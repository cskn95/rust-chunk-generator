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
use crate::chunk::Chunk;  // Yeni eklendi - chunk sistemi

/// Ana render durumu - artık chunk-based voksel rendering destekli
/// GPU kaynaklarını, kamerayı, chunk'ları ve render pipeline'ını yönetir
pub struct State {
    surface: wgpu::Surface<'static>,         // Çizim yüzeyi (ekran)
    device: wgpu::Device,                    // GPU cihazı
    queue: wgpu::Queue,                      // GPU komut kuyruğu
    surface_config: wgpu::SurfaceConfiguration, // Yüzey konfigürasyonu
    size: PhysicalSize<u32>,                 // Pencere boyutu
    render_pipeline: wgpu::RenderPipeline,   // Render pipeline
    
    // Chunk rendering için yeni alanlar
    chunk_vertex_buffer: wgpu::Buffer,       // Chunk mesh vertex buffer'ı
    chunk_index_buffer: wgpu::Buffer,        // Chunk mesh index buffer'ı
    chunk_vertex_count: u32,                 // Chunk'taki vertex sayısı
    chunk_index_count: u32,                  // Chunk'taki index sayısı
    chunk: Chunk,                            // Test chunk'ımız
    
    // Kamera sistemi (değişmedi)
    camera: Camera,                          
    camera_uniform: CameraUniform,           
    camera_buffer: wgpu::Buffer,             
    camera_bind_group: wgpu::BindGroup,      
    camera_controller: CameraController,     
    depth_texture: Texture,                  
    last_update_time: Instant,               
    is_active: bool,                         
}

impl State {
    // Yeni State oluşturur ve GPU kaynaklarını başlatır
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

        // Yüksek performanslı GPU adaptörü seç - voksel rendering GPU yoğun
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
                    label: Some("Voksel GPU Device"),
                    required_features: wgpu::Features::default(),
                    required_limits: wgpu::Limits::default(),
                    memory_hints: wgpu::MemoryHints::Performance,
                    trace: wgpu::Trace::Off,
                },
            )
            .await
            .unwrap();

        // Yüzey konfigürasyonu (değişmedi ama voksel rendering için optimize)
        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        // V-Sync kullan - chunk rendering'de frame pacing için önemli
        let present_mode = wgpu::PresentMode::Fifo;

        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        surface.configure(&device, &surface_config);

        // Kamera kurulumu - voksel dünyasını görmek için konumlandırılmış
        let camera = Camera::new(
            (8.0, 12.0, 8.0).into(),         // Chunk'ı görebilecek pozisyon
            std::f32::consts::PI * 1.25,     // Chunk merkezine doğru yönlendirilmiş
            -0.3,                            // Hafif aşağı bakıyor
            cgmath::Vector3::unit_y(),       
            surface_config.width as f32 / surface_config.height as f32,  
            45.0,                           
            0.1,                            
            100.0,                          
        );

        let mut camera_uniform = CameraUniform::new();
        camera_uniform.update_view_proj(&camera);

        // Kamera uniform buffer'ını oluştur (değişmedi)
        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        // Kamera bind group layout'u oluştur (değişmedi)
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

        // Kamera bind group oluştur (değişmedi)
        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("camera_bind_group"),
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
        });

        // Kamera kontrolcüsü - voksel dünyasında gezinmek için
        let camera_controller = CameraController::new(8.0); // Biraz daha hızlı hareket

        // Shader ve pipeline - artık renk desteği ile
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Voksel Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Voksel Render Pipeline Layout"),
            bind_group_layouts: &[&camera_bind_group_layout],
            push_constant_ranges: &[],
        });

        // Render pipeline oluştur - vertex layout artık renk bilgisi içeriyor
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Voksel Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[Vertex::desc()], // Artık position + color layout
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
                cull_mode: Some(wgpu::Face::Back), // Voksel face culling için önemli
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: Texture::DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less, // Voksel rendering için kritik
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

        // Test chunk'ı oluştur ve terrain generate et
        let mut chunk = Chunk::new((0, 0, 0)); // Orijinde bir chunk
        chunk.generate_test_terrain();

        // Chunk'tan mesh oluştur
        let (vertices, indices) = chunk.generate_mesh();
        
        log::info!("Chunk mesh oluşturuldu: {} vertex, {} triangle", 
                  vertices.len(), indices.len() / 3);

        // Chunk mesh için vertex ve index buffer'ları oluştur
        let chunk_vertex_buffer = if !vertices.is_empty() {
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Chunk Vertex Buffer"),
                contents: bytemuck::cast_slice(&vertices),
                usage: wgpu::BufferUsages::VERTEX,
            })
        } else {
            // Boş chunk için dummy buffer
            device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Empty Chunk Vertex Buffer"),
                size: 64, // Minimum boyut
                usage: wgpu::BufferUsages::VERTEX,
                mapped_at_creation: false,
            })
        };

        let chunk_index_buffer = if !indices.is_empty() {
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Chunk Index Buffer"),
                contents: bytemuck::cast_slice(&indices),
                usage: wgpu::BufferUsages::INDEX,
            })
        } else {
            // Boş chunk için dummy buffer
            device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Empty Chunk Index Buffer"),
                size: 64,
                usage: wgpu::BufferUsages::INDEX,
                mapped_at_creation: false,
            })
        };

        let chunk_vertex_count = vertices.len() as u32;
        let chunk_index_count = indices.len() as u32;

        // Derinlik texture'ı oluştur (değişmedi)
        let depth_texture = Texture::create_depth_texture(&device, &surface_config, "depth_texture");

        Ok(Self {
            surface,
            device,
            queue,
            surface_config,
            size,
            render_pipeline,
            chunk_vertex_buffer,
            chunk_index_buffer,
            chunk_vertex_count,
            chunk_index_count,
            chunk,
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

    /// Mevcut pencere boyutunu döndürür (değişmedi)
    pub fn get_size(&self) -> PhysicalSize<u32> {
        self.size
    }

    /// Pencere boyutu değiştiğinde yeniden boyutlandırma yapar
    /// Chunk rendering için depth buffer ve kamera aspect ratio güncellenir
    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.surface_config.width = new_size.width;
            self.surface_config.height = new_size.height;
            self.surface.configure(&self.device, &self.surface_config);
            self.depth_texture = Texture::create_depth_texture(&self.device, &self.surface_config, "depth_texture");
            
            // Kamera en/boy oranını güncelle - voksel dünyası doğru perspektifle görünsün
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

    /// Pencere girişlerini işler (değişmedi ama voksel editing için hazır)
    pub fn input(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::KeyboardInput { event: key_event, .. } => {
                if let winit::keyboard::PhysicalKey::Code(keycode) = key_event.physical_key {
                    let was_consumed = self.camera_controller.process_events(keycode, key_event.state);
                    
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

    /// Cihaz girişlerini işler - voksel dünyasında kamera kontrolü
    pub fn device_input(&mut self, event: &DeviceEvent) {
        match event {
            DeviceEvent::MouseMotion { delta } => {
                if !self.is_active {
                    self.last_update_time = Instant::now();
                    self.is_active = true;
                }
                self.camera_controller.process_mouse(delta.0, delta.1, &mut self.camera);
            }
            _ => {}
        }
    }

    /// Her frame'de kamera ve chunk durumunu günceller
    pub fn update(&mut self) {
        let now = Instant::now();
        let raw_delta = now.duration_since(self.last_update_time).as_secs_f32();
        let delta_time = raw_delta.min(0.1); // Chunk rendering için stabilite
        
        if self.is_active || raw_delta < 0.5 {
            self.last_update_time = now;
        }
        
        // Kamera hareketini güncelle - voksel dünyasında gezinmek için
        self.camera_controller.update_camera(&mut self.camera, delta_time);
        self.camera_uniform.update_view_proj(&self.camera);
        self.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[self.camera_uniform]),
        );
        
        self.is_active = self.camera_controller.has_movement();
        
        // Gelecekte burada chunk loading/unloading logic'i eklenecek
        // Şu anda tek chunk ile test ediyoruz
    }

    /// Render etmeye devam edilip edilmeyeceğini döndürür
    pub fn should_continue_rendering(&self) -> bool {
        self.camera_controller.has_movement()
    }

    /// Chunk'ları render eder - artık voksel mesh rendering
    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Voksel Render Encoder"),
        });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Voksel Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.3,  // Açık mavi gökyüzü - voksel dünyası için uygun
                            g: 0.7,
                            b: 1.0,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
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
            
            // Chunk mesh'ini render et (eğer vertex varsa)
            if self.chunk_index_count > 0 {
                render_pass.set_vertex_buffer(0, self.chunk_vertex_buffer.slice(..));
                render_pass.set_index_buffer(self.chunk_index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                render_pass.draw_indexed(0..self.chunk_index_count, 0, 0..1);
            }
        }

        // Komutları GPU'ya gönder ve ekranı güncelle
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}