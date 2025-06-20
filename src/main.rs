mod renderer;
mod vertex;
mod camera;
mod camera_controller;
mod voxel;
mod chunk;

use std::error::Error;
use std::sync::Arc;
use winit::application::ApplicationHandler;
use winit::event::{DeviceEvent, ElementState, KeyEvent, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowAttributes, WindowId};
use renderer::State;

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
    fn grab_cursor(&self, window: &Window) {
        use winit::window::CursorGrabMode;
        
        window.set_cursor_visible(false);
        
        if let Err(e) = window.set_cursor_grab(CursorGrabMode::Confined) {
            log::warn!("İmleç yakalama Confined modu ile başarısız: {}", e);
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
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            log::info!("Voksel Motor başlatılıyor - Winit ve WGPU inicializasyonu");
            let window_attributes = WindowAttributes::default()
                .with_title("Voksel Motoru - Chunk Sistemi")
                .with_fullscreen(Some(winit::window::Fullscreen::Borderless(None)));

            let window = Arc::new(
                event_loop
                    .create_window(window_attributes)
                    .expect("Pencere oluşturma başarısız"),
            );

            self.grab_cursor(&window);
            
            self.window = Some(window.clone());

            match pollster::block_on(State::new(window.clone())) {
                Ok(state) => {
                    self.state = Some(state);
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

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _window_id: WindowId, event: WindowEvent) {
        let state = match self.state.as_mut() {
            Some(s) => s,
            None => {
                if let WindowEvent::CloseRequested = event {
                    event_loop.exit();
                }
                return;
            }
        };

        let consumed_input = state.input(&event);
        
        if consumed_input {
            if let Some(window) = self.window.as_ref() {
                window.request_redraw();
            }
        }
        
        if !consumed_input {
            match event {
                WindowEvent::CloseRequested => event_loop.exit(),
                
                WindowEvent::KeyboardInput {
                    event: KeyEvent { 
                        state: ElementState::Pressed, 
                        physical_key: winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::Escape),
                        ..
                    },
                    ..
                } => {event_loop.exit()},
                
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
                
                WindowEvent::Resized(physical_size) => state.resize(physical_size),
                
                WindowEvent::RedrawRequested => {
                    state.update();
                    
                    match state.render() {
                        Ok(_) => {
                            if state.should_continue_rendering() {
                                if let Some(window) = self.window.as_ref() {
                                    window.request_redraw();
                                }
                            }
                        },
                        Err(wgpu::SurfaceError::Lost) | Err(wgpu::SurfaceError::Outdated) => {
                            state.resize(state.get_size())
                        },
                        Err(wgpu::SurfaceError::OutOfMemory) => {
                            log::error!("GPU belleği yetersiz - voksel verisi çok büyük olabilir");
                            event_loop.exit();
                        },
                        Err(wgpu::SurfaceError::Other) => {
                            log::error!("GPU surface hatası");
                            event_loop.exit();
                        },
                        Err(wgpu::SurfaceError::Timeout) => {
                            log::warn!("GPU render zaman aşımı")
                        },
                    }
                }
                _ => {}
            }
        }
    }

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: winit::event::DeviceId,
        event: DeviceEvent,
    ) {
        if let Some(state) = self.state.as_mut() {
            state.device_input(&event);
            
            if let DeviceEvent::MouseMotion { .. } = event {
                if let Some(window) = self.window.as_ref() {
                    window.request_redraw();
                }
            }
        }
    }

    fn exiting(&mut self, _event_loop: &ActiveEventLoop) {
        log::info!("Voksel motoru kapatılıyor - chunk verileri temizleniyor");
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    log::info!("Voksel motoru başlatılıyor - Chunk based rendering sistemi...");
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Wait);

    let mut app = App::default();
    event_loop.run_app(&mut app)?;

    Ok(())
}