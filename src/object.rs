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
    data::Vertex, gpu::Gpu, material::{Material, SimpleMaterial}, mesh::Mesh
};

#[repr(C, packed)]
#[derive(Copy, Clone, NoUninit)]
struct ModelUniform {
    pub model: Mat4,
    pub normal: Mat4
}

// TODO - Generalize this to multiple materials
pub struct Model {
    pub mesh: Mesh,
    pub material: Box<dyn Material>,
    model_uniform: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup
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

    pub fn new(gpu: &Gpu, mesh: Mesh, material: Box<dyn Material>) -> Self {
        let model_uniform = gpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Model uniform buffer"),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            size: size_of::<ModelUniform>() as u64,
            mapped_at_creation: false
        });
        
        let model_uniform_layout = gpu.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: "Model uniform variables layout".into(),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None
                },
                count: None
            }]
        });

        let bind_group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: "Model uniform bind group".into(),
            layout: &model_uniform_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &model_uniform,
                        offset: 0,
                        size: NonZero::new(size_of::<ModelUniform>() as u64)
                    })
                }
            ]
        });

        Self {
            mesh,
            material,
            model_uniform,
            bind_group
        }
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

            objs.push(Self::new(&gpu, mesh, material));
        }

        let result = Object::empty();
        for model in objs {
            result.add_child(Object::new(ObjectData::Model(Rc::new(model)), Mat4::default()));
        }

        Ok(result)
    }

    pub fn update_model_uniform(&self, gpu: &Gpu, xform: glam::Mat4) {
        let uniform_data = ModelUniform {
            model: xform,
            normal: xform.inverse().transpose()
        };

        gpu.queue.write_buffer(&self.model_uniform, 0, bytemuck::bytes_of(&uniform_data));
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
    pub fn new(data: ObjectData, xform: Mat4) -> Self {
        Self(Rc::new(RefCell::new(ObjectInternal {
            data,
            xform,
            parent: Weak::new(),
            children: Vec::new()
        })))
    }

    pub fn empty() -> Self {
        Self::new(ObjectData::Empty, Mat4::default())
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
