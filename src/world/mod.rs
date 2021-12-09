mod camera;
mod library;
pub mod components;
pub mod input;

use std::time::Instant;

use cgmath::{Matrix4, One, Quaternion, Vector3};
use winit::event::DeviceEvent;

use crate::transform::{Transform, TransformExtensions};
use crate::triangle_draw::{TriangleDraw, TriangleDrawSystem, TriangleDrawable};
use camera::CameraSystem;
use input::InputSystem;
use components::{ComponentSystem, avatar::AvatarControls};
use library::AssetLibrary;

pub trait WorldSystem {
    fn update(&mut self, draw_system: &TriangleDrawSystem, delta_time: f64);
    fn render(&self, renderer: &mut TriangleDraw);
}

struct WorldTime {
    last_frame: Instant,
}

pub struct World {
    time: WorldTime,
    input: InputSystem,
    camera: CameraSystem,
    assets: AssetLibrary,
    components: ComponentSystem,
}

impl World {
    pub fn new(draw_system: &TriangleDrawSystem) -> World {
        let mut assets = AssetLibrary::new();
        assets.create_standard_assets(draw_system);
        World {
            time: WorldTime { last_frame: Instant::now() },
            input: InputSystem::new(),
            camera: CameraSystem::new(draw_system.device()),
            assets,
            components: ComponentSystem::default(),
        }
    }
    pub fn init(&mut self, draw_system: &TriangleDrawSystem) {
        let avatar = TriangleDrawable {
            meshes: Vec::new(),
            transform: Transform::identity(),
        };
        let cube_id = self.components.drawables.add(avatar);
        let cube_id = self.components.avatars.add(cube_id, AvatarControls::new_flying());
        self.components.avatars.set_player_avatar(Some(cube_id));

        let cube_mesh = self.assets.get_mesh("cube").unwrap();
        let cube = TriangleDrawable {
            meshes: vec![(self.assets.get_material("white").unwrap(), cube_mesh.clone())],
            transform: Transform::from_translation(Vector3::new(4.0, 0.0, 0.0)),
        };
        self.components.drawables.add(cube);
        let cube = TriangleDrawable {
            meshes: vec![(self.assets.get_material("white").unwrap(), cube_mesh.clone())],
            transform: Transform::from_translation(Vector3::new(-4.0, 0.0, 0.0)),
        };
        self.components.drawables.add(cube);
    }
    pub fn handle_device_event(&mut self, event: DeviceEvent) {
        self.input.handle_device_event(event);
    }
    pub fn update(&mut self, draw_system: &TriangleDrawSystem) {
        let now = Instant::now();
        let delta_time = now.duration_since(self.time.last_frame).as_secs_f64();
        self.time.last_frame = now;
        if let Some(avatar_controls) = self.components.avatars.player_avatar().and_then(|id| self.components.avatars.get_mut(id)) {
            self.input.player().update_avatar(avatar_controls);
        }
        self.components.update(draw_system, delta_time);
    }
    pub fn camera_frame(&mut self, viewport_dimensions: [u32; 2]) -> Matrix4<f32> {
        self.camera.set_viewport_dimensions(viewport_dimensions);
        if let Some(drawable_id) = self.components.avatars.player_avatar().and_then(|id| self.components.avatars.get_parent(id)) {
            self.camera.set_camera_transform(self.components.drawables.get(drawable_id).unwrap().transform);
        } else {
            // TODO what should be rendered when no avatar is set?
            self.camera.set_camera_transform(Transform::identity());
        }
        self.camera.world_to_framebuffer()
    }
    pub fn render(&mut self, renderer: &mut TriangleDraw) {
        renderer.set_camera(self.camera.frame_data());
        self.components.render(renderer);
    }
}
