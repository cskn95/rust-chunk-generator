#[derive(Copy, Clone, Debug, PartialEq)]
pub enum VoxelType {
    Air,
    Stone,
    Grass,
    Dirt,
}

impl VoxelType {
    pub fn is_solid(&self) -> bool {
        match self {
            VoxelType::Air => false,
            _ => true,
        }
    }

    pub fn get_color(&self) -> [f32; 3] {
        match self {
            VoxelType::Air => [0.0, 0.0, 0.0],
            VoxelType::Stone => [0.5, 0.5, 0.5],
            VoxelType::Grass => [0.2, 0.8, 0.2],
            VoxelType::Dirt => [0.6, 0.4, 0.2],
        }
    }
}

impl Default for VoxelType {
    fn default() -> Self {
        VoxelType::Air
    }
}