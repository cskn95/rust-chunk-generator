use cgmath::InnerSpace;

/// OpenGL koordinat sistemini WGPU koordinat sistemine dönüştüren matris
/// OpenGL'de Y ekseni yukarı, Z ekseni gözlemciye doğru
/// WGPU'da Y ekseni yukarı, Z ekseni gözlemciden uzağa
#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::from_cols(
    cgmath::Vector4::new(1.0, 0.0, 0.0, 0.0),
    cgmath::Vector4::new(0.0, 1.0, 0.0, 0.0),
    cgmath::Vector4::new(0.0, 0.0, 0.5, 0.0),
    cgmath::Vector4::new(0.0, 0.0, 0.5, 1.0),
);

/// Perspektif kamera yapısı
/// 3D sahneyi 2D ekrana projekte etmek için gerekli tüm parametreleri içerir
pub struct Camera {
    eye: cgmath::Point3<f32>,        // Kameranın 3D uzaydaki konumu
    yaw: f32,                        // Yatay rotasyon (Y ekseni etrafında)
    pitch: f32,                      // Dikey rotasyon (X ekseni etrafında)
    up: cgmath::Vector3<f32>,        // Yukarı yön vektörü (genellikle Y ekseni)
    aspect: f32,                     // Görüntü en/boy oranı
    fovy: f32,                       // Görüş alanı açısı (derece)
    znear: f32,                      // Yakın düzlem mesafesi
    zfar: f32,                       // Uzak düzlem mesafesi
}

/// GPU'ya gönderilecek kamera uniform verileri
/// Shader'da kullanılmak üzere view-projection matrisini saklar
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    view_proj: [[f32; 4]; 4],
}

impl Camera {
    /// Yeni kamera oluşturur
    pub fn new(
        eye: cgmath::Point3<f32>,        // Kamera pozisyonu
        yaw: f32,                        // Başlangıç yatay rotasyonu
        pitch: f32,                      // Başlangıç dikey rotasyonu
        up: cgmath::Vector3<f32>,        // Yukarı vektörü
        aspect: f32,                     // Görüntü en/boy oranı
        fovy: f32,                       // Görüş alanı açısı
        znear: f32,                      // Yakın düzlem
        zfar: f32,                       // Uzak düzlem
    ) -> Self {
        Self {
            eye,
            yaw,
            pitch,
            up,
            aspect,
            fovy,
            znear,
            zfar,
        }
    }

    /// Kamera pozisyonunu döndürür
    pub fn eye(&self) -> cgmath::Point3<f32> {
        self.eye
    }

    /// Kamera pozisyonunu ayarlar
    pub fn set_eye(&mut self, eye: cgmath::Point3<f32>) {
        self.eye = eye;
    }

    /// Yukarı vektörünü döndürür
    pub fn up(&self) -> cgmath::Vector3<f32> {
        self.up
    }

    /// Yatay rotasyonu (yaw) döndürür
    pub fn yaw(&self) -> f32 {
        self.yaw
    }

    /// Yatay rotasyonu (yaw) ayarlar
    pub fn set_yaw(&mut self, yaw: f32) {
        self.yaw = yaw;
    }

    /// Dikey rotasyonu (pitch) döndürür
    pub fn pitch(&self) -> f32 {
        self.pitch
    }

    /// Dikey rotasyonu (pitch) ayarlar
    /// Aşırı dönüşü önlemek için -90° ile +90° arasında sınırlar
    pub fn set_pitch(&mut self, pitch: f32) {
        self.pitch = pitch.clamp(-std::f32::consts::FRAC_PI_2 + 0.1, std::f32::consts::FRAC_PI_2 - 0.1);
    }

    /// Kameranın baktığı yönü (ileri vektörü) hesaplar
    /// Yaw ve pitch açılarından sferik koordinat sistemini kullanarak hesaplar
    pub fn forward(&self) -> cgmath::Vector3<f32> {
        cgmath::Vector3::new(
            self.yaw.cos() * self.pitch.cos(),   // X bileşeni
            self.pitch.sin(),                    // Y bileşeni (dikey)
            self.yaw.sin() * self.pitch.cos(),   // Z bileşeni
        )
    }

    /// Kameranın sağ yönünü hesaplar
    /// İleri vektör ile yukarı vektörün çapraz çarpımından elde edilir
    pub fn right(&self) -> cgmath::Vector3<f32> {
        self.forward().cross(self.up).normalize()
    }

    /// View-Projection matrisini oluşturur
    /// Bu matris 3D dünya koordinatlarını 2D ekran koordinatlarına dönüştürür
    pub fn build_view_projection_matrix(&self) -> cgmath::Matrix4<f32> {
        // İleri vektörünü yaw ve pitch değerlerine göre hesapla
        let forward = self.forward();

        // Hedef noktayı ileri vektörünü kullanarak hesapla
        let target = self.eye + forward;

        // View matrisini oluştur (dünya koordinatlarını kamera koordinatlarına dönüştürür)
        let view = cgmath::Matrix4::look_at_rh(self.eye, target, self.up);
        
        // Projection matrisini oluştur (kamera koordinatlarını ekran koordinatlarına dönüştürür)
        let proj = cgmath::perspective(cgmath::Deg(self.fovy), self.aspect, self.znear, self.zfar);

        // Koordinat sistemi dönüşümü ile birlikte final matrisi döndür
        return OPENGL_TO_WGPU_MATRIX * proj * view;
    }
}

impl CameraUniform {
    /// Birim matris ile yeni CameraUniform oluşturur
    pub fn new() -> Self {
        use cgmath::SquareMatrix;
        Self {
            view_proj: cgmath::Matrix4::identity().into(),
        }
    }

    /// Kamera nesnesinden view-projection matrisini günceller
    /// Her frame'de çağrılarak GPU'daki uniform buffer'ı günceller
    pub fn update_view_proj(&mut self, camera: &Camera) {
        self.view_proj = camera.build_view_projection_matrix().into();
    }
}