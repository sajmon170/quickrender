use std::{ops::Deref, path::Path, rc::Weak};

use glam::{Mat4, Vec3, Vec4};
use tobj::LoadError;
use std::rc::Rc;

use crate::{
    camera::Camera, data::Vertex, gpu::Gpu, material::{Material, SimpleMaterial}, mesh::Mesh, object::{Object, DataToken}, model::Model
};

pub struct Scene {
    pub root: Object,
    camera: Option<Object>
}

impl Scene {
    pub fn new(objs: Vec<Object>) -> Self {
        let camera = objs.iter().find(|obj| match obj.get_data() {
            DataToken::Camera(_) => true,
            _ => false
        }).cloned();

        let root = Object::empty().with_children(objs);
        
        Self {
            root,
            camera
        }
    }
    
    pub fn set_camera(&mut self, camera: Object) {
        self.camera = Some(camera);
    }

    pub fn get_camera_object(&mut self) -> Option<&mut Object> {
        self.camera.as_mut()
    }
}
