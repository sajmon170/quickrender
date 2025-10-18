use std::{default::Default, mem::size_of, num::NonZero, path::Path};
use crate::{data::Vertex, gpu::Gpu};
use bytemuck::NoUninit;
use glam::{Mat4, Vec3};
use wgpu::{Extent3d, TexelCopyBufferLayout};

// TODO - refactor camera position out of this
pub trait Material {
    fn set_render_pass(&self, render_pass: &mut wgpu::RenderPass, queue: &wgpu::Queue, data: &ExternalData);
}

#[repr(C, packed)]
#[derive(Copy, Clone, NoUninit)]
struct UniformData {
    pub projection: Mat4,
    pub view: Mat4,
    pub model: Mat4,
    pub normal: Mat4,
    pub camera_pos: Vec3,
    pub time: f32,
}

struct ExternalData {
    start_time: std::time::Instant,
    projection: Mat4,
    view: Mat4,
    model: Mat4,
    camera_pos: Vec3
}

pub struct SimpleMaterial {
    pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
    uniform_buffer: wgpu::Buffer,
    texture: wgpu::Texture,
}

impl SimpleMaterial {
    // TODO - refactor texture inputs into another struct
    fn setup_bind_group(device: &wgpu::Device, uniform_buffer: &wgpu::Buffer,
                        texture: &wgpu::Texture, normal_map: &wgpu::Texture)
                        -> (wgpu::BindGroup, wgpu::PipelineLayout) {

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: "Simple material bind group layout".into(),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None
                    },
                    count: None
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float {
                            filterable: true
                        },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false
                    },
                    count: None
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float {
                            filterable: true
                        },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false
                    },
                    count: None
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None
                }
            ]
        });


        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: "Uniform buffer layout".into(),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[]
        });

        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let normal_view = normal_map.create_view(&wgpu::TextureViewDescriptor::default());
        
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: "Simple texture sampler".into(),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: "Bind group".into(),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &uniform_buffer,
                        offset: 0,
                        size: NonZero::new(size_of::<UniformData>() as u64)
                    })
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&texture_view)
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&normal_view)
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::Sampler(&sampler)
                }
            ]
        });

        (bind_group, pipeline_layout)
    }
 
    fn make_pipeline(device: &wgpu::Device,
                     config: &wgpu::SurfaceConfiguration,
                     pipeline_layout: &wgpu::PipelineLayout) -> wgpu::RenderPipeline {
        let shader_module = device.create_shader_module(
            wgpu::include_wgsl!("shaders/simple.wgsl")
        );

        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Triangle render"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader_module,
                entry_point: Some("vs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[
                    wgpu::VertexBufferLayout {
                        array_stride: size_of::<Vertex>() as u64,
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &wgpu::vertex_attr_array![
                            0 => Float32x3,
                            1 => Float32x3,
                            2 => Float32x3,
                            3 => Float32x3,
                            4 => Float32x2
                        ]
                    }
                ]
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None, //Some(wgpu::Face::Back),
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader_module,
                entry_point: Some("fs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL
                })],
            }),
            depth_stencil: Some(wgpu::DepthStencilState {
                // TODO - grab this info from outside
                format: wgpu::TextureFormat::Depth24Plus,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default()
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0u64,
                alpha_to_coverage_enabled: false
            },
            multiview: None,
            cache: None
        })
    }

    fn make_uniform_buffer(device: &wgpu::Device) -> wgpu::Buffer {
        let descriptor = wgpu::BufferDescriptor {
            label: "Uniform buffer".into(),
            // TODO: round the size to a multiple of 4 f32 values
            size: size_of::<UniformData>() as u64,
            mapped_at_creation: false,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM
        };

        device.create_buffer(&descriptor)
    }

    fn make_texture(device: &wgpu::Device, queue: &wgpu::Queue,
                    path: &Path, format: wgpu::TextureFormat) -> wgpu::Texture {
        let texture_bytes = std::fs::read(path).unwrap();
        let texture_rgba = image::load_from_memory(&texture_bytes).unwrap()
            .to_rgba8();
        let (tex_width, tex_height) = texture_rgba.dimensions();
        let extent = Extent3d {
            width: tex_width,
            height: tex_height,
            depth_or_array_layers: 1
        };
        
        let descriptor = wgpu::TextureDescriptor {
            label: "Simple texture".into(),
            dimension: wgpu::TextureDimension::D2,
            size: extent,
            format,
            sample_count: 1,
            mip_level_count: 1,
            usage: wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[]
        };

        let texture = device.create_texture(&descriptor);

        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All
            },
            &texture_rgba,
            TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(tex_width * 4),
                rows_per_image: Some(tex_height)
            },
            extent
        );

        texture
    }
    
    pub fn new(gpu: &Gpu, texture_path: &Path, normal_path: &Path) -> Self {
        let uniform_buffer = Self::make_uniform_buffer(&gpu.device);
        let texture = Self::make_texture(
            &gpu.device,
            &gpu.queue,
            texture_path,
            wgpu::TextureFormat::Rgba8UnormSrgb
        );
        let normal_map = Self::make_texture(
            &gpu.device,
            &gpu.queue,
            normal_path,
            wgpu::TextureFormat::Rgba8Unorm
        );
        let (bind_group, pipeline_layout) = Self::setup_bind_group(
            &gpu.device,
            &uniform_buffer,
            &texture,
            &normal_map
        );
        let pipeline = Self::make_pipeline(&gpu.device, &gpu.config, &pipeline_layout);

        Self {
            bind_group,
            uniform_buffer,
            pipeline,
            texture,
        }
    }
}

impl Material for SimpleMaterial {
    fn set_render_pass(&self, render_pass: &mut wgpu::RenderPass, queue: &wgpu::Queue, data: &ExternalData) {
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.bind_group, &[]);

        // TODO - set uniform buffers somewhere else instead
        let time = std::time::Instant::now()
            .duration_since(data.start_time)
            .as_secs_f32();

        let uniform_data = UniformData {
            projection: data.projection,
            view: data.view,
            model: data.model,
            normal: data.model.inverse().transpose(),
            camera_pos: data.camera_pos,
            time,
        };

        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::bytes_of(&uniform_data));
    }
}
