use crate::voxel::VoxelType;
use crate::vertex::Vertex;

pub const CHUNK_SIZE: usize = 16;

#[derive(Clone)]
pub struct Chunk {
    pub voxels: [[[VoxelType; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE],
    pub world_pos: (i32, i32, i32),
    pub is_dirty: bool,
}

impl Chunk {
    pub fn new(world_pos: (i32, i32, i32)) -> Self {
        Self {
            voxels: [[[VoxelType::Air; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE],
            world_pos,
            is_dirty: true,
        }
    }

    pub fn generate_test_terrain(&mut self) {
        for x in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                for z in 0..CHUNK_SIZE {
                    if y < CHUNK_SIZE / 4 {
                        self.voxels[x][y][z] = VoxelType::Stone;
                    } else if y < CHUNK_SIZE / 2 {
                        self.voxels[x][y][z] = VoxelType::Dirt;
                    } else if y == CHUNK_SIZE / 2 {
                        self.voxels[x][y][z] = VoxelType::Grass;
                    }
                }
            }
        }
        self.is_dirty = true;
    }

    pub fn get_voxel(&self, x: usize, y: usize, z: usize) -> Option<VoxelType> {
        if x < CHUNK_SIZE && y < CHUNK_SIZE && z < CHUNK_SIZE {
            Some(self.voxels[x][y][z])
        } else {
            None
        }
    }

    pub fn set_voxel(&mut self, x: usize, y: usize, z: usize, voxel_type: VoxelType) {
        if x < CHUNK_SIZE && y < CHUNK_SIZE && z < CHUNK_SIZE {
            if self.voxels[x][y][z] != voxel_type {
                self.voxels[x][y][z] = voxel_type;
                self.is_dirty = true;
            }
        }
    }

    pub fn generate_mesh(&self) -> (Vec<Vertex>, Vec<u16>) {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        let mut vertex_count = 0u16;

        // Her voksel için
        for x in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                for z in 0..CHUNK_SIZE {
                    let voxel = self.voxels[x][y][z];
                    
                    // Sadece solid vokselleri render et
                    if !voxel.is_solid() {
                        continue;
                    }

                    let voxel_color = voxel.get_color();
                    let world_x = (self.world_pos.0 * CHUNK_SIZE as i32) + x as i32;
                    let world_y = (self.world_pos.1 * CHUNK_SIZE as i32) + y as i32;
                    let world_z = (self.world_pos.2 * CHUNK_SIZE as i32) + z as i32;
                    
                    let pos = [world_x as f32, world_y as f32, world_z as f32];

                    // Her yüz için komşu kontrolü yap
                    // +X yüzü (sağ)
                    if x + 1 >= CHUNK_SIZE || !self.voxels[x + 1][y][z].is_solid() {
                        self.add_quad_face(
                            &mut vertices, &mut indices, &mut vertex_count,
                            pos, 0, voxel_color
                        );
                    }
                    
                    // -X yüzü (sol)
                    if x == 0 || !self.voxels[x - 1][y][z].is_solid() {
                        self.add_quad_face(
                            &mut vertices, &mut indices, &mut vertex_count,
                            pos, 1, voxel_color
                        );
                    }
                    
                    // +Y yüzü (üst)
                    if y + 1 >= CHUNK_SIZE || !self.voxels[x][y + 1][z].is_solid() {
                        self.add_quad_face(
                            &mut vertices, &mut indices, &mut vertex_count,
                            pos, 2, voxel_color
                        );
                    }
                    
                    // -Y yüzü (alt)
                    if y == 0 || !self.voxels[x][y - 1][z].is_solid() {
                        self.add_quad_face(
                            &mut vertices, &mut indices, &mut vertex_count,
                            pos, 3, voxel_color
                        );
                    }
                    
                    // +Z yüzü (ön)
                    if z + 1 >= CHUNK_SIZE || !self.voxels[x][y][z + 1].is_solid() {
                        self.add_quad_face(
                            &mut vertices, &mut indices, &mut vertex_count,
                            pos, 4, voxel_color
                        );
                    }
                    
                    // -Z yüzü (arka)
                    if z == 0 || !self.voxels[x][y][z - 1].is_solid() {
                        self.add_quad_face(
                            &mut vertices, &mut indices, &mut vertex_count,
                            pos, 5, voxel_color
                        );
                    }
                }
            }
        }

        (vertices, indices)
    }

    // Belirtilen yüz için 4 vertex ve 6 index ekler
    fn add_quad_face(
        &self,
        vertices: &mut Vec<Vertex>,
        indices: &mut Vec<u16>,
        vertex_count: &mut u16,
        center_pos: [f32; 3],
        face: u8, // 0: +X, 1: -X, 2: +Y, 3: -Y, 4: +Z, 5: -Z
        color: [f32; 3]
    ) {
        const HALF: f32 = 0.5;
        
        // Her yüz için 4 köşe pozisyonu
        let face_vertices = match face {
            0 => [ // +X (sağ)
                [center_pos[0] + HALF, center_pos[1] - HALF, center_pos[2] - HALF],
                [center_pos[0] + HALF, center_pos[1] - HALF, center_pos[2] + HALF],
                [center_pos[0] + HALF, center_pos[1] + HALF, center_pos[2] + HALF],
                [center_pos[0] + HALF, center_pos[1] + HALF, center_pos[2] - HALF],
            ],
            1 => [ // -X (sol)
                [center_pos[0] - HALF, center_pos[1] - HALF, center_pos[2] + HALF],
                [center_pos[0] - HALF, center_pos[1] - HALF, center_pos[2] - HALF],
                [center_pos[0] - HALF, center_pos[1] + HALF, center_pos[2] - HALF],
                [center_pos[0] - HALF, center_pos[1] + HALF, center_pos[2] + HALF],
            ],
            2 => [ // +Y (üst)
                [center_pos[0] - HALF, center_pos[1] + HALF, center_pos[2] - HALF],
                [center_pos[0] + HALF, center_pos[1] + HALF, center_pos[2] - HALF],
                [center_pos[0] + HALF, center_pos[1] + HALF, center_pos[2] + HALF],
                [center_pos[0] - HALF, center_pos[1] + HALF, center_pos[2] + HALF],
            ],
            3 => [ // -Y (alt)
                [center_pos[0] - HALF, center_pos[1] - HALF, center_pos[2] + HALF],
                [center_pos[0] + HALF, center_pos[1] - HALF, center_pos[2] + HALF],
                [center_pos[0] + HALF, center_pos[1] - HALF, center_pos[2] - HALF],
                [center_pos[0] - HALF, center_pos[1] - HALF, center_pos[2] - HALF],
            ],
            4 => [ // +Z (ön)
                [center_pos[0] - HALF, center_pos[1] - HALF, center_pos[2] + HALF],
                [center_pos[0] + HALF, center_pos[1] - HALF, center_pos[2] + HALF],
                [center_pos[0] + HALF, center_pos[1] + HALF, center_pos[2] + HALF],
                [center_pos[0] - HALF, center_pos[1] + HALF, center_pos[2] + HALF],
            ],
            5 => [ // -Z (arka)
                [center_pos[0] + HALF, center_pos[1] - HALF, center_pos[2] - HALF],
                [center_pos[0] - HALF, center_pos[1] - HALF, center_pos[2] - HALF],
                [center_pos[0] - HALF, center_pos[1] + HALF, center_pos[2] - HALF],
                [center_pos[0] + HALF, center_pos[1] + HALF, center_pos[2] - HALF],
            ],
            _ => return, // Geçersiz yüz
        };

        // 4 vertex ekle
        for position in face_vertices.iter() {
            vertices.push(Vertex {
                position: *position,
                color,
            });
        }

        // 2 üçgen için index'ler (counter-clockwise)
        let base = *vertex_count;
        indices.extend_from_slice(&[
            base, base + 1, base + 2,
            base + 2, base + 3, base,
        ]);
        
        *vertex_count += 4;
    }
}