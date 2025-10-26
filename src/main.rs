mod gpu;
mod renderer;
mod object;
mod material;
mod mesh;
mod model;
mod data;
mod camera;
mod scene;
mod globals;

use std::{path::Path, rc::Rc};

use camera::Camera;
use glam::{Mat4, Vec3};
use object::{Object, ObjectData};
use model::Model;
use scene::Scene;
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::WindowEvent,
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

use crate::{
    gpu::Gpu,
    renderer::Renderer,
};

#[derive(Default)]
struct App {
    renderer: Option<Renderer>
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let size = PhysicalSize::new(640, 480);

        let attrs = Window::default_attributes()
            .with_inner_size(size.clone())
            .with_resizable(false);

        let window = event_loop.create_window(attrs).unwrap();
        let gpu = pollster::block_on(Gpu::new(window, size)).unwrap();

        let scene = Scene::new(vec![
            Model::load_obj(&gpu, &Path::new("src/res/models/sus/sus.obj"))
                .unwrap()
                .with_rotation_x(-2.0 * std::f32::consts::PI / 4.0),
            Camera::new(&gpu)
                .with_translation(Vec3::new(-2.0, 0.0, 6.0))
        ]);

        self.renderer = Some(Renderer::new(gpu, scene));
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        let renderer = if let Some(renderer) = &mut self.renderer {
            renderer
        } else {
            return;
        };

        match event {
            WindowEvent::CloseRequested => {
                println!("Closing window.");
                event_loop.exit();
            }
            WindowEvent::Resized(size) => {
                renderer.resize(size);
            }
            WindowEvent::RedrawRequested => {
                renderer.render().unwrap();
            }
            _ => (),
        }
    }
}

fn main() {
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);
    event_loop.run_app(&mut App::default()).unwrap();
}
