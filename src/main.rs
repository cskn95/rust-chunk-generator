use std::error::Error;
use std::sync::Arc;
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::{KeyEvent, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{Key, NamedKey};
use winit::window::{Window, WindowAttributes, WindowId};

struct State {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface_config: wgpu::SurfaceConfiguration,
    size: PhysicalSize<u32>,
}

impl State {
    async fn new(window: Arc<Window>) -> Result<Self, Box<dyn Error>> {
        let size = window.inner_size();
        // Pencere boyutu 0 ise wgpu başlamaz
        if size.width == 0 || size.height == 0 {
            return Err("Pencere boyutu sıfır olamaz.".into());
        }

        let instance_descriptor = wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        };

        let instance = wgpu::Instance::new(&instance_descriptor);

        let surface = instance.create_surface(window.clone()).unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            }).await.unwrap();

        log::info!("Adaptör: {:?}", adapter.get_info());

        let device_descriptor = wgpu::DeviceDescriptor {
            label: Some("Device"),
            required_features: wgpu::Features::default(),
            required_limits: wgpu::Limits::default(),
            memory_hints: wgpu::MemoryHints::Performance,
            trace: wgpu::Trace::Off
        };
        let (device, queue) = adapter.request_device(&device_descriptor).await.unwrap();

        let surface_caps = surface.get_capabilities(&adapter);

        let surface_format = surface_caps.formats.iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        Ok(Self {
            surface,
            device,
            queue,
            surface_config,
            size,
        })
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 && self.size != new_size {
            self.size = new_size;
            self.surface_config.width = new_size.width;
            self.surface_config.height = new_size.height;
            self.surface.configure(&self.device, &self.surface_config);
        }
    }

    #[allow(unused_variables)]
    fn input(&mut self, event: &WindowEvent) -> bool {
        false
    }

    fn update(&mut self) {
        todo!()
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        todo!()
    }
}

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
            log::info!("winit & wgpu başlatılıyor");
            let window_attributes = WindowAttributes::default().with_title("winit & wgpu");

            let window = Arc::new(
                event_loop
                    .create_window(window_attributes)
                    .expect("Pencere oluşturulamadı"),
            );
            self.window = Some(window.clone());

            match pollster::block_on(State::new(window)) {
                Ok(state) => {
                    self.state = Some(state);
                    log::info!("window & state hazır");
                }
                Err(e) => {
                    log::error!("bir error yaklaşıyor efendim: {}", e);
                    event_loop.exit();
                }
            }
        } else {
            log::info!("window resume");
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _window_id: WindowId, event: WindowEvent) {

        let state = match self.state.as_mut() {
            Some(s) => s,
            None => {
                if let WindowEvent::CloseRequested = event {
                    log::warn!("state yok, kapatma isteği");
                    event_loop.exit();
                }
                return;
            }
        };

        match event {
            winit::event::Event::WindowEvent { event, ..} if !state.input(&event)
        }
    }

    fn exiting(&mut self, _event_loop: &ActiveEventLoop) {
        log::info!("App: Çıkılıyor.");
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    log::info!("Olay döngüsü oluşturuluyor...");
    let event_loop = EventLoop::new().unwrap();

    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App::default();

    event_loop.run_app(&mut app)?;

    log::info!("Olay döngüsü tamamlandı.");
    Ok(())
}