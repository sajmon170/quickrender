use glam::{Mat4, Vec3};

use crate::{object::DataStore, scene::Scene};

#[derive(Default, Copy, Clone)]
pub struct UserInput {
    pub move_forward: bool,
    pub move_backward: bool,
    pub move_left: bool,
    pub move_right: bool,
    pub move_up: bool,
    pub move_down: bool,
    pub yaw: f32,
    pub pitch: f32
}

impl UserInput {
    pub fn direction(&self) -> Vec3 {
        let mut direction = Vec3::ZERO;

        if self.move_forward {
            direction -= Vec3::Z;
        }

        if self.move_backward {
            direction += Vec3::Z;
        }

        if self.move_left {
            direction += Vec3::X;
        }

        if self.move_right {
            direction -= Vec3::X;
        }

        if self.move_up {
            direction -= Vec3::Y;
        }

        if self.move_down {
            direction += Vec3::Y;
        }

        if direction.element_sum() > 0.0 {
            direction = direction.normalize();
        }
        
        direction
    }
}

#[derive(Default)]
pub struct PhysicsController;

impl PhysicsController {
    pub fn update(&self, scene: &mut Scene, input: UserInput) {
        if let Some(camera) = scene.get_camera_object() {
            let (_, _, pos) = camera.get_local_xform().to_scale_rotation_translation();

            let xform =
                Mat4::from_rotation_y(-input.yaw * 0.0025)
                * Mat4::from_rotation_x(-input.pitch * 0.0025)
                * Mat4::from_translation(pos)
                * camera.get_parent_xform();

            camera.set_xform(xform);
        }
    }
}
