use crate::voxel::VoxelType;
use crate::vertex::Vertex;

/// Chunk boyutları - 16x16x16 standart Minecraft boyutu
/// Bu boyut performans ve memory arasında iyi bir denge sağlar
pub const CHUNK_SIZE: usize = 16;

/// Bir chunk'ı temsil eden yapı
/// 16x16x16 boyutunda voksel array'i içerir
#[derive(Clone)]
pub struct Chunk {
    /// 3D voksel verisi [x][y][z] şeklinde erişilir
    /// Her pozisyonda bir VoxelType saklanır
    pub voxels: [[[VoxelType; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE],
    
    /// Chunk'ın dünya koordinatlarındaki pozisyonu
    /// Her chunk CHUNK_SIZE kadar alan kaplar
    pub world_pos: (i32, i32, i32),
    
    /// Mesh'in yeniden oluşturulması gerekip gerekmediği
    /// Voksel değiştiğinde true olur
    pub is_dirty: bool,
}

impl Chunk {
    /// Belirtilen dünya pozisyonunda yeni chunk oluşturur
    /// Başlangıçta tüm vokseller Air olarak ayarlanır
    pub fn new(world_pos: (i32, i32, i32)) -> Self {
        Self {
            voxels: [[[VoxelType::Air; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE],
            world_pos,
            is_dirty: true, // Yeni chunk her zaman mesh generation gerektirir
        }
    }

    /// Test amaçlı basit bir dünya oluşturur
    /// Alt katmanları dirt/stone ile, üst katmanı grass ile doldurur
    pub fn generate_test_terrain(&mut self) {
        for x in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                for z in 0..CHUNK_SIZE {
                    // Y koordinatına göre farklı blok türleri yerleştir
                    // Alt kısımlar stone, orta kısımlar dirt, üst kısım grass
                    if y < CHUNK_SIZE / 4 {
                        self.voxels[x][y][z] = VoxelType::Stone;
                    } else if y < CHUNK_SIZE / 2 {
                        self.voxels[x][y][z] = VoxelType::Dirt;
                    } else if y == CHUNK_SIZE / 2 {
                        self.voxels[x][y][z] = VoxelType::Grass;
                    }
                    // Geri kalan üst kısım Air olarak kalır (default)
                }
            }
        }
        self.is_dirty = true;
    }

    /// Belirtilen yerel koordinatlardaki vokseli döndürür
    /// Koordinatlar chunk sınırları içinde olmalı (0..CHUNK_SIZE)
    pub fn get_voxel(&self, x: usize, y: usize, z: usize) -> Option<VoxelType> {
        if x < CHUNK_SIZE && y < CHUNK_SIZE && z < CHUNK_SIZE {
            Some(self.voxels[x][y][z])
        } else {
            None
        }
    }

    /// Belirtilen pozisyondaki vokseli değiştirir
    /// Değişiklik olduğunda chunk'ı dirty olarak işaretler
    pub fn set_voxel(&mut self, x: usize, y: usize, z: usize, voxel_type: VoxelType) {
        if x < CHUNK_SIZE && y < CHUNK_SIZE && z < CHUNK_SIZE {
            if self.voxels[x][y][z] != voxel_type {
                self.voxels[x][y][z] = voxel_type;
                self.is_dirty = true; // Mesh yeniden oluşturulmalı
            }
        }
    }

    /// Chunk'tan render edilebilir mesh oluşturur
    /// Sadece görünür yüzleri mesh'e ekler (komşu analizi yapar)
    /// Bu algoritma "greedy meshing" olmadan basit face culling yapar
    pub fn generate_mesh(&self) -> (Vec<Vertex>, Vec<u16>) {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        let mut vertex_count = 0u16;

        // Her voksel pozisyonunu kontrol et
        for x in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                for z in 0..CHUNK_SIZE {
                    let voxel = self.voxels[x][y][z];
                    
                    // Sadece solid vokseller için yüz oluştur
                    if !voxel.is_solid() {
                        continue;
                    }

                    let voxel_color = voxel.get_color();
                    // Voksel merkezi pozisyonu (dünya koordinatlarında)
                    let world_x = (self.world_pos.0 * CHUNK_SIZE as i32) + x as i32;
                    let world_y = (self.world_pos.1 * CHUNK_SIZE as i32) + y as i32;
                    let world_z = (self.world_pos.2 * CHUNK_SIZE as i32) + z as i32;
                    
                    let pos = [world_x as f32, world_y as f32, world_z as f32];

                    // 6 yüzü kontrol et ve gerektiğinde ekle
                    // Her yüz için komşu voksel kontrolü yapılır
                    
                    // +X yüzü (sağ)
                    if self.should_render_face(x + 1, y, z) {
                        self.add_face_vertices_indices(
                            &mut vertices, &mut indices, &mut vertex_count,
                            pos, [1.0, 0.0, 0.0], voxel_color // Normal: +X yönü
                        );
                    }
                    
                    // -X yüzü (sol)
                    if self.should_render_face_negative(x, y, z, 0) {
                        self.add_face_vertices_indices(
                            &mut vertices, &mut indices, &mut vertex_count,
                            pos, [-1.0, 0.0, 0.0], voxel_color // Normal: -X yönü
                        );
                    }
                    
                    // +Y yüzü (üst)
                    if self.should_render_face(x, y + 1, z) {
                        self.add_face_vertices_indices(
                            &mut vertices, &mut indices, &mut vertex_count,
                            pos, [0.0, 1.0, 0.0], voxel_color // Normal: +Y yönü
                        );
                    }
                    
                    // -Y yüzü (alt)
                    if self.should_render_face_negative(x, y, z, 1) {
                        self.add_face_vertices_indices(
                            &mut vertices, &mut indices, &mut vertex_count,
                            pos, [0.0, -1.0, 0.0], voxel_color // Normal: -Y yönü
                        );
                    }
                    
                    // +Z yüzü (ön)
                    if self.should_render_face(x, y, z + 1) {
                        self.add_face_vertices_indices(
                            &mut vertices, &mut indices, &mut vertex_count,
                            pos, [0.0, 0.0, 1.0], voxel_color // Normal: +Z yönü
                        );
                    }
                    
                    // -Z yüzü (arka)
                    if self.should_render_face_negative(x, y, z, 2) {
                        self.add_face_vertices_indices(
                            &mut vertices, &mut indices, &mut vertex_count,
                            pos, [0.0, 0.0, -1.0], voxel_color // Normal: -Z yönü
                        );
                    }
                }
            }
        }

        (vertices, indices)
    }

    /// Belirtilen pozisyondaki yüzün render edilip edilmeyeceğini kontrol eder
    /// Komşu pozisyonda solid voksel varsa yüz gizlenir (face culling)
    fn should_render_face(&self, x: usize, y: usize, z: usize) -> bool {
        // Chunk sınırları dışına çıkıyorsa yüzü render et
        // Gerçek uygulamada komşu chunk kontrolü yapılmalı
        if x >= CHUNK_SIZE || y >= CHUNK_SIZE || z >= CHUNK_SIZE {
            return true;
        }
        
        // Komşu pozisyondaki voksel solid değilse yüzü render et
        !self.voxels[x][y][z].is_solid()
    }

    /// Negatif yöndeki komşu kontrolü için ayrı fonksiyon
    /// axis: 0=X, 1=Y, 2=Z
    fn should_render_face_negative(&self, x: usize, y: usize, z: usize, axis: usize) -> bool {
        let (check_x, check_y, check_z) = match axis {
            0 => if x == 0 { return true; } else { (x - 1, y, z) },
            1 => if y == 0 { return true; } else { (x, y - 1, z) },
            2 => if z == 0 { return true; } else { (x, y, z - 1) },
            _ => return true,
        };
        
        !self.voxels[check_x][check_y][check_z].is_solid()
    }

    /// Bir yüz için 4 vertex ve 6 index (2 üçgen) ekler
    /// Normal vektörü yüzün yönünü belirler, renk her vertex'e uygulanır
    fn add_face_vertices_indices(
        &self,
        vertices: &mut Vec<Vertex>,
        indices: &mut Vec<u16>,
        vertex_count: &mut u16,
        center_pos: [f32; 3],
        normal: [f32; 3],
        color: [f32; 3]
    ) {
        // Yüz normal vektörüne göre 4 köşe pozisyonunu hesapla
        // Bu basit implementasyon, normal'a göre uygun offset'ler kullanır
        let offsets = self.get_face_offsets(normal);
        
        // 4 vertex ekle
        for offset in offsets.iter() {
            vertices.push(Vertex {
                position: [
                    center_pos[0] + offset[0],
                    center_pos[1] + offset[1],
                    center_pos[2] + offset[2],
                ],
                color,
            });
        }

        // 2 üçgen için 6 index ekle (counter-clockwise sıralama)
        let base = *vertex_count;
        indices.extend_from_slice(&[
            base, base + 1, base + 2,    // İlk üçgen
            base + 2, base + 3, base,    // İkinci üçgen
        ]);
        
        *vertex_count += 4;
    }

    /// Normal vektörüne göre yüz köşelerinin offset'lerini döndürür
    /// Her yüz için 4 köşe pozisyonu hesaplanır
    fn get_face_offsets(&self, normal: [f32; 3]) -> [[f32; 3]; 4] {
        const HALF_SIZE: f32 = 0.5;
        
        match normal {
            // +X yüzü (sağ)
            [1.0, 0.0, 0.0] => [
                [HALF_SIZE, -HALF_SIZE, -HALF_SIZE],
                [HALF_SIZE, -HALF_SIZE,  HALF_SIZE],
                [HALF_SIZE,  HALF_SIZE,  HALF_SIZE],
                [HALF_SIZE,  HALF_SIZE, -HALF_SIZE],
            ],
            // -X yüzü (sol)
            [-1.0, 0.0, 0.0] => [
                [-HALF_SIZE, -HALF_SIZE,  HALF_SIZE],
                [-HALF_SIZE, -HALF_SIZE, -HALF_SIZE],
                [-HALF_SIZE,  HALF_SIZE, -HALF_SIZE],
                [-HALF_SIZE,  HALF_SIZE,  HALF_SIZE],
            ],
            // +Y yüzü (üst)
            [0.0, 1.0, 0.0] => [
                [-HALF_SIZE,  HALF_SIZE, -HALF_SIZE],
                [-HALF_SIZE,  HALF_SIZE,  HALF_SIZE],
                [ HALF_SIZE,  HALF_SIZE,  HALF_SIZE],
                [ HALF_SIZE,  HALF_SIZE, -HALF_SIZE],
            ],
            // -Y yüzü (alt)
            [0.0, -1.0, 0.0] => [
                [-HALF_SIZE, -HALF_SIZE,  HALF_SIZE],
                [-HALF_SIZE, -HALF_SIZE, -HALF_SIZE],
                [ HALF_SIZE, -HALF_SIZE, -HALF_SIZE],
                [ HALF_SIZE, -HALF_SIZE,  HALF_SIZE],
            ],
            // +Z yüzü (ön)
            [0.0, 0.0, 1.0] => [
                [-HALF_SIZE, -HALF_SIZE,  HALF_SIZE],
                [ HALF_SIZE, -HALF_SIZE,  HALF_SIZE],
                [ HALF_SIZE,  HALF_SIZE,  HALF_SIZE],
                [-HALF_SIZE,  HALF_SIZE,  HALF_SIZE],
            ],
            // -Z yüzü (arka)
            [0.0, 0.0, -1.0] => [
                [ HALF_SIZE, -HALF_SIZE, -HALF_SIZE],
                [-HALF_SIZE, -HALF_SIZE, -HALF_SIZE],
                [-HALF_SIZE,  HALF_SIZE, -HALF_SIZE],
                [ HALF_SIZE,  HALF_SIZE, -HALF_SIZE],
            ],
            _ => [[0.0; 3]; 4], // Fallback
        }
    }
}