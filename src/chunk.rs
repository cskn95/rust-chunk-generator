use crate::voxel::VoxelType;
use crate::vertex::Vertex;
use noise::{NoiseFn, Perlin};

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

    pub fn generate_terrain(&mut self, noise: &Perlin) {
        for x in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                let world_x = (self.world_pos.0 * CHUNK_SIZE as i32 + x as i32) as f64;
                let world_z = (self.world_pos.2 * CHUNK_SIZE as i32 + z as i32) as f64;

                // Gürültü fonksiyonunu kullanarak yükseklik haritası oluştur
                let height_val = noise.get([world_x * 0.01, world_z * 0.01]);
                
                // Yüksekliği ölçeklendir ve chunk yüksekliğine ayarla
                let height = (height_val * 10.0 + 20.0).round() as i32;
                
                let chunk_world_y_start = self.world_pos.1 * CHUNK_SIZE as i32;

                for y in 0..CHUNK_SIZE {
                    let world_y = chunk_world_y_start + y as i32;
                    
                    if world_y < height {
                        self.voxels[x][y][z] = VoxelType::Stone;
                    } else if world_y == height {
                        self.voxels[x][y][z] = VoxelType::Grass;
                    } else {
                        self.voxels[x][y][z] = VoxelType::Air;
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

                    // Face culling optimizasyonu - sadece görünen yüzleri render et
                    
                    // +X yüzü (sağ) - chunk sınırında veya komşu solid değilse render et
                    if x + 1 >= CHUNK_SIZE || !self.voxels[x + 1][y][z].is_solid() {
                        self.add_quad_face(
                            &mut vertices, &mut indices, &mut vertex_count,
                            pos, 0, voxel_color
                        );
                    }
                    
                    // -X yüzü (sol) - chunk sınırında veya komşu solid değilse render et
                    if x == 0 || !self.voxels[x - 1][y][z].is_solid() {
                        self.add_quad_face(
                            &mut vertices, &mut indices, &mut vertex_count,
                            pos, 1, voxel_color
                        );
                    }
                    
                    // +Y yüzü (üst) - chunk sınırında veya komşu solid değilse render et
                    if y + 1 >= CHUNK_SIZE || !self.voxels[x][y + 1][z].is_solid() {
                        self.add_quad_face(
                            &mut vertices, &mut indices, &mut vertex_count,
                            pos, 2, voxel_color
                        );
                    }
                    
                    // -Y yüzü (alt) - chunk sınırında veya komşu solid değilse render et
                    if y == 0 || !self.voxels[x][y - 1][z].is_solid() {
                        self.add_quad_face(
                            &mut vertices, &mut indices, &mut vertex_count,
                            pos, 3, voxel_color
                        );
                    }
                    
                    // +Z yüzü (ön) - chunk sınırında veya komşu solid değilse render et
                    if z + 1 >= CHUNK_SIZE || !self.voxels[x][y][z + 1].is_solid() {
                        self.add_quad_face(
                            &mut vertices, &mut indices, &mut vertex_count,
                            pos, 4, voxel_color
                        );
                    }
                    
                    // -Z yüzü (arka) - chunk sınırında veya komşu solid değilse render et
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
        
        // Standart cube face vertex tanımları - CCW winding order
        let face_vertices = match face {
            0 => [ // +X (sağ yüz) 
                [center_pos[0] + HALF, center_pos[1] - HALF, center_pos[2] + HALF], // 0
                [center_pos[0] + HALF, center_pos[1] + HALF, center_pos[2] + HALF], // 1
                [center_pos[0] + HALF, center_pos[1] + HALF, center_pos[2] - HALF], // 2
                [center_pos[0] + HALF, center_pos[1] - HALF, center_pos[2] - HALF], // 3
            ],
            1 => [ // -X (sol yüz)
                [center_pos[0] - HALF, center_pos[1] - HALF, center_pos[2] - HALF], // 0
                [center_pos[0] - HALF, center_pos[1] + HALF, center_pos[2] - HALF], // 1
                [center_pos[0] - HALF, center_pos[1] + HALF, center_pos[2] + HALF], // 2
                [center_pos[0] - HALF, center_pos[1] - HALF, center_pos[2] + HALF], // 3
            ],
            2 => [ // +Y (üst yüz)
                [center_pos[0] - HALF, center_pos[1] + HALF, center_pos[2] + HALF], // 0
                [center_pos[0] + HALF, center_pos[1] + HALF, center_pos[2] + HALF], // 1
                [center_pos[0] + HALF, center_pos[1] + HALF, center_pos[2] - HALF], // 2
                [center_pos[0] - HALF, center_pos[1] + HALF, center_pos[2] - HALF], // 3
            ],
            3 => [ // -Y (alt yüz)
                [center_pos[0] - HALF, center_pos[1] - HALF, center_pos[2] - HALF], // 0
                [center_pos[0] + HALF, center_pos[1] - HALF, center_pos[2] - HALF], // 1
                [center_pos[0] + HALF, center_pos[1] - HALF, center_pos[2] + HALF], // 2
                [center_pos[0] - HALF, center_pos[1] - HALF, center_pos[2] + HALF], // 3
            ],
            4 => [ // +Z (ön yüz)
                [center_pos[0] - HALF, center_pos[1] - HALF, center_pos[2] + HALF], // 0
                [center_pos[0] - HALF, center_pos[1] + HALF, center_pos[2] + HALF], // 1
                [center_pos[0] + HALF, center_pos[1] + HALF, center_pos[2] + HALF], // 2
                [center_pos[0] + HALF, center_pos[1] - HALF, center_pos[2] + HALF], // 3
            ],
            5 => [ // -Z (arka yüz)
                [center_pos[0] + HALF, center_pos[1] - HALF, center_pos[2] - HALF], // 0
                [center_pos[0] + HALF, center_pos[1] + HALF, center_pos[2] - HALF], // 1
                [center_pos[0] - HALF, center_pos[1] + HALF, center_pos[2] - HALF], // 2
                [center_pos[0] - HALF, center_pos[1] - HALF, center_pos[2] - HALF], // 3
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

        // Quad için triangle indexleri - CCW winding order
        let base = *vertex_count;
        indices.extend_from_slice(&[
            base, base + 1, base + 2,      // İlk üçgen: 0->1->2
            base, base + 2, base + 3,      // İkinci üçgen: 0->2->3
        ]);
        
        *vertex_count += 4;
    }
}