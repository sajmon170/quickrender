use std::{ops::Deref, path::Path, rc::Weak};

use glam::{Mat4, Vec3, Vec4};
use tobj::LoadError;
use std::rc::Rc;

use crate::{
    camera::Camera, data::Vertex, gpu::Gpu, material::{Material, SimpleMaterial}, mesh::Mesh, object::{Object, ObjectData}, model::Model
};

pub struct Scene {
    pub root: Object,
    camera: Weak<Camera>
}

impl Scene {
    pub fn new() -> Self {
        Self {
            root: Object::empty(),
            camera: Default::default()
        }
    }
    
    pub fn set_camera(&mut self, camera: &Rc<Camera>) {
        self.camera = Rc::downgrade(camera);
    }

    pub fn get_camera(&self) -> Rc<Camera> {
        self.camera.upgrade().unwrap()
    }
}
