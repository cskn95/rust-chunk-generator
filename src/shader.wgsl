// Kamera uniform yapısı - CPU'dan GPU'ya gönderilen kamera verileri
struct CameraUniform {
    view_proj: mat4x4<f32>,  // View-Projection matrisi (3D dünyayı 2D ekrana dönüştürür)
};

// Bind group 0, binding 0'da bulunan kamera uniform'una erişim
@group(0) @binding(0)
var<uniform> camera: CameraUniform;

// Vertex shader'a gelen giriş verileri - artık renk bilgisi de var
struct VertexInput {
    @location(0) position: vec3<f32>,  // Vertex pozisyonu (x, y, z)
    @location(1) color: vec3<f32>,     // Vertex rengi (r, g, b) - yeni eklendi
};

// Vertex shader'dan fragment shader'a gönderilen veriler
struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,  // Ekran koordinatlarındaki pozisyon
    @location(0) color: vec3<f32>,                // Renk bilgisi fragment'a aktarılır
};

// Vertex shader - her vertex için çağrılır
// 3D dünya koordinatlarını ekran koordinatlarına dönüştürür ve renk bilgisini aktarır
@vertex
fn vs_main(model: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    
    // Vertex pozisyonunu kamera matrisi ile çarparak ekran koordinatlarına dönüştür
    out.clip_position = camera.view_proj * vec4<f32>(model.position, 1.0);
    
    // Renk bilgisini fragment shader'a aktar
    // Bu sayede her voksel türü farklı renkte görünebilir
    out.color = model.color;
    
    return out;
}

// Fragment shader - her pixel için çağrılır
// Vertex'lerden interpolate edilmiş renk değerini kullanarak pixel rengini belirler
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Vertex'lerden gelen renk bilgisini kullan
    // Alpha değerini 1.0 yaparak opaque (şeffaf olmayan) render
    return vec4<f32>(in.color, 1.0);
}