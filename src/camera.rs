use cgmath::InnerSpace;

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::from_cols(
    cgmath::Vector4::new(1.0, 0.0, 0.0, 0.0),
    cgmath::Vector4::new(0.0, 1.0, 0.0, 0.0),
    cgmath::Vector4::new(0.0, 0.0, 0.5, 0.0),
    cgmath::Vector4::new(0.0, 0.0, 0.5, 1.0),
);

pub struct Camera {
    eye: cgmath::Point3<f32>,
    yaw: f32,
    pitch: f32,
    up: cgmath::Vector3<f32>,
    aspect: f32,
    fovy: f32,
    znear: f32,
    zfar: f32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    view_proj: [[f32; 4]; 4],
}

impl Camera {
    pub fn new(
        eye: cgmath::Point3<f32>, 
        yaw: f32,
        pitch: f32,
        up: cgmath::Vector3<f32>,
        aspect: f32,
        fovy: f32,
        znear: f32,
        zfar: f32,
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

    pub fn eye(&self) -> cgmath::Point3<f32> {
        self.eye
    }

    pub fn set_eye(&mut self, eye: cgmath::Point3<f32>) {
        self.eye = eye;
    }

    pub fn up(&self) -> cgmath::Vector3<f32> {
        self.up
    }

    pub fn yaw(&self) -> f32 {
        self.yaw
    }

    pub fn set_yaw(&mut self, yaw: f32) {
        self.yaw = yaw;
    }

    pub fn pitch(&self) -> f32 {
        self.pitch
    }

    pub fn set_pitch(&mut self, pitch: f32) {
        self.pitch = pitch.clamp(-std::f32::consts::FRAC_PI_2 + 0.1, std::f32::consts::FRAC_PI_2 - 0.1);
    }

    pub fn set_rotation(&mut self, yaw: f32, pitch: f32) {
        self.yaw = yaw;
        self.set_pitch(pitch);
    }

    pub fn forward(&self) -> cgmath::Vector3<f32> {
        cgmath::Vector3::new(
            self.yaw.cos() * self.pitch.cos(),
            self.pitch.sin(),
            self.yaw.sin() * self.pitch.cos(),
        )
    }

    pub fn right(&self) -> cgmath::Vector3<f32> {
        self.forward().cross(self.up).normalize()
    }

    pub fn build_view_projection_matrix(&self) -> cgmath::Matrix4<f32> {
        // Forward vektörünü yaw ve pitch değerlerine göre hesapla
        let forward = self.forward();

        // Target'i forward vektörünü kullanarak hesapla
        let target = self.eye + forward;

        // View matrisini oluştur
        let view = cgmath::Matrix4::look_at_rh(self.eye, target, self.up);
        let proj = cgmath::perspective(cgmath::Deg(self.fovy), self.aspect, self.znear, self.zfar);

        return OPENGL_TO_WGPU_MATRIX * proj * view;
    }
}

impl CameraUniform {
    pub fn new() -> Self {
        use cgmath::SquareMatrix;
        Self {
            view_proj: cgmath::Matrix4::identity().into(),
        }
    }

    pub fn update_view_proj(&mut self, camera: &Camera) {
        self.view_proj = camera.build_view_projection_matrix().into();
    }
}