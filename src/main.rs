mod renderer;
mod vertex;
mod texture;
mod camera;
mod camera_controller;

use std::error::Error;
use std::sync::Arc;
use winit::application::ApplicationHandler;
use winit::event::{ElementState, KeyEvent, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowAttributes, WindowId};
use renderer::State;

struct App {
    window: Option<Arc<Window>>,
    state: Option<State>,
}

impl Default for App {
    fn default() -> Self {
        Self { window: None, state: None }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            log::info!("Initializing winit & wgpu");
            let window_attributes = WindowAttributes::default().with_title("Voxel Engine");

            let window = Arc::new(
                event_loop
                    .create_window(window_attributes)
                    .expect("Failed to create window"),
            );
            self.window = Some(window.clone());

            match pollster::block_on(State::new(window)) {
                Ok(state) => {
                    self.state = Some(state);
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

        if !state.input(&event) {
            match event {
                WindowEvent::CloseRequested
                | WindowEvent::KeyboardInput {
                    event: KeyEvent { 
                        state: ElementState::Pressed, 
                        physical_key: winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::Escape),
                        ..
                    },
                    ..
                } => event_loop.exit(),
                
                WindowEvent::Resized(physical_size) => state.resize(physical_size),
                
                WindowEvent::RedrawRequested => {
                    if let Some(window) = self.window.as_ref() {
                        window.request_redraw();
                    }
                    
                    state.update();
                    
                    match state.render() {
                        Ok(_) => {},
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

    fn exiting(&mut self, _event_loop: &ActiveEventLoop) {
        log::info!("Exiting voxel engine");
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    log::info!("Starting voxel engine...");
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App::default();
    event_loop.run_app(&mut app)?;

    log::info!("Voxel engine shutdown complete");
    Ok(())
}