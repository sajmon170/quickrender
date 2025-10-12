use glam::{Vec3, Mat4};

#[derive(Clone)]
pub struct Camera {
    pub fov: f32,
    pub near: f32,
    pub far: f32
}

impl Camera {
    fn new() -> Self {
        Self {
            fov: 45.0 * std::f32::consts::PI / 180.0,
            near: 0.01,
            far: 100.0
        }
    }

    fn get_projection_matrix(&self, ratio: f32) -> Mat4 {
        Mat4::perspective_lh(self.fov, ratio, self.near, self.far)
    }

    pub const CAMERA_POS: Vec3 = Vec3::new(0.0, 0.0, 3.0); 
}
