use crate::camera::Camera;
use cgmath::{InnerSpace, Vector3};
use winit::event::ElementState;
use winit::keyboard::KeyCode;

/// FPS tarzı kamera kontrolcüsü
/// Klavye ve fare girişlerini işleyerek kamerayı hareket ettirir
pub struct CameraController {
    speed: f32,                    // Hareket hızı (birim/saniye)
    mouse_sensitivity: f32,        // Fare hassasiyeti
    is_forward_pressed: bool,      // W tuşu basılı mı?
    is_backward_pressed: bool,     // S tuşu basılı mı?
    is_left_pressed: bool,         // A tuşu basılı mı?
    is_right_pressed: bool,        // D tuşu basılı mı?
    is_up_pressed: bool,           // Space tuşu basılı mı?
    is_down_pressed: bool,         // Ctrl tuşu basılı mı?
    has_movement: bool,            // Hareket var mı? (güç tasarrufu için)
}

impl CameraController {
    /// Belirtilen hızla yeni kamera kontrolcüsü oluşturur
    pub fn new(speed: f32) -> Self {
        Self {
            speed,
            // Pürüzsüz kamera kontrolü için optimize edilmiş fare hassasiyeti
            mouse_sensitivity: 0.001, 
            is_forward_pressed: false,
            is_backward_pressed: false,
            is_left_pressed: false,
            is_right_pressed: false,
            is_up_pressed: false,
            is_down_pressed: false,
            has_movement: false,
        }
    }

    /// Klavye girişlerini işler
    /// Döndürülen bool değeri girişin işlenip işlenmediğini belirtir
    pub fn process_events(&mut self, keycode: KeyCode, state: ElementState) -> bool {
        let is_pressed = state == ElementState::Pressed;

        match keycode {
            KeyCode::KeyW => {
                self.is_forward_pressed = is_pressed;
                true
            }
            KeyCode::KeyA => {
                self.is_left_pressed = is_pressed;
                true
            }
            KeyCode::KeyS => {
                self.is_backward_pressed = is_pressed;
                true
            }
            KeyCode::KeyD => {
                self.is_right_pressed = is_pressed;
                true
            }
            KeyCode::Space => {
                self.is_up_pressed = is_pressed;
                true
            }
            KeyCode::ControlLeft | KeyCode::ControlRight => {
                self.is_down_pressed = is_pressed;
                true
            }
            _ => false,
        }
    }

    /// Fare girişlerini işler ve kamerayı döndürür
    /// mouse_dx: Yatay fare hareketi (piksel)
    /// mouse_dy: Dikey fare hareketi (piksel)
    pub fn process_mouse(&mut self, mouse_dx: f64, mouse_dy: f64, camera: &mut Camera) {
        let dx = mouse_dx as f32;
        let dy = mouse_dy as f32;
        
        // Gerçek fare hareketi olup olmadığını kontrol et (gürültü filtresi)
        let has_mouse_movement = dx.abs() > 0.001 || dy.abs() > 0.001;
        if has_mouse_movement {
            self.has_movement = true;
        }
        
        // Yaw (yatay rotasyon) güncelle - X ekseni ters çevrildi
        let new_yaw = camera.yaw() + dx * self.mouse_sensitivity;
        camera.set_yaw(new_yaw);
        
        // Pitch (dikey rotasyon) güncelle
        let new_pitch = camera.pitch() - dy * self.mouse_sensitivity;
        camera.set_pitch(new_pitch); // Camera::set_pitch içinde otomatik sınırlama var
    }

    /// Kamera pozisyonunu günceller
    /// Basılı tuşlara göre kamerayı hareket ettirir
    /// delta_time: Son güncellemeden geçen süre (saniye)
    pub fn update_camera(&mut self, camera: &mut Camera, delta_time: f32) {
        // Kameradan yön vektörlerini al
        let forward = camera.forward();
        let right = camera.right();
        let up = camera.up();
        
        let mut move_dir = Vector3::new(0.0, 0.0, 0.0);
        
        // İleri/Geri hareket
        if self.is_forward_pressed {
            move_dir += forward;
        }
        if self.is_backward_pressed {
            move_dir -= forward;
        }
        
        // Yan hareket (sola/sağa)
        if self.is_right_pressed {
            move_dir += right;
        }
        if self.is_left_pressed {
            move_dir -= right;
        }

        // Y ekseni hareketi (yukarı/aşağı)
        if self.is_up_pressed {
            move_dir += up;
        }
        if self.is_down_pressed {
            move_dir -= up;
        }

        if move_dir.magnitude2() > 0.0 {
            let new_pos = camera.eye() + move_dir.normalize() * self.speed * delta_time;
            camera.set_eye(new_pos);
        }
        
        // Güç tasarrufu için hareket durumunu güncelle
        self.has_movement = self.is_forward_pressed || self.is_backward_pressed || self.is_left_pressed || self.is_right_pressed || self.is_up_pressed || self.is_down_pressed;
    }
}
