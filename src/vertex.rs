/// Temel vertex yapısı
/// 3D uzayda bir noktayı temsil eder
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    position: [f32; 3],    // X, Y, Z koordinatları
}

/// Basit küp vertex'leri (1x1x1 birim küp)
/// Her yüz için 4 vertex tanımlanmıştır
pub const VERTICES: &[Vertex] = &[
    // Ön yüz (Z = 0.5)
    Vertex { position: [-0.5, -0.5,  0.5] },  // Sol alt
    Vertex { position: [ 0.5, -0.5,  0.5] },  // Sağ alt
    Vertex { position: [ 0.5,  0.5,  0.5] },  // Sağ üst
    Vertex { position: [-0.5,  0.5,  0.5] },  // Sol üst
    
    // Arka yüz (Z = -0.5)
    Vertex { position: [-0.5, -0.5, -0.5] },  // Sol alt
    Vertex { position: [-0.5,  0.5, -0.5] },  // Sol üst
    Vertex { position: [ 0.5,  0.5, -0.5] },  // Sağ üst
    Vertex { position: [ 0.5, -0.5, -0.5] },  // Sağ alt
    
    // Üst yüz (Y = 0.5)
    Vertex { position: [-0.5,  0.5, -0.5] },  // Sol arka
    Vertex { position: [-0.5,  0.5,  0.5] },  // Sol ön
    Vertex { position: [ 0.5,  0.5,  0.5] },  // Sağ ön
    Vertex { position: [ 0.5,  0.5, -0.5] },  // Sağ arka
    
    // Alt yüz (Y = -0.5)
    Vertex { position: [-0.5, -0.5, -0.5] },  // Sol arka
    Vertex { position: [ 0.5, -0.5, -0.5] },  // Sağ arka
    Vertex { position: [ 0.5, -0.5,  0.5] },  // Sağ ön
    Vertex { position: [-0.5, -0.5,  0.5] },  // Sol ön
    
    // Sağ yüz (X = 0.5)
    Vertex { position: [ 0.5, -0.5, -0.5] },  // Alt arka
    Vertex { position: [ 0.5,  0.5, -0.5] },  // Üst arka
    Vertex { position: [ 0.5,  0.5,  0.5] },  // Üst ön
    Vertex { position: [ 0.5, -0.5,  0.5] },  // Alt ön
    
    // Sol yüz (X = -0.5)
    Vertex { position: [-0.5, -0.5, -0.5] },  // Alt arka
    Vertex { position: [-0.5, -0.5,  0.5] },  // Alt ön
    Vertex { position: [-0.5,  0.5,  0.5] },  // Üst ön
    Vertex { position: [-0.5,  0.5, -0.5] },  // Üst arka
];

/// Küp indeksleri - her yüz için 2 üçgen (6 yüz = 12 üçgen)
/// Her üçgen 3 vertex indeksi ile tanımlanır
/// Vertex'ler saat yönünün tersine (counter-clockwise) sıralanmıştır
pub const INDICES: &[u16] = &[
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
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem::size_of;
        
        wgpu::VertexBufferLayout {
            // Her vertex'in byte cinsinden boyutu
            array_stride: size_of::<Self>() as wgpu::BufferAddress,
            // Vertex başına veri okuma modu
            step_mode: wgpu::VertexStepMode::Vertex,
            // Vertex attributeları (position, normal, texture coordinates vb.)
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,                              // Vertex içindeki offset
                    shader_location: 0,                     // Shader'daki location
                    format: wgpu::VertexFormat::Float32x3,  // 3 float (x, y, z)
                },
            ]
        }
    }
}