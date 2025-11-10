use std::path::Path;

use glam::Mat4;
use gltf::camera::{Perspective, Projection};
use gltf::mesh::util::{ReadNormals, ReadPositions};
use image::RgbaImage;
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

    fn parse_gltf_camera(gpu: &Gpu, store: &mut DataStore, perspective: Perspective) -> Option<Object> {
        let fov = perspective.yfov();
        let far = perspective.zfar().unwrap_or(Camera::DEFAULT_FAR);
        let near = perspective.znear();

        Some(Camera::new_custom(gpu, store, fov, near, far))
    }

    fn parse_gltf_mesh(
        gpu: &Gpu,
        store: &mut DataStore,
        mesh: gltf::Mesh,
        buffers: &Vec<gltf::buffer::Data>,
        images: &Vec<gltf::image::Data>,
    ) -> Option<Object> {
        let mut children: Vec<Object> = Vec::new();
        for primitive in mesh.primitives() {
            let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));
            let indices: Vec<u32> = reader.read_indices()?.into_u32().collect();
            let positions: Vec<[f32; 3]> = match reader.read_positions()? {
                ReadPositions::Standard(pos) => pos.collect(),
                ReadPositions::Sparse(pos) => pos.collect(),
            };
            let normals: Vec<[f32; 3]> = match reader.read_normals()? {
                ReadNormals::Standard(normals) => normals.collect(),
                ReadNormals::Sparse(normals) => normals.collect(),
            };
            let uv: Vec<[f32; 2]> = reader.read_tex_coords(0)?.into_f32().collect();

            let mut vertices: Vec<_> = positions
                .into_iter()
                .zip(normals)
                .zip(uv)
                .map(|((pos, normal), uv)| Vertex {
                    pos: [pos[0], -pos[2], pos[1]],
                    normal,
                    uv,
                    ..Default::default()
                })
                .collect();

            for point_idx in indices.chunks_exact(3) {
                let (a, b, c) = Self::fill_tangents(
                    vertices[point_idx[0] as usize],
                    vertices[point_idx[1] as usize],
                    vertices[point_idx[2] as usize],
                );

                vertices[point_idx[0] as usize] = a;
                vertices[point_idx[1] as usize] = b;
                vertices[point_idx[2] as usize] = c;
            }

            let idx = primitive
                .material()
                .pbr_metallic_roughness()
                .base_color_texture()?
                .texture()
                .index();
            let data = &images[idx];
            let texture_rgba = RgbaImage::from_raw(
                data.width, data.height, data.pixels.clone()
            )?;

            let idx = primitive
                .material()
                .normal_texture()?
                .texture()
                .index();
            let data = &images[idx];
            let normal_rgba = RgbaImage::from_raw(
                data.width, data.height, data.pixels.clone()
            )?;

            let material = Box::new(SimpleMaterial::new(&gpu, &texture_rgba, &normal_rgba));
            let mesh = Mesh::new(gpu, vertices, indices);
            let model = Self::new(gpu, mesh, material);
            let obj = Object::new(model, store);
            children.push(obj);
        }
        Some(Object::empty().with_children(children))
    }

    fn parse_node(
        gpu: &Gpu,
        store: &mut DataStore,
        node: gltf::Node,
        buffers: &Vec<gltf::buffer::Data>,
        images: &Vec<gltf::image::Data>,
    ) -> Option<Object> {
        let children: Vec<_> = node
            .children()
            .flat_map(|child| Self::parse_node(gpu, store, child, buffers, images))
            .collect();
        
        let obj = if let Some(camera) = node.camera()
            && let Projection::Perspective(perspective) = camera.projection()
        {
            Self::parse_gltf_camera(gpu, store, perspective)
        } else if let Some(mesh) = node.mesh() {
            Self::parse_gltf_mesh(gpu, store, mesh, buffers, images)
        } else {
            None
        };

        obj.map(|mut object| {
            let matrix = node.transform().matrix();
            object.set_xform(Mat4::from_cols_array_2d(&matrix));
            object.add_children(children);
            object
        })
    }

    // Note: This should return a full scene
    // Maybe move this to Scene instead?
    pub fn load_gltf(gpu: &Gpu, store: &mut DataStore, path: &Path) -> gltf::Result<Object> {
        let (gltf, buffers, images) = gltf::import(path)?;
        let scene = gltf.scenes().next().unwrap();

        let objs: Vec<_> = scene
            .nodes()
            .filter_map(|node| Self::parse_node(gpu, store, node, &buffers, &images))
            .collect();

        Ok(Object::empty().with_children(objs))
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

            let texture_bytes = std::fs::read(texture_path).unwrap();
            let texture_rgba = image::load_from_memory(&texture_bytes).unwrap().to_rgba8();
            let normal_bytes = std::fs::read(normal_path).unwrap();
            let normal_rgba = image::load_from_memory(&texture_bytes).unwrap().to_rgba8();

            let material = Box::new(SimpleMaterial::new(&gpu, &texture_rgba, &normal_rgba));

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
