use std::{path::Path, rc::Weak};

use glam::{Mat4, Vec3, Vec4};
use tobj::LoadError;
use std::rc::Rc;

use crate::{
    camera::Camera, data::Vertex, gpu::Gpu, material::{Material, SimpleMaterial}, mesh::Mesh, object::{Object, ObjectData, Model}
};

struct Scene {
    root: Object,
    camera: Weak<Camera>
}

impl Scene {
    fn set_camera(&mut self, camera: &Rc<Camera>) {
        self.camera = Rc::downgrade(camera);
    }
    
    pub fn set_render_pass(&mut self, render_pass: &mut wgpu::RenderPass, queue: &wgpu::Queue) {
        /*
        for Model { mesh, material } in &mut self.objs {
            material.set_projection_xform(self.projection_xform);
            material.set_view_xform(self.view_xform);
            material.set_model_xform(self.0.borrow_mut().xform);
            material.set_render_pass(render_pass, queue, Camera::CAMERA_POS);
            
            mesh.set_render_pass(render_pass);
        }
        */
    }
}
