use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::sync::Arc;
use std::time::Instant;
use wgpu::util::DeviceExt;
use winit::dpi::PhysicalSize;
use winit::event::{DeviceEvent, WindowEvent};
use winit::window::Window;
use noise::{Perlin};

use crate::vertex::*;
use crate::camera::{Camera, CameraUniform};
use crate::camera_controller::CameraController;
use crate::chunk::{Chunk, CHUNK_SIZE};

const RENDER_DISTANCE: i32 = 10; // Render mesafesi (chunk cinsinden yarıçap)
const VERTICAL_RENDER_DISTANCE_CHUNKS: i32 = 3;

struct ChunkRenderData {
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    index_count: u32,
}

pub struct State {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface_config: wgpu::SurfaceConfiguration,
    size: PhysicalSize<u32>,
    render_pipeline: wgpu::RenderPipeline,
    
    chunks: HashMap<(i32, i32, i32), Chunk>,
    chunk_render_data: HashMap<(i32, i32, i32), ChunkRenderData>,
    noise: Perlin,
    
    camera: Camera,
    camera_uniform: CameraUniform,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    camera_controller: CameraController,
    depth_texture_view: wgpu::TextureView,
    last_update_time: Instant,
    is_active: bool,
}

impl State {
    pub async fn new(window: Arc<Window>) -> Result<Self, Box<dyn Error>> {
        let size = window.inner_size();
        
        if size.width == 0 || size.height == 0 {
            return Err("Pencere boyutu sıfır olamaz.".into());
        }

        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        let surface = instance.create_surface(window.clone()).unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

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

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

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

        let camera = Camera::new(
            (6.0, 40.0, 6.0).into(),
            std::f32::consts::PI * 1.1,
            -0.2,
            cgmath::Vector3::unit_y(),
            surface_config.width as f32 / surface_config.height as f32,
            65.0,
            0.01,
            1000.0,
        );

        let mut camera_uniform = CameraUniform::new();
        camera_uniform.update_view_proj(&camera);

        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

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

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("camera_bind_group"),
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
        });

        let camera_controller = CameraController::new(8.0);

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Voksel Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Voksel Render Pipeline Layout"),
            bind_group_layouts: &[&camera_bind_group_layout],
            push_constant_ranges: &[],
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Voksel Render Pipeline"),
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
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState {
                    constant: 0,
                    slope_scale: 0.0,
                    clamp: 0.0,
                },
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        });

        let noise = Perlin::new(123); // Seed değeri
        let mut chunks = HashMap::new();
        
        // Başlangıçta render mesafesindeki chunkları oluştur
        for y in 0..=VERTICAL_RENDER_DISTANCE_CHUNKS {
             for x in -RENDER_DISTANCE..=RENDER_DISTANCE {
                for z in -RENDER_DISTANCE..=RENDER_DISTANCE {
                    let chunk_pos = (x, y, z);
                    let mut chunk = Chunk::new(chunk_pos);
                    chunk.generate_terrain(&noise);
                    chunks.insert(chunk_pos, chunk);
                }
            }
        }
        
        // Render verisini `update()` döngüsü oluşturacak.
        let chunk_render_data = HashMap::new();

        let depth_texture_view = Self::create_depth_texture(&device, &surface_config);

        Ok(Self {
            surface,
            device,
            queue,
            surface_config,
            size,
            render_pipeline,
            chunks,
            chunk_render_data,
            noise,
            camera,
            camera_uniform,
            camera_buffer,
            camera_bind_group,
            camera_controller,
            depth_texture_view,
            last_update_time: Instant::now(),
            is_active: true,
        })
    }

    fn create_depth_texture(device: &wgpu::Device, config: &wgpu::SurfaceConfiguration) -> wgpu::TextureView {
        let size = wgpu::Extent3d {
            width: config.width.max(1),
            height: config.height.max(1),
            depth_or_array_layers: 1,
        };
        
        let desc = wgpu::TextureDescriptor {
            label: Some("Depth Texture"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        };
        
        let texture = device.create_texture(&desc);
        texture.create_view(&wgpu::TextureViewDescriptor::default())
    }

    pub fn get_size(&self) -> PhysicalSize<u32> {
        self.size
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.surface_config.width = new_size.width;
            self.surface_config.height = new_size.height;
            self.surface.configure(&self.device, &self.surface_config);
            self.depth_texture_view = Self::create_depth_texture(&self.device, &self.surface_config);
            
            self.camera = Camera::new(
                self.camera.eye(),
                self.camera.yaw(),
                self.camera.pitch(),
                self.camera.up(),
                new_size.width as f32 / new_size.height as f32,
                65.0,
                0.01,
                1000.0,
            );
        }
    }

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

    pub fn device_input(&mut self, event: &DeviceEvent) {
        if let DeviceEvent::MouseMotion { delta } = event {
            self.camera_controller.process_mouse(delta.0, delta.1, &mut self.camera);
            self.is_active = true;
        }
    }

    pub fn update(&mut self) {
        let now = Instant::now();
        let delta_time = (now - self.last_update_time).as_secs_f32();
        self.last_update_time = now;

        self.camera_controller.update_camera(&mut self.camera, delta_time);

        self.camera_uniform.update_view_proj(&self.camera);
        self.queue.write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(&[self.camera_uniform]));
        
        self.update_chunks();

        let mut dirty_chunks = Vec::new();
        for (pos, chunk) in &mut self.chunks {
            if chunk.is_dirty {
                dirty_chunks.push(*pos);
                chunk.is_dirty = false;
            }
        }

        for pos in dirty_chunks {
            if let Some(chunk) = self.chunks.get(&pos) {
                let (vertices, indices) = chunk.generate_mesh();
                
                self.chunk_render_data.remove(&pos);

                if !vertices.is_empty() {
                    let vertex_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some(&format!("Chunk VB {:?}", pos)),
                        contents: bytemuck::cast_slice(&vertices),
                        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                    });

                    let index_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some(&format!("Chunk IB {:?}", pos)),
                        contents: bytemuck::cast_slice(&indices),
                        usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
                    });

                    self.chunk_render_data.insert(pos, ChunkRenderData {
                        vertex_buffer,
                        index_buffer,
                        index_count: indices.len() as u32,
                    });
                }
            }
        }
    }

    fn update_chunks(&mut self) {
        let cam_pos = self.camera.eye();
        let cam_chunk_pos = (
            (cam_pos.x / CHUNK_SIZE as f32).floor() as i32,
            (cam_pos.y / CHUNK_SIZE as f32).floor() as i32,
            (cam_pos.z / CHUNK_SIZE as f32).floor() as i32,
        );

        let mut required_chunks = HashSet::new();
        for y in 0..=VERTICAL_RENDER_DISTANCE_CHUNKS {
            for x in (cam_chunk_pos.0 - RENDER_DISTANCE)..=(cam_chunk_pos.0 + RENDER_DISTANCE) {
                for z in (cam_chunk_pos.2 - RENDER_DISTANCE)..=(cam_chunk_pos.2 + RENDER_DISTANCE) {
                    required_chunks.insert((x, y, z));
                }
            }
        }

        // Unload chunks that are out of range
        self.chunks.retain(|pos, _| required_chunks.contains(pos));
        self.chunk_render_data.retain(|pos, _| required_chunks.contains(pos));

        // Load new chunks
        for pos in required_chunks {
            self.chunks.entry(pos).or_insert_with(|| {
                let mut new_chunk = Chunk::new(pos);
                new_chunk.generate_terrain(&self.noise);
                new_chunk
            });
        }
    }

    pub fn should_continue_rendering(&self) -> bool {
        self.is_active
    }

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
                            r: 0.3,
                            g: 0.7,
                            b: 1.0,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
            
            for (_pos, data) in &self.chunk_render_data {
                render_pass.set_vertex_buffer(0, data.vertex_buffer.slice(..));
                render_pass.set_index_buffer(data.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                render_pass.draw_indexed(0..data.index_count, 0, 0..1);
            }
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}