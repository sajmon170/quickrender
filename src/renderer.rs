use crate::{globals::Globals, gpu::Gpu, material::SimpleMaterial, mesh::Mesh, object::{Object, ObjectData}, scene::Scene, model::Model};
use winit::dpi::PhysicalSize;
use anyhow::Result;
use glam::{Vec2, Vec3};
use std::{ops::Deref, path::Path};

pub struct Renderer {
    gpu: Gpu,
    scene: Scene,
    globals: Globals
}

impl Renderer {
    pub fn render(&mut self) -> Result<()> {
        self.globals.update_globals(&self.gpu);
        self.gpu.render(|render_pass| {
            for (obj, xform) in self.scene.root.get_all() {
                match obj.get().deref() {
                    ObjectData::Model(model) => {
                        model.update_model_uniform(&self.gpu, xform);
                        model.material
                            .as_gpu(&self.globals, &self.scene.get_camera(), &model)
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

    pub fn new(gpu: Gpu, scene: Scene) -> Self {
        let globals = Globals::new(&gpu);
        Self { gpu, scene, globals }
    }

    pub fn resize(&mut self, size: PhysicalSize<u32>) {
        self.gpu.resize(size);
    }
}
