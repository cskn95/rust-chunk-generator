use image::GenericImageView;
use anyhow::*;

/// GPU texture ve ilgili bileşenlerini tutan yapı
/// Texture verisi, view ve sampler'ı bir arada saklar
pub struct Texture {
    #[allow(unused)]
    pub texture: wgpu::Texture,        // Ham texture verisi
    pub view: wgpu::TextureView,       // Shader'da kullanılacak view
    pub sampler: wgpu::Sampler,        // Texture örnekleme ayarları
}

impl Texture {
    /// Byte array'inden texture oluşturur
    /// Resim dosyası byte'larından direkt texture yaratır
    pub fn from_bytes(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bytes: &[u8],
        label: &str
    ) -> Result<Self> {
        // Resim dosyasını byte'lardan yükle
        let img = image::load_from_memory(bytes)?;
        Self::from_image(device, queue, &img, Some(label))
    }

    /// DynamicImage nesnesinden texture oluşturur
    /// Resim verisini GPU'ya yükler ve gerekli bileşenleri oluşturur
    pub fn from_image(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        img: &image::DynamicImage,
        label: Option<&str>
    ) -> Result<Self> {
        // Resmi RGBA formatına dönüştür (4 kanal: kırmızı, yeşil, mavi, alfa)
        let rgba = img.to_rgba8();
        let dimensions = img.dimensions();

        // Texture boyutlarını tanımla
        let size = wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: 1,  // 2D texture için 1
        };
        
        // GPU'da texture oluştur
        let texture = device.create_texture(
            &wgpu::TextureDescriptor {
                label,
                size,
                mip_level_count: 1,                                    // Mipmap yok
                sample_count: 1,                                       // Multisampling yok
                dimension: wgpu::TextureDimension::D2,                 // 2D texture
                format: wgpu::TextureFormat::Rgba8UnormSrgb,           // RGBA 8-bit sRGB
                usage: wgpu::TextureUsages::TEXTURE_BINDING |          // Shader'da kullanım
                       wgpu::TextureUsages::COPY_DST,                  // Veri kopyalama
                view_formats: &[],
            }
        );

        // Resim verisini GPU'ya kopyala
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                aspect: wgpu::TextureAspect::All,
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            &rgba,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * dimensions.0),    // 4 byte per pixel (RGBA)
                rows_per_image: Some(dimensions.1),
            },
            size,
        );

        // Texture view oluştur (shader'da kullanım için)
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        
        // Sampler oluştur (texture örnekleme ayarları)
        let sampler = device.create_sampler(
            &wgpu::SamplerDescriptor {
                address_mode_u: wgpu::AddressMode::ClampToEdge,  // U koordinatı kenar kırpma
                address_mode_v: wgpu::AddressMode::ClampToEdge,  // V koordinatı kenar kırpma
                address_mode_w: wgpu::AddressMode::ClampToEdge,  // W koordinatı kenar kırpma
                mag_filter: wgpu::FilterMode::Linear,            // Büyütme filtresi
                min_filter: wgpu::FilterMode::Nearest,           // Küçültme filtresi
                mipmap_filter: wgpu::FilterMode::Nearest,        // Mipmap filtresi
                ..Default::default()
            }
        );

        Ok(Self { texture, view, sampler })
    }

    /// Depth buffer için kullanılacak format
    pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;
    
    /// Depth texture oluşturur
    /// 3D sahne derinlik testi için gerekli
    pub fn create_depth_texture(device: &wgpu::Device, config: &wgpu::SurfaceConfiguration, label: &str) -> Self {
        // Depth texture boyutları (ekran boyutu ile aynı)
        let size = wgpu::Extent3d {
            width: config.width.max(1),     // En az 1 pixel genişlik
            height: config.height.max(1),   // En az 1 pixel yükseklik
            depth_or_array_layers: 1,
        };
        
        // Depth texture tanımı
        let desc = wgpu::TextureDescriptor {
            label: Some(label),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: Self::DEPTH_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT |  // Render hedefi olarak kullanım
                   wgpu::TextureUsages::TEXTURE_BINDING,     // Shader'da kullanım
            view_formats: &[],
        };
        
        // Depth texture oluştur
        let texture = device.create_texture(&desc);

        // Depth texture view oluştur
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        
        // Depth texture sampler oluştur
        let sampler = device.create_sampler(
            &wgpu::SamplerDescriptor {
                address_mode_u: wgpu::AddressMode::ClampToEdge,
                address_mode_v: wgpu::AddressMode::ClampToEdge,
                address_mode_w: wgpu::AddressMode::ClampToEdge,
                mag_filter: wgpu::FilterMode::Linear,
                min_filter: wgpu::FilterMode::Linear,
                mipmap_filter: wgpu::FilterMode::Nearest,
                compare: Some(wgpu::CompareFunction::LessEqual),  // Derinlik karşılaştırması
                lod_min_clamp: 0.0,
                lod_max_clamp: 100.0,
                ..Default::default()
            }
        );

        Self { texture, view, sampler }
    }
}