use crate::gpu::Gpu;
use bytemuck::NoUninit;
use std::num::NonZero;

pub struct Globals {
    begin: std::time::Instant,
    globals_uniform: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup
}

#[repr(C, packed)]
#[derive(Copy, Clone, NoUninit)]
struct GlobalsUniform {
    pub time: f32,
}

impl Globals {
    pub fn new(gpu: &Gpu) -> Self {
        let globals_uniform = gpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Globals uniform buffer"),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            size: size_of::<GlobalsUniform>() as u64,
            mapped_at_creation: false
        });
        
        let globals_uniform_layout = gpu.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: "Global uniform variables layout".into(),
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
            label: "Global uniform bind group".into(),
            layout: &globals_uniform_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &globals_uniform,
                        offset: 0,
                        size: NonZero::new(size_of::<GlobalsUniform>() as u64)
                    })
                }
            ]
        });
        
        let begin = std::time::Instant::now();

        Self {
            begin,
            globals_uniform,
            bind_group
        }
    }

    pub fn update_globals(&mut self, gpu: &Gpu) {
        let uniform_data = GlobalsUniform {
            time: std::time::Instant::now()
                .duration_since(self.begin)
                .as_secs_f32()
        };

        gpu.queue.write_buffer(&self.globals_uniform, 0, bytemuck::bytes_of(&uniform_data));
    }
}
