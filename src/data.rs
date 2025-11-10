use bytemuck::NoUninit;

#[repr(C)]
#[derive(Copy, Clone, Debug, NoUninit, Default)]
pub struct Vertex {
    pub pos: [f32; 3],
    pub tangent: [f32; 3],
    pub bitangent: [f32; 3],
    pub normal: [f32; 3],
    pub uv: [f32; 2],
}
