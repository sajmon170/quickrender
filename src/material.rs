use std::{default::Default, mem::size_of, path::Path};
use crate::{camera::Camera, data::Vertex, globals::Globals, gpu::Gpu, object::Model};
use wgpu::{Extent3d, TexelCopyBufferLayout};

pub trait Material {
    fn as_gpu<'a>(&'a self, globals: &'a Globals, camera: &'a Camera, model: &'a Model) -> GpuMaterial<'a>;
}

pub struct SimpleMaterial {
    pipeline: wgpu::RenderPipeline,
    texture: wgpu::Texture,
    sampler: wgpu::Sampler,
    bind_group: wgpu::BindGroup
}

impl Material for SimpleMaterial {
    fn as_gpu<'a>(&'a self, globals: &'a Globals, camera: &'a Camera, model: &'a Model) -> GpuMaterial {
        GpuMaterial {
            pipeline: &self.pipeline,
            bind_groups: vec![
                (0, &globals.bind_group),
                (1, &camera.bind_group),
                (2, &model.bind_group),
                (3, &self.bind_group)
            ]
        }
    }
}

pub struct GpuMaterial<'a> {
    pipeline: &'a wgpu::RenderPipeline,
    bind_groups: Vec<(u32, &'a wgpu::BindGroup)>
}

impl<'a> GpuMaterial<'a> {
    pub fn setup(&self, render_pass: &mut wgpu::RenderPass) {
        render_pass.set_pipeline(self.pipeline);
        for (idx, bind_group) in &self.bind_groups {
            render_pass.set_bind_group(*idx, *bind_group, &[]);
        }
    }
}

impl SimpleMaterial {
    fn get_texture_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: "Simple material textures bind group layout".into(),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
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
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None
                }
            ]
        })
    }

    fn get_pipeline_layout(device: &wgpu::Device, textures_group_layout: &wgpu::BindGroupLayout) -> wgpu::PipelineLayout {
        let simple_entries = [
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None
                },
                count: None
            }
        ];

        let global_uniform_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: "Global uniform variables layout".into(),
            entries: &simple_entries
        });

        let camera_uniform_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: "Camera uniform variables layout".into(),
            entries: &simple_entries
        });

        let model_uniform_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: "Model uniform variables layout".into(),
            entries: &simple_entries
        });

        device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: "Uniform buffer layout".into(),
            bind_group_layouts: &[
                &global_uniform_layout,
                &camera_uniform_layout,
                &model_uniform_layout,
                &textures_group_layout
            ],
            push_constant_ranges: &[]
        })
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
        let texture_bind_group_layout = Self::get_texture_bind_group_layout(
            &gpu.device
        );

        let pipeline = gpu.get_render_pipelines()
            .entry("Simple texture pipeline".into())
            .or_insert_with(|| {
                let pipeline_layout = Self::get_pipeline_layout(
                    &gpu.device,
                    &texture_bind_group_layout
                );
                Self::make_pipeline(&gpu.device, &gpu.config, &pipeline_layout)
            })
            .clone();

        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let normal_view = normal_map.create_view(&wgpu::TextureViewDescriptor::default());

        let sampler = gpu.device.create_sampler(&wgpu::SamplerDescriptor {
            label: "Simple texture sampler".into(),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let bind_group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: "Simple material texture data bind group".into(),
            layout: &texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture_view)
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&normal_view)
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&sampler)
                }
            ]
        });

        Self {
            pipeline,
            texture,
            sampler,
            bind_group,
        }
    }
}
