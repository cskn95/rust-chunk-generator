mod renderer;
mod vertex;
mod texture;
mod camera;
mod camera_controller;

use std::error::Error;
use std::sync::Arc;
use winit::application::ApplicationHandler;
use winit::event::{DeviceEvent, ElementState, KeyEvent, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowAttributes, WindowId};
use renderer::State;

/// Ana uygulama yapısı - pencere ve render durumunu saklar
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
    fn grab_cursor(&self, window: &Window) {
        use winit::window::CursorGrabMode;
        
        // İmleci gizle
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
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            log::info!("Winit ve WGPU başlatılıyor");
            let window_attributes = WindowAttributes::default()
                .with_title("Voxel Motoru")
                .with_fullscreen(Some(winit::window::Fullscreen::Borderless(None)));

            let window = Arc::new(
                event_loop
                    .create_window(window_attributes)
                    .expect("Pencere oluşturma başarısız"),
            );

            // İmleci yakalayıp gizle
            self.grab_cursor(&window);
            
            self.window = Some(window.clone());

            // Render durumunu asenkron olarak başlat
            match pollster::block_on(State::new(window.clone())) {
                Ok(state) => {
                    self.state = Some(state);
                    // Render döngüsünü başlatmak için ilk çizim isteği
                    window.request_redraw();
                    log::info!("Voxel motoru başarıyla başlatıldı");
                }
                Err(e) => {
                    log::error!("Başlatma başarısız: {}", e);
                    event_loop.exit();
                }
            }
        }
    }

    /// Pencere olaylarını işler
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

        // Giriş olayını state'e gönder
        let consumed_input = state.input(&event);
        
        // Herhangi bir giriş olduğunda hızlı tepki için çizim iste
        if consumed_input {
            if let Some(window) = self.window.as_ref() {
                window.request_redraw();
            }
        }
        
        // State tarafından işlenmemiş olayları burada ele al
        if !consumed_input {
            match event {
                WindowEvent::CloseRequested => event_loop.exit(),
                
                // Escape tuşu ile çıkış
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
                WindowEvent::Resized(physical_size) => state.resize(physical_size),
                
                // Çizim isteği geldiğinde render et
                WindowEvent::RedrawRequested => {
                    // Kamera ve oyun durumunu güncelle
                    state.update();
                    
                    // Sahneyi render et
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
                            log::error!("Bellek yetersiz");
                            event_loop.exit();
                        },
                        // Diğer surface hataları
                        Err(wgpu::SurfaceError::Other) => {
                            log::error!("Surface hatası");
                            event_loop.exit();
                        },
                        // Zaman aşımı (genellikle sorun değil)
                        Err(wgpu::SurfaceError::Timeout) => {
                            log::warn!("Surface zaman aşımı")
                        },
                    }
                }
                _ => {}
            }
        }
    }

    /// Cihaz girişlerini işler (fare hareketi gibi)
    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: winit::event::DeviceId,
        event: DeviceEvent,
    ) {
        if let Some(state) = self.state.as_mut() {
            // Cihaz girişini state'e gönder
            state.device_input(&event);
            
            // Fare hareketi durumunda hızlı kamera kontrolü için çizim iste
            if let DeviceEvent::MouseMotion { .. } = event {
                if let Some(window) = self.window.as_ref() {
                    window.request_redraw();
                }
            }
        }
    }

    /// Uygulama kapanırken çağrılır
    fn exiting(&mut self, _event_loop: &ActiveEventLoop) {
        log::info!("Voxel motoru kapatılıyor");
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    // Loglama sistemini başlat
    env_logger::init();

    log::info!("Voxel motoru başlatılıyor...");
    let event_loop = EventLoop::new().unwrap();
    // V-Sync ile güç tasarrufu modu
    event_loop.set_control_flow(ControlFlow::Wait);

    let mut app = App::default();
    event_loop.run_app(&mut app)?;

    Ok(())
}