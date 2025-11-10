use crate::{globals::Globals, gpu::Gpu, material::SimpleMaterial, mesh::Mesh, model::Model, object::{DataStore, DataToken, Object}, scene::Scene};
use winit::dpi::PhysicalSize;
use anyhow::Result;
use glam::{Vec2, Vec3};
use std::{ops::Deref, path::Path};

pub struct Renderer {
    gpu: Gpu,
    globals: Globals
}

impl Renderer {
    pub fn render(&mut self, scene: &mut Scene, store: &mut DataStore) -> Result<()> {
        self.globals.update_globals(&self.gpu);
        self.gpu.render(|render_pass| {
            for (obj, xform) in scene.root.get_all() {
                match obj.get_data() {
                    DataToken::Model(id) => {
                        // TODO - refactor this unwrap and clone mess
                        let token = scene.get_camera_object().unwrap().get_data();
                        let camera = store.get_camera(token.try_as_camera().unwrap()).unwrap().clone();
                        let model = store.get_model(id).unwrap();
                        model.update_model_uniform(&self.gpu, xform);
                        model.material
                            .as_gpu(&self.globals, &camera, model)
                            .setup(render_pass);
                        model.mesh.set_render_pass(render_pass);
                    },
                    DataToken::Camera(id) => {
                        let camera = store.get_camera(id).unwrap();
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
