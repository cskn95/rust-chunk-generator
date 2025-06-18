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
        
        // Hide cursor
        window.set_cursor_visible(false);
        
        // Try to grab cursor
        if let Err(e) = window.set_cursor_grab(CursorGrabMode::Confined) {
            log::warn!("Failed to grab cursor with Confined mode: {}", e);
            // Fallback to locked mode
            if let Err(e) = window.set_cursor_grab(CursorGrabMode::Locked) {
                log::warn!("Failed to grab cursor with Locked mode: {}", e);
            } else {
                log::info!("Cursor grabbed with Locked mode");
            }
        } else {
            log::info!("Cursor grabbed with Confined mode");
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            log::info!("Initializing winit & wgpu");
            let window_attributes = WindowAttributes::default()
                .with_title("Voxel Engine")
                .with_fullscreen(Some(winit::window::Fullscreen::Borderless(None)));

            let window = Arc::new(
                event_loop
                    .create_window(window_attributes)
                    .expect("Failed to create window"),
            );

            self.grab_cursor(&window);
            
            self.window = Some(window.clone());

            match pollster::block_on(State::new(window.clone())) {
                Ok(state) => {
                    self.state = Some(state);
                    // Request initial redraw to start the render loop
                    window.request_redraw();
                    log::info!("Voxel engine initialized");
                }
                Err(e) => {
                    log::error!("Failed to initialize: {}", e);
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
        
        // Request redraw on any input to ensure responsive UI
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
                            log::info!("Exited fullscreen mode");
                        } else {
                            window.set_fullscreen(Some(winit::window::Fullscreen::Borderless(None)));
                            log::info!("Entered fullscreen mode");
                        }
                    }
                },
                
                WindowEvent::Resized(physical_size) => state.resize(physical_size),
                
                WindowEvent::RedrawRequested => {
                    state.update();
                    
                    match state.render() {
                        Ok(_) => {
                            // Request next frame only if there's movement (power saving with V-Sync)
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
                            log::error!("Out of memory");
                            event_loop.exit();
                        },
                        Err(wgpu::SurfaceError::Other) => {
                            log::error!("Surface error");
                            event_loop.exit();
                        },
                        Err(wgpu::SurfaceError::Timeout) => {
                            log::warn!("Surface timeout")
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
            
            // Request redraw on mouse movement for responsive camera control
            if let DeviceEvent::MouseMotion { .. } = event {
                if let Some(window) = self.window.as_ref() {
                    window.request_redraw();
                }
            }
        }
    }

    fn exiting(&mut self, _event_loop: &ActiveEventLoop) {
        log::info!("Exiting voxel engine");
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    log::info!("Starting voxel engine...");
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Wait); // Power-saving mode with V-Sync

    let mut app = App::default();
    event_loop.run_app(&mut app)?;

    log::info!("Voxel engine shutdown complete");
    Ok(())
}