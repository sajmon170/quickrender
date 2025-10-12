use std::ops::Deref;
use std::path::Path;

use glam::{Mat4, Vec3, Vec4};
use tobj::LoadError;

use std::rc::{Rc, Weak};
use std::cell::{Ref, RefCell};

use crate::camera::Camera;
use crate::{
    data::Vertex, gpu::Gpu, material::{Material, SimpleMaterial}, mesh::Mesh
};

// TODO - Generalize this to multiple materials
struct Model {
    mesh: Mesh,
    material: Box<dyn Material>
}

impl Model {
    fn fill_tangents(mut a: Vertex, mut b: Vertex, mut c: Vertex)
                     -> (Vertex, Vertex, Vertex) {
        let e_pos_b = glam::Vec3::from(b.pos) - glam::Vec3::from(a.pos);
        let e_pos_c = glam::Vec3::from(c.pos) - glam::Vec3::from(a.pos);

        let e_uv_b = glam::Vec2::from(b.uv) - glam::Vec2::from(a.uv);
        let e_uv_c = glam::Vec2::from(c.uv) - glam::Vec2::from(a.uv);

        let t_vec = (e_pos_b * e_uv_c.y - e_pos_c * e_uv_b.y).normalize();
        let b_vec = (e_pos_c * e_uv_b.x - e_pos_b * e_uv_c.x).normalize();

        for vtx in [&mut a, &mut b, &mut c] {
            vtx.tangent = t_vec.into();
            vtx.bitangent = b_vec.into();
        }

        (a, b, c)
    }

    pub fn load_obj(gpu: &Gpu, path: &Path) -> Result<Object, LoadError> {
        let (models, materials) = tobj::load_obj(&path, &tobj::GPU_LOAD_OPTIONS)?;
        let materials = materials.unwrap();
        let mut objs = Vec::<Model>::new();
 
        for model in models.iter() {
            let mut vertices: Vec<_> = model.mesh.positions.chunks_exact(3)
                .zip(model.mesh.normals.chunks_exact(3))
                .zip(model.mesh.texcoords.chunks_exact(2))
                .map(|((pos, normal), uv)| Vertex {
                    pos: [pos[0], -pos[2], pos[1]],
                    normal: [normal[0], normal[1], normal[2]],
                    uv: [uv[0], 1.0 - uv[1]],
                    ..Default::default()
                })
                .collect();

            // TODO - refactor texture extraction code

            let texture_path = if let Some(id) = model.mesh.material_id
                && let Some(diffuse) = &materials[id].diffuse_texture {
                diffuse
            }
            else {
                &"src/res/star.png".into()
            };

            let normal_path = if let Some(id) = model.mesh.material_id
                && let Some(normal) = &materials[id].normal_texture {
                if let Some("-bm") = normal.split_whitespace().next() {
                    normal.splitn(3, " ").last().unwrap()
                }
                else {
                    normal
                }
            }
            else {
                "src/res/star.png"
            };

            let material = Box::new(SimpleMaterial::new(&gpu,
                                                        &Path::new(texture_path),
                                                        &Path::new(normal_path)));


            for point_idx in model.mesh.indices.chunks_exact(3) {
                let (a, b, c) = Self::fill_tangents(
                    vertices[point_idx[0] as usize],
                    vertices[point_idx[1] as usize],
                    vertices[point_idx[2] as usize]
                );

                vertices[point_idx[0] as usize] = a;
                vertices[point_idx[1] as usize] = b;
                vertices[point_idx[2] as usize] = c;
            }
            
            let mesh = Mesh::new(gpu, vertices, model.mesh.indices.clone());

            objs.push(Model { mesh, material });
        }

        let result = Object::empty();
        for model in objs {
            result.add_child(ObjectData::Model(Rc::new(model)));
        }

        Ok(result)
    }
}

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
            xform: Mat4::default(),
            parent: Weak::new(),
            children: Vec::new()
        })))
    }

    pub fn empty() -> Self {
        Self::new(ObjectData::Empty)
    }

    pub fn with_children(self, children: Vec<ObjectData>) -> Self {
        for child in children {
            self.add_child(child);
        }
        self
    }
    
    pub fn set(&mut self, xform: Mat4) {
        self.0.borrow_mut().xform = xform;
    }

    pub fn get(&self) -> impl Deref<Target = ObjectData> + '_ {
        Ref::map(self.0.borrow(), |borrow| &borrow.data)
    }

    pub fn add_child(&self, data: ObjectData) {
        self.0.borrow_mut().children.push(
            Self(Rc::new(RefCell::new(ObjectInternal{
                parent: Rc::downgrade(&self.0),
                data,
                xform: Mat4::default(),
                children: Vec::new()
            }
        ))));
    }

    pub fn get_child(&self, idx: usize) -> Option<Self> {
        self.0.borrow().children.get(idx).cloned()
    }

    /*
    fn print_internal(&self, level: usize) {
        println!("{}Node: {}", "  ".repeat(level), self.get());
        for child in &self.0.borrow().children {
            child.print_internal(level + 1);
        }
    }

    pub fn print(&self) {
        self.print_internal(0);
    }
    */
}


impl Object {
   pub fn translate(&mut self, translation: Vec3) {
        self.0.borrow_mut().xform *= Mat4::from_translation(translation);
    }

    pub fn rotate_x(&mut self, rotation: f32) {
        self.0.borrow_mut().xform *= Mat4::from_rotation_x(rotation);
    }

    pub fn rotate_y(&mut self, rotation: f32) {
        self.0.borrow_mut().xform *= Mat4::from_rotation_y(rotation);
    }

    pub fn rotate_z(&mut self, rotation: f32) {
        self.0.borrow_mut().xform *= Mat4::from_rotation_z(rotation);
    }

    pub fn scale(&mut self, scale: Vec3) {
        self.0.borrow_mut().xform *= Mat4::from_scale(scale);
    }

    pub fn reset(&mut self) {
        self.0.borrow_mut().xform = Mat4::IDENTITY;
    }
}
