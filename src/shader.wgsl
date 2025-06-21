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
    @location(1) world_position: vec3<f32>,       // 3D dünya pozisyonu (depth shading için)
    @location(2) depth: f32,                      // Derinlik bilgisi (fog effect için)
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
    
    // Dünya pozisyonunu da aktar (depth-based shading için)
    out.world_position = model.position;
    
    // Derinlik bilgisini hesapla (0.0 = en yakın, 1.0 = en uzak)
    out.depth = out.clip_position.z / out.clip_position.w;
    
    return out;
}

// Fragment shader - her pixel için çağrılır
// Vertex'lerden interpolate edilmiş renk değerini kullanarak pixel rengini belirler
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Derinlik bazlı renk modifikasyonu (uzak objeler daha koyu)
    var depth_factor = 1.0 - (in.depth * 0.1);  // Depth fog effect
    depth_factor = max(depth_factor, 0.3);       // Minimum görünürlük
    
    // Vertex'lerden gelen renk bilgisini depth factor ile çarp
    var final_color = in.color * depth_factor;
    
    // Depth based ambient lighting simulation
    var ambient_strength = 0.1;
    var ambient_light = vec3<f32>(0.3, 0.3, 0.4);  // Hafif mavi ambient
    
    // Y pozisyonuna göre basit lighting (yüksekte daha aydınlık)
    var height_factor = (in.world_position.y + 16.0) / 32.0;
    height_factor = clamp(height_factor, 0.0, 1.0);
    
    // Final renk hesaplama
    final_color = final_color * (ambient_strength + (1.0 - ambient_strength) * height_factor);
    final_color = final_color + ambient_light * ambient_strength;
    
    // Alpha değerini 1.0 yaparak opaque (şeffaf olmayan) render
    return vec4<f32>(final_color, 1.0);
}