use std::ops::Deref;
use std::path::Path;

use glam::{Mat4, Vec3, Vec4};
use tobj::LoadError;

use bytemuck::NoUninit;
use std::num::NonZero;

use std::rc::{Rc, Weak};
use std::cell::{Ref, RefCell};

use crate::camera::Camera;
use crate::{
    data::Vertex, gpu::Gpu, material::{Material, SimpleMaterial}, mesh::Mesh, model::Model
};

pub enum ObjectData {
    Empty,
    Model(Rc<Model>),
    Camera(Rc<Camera>)
}

struct ObjectInternal {
    xform: Mat4,
    data: ObjectData,
    parent: Weak<RefCell<ObjectInternal>>,
    children: Vec<Object>
}

#[derive(Clone)]
pub struct Object(Rc<RefCell<ObjectInternal>>);

// TODO - implement ObjectRef with Weak

impl Object {
    pub fn new(data: ObjectData) -> Self {
        Self(Rc::new(RefCell::new(ObjectInternal {
            data,
            xform: Default::default(),
            parent: Weak::new(),
            children: Vec::new()
        })))
    }

    pub fn empty() -> Self {
        Self::new(ObjectData::Empty)
    }

    pub fn with_children(self, children: Vec<Object>) -> Self {
        self.0.borrow_mut().children = children;
        self
    }
    
    pub fn set(&mut self, xform: Mat4) {
        self.0.borrow_mut().xform = xform;
    }

    pub fn get(&self) -> impl Deref<Target = ObjectData> + '_ {
        Ref::map(self.0.borrow(), |borrow| &borrow.data)
    }

    pub fn add_child(&self, obj: Object) {
        self.0.borrow_mut().children.push(obj);
            /*
            Self(Rc::new(RefCell::new(ObjectInternal{
                parent: Rc::downgrade(&self.0),
                data,
                xform: Mat4::default(),
                children: Vec::new()
            }
        ))));*/
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

    pub fn get_all_models(&self) -> Vec<(Rc<Model>, Mat4)> {
        self.get_all()
            .into_iter()
            .filter_map(|(obj, xform)| {
                match &obj.0.borrow().data {
                    ObjectData::Model(model) => Some((model.clone(), xform)),
                    _ => None
                }
            })
            .collect()
    }

    pub fn get_all_cameras(&self) -> Vec<(Rc<Camera>, Mat4)> {
        self.get_all()
            .into_iter()
            .filter_map(|(obj, xform)| {
                match &obj.0.borrow().data {
                    ObjectData::Camera(camera) => Some((camera.clone(), xform)),
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
