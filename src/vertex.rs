#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    position: [f32; 3],
}

// Simple cube vertices (1x1x1 unit cube)
pub const VERTICES: &[Vertex] = &[
    // Front face
    Vertex { position: [-0.5, -0.5,  0.5] },
    Vertex { position: [ 0.5, -0.5,  0.5] },
    Vertex { position: [ 0.5,  0.5,  0.5] },
    Vertex { position: [-0.5,  0.5,  0.5] },
    
    // Back face
    Vertex { position: [-0.5, -0.5, -0.5] },
    Vertex { position: [-0.5,  0.5, -0.5] },
    Vertex { position: [ 0.5,  0.5, -0.5] },
    Vertex { position: [ 0.5, -0.5, -0.5] },
    
    // Top face
    Vertex { position: [-0.5,  0.5, -0.5] },
    Vertex { position: [-0.5,  0.5,  0.5] },
    Vertex { position: [ 0.5,  0.5,  0.5] },
    Vertex { position: [ 0.5,  0.5, -0.5] },
    
    // Bottom face
    Vertex { position: [-0.5, -0.5, -0.5] },
    Vertex { position: [ 0.5, -0.5, -0.5] },
    Vertex { position: [ 0.5, -0.5,  0.5] },
    Vertex { position: [-0.5, -0.5,  0.5] },
    
    // Right face
    Vertex { position: [ 0.5, -0.5, -0.5] },
    Vertex { position: [ 0.5,  0.5, -0.5] },
    Vertex { position: [ 0.5,  0.5,  0.5] },
    Vertex { position: [ 0.5, -0.5,  0.5] },
    
    // Left face
    Vertex { position: [-0.5, -0.5, -0.5] },
    Vertex { position: [-0.5, -0.5,  0.5] },
    Vertex { position: [-0.5,  0.5,  0.5] },
    Vertex { position: [-0.5,  0.5, -0.5] },
];

// Cube indices (2 triangles per face, 6 faces = 12 triangles)
pub const INDICES: &[u16] = &[
    0,  1,  2,   2,  3,  0,   // Front
    4,  5,  6,   6,  7,  4,   // Back
    8,  9,  10,  10, 11, 8,   // Top
    12, 13, 14,  14, 15, 12,  // Bottom
    16, 17, 18,  18, 19, 16,  // Right
    20, 21, 22,  22, 23, 20,  // Left
];

impl Vertex {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem::size_of;
        
        wgpu::VertexBufferLayout {
            array_stride: size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ]
        }
    }
}