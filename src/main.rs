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
mod physics;

use std::{path::Path, rc::Rc};

use camera::Camera;
use glam::{Mat4, Vec2, Vec3};
use object::{Object, ObjectData};
use model::Model;
use physics::UserInput;
use scene::Scene;
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::{DeviceEvent, KeyEvent, Modifiers, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    keyboard::{Key, KeyCode, KeyLocation, ModifiersKeyState, NamedKey, PhysicalKey},
    window::{CursorGrabMode, Window}
};

use crate::{
    gpu::Gpu,
    renderer::Renderer,
    physics::PhysicsController
};

#[derive(Default)]
struct App {
    renderer: Option<Renderer>,
    scene: Option<Scene>,
    physics: PhysicsController,
    input_modifiers: Modifiers,
    key_event: Option<KeyEvent>,
    mouse_motion: Vec2
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let size = PhysicalSize::new(640, 480);

        let attrs = Window::default_attributes()
            .with_inner_size(size.clone())
            .with_resizable(false);

        let window = event_loop.create_window(attrs).unwrap();
        let _ = window.set_cursor_grab(CursorGrabMode::Confined);
        window.set_cursor_visible(false);
        let gpu = pollster::block_on(Gpu::new(window, size)).unwrap();

        let scene = Scene::new(vec![
            Model::load_obj(&gpu, &Path::new("src/res/models/sus/sus.obj"))
                .unwrap()
                .with_rotation_x(-2.0 * std::f32::consts::PI / 4.0),
            Camera::new(&gpu)
                .with_translation(Vec3::new(-2.0, 0.0, 6.0))
        ]);

        self.scene = Some(scene);
        self.renderer = Some(Renderer::new(gpu));
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                println!("Closing window.");
                event_loop.exit();
            }
            WindowEvent::Resized(size) => {
                if let Some(renderer) = &mut self.renderer {
                    renderer.resize(size);
                }
            }
            WindowEvent::RedrawRequested => {
                let user_input = self.handle_input();

                if let Some(renderer) = &mut self.renderer && let Some(scene) = &mut self.scene {
                    self.physics.update(scene, user_input);
                    renderer.render(&scene).unwrap();
                }
            }
            WindowEvent::ModifiersChanged(modifiers) => {
                self.input_modifiers = modifiers;
            }
            WindowEvent::KeyboardInput { event, .. } => {
                self.key_event = Some(event);
            }
            _ => (),
        }
    }

    fn device_event(
        &mut self,
        _event_loop: &winit::event_loop::ActiveEventLoop,
        _device_id: winit::event::DeviceId,
        event: DeviceEvent,
    ) {
        match event {
            DeviceEvent::MouseMotion { delta } => {
                let motion = Vec2::from([delta.0 as f32, delta.1 as f32]);
                self.mouse_motion += motion;
            },
            _ => {}
        }
    }
}

impl App {
    fn handle_input(&mut self) -> UserInput {
        let mut input = UserInput::default();

        if let Some(key_event) = &self.key_event {
            if let PhysicalKey::Code(code) = key_event.physical_key {
                match code {
                    KeyCode::KeyW => { input.move_forward = true },
                    KeyCode::KeyA => { input.move_left = true },
                    KeyCode::KeyS => { input.move_backward = true },
                    KeyCode::KeyD => { input.move_right = true },
                    KeyCode::Space => { input.move_up = true },
                    KeyCode::KeyC => { input.move_down = true }
                    _ => {}
                }
            }

            self.key_event = None;
        }

        input.yaw = self.mouse_motion[0];
        input.pitch = self.mouse_motion[1];
        self.mouse_motion = Vec2::ZERO;
        println!("Done");

        input
    }
}

fn main() {
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);
    event_loop.run_app(&mut App::default()).unwrap();
}
