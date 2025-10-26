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
    pub fn new(objs: Vec<Object>) -> Self {
        let root = Object::empty().with_children(objs);
        let camera = root.get_all_cameras()
            .first()
            .map(|(camera, _)| Rc::downgrade(camera))
            .unwrap_or_default();
        
        Self {
            root,
            camera
        }
    }
    
    pub fn set_camera(&mut self, camera: &Rc<Camera>) {
        self.camera = Rc::downgrade(camera);
    }

    pub fn get_camera(&self) -> Rc<Camera> {
        self.camera.upgrade().unwrap()
    }
}
