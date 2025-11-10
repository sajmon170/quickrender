use std::path::Path;

use glam::Mat4;
use gltf::camera::{Perspective, Projection};
use gltf::Gltf;
use tobj::LoadError;

use bytemuck::NoUninit;
use std::num::NonZero;

use crate::camera::Camera;
use crate::object::DataStore;
use crate::{
    data::Vertex,
    gpu::Gpu,
    material::{Material, SimpleMaterial},
    mesh::Mesh,
    object::Object,
};

#[repr(C, packed)]
#[derive(Copy, Clone, NoUninit)]
struct ModelUniform {
    pub model: Mat4,
    pub normal: Mat4,
}

// TODO - Generalize this to multiple materials
pub struct Model {
    pub mesh: Mesh,
    pub material: Box<dyn Material>,
    model_uniform: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
}

impl Model {
    fn fill_tangents(mut a: Vertex, mut b: Vertex, mut c: Vertex) -> (Vertex, Vertex, Vertex) {
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
            mapped_at_creation: false,
        });

        let model_uniform_layout =
            gpu.device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: "Model uniform variables layout".into(),
                    entries: &[wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }],
                });

        let bind_group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: "Model uniform bind group".into(),
            layout: &model_uniform_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &model_uniform,
                    offset: 0,
                    size: NonZero::new(size_of::<ModelUniform>() as u64),
                }),
            }],
        });

        Self {
            mesh,
            material,
            model_uniform,
            bind_group,
        }
    }

    fn parse_node(node: gltf::Node, gpu: &Gpu, store: &mut DataStore) -> Option<Object> {
        if let Some(camera) = node.camera()
            && let Projection::Perspective(perspective) = camera.projection() {
                let fov = perspective.yfov();
                let far = perspective.zfar().unwrap_or(Camera::DEFAULT_FAR);
                let near = perspective.znear();

                Some(Camera::new_custom(gpu, store, fov, near, far))
        }
        else {
            None
        }
    }

    // Note: This should return a full scene
    // Maybe move this to Scene instead?
    pub fn load_gltf(gpu: &Gpu, store: &mut DataStore, path: &Path) -> gltf::Result<Vec<Object>> {
        let gltf = Gltf::open(path)?;
        let scene = gltf.scenes().next().unwrap();

        let objs: Vec<_> = scene.nodes()
            .filter_map(|node| Self::parse_node(node, gpu, store))
            .collect();
        
        Ok(objs)
    }

    pub fn load_obj(gpu: &Gpu, store: &mut DataStore, path: &Path) -> Result<Object, LoadError> {
        let (models, materials) = tobj::load_obj(&path, &tobj::GPU_LOAD_OPTIONS)?;
        let materials = materials.unwrap();
        let mut objs = Vec::<Model>::new();

        for model in models.iter() {
            let mut vertices: Vec<_> = model
                .mesh
                .positions
                .chunks_exact(3)
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
                && let Some(diffuse) = &materials[id].diffuse_texture
            {
                diffuse
            } else {
                &"src/res/star.png".into()
            };

            let normal_path = if let Some(id) = model.mesh.material_id
                && let Some(normal) = &materials[id].normal_texture
            {
                if let Some("-bm") = normal.split_whitespace().next() {
                    normal.splitn(3, " ").last().unwrap()
                } else {
                    normal
                }
            } else {
                "src/res/star.png"
            };

            let material = Box::new(SimpleMaterial::new(
                &gpu,
                &Path::new(texture_path),
                &Path::new(normal_path),
            ));

            for point_idx in model.mesh.indices.chunks_exact(3) {
                let (a, b, c) = Self::fill_tangents(
                    vertices[point_idx[0] as usize],
                    vertices[point_idx[1] as usize],
                    vertices[point_idx[2] as usize],
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
            result.add_child(Object::new(model, store));
        }

        Ok(result)
    }

    pub fn update_model_uniform(&self, gpu: &Gpu, xform: glam::Mat4) {
        let uniform_data = ModelUniform {
            model: xform,
            normal: xform.inverse().transpose(),
        };

        gpu.queue
            .write_buffer(&self.model_uniform, 0, bytemuck::bytes_of(&uniform_data));
    }
}
