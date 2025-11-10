use bytemuck::NoUninit;
use glam::{Mat4, Vec3};
use std::num::NonZero;

use crate::{
    gpu::Gpu,
    object::{DataStore, Object},
};

#[derive(Clone)]
pub struct Camera {
    pub fov: f32,
    pub near: f32,
    pub far: f32,
    pub yaw: f32,
    pub pitch: f32,
    uniform_buffer: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
}

#[repr(C, packed)]
#[derive(Copy, Clone, NoUninit)]
pub struct CameraUniform {
    pub projection: Mat4,
    pub view: Mat4,
    pub camera_pos: Vec3,
    _padding: f32,
}

impl Camera {
    pub fn new(gpu: &Gpu, store: &mut DataStore) -> Object {
        let uniform_buffer = gpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Camera uniform buffer"),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            size: size_of::<CameraUniform>() as u64,
            mapped_at_creation: false,
        });

        let camera_uniform_layout =
            gpu.device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: "Camera uniform variables layout".into(),
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
            label: "Camera uniform bind group".into(),
            layout: &camera_uniform_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &uniform_buffer,
                    offset: 0,
                    size: NonZero::new(size_of::<CameraUniform>() as u64),
                }),
            }],
        });

        let camera = Self {
            fov: 45.0 * std::f32::consts::PI / 180.0,
            near: 0.01,
            far: 100.0,
            // TODO - read this from constructor
            yaw: 0.0,
            pitch: 0.0,
            uniform_buffer,
            bind_group,
        };

        Object::new(camera, store)
    }

    pub fn update_camera_uniform(&self, gpu: &Gpu, xform: glam::Mat4, ratio: f32) {
        let uniform_data = CameraUniform {
            projection: self.get_projection_matrix(ratio),
            view: xform.inverse(),
            camera_pos: xform.to_scale_rotation_translation().2,
            _padding: Default::default(),
        };

        gpu.queue
            .write_buffer(&self.uniform_buffer, 0, bytemuck::bytes_of(&uniform_data));
    }

    fn get_projection_matrix(&self, ratio: f32) -> Mat4 {
        Mat4::perspective_lh(self.fov, ratio, self.near, self.far)
    }
}
