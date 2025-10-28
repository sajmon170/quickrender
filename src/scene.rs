use std::{ops::Deref, path::Path, rc::Weak};

use glam::{Mat4, Vec3, Vec4};
use tobj::LoadError;
use std::rc::Rc;

use crate::{
    camera::Camera, data::Vertex, gpu::Gpu, material::{Material, SimpleMaterial}, mesh::Mesh, object::{Object, ObjectData}, model::Model
};

pub struct Scene {
    pub root: Object,
    camera: Option<Object>
}

impl Scene {
    pub fn new(objs: Vec<Object>) -> Self {
        let root = Object::empty().with_children(objs);
        let camera = root.get_all_cameras()
            .first()
            .map(|(camera, _)| camera)
            .cloned();
        
        Self {
            root,
            camera
        }
    }
    
    pub fn set_camera(&mut self, camera: Object) {
        self.camera = Some(camera);
    }

    pub fn get_camera(&self) -> Option<Rc<Camera>> {
        self.camera.as_ref().map(|obj| match obj.get().deref() {
            ObjectData::Camera(camera) => camera.clone(),
            _ => panic!()
        })
    }

    pub fn get_camera_object(&mut self) -> &mut Option<Object> {
        &mut self.camera
    }
}
