use crate::{globals::Globals, gpu::Gpu, material::SimpleMaterial, mesh::Mesh, object::{Object, ObjectData}, scene::Scene, model::Model};
use winit::dpi::PhysicalSize;
use anyhow::Result;
use glam::{Vec2, Vec3};
use std::{ops::Deref, path::Path};

pub struct Renderer {
    gpu: Gpu,
    globals: Globals
}

impl Renderer {
    pub fn render(&mut self, scene: &Scene) -> Result<()> {
        self.globals.update_globals(&self.gpu);
        self.gpu.render(|render_pass| {
            for (obj, xform) in scene.root.get_all() {
                match obj.get().deref() {
                    ObjectData::Model(model) => {
                        model.update_model_uniform(&self.gpu, xform);
                        model.material
                            .as_gpu(&self.globals, &scene.get_camera().unwrap(), &model)
                            .setup(render_pass);
                        model.mesh.set_render_pass(render_pass);
                    },
                    ObjectData::Camera(camera) => {
                        camera.update_camera_uniform(&self.gpu, xform, 640.0/480.0);
                    },
                    _ => {}
                }
            }
        })
    }

    pub fn new(gpu: Gpu) -> Self {
        let globals = Globals::new(&gpu);
        Self { gpu, globals }
    }

    pub fn resize(&mut self, size: PhysicalSize<u32>) {
        self.gpu.resize(size);
    }
}
