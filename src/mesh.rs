use std::mem::size_of;
use crate::{Gpu, data::Vertex};

pub struct Mesh {
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    vtx_count: u32
}

impl Mesh {
    fn make_vertex_buffer(device: &wgpu::Device, vtx: &[Vertex]) -> wgpu::Buffer {
        let descriptor = wgpu::BufferDescriptor {
            label: "Vertex buffer".into(),
            size: (vtx.len() * size_of::<Vertex>()) as u64,
            mapped_at_creation: false,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::VERTEX
        };

        device.create_buffer(&descriptor)
    }

    fn make_index_buffer(device: &wgpu::Device, idx: &[u32]) -> wgpu::Buffer {
        let descriptor = wgpu::BufferDescriptor {
            label: "Index buffer".into(),
            size: (idx.len() * size_of::<u32>()) as u64,
            mapped_at_creation: false,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::INDEX
        };

        device.create_buffer(&descriptor)
    }

    pub fn new(gpu: &Gpu, vertices: Vec<Vertex>, indices: Vec<u32>) -> Self {
        let vertex_buffer = Self::make_vertex_buffer(&gpu.device, &vertices);
        gpu.queue.write_buffer(&vertex_buffer, 0, &bytemuck::cast_slice(&vertices));
        let index_buffer = Self::make_index_buffer(&gpu.device, &indices);
        gpu.queue.write_buffer(&index_buffer, 0, &bytemuck::cast_slice(&indices));
        
        Self {
            vertex_buffer,
            index_buffer,
            vtx_count: indices.len() as u32,
        }
    }

    pub fn set_render_pass(&self, render_pass: &mut wgpu::RenderPass) {
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        render_pass.draw_indexed(0..self.vtx_count, 0, 0..1);
    }
}
