// Kamera uniform yapısı - CPU'dan GPU'ya gönderilen kamera verileri
struct CameraUniform {
    view_proj: mat4x4<f32>,  // View-Projection matrisi (3D dünyayı 2D ekrana dönüştürür)
};

// Bind group 0, binding 0'da bulunan kamera uniform'una erişim
@group(0) @binding(0)
var<uniform> camera: CameraUniform;

// Vertex shader'a gelen giriş verileri
struct VertexInput {
    @location(0) position: vec3<f32>,  // Vertex pozisyonu (x, y, z)
};

// Vertex shader'dan fragment shader'a gönderilen veriler
struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,  // Ekran koordinatlarındaki pozisyon
};

// Vertex shader - her vertex için çağrılır
// 3D dünya koordinatlarını ekran koordinatlarına dönüştürür
@vertex
fn vs_main(model: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    // Vertex pozisyonunu kamera matrisi ile çarparak ekran koordinatlarına dönüştür
    out.clip_position = camera.view_proj * vec4<f32>(model.position, 1.0);
    return out;
}

// Fragment shader - her pixel için çağrılır
// Pixel'in rengini belirler
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(0.2, 0.8, 0.2, 1.0); // Düz yeşil renk (R, G, B, A)
}