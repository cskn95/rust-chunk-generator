use crate::camera::Camera;
use winit::event::ElementState;
use winit::keyboard::KeyCode;

pub struct CameraController {
    speed: f32,
    mouse_sensitivity: f32,
    is_forward_pressed: bool,
    is_backward_pressed: bool,
    is_left_pressed: bool,
    is_right_pressed: bool,
    has_movement: bool,
}

impl CameraController {
    pub fn new(speed: f32) -> Self {
        Self {
            speed,
            mouse_sensitivity: 0.001, // Optimized mouse sensitivity for smooth camera control
            is_forward_pressed: false,
            is_backward_pressed: false,
            is_left_pressed: false,
            is_right_pressed: false,
            has_movement: false,
        }
    }

    pub fn process_events(&mut self, keycode: KeyCode, state: ElementState) -> bool {
        let is_pressed = state == ElementState::Pressed;

        match keycode {
            KeyCode::KeyW | KeyCode::ArrowUp => {
                self.is_forward_pressed = is_pressed;
                true
            }
            KeyCode::KeyA | KeyCode::ArrowLeft => {
                self.is_left_pressed = is_pressed;
                true
            }
            KeyCode::KeyS | KeyCode::ArrowDown => {
                self.is_backward_pressed = is_pressed;
                true
            }
            KeyCode::KeyD | KeyCode::ArrowRight => {
                self.is_right_pressed = is_pressed;
                true
            }
            _ => false,
        }
    }

    pub fn process_mouse(&mut self, mouse_dx: f64, mouse_dy: f64, camera: &mut Camera) {
        let dx = mouse_dx as f32;
        let dy = mouse_dy as f32;
        
        // Check if there's actual mouse movement (threshold for noise filtering)
        let has_mouse_movement = dx.abs() > 0.001 || dy.abs() > 0.001;
        if has_mouse_movement {
            self.has_movement = true;
        }
        
        // Update yaw (horizontal rotation) - reversed X axis
        let new_yaw = camera.yaw() + dx * self.mouse_sensitivity;
        camera.set_yaw(new_yaw);
        
        // Update pitch (vertical rotation)
        let new_pitch = camera.pitch() - dy * self.mouse_sensitivity;
        camera.set_pitch(new_pitch); // Automatic clamping in Camera::set_pitch
    }

    pub fn update_camera(&mut self, camera: &mut Camera, delta_time: f32) {
        // Get direction vectors from camera
        let forward = camera.forward();
        let right = camera.right();
        
        // Calculate velocity based on pressed keys with delta time for smooth movement
        let mut velocity = cgmath::Vector3::new(0.0, 0.0, 0.0);
        let movement_speed = self.speed * delta_time;
        
        // Check if any movement keys are pressed
        let has_keyboard_input = self.is_forward_pressed || self.is_backward_pressed || 
                                self.is_left_pressed || self.is_right_pressed;
        
        // Forward/Backward movement
        if self.is_forward_pressed {
            velocity += forward * movement_speed;
        }
        if self.is_backward_pressed {
            velocity -= forward * movement_speed;
        }
        
        // Strafe movement (left/right)
        if self.is_right_pressed {
            velocity += right * movement_speed;
        }
        if self.is_left_pressed {
            velocity -= right * movement_speed;
        }
        
        // Update movement status for power saving
        self.has_movement = has_keyboard_input;
        
        // Update camera position
        camera.set_eye(camera.eye() + velocity);
    }
    
    pub fn has_movement(&self) -> bool {
        self.has_movement
    }

    pub fn set_speed(&mut self, speed: f32) {
        self.speed = speed;
    }

    pub fn set_mouse_sensitivity(&mut self, sensitivity: f32) {
        self.mouse_sensitivity = sensitivity;
    }
}
