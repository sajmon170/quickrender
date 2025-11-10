use std::ops::Deref;
use std::path::Path;

use glam::{Mat4, Vec3, Vec4};
use slab::Slab;
use tobj::LoadError;

use strum::EnumTryAs;

use bytemuck::NoUninit;
use std::num::NonZero;

use std::rc::{Rc, Weak};
use std::cell::{Ref, RefCell};

use crate::camera::Camera;
use crate::{
    data::Vertex, gpu::Gpu, material::{Material, SimpleMaterial}, mesh::Mesh, model::Model
};

pub struct DataStore {
    models: Slab<Model>,
    cameras: Slab<Camera>
}

impl DataStore {
    pub fn add_model(&mut self, model: Model) -> DataToken {
        let id = self.models.insert(model);
        DataToken::Model(id)
    }

    pub fn add_camera(&mut self, camera: Camera) -> DataToken {
        let id = self.cameras.insert(camera);
        DataToken::Camera(id)
    }

    pub fn get_model(&mut self, id: usize) -> Option<&mut Model> {
        self.models.get_mut(id)
    }

    pub fn get_camera(&mut self, id: usize) -> Option<&mut Camera> {
        self.cameras.get_mut(id)
    }
}

#[derive(Copy, Clone, Debug)]
pub enum DataToken {
    Empty,
    Model(usize),
    Camera(usize)
}

struct ObjectInternal {
    xform: Mat4,
    data: DataToken,
    parent: Weak<RefCell<ObjectInternal>>,
    children: Vec<Object>
}

pub trait IntoData {
    fn into_data(self, store: &mut DataStore) -> DataToken;
}

impl IntoData for Camera {
    fn into_data(self, store: &mut DataStore) -> DataToken {
        store.add_camera(self)
    }
}

impl IntoData for Model {
    fn into_data(self, store: &mut DataStore) -> DataToken {
        store.add_model(self)
    }
}

#[derive(Clone)]
pub struct Object(Rc<RefCell<ObjectInternal>>);

impl Object {
    pub fn new(data: impl IntoData, store: &mut DataStore) -> Self {
        Self(Rc::new(RefCell::new(ObjectInternal {
            data: data.into_data(store),
            xform: Default::default(),
            parent: Weak::new(),
            children: Vec::new()
        })))
    }

    pub fn empty() -> Self {
        Self(Rc::new(RefCell::new(ObjectInternal {
            data: DataToken::Empty,
            xform: Default::default(),
            parent: Weak::new(),
            children: Vec::new()
        })))
    }

    pub fn with_children(self, children: Vec<Object>) -> Self {
        self.0.borrow_mut().children = children;
        self
    }

    pub fn get_parent_xform(&self) -> Mat4 {
        self.0.borrow()
            .parent
            .upgrade()
            .map(|parent| Object(parent).get_local_xform())
            .unwrap_or_default()
    }

    pub fn get_local_xform(&self) -> Mat4 {
        self.0.borrow().xform
    }
    
    pub fn set_xform(&mut self, xform: Mat4) {
        self.0.borrow_mut().xform = xform;
    }

    pub fn get_data(&self) -> DataToken {
        self.0.borrow().data
    }

    pub fn add_child(&self, obj: Object) {
        self.0.borrow_mut().children.push(obj);
    }

    pub fn get_child(&self, idx: usize) -> Option<Self> {
        self.0.borrow().children.get(idx).cloned()
    }

    fn get_all_internal(&self, objs: &mut Vec<(Object, Mat4)>, prev_xforms: &Mat4) {
        let current_xform = self.0.borrow().xform * prev_xforms;
        objs.push((self.clone(), current_xform));

        for child in &self.0.borrow().children {
            child.get_all_internal(objs, &current_xform);
        }
    }

    pub fn get_all(&self) -> Vec<(Object, Mat4)> {
        let mut objs = Vec::new();
        self.get_all_internal(&mut objs, &Mat4::IDENTITY);

        objs
    }

    pub fn get_all_models(&self) -> Vec<(DataToken, Mat4)> {
        self.get_all()
            .into_iter()
            .filter_map(|(obj, xform)| {
                let data = obj.0.borrow().data;
                match data {
                    DataToken::Model(_) => Some((data, xform)),
                    _ => None
                }
            })
            .collect()
    }

    pub fn get_all_cameras(&self) -> Vec<(DataToken, Mat4)> {
        self.get_all()
            .into_iter()
            .filter_map(|(obj, xform)| {
                let data = obj.0.borrow().data;
                match data {
                    DataToken::Camera(_) => Some((data, xform)),
                    _ => None
                }
            })
            .collect()
    }

    pub fn translate(&mut self, translation: Vec3) {
        self.0.borrow_mut().xform *= Mat4::from_translation(translation);
    }

    pub fn with_translation(mut self, translation: Vec3) -> Self {
        self.translate(translation);
        self
    }

    pub fn rotate_x(&mut self, rotation: f32) {
        self.0.borrow_mut().xform *= Mat4::from_rotation_x(rotation);
    }

    pub fn with_rotation_x(mut self, rotation: f32) -> Self {
        self.rotate_x(rotation);
        self
    }

    pub fn rotate_y(&mut self, rotation: f32) {
        self.0.borrow_mut().xform *= Mat4::from_rotation_y(rotation);
    }

    pub fn with_rotation_y(mut self, rotation: f32) -> Self {
        self.rotate_y(rotation);
        self
    }

    pub fn rotate_z(&mut self, rotation: f32) {
        self.0.borrow_mut().xform *= Mat4::from_rotation_z(rotation);
    }

    pub fn with_rotation_z(mut self, rotation: f32) -> Self {
        self.rotate_z(rotation);
        self
    }

    pub fn scale(&mut self, scale: Vec3) {
        self.0.borrow_mut().xform *= Mat4::from_scale(scale);
    }

    pub fn with_scale(mut self, scale: Vec3) -> Self {
        self.scale(scale);
        self
    }

    pub fn reset(&mut self) {
        self.0.borrow_mut().xform = Mat4::IDENTITY;
    }
}
