use std::error::Error;
use std::sync::Arc;
use std::time::Instant;
use wgpu::util::DeviceExt;
use winit::dpi::PhysicalSize;
use winit::event::{DeviceEvent, WindowEvent};
use winit::window::Window;

use crate::vertex::*;
use crate::texture::Texture;
use crate::camera::{Camera, CameraUniform};
use crate::camera_controller::CameraController;

/// Ana render durumu
/// GPU kaynaklarını, kamerayı ve render pipeline'ını yönetir
pub struct State {
    surface: wgpu::Surface<'static>,         // Çizim yüzeyi (ekran)
    device: wgpu::Device,                    // GPU cihazı
    queue: wgpu::Queue,                      // GPU komut kuyruğu
    surface_config: wgpu::SurfaceConfiguration, // Yüzey konfigürasyonu
    size: PhysicalSize<u32>,                 // Pencere boyutu
    render_pipeline: wgpu::RenderPipeline,   // Render pipeline
    vertex_buffer: wgpu::Buffer,             // Vertex buffer
    index_buffer: wgpu::Buffer,              // Index buffer
    num_indices: u32,                        // Index sayısı
    camera: Camera,                          // Kamera
    camera_uniform: CameraUniform,           // Kamera uniform verileri
    camera_buffer: wgpu::Buffer,             // Kamera buffer'ı
    camera_bind_group: wgpu::BindGroup,      // Kamera bind group
    camera_controller: CameraController,     // Kamera kontrolcüsü
    depth_texture: Texture,                  // Derinlik texture'ı
    last_update_time: Instant,               // Son güncelleme zamanı
    is_active: bool,                         // Aktif güncellemeler yapılıyor mu?
}

impl State {
    /// Yeni State oluşturur ve GPU kaynaklarını başlatır
    pub async fn new(window: Arc<Window>) -> Result<Self, Box<dyn Error>> {
        let size = window.inner_size();
        
        if size.width == 0 || size.height == 0 {
            return Err("Pencere boyutu sıfır olamaz.".into());
        }

        // WGPU instance ve adapter kurulumu
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        let surface = instance.create_surface(window.clone()).unwrap();

        // Yüksek performanslı GPU adaptörü seç
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        // GPU cihazı ve komut kuyruğu oluştur
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("Device"),
                    required_features: wgpu::Features::default(),
                    required_limits: wgpu::Limits::default(),
                    memory_hints: wgpu::MemoryHints::Performance,
                    trace: wgpu::Trace::Off,
                },
            )
            .await
            .unwrap();

        // Yüzey konfigürasyonu
        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        // V-Sync kullan (Fifo modu) - FPS'i ekran yenileme hızına sınırlar ve güç tasarrufu sağlar
        let present_mode = wgpu::PresentMode::Fifo;

        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2, // V-Sync kararlılığı için optimal
        };

        surface.configure(&device, &surface_config);

        // Kamera kurulumu
        let camera = Camera::new(
            (0.0, 5.0, 10.0).into(),        // Göz pozisyonu
            std::f32::consts::PI,           // Yaw: 180° (orijine doğru bak)
            0.0,                            // Pitch: 0° (yatay)
            cgmath::Vector3::unit_y(),       // Yukarı vektörü
            surface_config.width as f32 / surface_config.height as f32,  // En/boy oranı
            45.0,                           // Görüş alanı
            0.1,                            // Yakın düzlem
            100.0,                          // Uzak düzlem
        );

        let mut camera_uniform = CameraUniform::new();
        camera_uniform.update_view_proj(&camera);

        // Kamera uniform buffer'ını oluştur
        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        // Kamera bind group layout'u oluştur
        let camera_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
            label: Some("camera_bind_group_layout"),
        });

        // Kamera bind group oluştur
        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("camera_bind_group"),
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
        });

        // Kamera kontrolcüsü (delta time ile saniye başına 5 birim hareket hızı)
        let camera_controller = CameraController::new(5.0);

        // Shader ve pipeline
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[&camera_bind_group_layout],
            push_constant_ranges: &[],
        });

        // Render pipeline oluştur
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[Vertex::desc()],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: Texture::DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        });

        // Vertex ve index buffer'larını oluştur
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });

        let num_indices = INDICES.len() as u32;

        // Derinlik texture'ı oluştur
        let depth_texture = Texture::create_depth_texture(&device, &surface_config, "depth_texture");

        Ok(Self {
            surface,
            device,
            queue,
            surface_config,
            size,
            render_pipeline,
            vertex_buffer,
            index_buffer,
            num_indices,
            camera,
            camera_uniform,
            camera_buffer,
            camera_bind_group,
            camera_controller,
            depth_texture,
            last_update_time: Instant::now(),
            is_active: false,
        })
    }

    /// Mevcut pencere boyutunu döndürür
    pub fn get_size(&self) -> PhysicalSize<u32> {
        self.size
    }

    /// Pencere boyutu değiştiğinde yeniden boyutlandırma yapar
    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.surface_config.width = new_size.width;
            self.surface_config.height = new_size.height;
            self.surface.configure(&self.device, &self.surface_config);
            self.depth_texture = Texture::create_depth_texture(&self.device, &self.surface_config, "depth_texture");
            
            // Kamera en/boy oranını güncelle
            self.camera = Camera::new(
                self.camera.eye(),
                self.camera.yaw(),
                self.camera.pitch(),
                self.camera.up(),
                new_size.width as f32 / new_size.height as f32,
                45.0,
                0.1,
                100.0,
            );
        }
    }

    /// Pencere girişlerini işler
    /// Döndürülen bool değeri girişin işlenip işlenmediğini belirtir
    pub fn input(&mut self, event: &WindowEvent) -> bool {
        match event {
            // Klavye girişlerini işle
            WindowEvent::KeyboardInput { event: key_event, .. } => {
                if let winit::keyboard::PhysicalKey::Code(keycode) = key_event.physical_key {
                    let was_consumed = self.camera_controller.process_events(keycode, key_event.state);
                    
                    // Hareket başladıysa ve aktif değildiysek, zamanlayıcıyı sıfırla
                    if was_consumed && !self.is_active {
                        self.last_update_time = Instant::now();
                        self.is_active = true;
                    }
                    
                    was_consumed
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    /// Cihaz girişlerini işler (fare hareketi gibi)
    pub fn device_input(&mut self, event: &DeviceEvent) {
        match event {
            // İmleç yakalandığında FPS kamera bakışı için ham fare hareketini işle
            DeviceEvent::MouseMotion { delta } => {
                // Fare hareketi de güncellemeleri etkinleştirmeli
                if !self.is_active {
                    self.last_update_time = Instant::now();
                    self.is_active = true;
                }
                self.camera_controller.process_mouse(delta.0, delta.1, &mut self.camera);
            }
            _ => {}
        }
    }

    /// Her frame'de kamera ve oyun durumunu günceller
    pub fn update(&mut self) {
        let now = Instant::now();
        
        // Büyük zıplamaları önlemek için delta time hesapla (maksimum sınır ile)
        let raw_delta = now.duration_since(self.last_update_time).as_secs_f32();
        let delta_time = raw_delta.min(0.1); // 100ms'de sınırla (minimum 10 FPS)
        
        // Sadece aktif render yapıyorsak zamanlayıcıyı güncelle
        // Bu, boşta kalma dönemlerinde delta time birikimini önler
        if self.is_active || raw_delta < 0.5 {
            self.last_update_time = now;
        }
        
        // Frame hızından bağımsız hareket için delta time'ı kamera hareketine uygula
        self.camera_controller.update_camera(&mut self.camera, delta_time);
        self.camera_uniform.update_view_proj(&self.camera);
        self.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[self.camera_uniform]),
        );
        
        // Hareket olup olmadığına göre aktif durumu güncelle
        self.is_active = self.camera_controller.has_movement();
    }

    /// Render etmeye devam edilip edilmeyeceğini döndürür
    pub fn should_continue_rendering(&self) -> bool {
        // Pürüzsüz etkileşim için kamera hareketi varsa render etmeye devam et
        // V-Sync frame hızını otomatik olarak ekran yenileme hızına sınırlar
        self.camera_controller.has_movement()
    }

    /// Sahneyi render eder
    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,  // Koyu mavi arkaplan
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),  // Derinlik buffer'ını temizle
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            // Render pipeline ve kaynakları ayarla
            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            
            // Küpü çiz
            render_pass.draw_indexed(0..self.num_indices, 0, 0..1);
        }

        // Komutları GPU'ya gönder ve ekranı güncelle
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}