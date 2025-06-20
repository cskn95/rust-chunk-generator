/// Voksel türlerini tanımlayan enum
/// Her voksel türü farklı bir renk ve özellik taşıyabilir
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum VoxelType {
    Air,      // Boş alan - render edilmez
    Stone,    // Taş blok - gri renk
    Grass,    // Çim blok - yeşil renk
    Dirt,     // Toprak blok - kahverengi
}

impl VoxelType {
    /// Voksel türünün solid (katı) olup olmadığını döndürür
    /// Air dışındaki tüm vokseller solid kabul edilir
    pub fn is_solid(&self) -> bool {
        match self {
            VoxelType::Air => false,
            _ => true,
        }
    }

    /// Voksel türüne göre renk döndürür
    /// Bu renkler shader'da kullanılacak
    pub fn get_color(&self) -> [f32; 3] {
        match self {
            VoxelType::Air => [0.0, 0.0, 0.0],      // Air render edilmeyeceği için önemli değil
            VoxelType::Stone => [0.5, 0.5, 0.5],    // Gri
            VoxelType::Grass => [0.2, 0.8, 0.2],    // Yeşil
            VoxelType::Dirt => [0.6, 0.4, 0.2],     // Kahverengi
        }
    }
}

impl Default for VoxelType {
    fn default() -> Self {
        VoxelType::Air
    }
}