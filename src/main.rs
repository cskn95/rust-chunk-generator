// Mevcut modüller
mod renderer;
mod vertex;
mod texture;
mod camera;
mod camera_controller;

// Yeni voksel motor modülleri
mod voxel;     // Voksel türlerini tanımlayan modül
mod chunk;     // Chunk sistemi ve mesh generation

use std::error::Error;
use std::sync::Arc;
use winit::application::ApplicationHandler;
use winit::event::{DeviceEvent, ElementState, KeyEvent, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowAttributes, WindowId};
use renderer::State;

/// Ana uygulama yapısı - pencere ve render durumunu saklar
/// Artık voksel motor desteği ile geliştirilmiş durumda
struct App {
    window: Option<Arc<Window>>,
    state: Option<State>,
}

impl Default for App {
    fn default() -> Self {
        Self { 
            window: None, 
            state: None,
        }
    }
}

impl App {
    /// İmleci yakalayıp gizler (FPS oyunu benzeri kontrol için)
    /// Voksel dünyasında hareket etmek için gerekli
    fn grab_cursor(&self, window: &Window) {
        use winit::window::CursorGrabMode;
        
        // İmleci gizle - voksel oyunlarda standart davranış
        window.set_cursor_visible(false);
        
        // İmleci yakalamaya çalış (Confined modu ile)
        if let Err(e) = window.set_cursor_grab(CursorGrabMode::Confined) {
            log::warn!("İmleç yakalama Confined modu ile başarısız: {}", e);
            // Başarısızlık durumunda Locked moduna geç
            if let Err(e) = window.set_cursor_grab(CursorGrabMode::Locked) {
                log::warn!("İmleç yakalama Locked modu ile başarısız: {}", e);
            } else {
                log::info!("İmleç Locked modu ile yakalandı");
            }
        } else {
            log::info!("İmleç Confined modu ile yakalandı");
        }
    }
}

impl ApplicationHandler for App {
    /// Uygulama etkinleştirildiğinde çağrılır
    /// Voksel motor inicializasyonu burada yapılır
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            log::info!("Voksel Motor başlatılıyor - Winit ve WGPU inicializasyonu");
            let window_attributes = WindowAttributes::default()
                .with_title("Voksel Motoru - Chunk Sistemi")  // Başlık güncellendi
                .with_fullscreen(Some(winit::window::Fullscreen::Borderless(None)));

            let window = Arc::new(
                event_loop
                    .create_window(window_attributes)
                    .expect("Pencere oluşturma başarısız"),
            );

            // FPS tarzı kamera kontrolü için imleci yakala
            self.grab_cursor(&window);
            
            self.window = Some(window.clone());

            // Render durumunu asenkron olarak başlat - artık chunk sistemiyle
            match pollster::block_on(State::new(window.clone())) {
                Ok(state) => {
                    self.state = Some(state);
                    // İlk chunk render'ı için çizim isteği
                    window.request_redraw();
                    log::info!("Voksel motoru ve chunk sistemi başarıyla başlatıldı");
                }
                Err(e) => {
                    log::error!("Voksel motor başlatma başarısız: {}", e);
                    event_loop.exit();
                }
            }
        }
    }

    /// Pencere olaylarını işler
    /// Kamera kontrolü ve voksel etkileşimi için gerekli
    fn window_event(&mut self, event_loop: &ActiveEventLoop, _window_id: WindowId, event: WindowEvent) {
        let state = match self.state.as_mut() {
            Some(s) => s,
            None => {
                // State henüz hazır değilse sadece kapatma olayını dinle
                if let WindowEvent::CloseRequested = event {
                    event_loop.exit();
                }
                return;
            }
        };

        // Giriş olayını state'e gönder (kamera kontrolü dahil)
        let consumed_input = state.input(&event);
        
        // Herhangi bir giriş olduğunda hızlı tepki için çizim iste
        // Voksel dünyasında smooth hareket için önemli
        if consumed_input {
            if let Some(window) = self.window.as_ref() {
                window.request_redraw();
            }
        }
        
        // State tarafından işlenmemiş olayları burada ele al
        if !consumed_input {
            match event {
                WindowEvent::CloseRequested => event_loop.exit(),
                
                // Escape tuşu ile çıkış - voksel editörlerinde standart
                WindowEvent::KeyboardInput {
                    event: KeyEvent { 
                        state: ElementState::Pressed, 
                        physical_key: winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::Escape),
                        ..
                    },
                    ..
                } => {event_loop.exit()},
                
                // F11 tuşu ile tam ekran geçişi
                WindowEvent::KeyboardInput {
                    event: KeyEvent { 
                        state: ElementState::Pressed, 
                        physical_key: winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::F11),
                        ..
                    },
                    ..
                } => {
                    if let Some(window) = self.window.as_ref() {
                        let is_fullscreen = window.fullscreen().is_some();
                        if is_fullscreen {
                            window.set_fullscreen(None);
                            log::info!("Tam ekran modundan çıkıldı");
                        } else {
                            window.set_fullscreen(Some(winit::window::Fullscreen::Borderless(None)));
                            log::info!("Tam ekran moduna geçildi");
                        }
                    }
                },
                
                // Pencere boyutu değiştiğinde yeniden boyutlandır
                // Chunk render'ı için önemli
                WindowEvent::Resized(physical_size) => state.resize(physical_size),
                
                // Çizim isteği geldiğinde chunk'ları render et
                WindowEvent::RedrawRequested => {
                    // Kamera ve voksel dünyası durumunu güncelle
                    state.update();
                    
                    // Chunk'ları render et
                    match state.render() {
                        Ok(_) => {
                            // Hareket varsa bir sonraki frame'i iste (V-Sync ile güç tasarrufu)
                            if state.should_continue_rendering() {
                                if let Some(window) = self.window.as_ref() {
                                    window.request_redraw();
                                }
                            }
                        },
                        // Surface kayboldu veya güncel değil - yeniden boyutlandır
                        Err(wgpu::SurfaceError::Lost) | Err(wgpu::SurfaceError::Outdated) => {
                            state.resize(state.get_size())
                        },
                        // Bellek yetersiz - uygulamayı kapat
                        Err(wgpu::SurfaceError::OutOfMemory) => {
                            log::error!("GPU belleği yetersiz - voksel verisi çok büyük olabilir");
                            event_loop.exit();
                        },
                        // Diğer surface hataları
                        Err(wgpu::SurfaceError::Other) => {
                            log::error!("GPU surface hatası");
                            event_loop.exit();
                        },
                        // Zaman aşımı (genellikle sorun değil)
                        Err(wgpu::SurfaceError::Timeout) => {
                            log::warn!("GPU render zaman aşımı")
                        },
                    }
                }
                _ => {}
            }
        }
    }

    /// Cihaz girişlerini işler (fare hareketi gibi)
    /// Voksel dünyasında kamera kontrolü için kritik
    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: winit::event::DeviceId,
        event: DeviceEvent,
    ) {
        if let Some(state) = self.state.as_mut() {
            // Cihaz girişini state'e gönder (fare hareketi dahil)
            state.device_input(&event);
            
            // Fare hareketi durumunda hızlı kamera kontrolü için çizim iste
            // Voksel dünyasında smooth bakış için gerekli
            if let DeviceEvent::MouseMotion { .. } = event {
                if let Some(window) = self.window.as_ref() {
                    window.request_redraw();
                }
            }
        }
    }

    /// Uygulama kapanırken çağrılır
    fn exiting(&mut self, _event_loop: &ActiveEventLoop) {
        log::info!("Voksel motoru kapatılıyor - chunk verileri temizleniyor");
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    // Loglama sistemini başlat
    env_logger::init();

    log::info!("Voksel motoru başlatılıyor - Chunk based rendering sistemi...");
    let event_loop = EventLoop::new().unwrap();
    // V-Sync ile güç tasarrufu modu - chunk rendering için optimal
    event_loop.set_control_flow(ControlFlow::Wait);

    let mut app = App::default();
    event_loop.run_app(&mut app)?;

    Ok(())
}