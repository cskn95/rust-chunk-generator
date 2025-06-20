/// Geliştirilmiş vertex yapısı
/// 3D uzayda bir noktayı ve rengini temsil eder
/// Voksel motoru için position ve color bilgilerini saklar
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],    // X, Y, Z koordinatları (chunk sistemi için public)
    pub color: [f32; 3],       // R, G, B renk değerleri (0.0-1.0 aralığında)
}

/// Test amaçlı basit küp vertex'leri - artık kullanılmayacak
/// Chunk sistemi kendi mesh'ini oluşturacak, ama test için tutuyoruz
pub const _VERTICES: &[Vertex] = &[
    // Ön yüz (Z = 0.5) - yeşil renk
    Vertex { position: [-0.5, -0.5,  0.5], color: [0.2, 0.8, 0.2] },
    Vertex { position: [ 0.5, -0.5,  0.5], color: [0.2, 0.8, 0.2] },
    Vertex { position: [ 0.5,  0.5,  0.5], color: [0.2, 0.8, 0.2] },
    Vertex { position: [-0.5,  0.5,  0.5], color: [0.2, 0.8, 0.2] },
    
    // Arka yüz (Z = -0.5) - mavi renk
    Vertex { position: [-0.5, -0.5, -0.5], color: [0.2, 0.2, 0.8] },
    Vertex { position: [-0.5,  0.5, -0.5], color: [0.2, 0.2, 0.8] },
    Vertex { position: [ 0.5,  0.5, -0.5], color: [0.2, 0.2, 0.8] },
    Vertex { position: [ 0.5, -0.5, -0.5], color: [0.2, 0.2, 0.8] },
    
    // Üst yüz (Y = 0.5) - kırmızı renk
    Vertex { position: [-0.5,  0.5, -0.5], color: [0.8, 0.2, 0.2] },
    Vertex { position: [-0.5,  0.5,  0.5], color: [0.8, 0.2, 0.2] },
    Vertex { position: [ 0.5,  0.5,  0.5], color: [0.8, 0.2, 0.2] },
    Vertex { position: [ 0.5,  0.5, -0.5], color: [0.8, 0.2, 0.2] },
    
    // Alt yüz (Y = -0.5) - sarı renk
    Vertex { position: [-0.5, -0.5, -0.5], color: [0.8, 0.8, 0.2] },
    Vertex { position: [ 0.5, -0.5, -0.5], color: [0.8, 0.8, 0.2] },
    Vertex { position: [ 0.5, -0.5,  0.5], color: [0.8, 0.8, 0.2] },
    Vertex { position: [-0.5, -0.5,  0.5], color: [0.8, 0.8, 0.2] },
    
    // Sağ yüz (X = 0.5) - mor renk
    Vertex { position: [ 0.5, -0.5, -0.5], color: [0.8, 0.2, 0.8] },
    Vertex { position: [ 0.5,  0.5, -0.5], color: [0.8, 0.2, 0.8] },
    Vertex { position: [ 0.5,  0.5,  0.5], color: [0.8, 0.2, 0.8] },
    Vertex { position: [ 0.5, -0.5,  0.5], color: [0.8, 0.2, 0.8] },
    
    // Sol yüz (X = -0.5) - turkuaz renk
    Vertex { position: [-0.5, -0.5, -0.5], color: [0.2, 0.8, 0.8] },
    Vertex { position: [-0.5, -0.5,  0.5], color: [0.2, 0.8, 0.8] },
    Vertex { position: [-0.5,  0.5,  0.5], color: [0.2, 0.8, 0.8] },
    Vertex { position: [-0.5,  0.5, -0.5], color: [0.2, 0.8, 0.8] },
];

/// Test küpü indeksleri - chunk sistemi için artık kullanılmayacak
/// Ama geriye uyumluluk için tutuyoruz
pub const _INDICES: &[u16] = &[
    0,  1,  2,   2,  3,  0,   // Ön yüz
    4,  5,  6,   6,  7,  4,   // Arka yüz
    8,  9,  10,  10, 11, 8,   // Üst yüz
    12, 13, 14,  14, 15, 12,  // Alt yüz
    16, 17, 18,  18, 19, 16,  // Sağ yüz
    20, 21, 22,  22, 23, 20,  // Sol yüz
];

impl Vertex {
    /// Vertex buffer layout tanımını döndürür
    /// GPU'ya vertex verilerinin nasıl okunacağını söyler
    /// Artık position ve color attribute'ları içerir
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem::size_of;
        
        wgpu::VertexBufferLayout {
            // Her vertex'in byte cinsinden boyutu (position + color = 6 float)
            array_stride: size_of::<Self>() as wgpu::BufferAddress,
            // Vertex başına veri okuma modu
            step_mode: wgpu::VertexStepMode::Vertex,
            // Vertex attributeları - şimdi position ve color var
            attributes: &[
                // Position attribute (location 0)
                wgpu::VertexAttribute {
                    offset: 0,                              // Vertex başından offset
                    shader_location: 0,                     // Shader'daki @location(0)
                    format: wgpu::VertexFormat::Float32x3,  // 3 float (x, y, z)
                },
                // Color attribute (location 1)
                wgpu::VertexAttribute {
                    offset: size_of::<[f32; 3]>() as wgpu::BufferAddress, // Position'dan sonra
                    shader_location: 1,                     // Shader'daki @location(1)
                    format: wgpu::VertexFormat::Float32x3,  // 3 float (r, g, b)
                },
            ]
        }
    }
}