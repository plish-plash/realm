mod camera;
mod library;
pub mod entity;
pub mod input;

use std::time::Instant;

use cgmath::{Matrix4, One, Quaternion, Vector3};
use winit::event::DeviceEvent;

use crate::transform::{Transform, TransformExtensions};
use crate::triangle_draw::{TriangleDraw, TriangleDrawSystem};
use crate::terrain::{TerrainSystem, BlockyMesh};
use camera::CameraSystem;
use input::InputSystem;
use entity::{Entity, EntitySystem, avatar::AvatarControls};
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
    entities: EntitySystem,
    assets: AssetLibrary,
    terrain: TerrainSystem<BlockyMesh>,
}

impl World {
    pub fn new(draw_system: &TriangleDrawSystem) -> World {
        let mut assets = AssetLibrary::new();
        assets.create_standard_assets(draw_system);
        World {
            time: WorldTime { last_frame: Instant::now() },
            input: InputSystem::new(),
            camera: CameraSystem::new(draw_system.device()),
            entities: EntitySystem::new(),
            assets,
            terrain: TerrainSystem::new(draw_system),
        }
    }
    pub fn init(&mut self, draw_system: &TriangleDrawSystem) {
        let cube_mesh = self.assets.get_mesh("cube").unwrap();
        let cube_avatar = Entity {
            mesh: cube_mesh.clone(),
            material: self.assets.get_material("white").unwrap(),
            transform: Transform::identity(),
        };
        let cube_avatar_id = self.entities.spawn(cube_avatar);
        self.entities.avatar_mut().insert(cube_avatar_id, AvatarControls::new_flying());
        self.entities.avatar_mut().set_player_avatar(Some(cube_avatar_id));

        // let cube = Entity {
        //     mesh: cube_mesh.clone(),
        //     material: self.assets.get_material("white").unwrap(),
        //     transform: Transform::from_translation(Vector3::new(0.0, 0.0, 0.0)),
        // };
        // self.entities.spawn(cube);
        let cube = Entity {
            mesh: cube_mesh.clone(),
            material: self.assets.get_material("white").unwrap(),
            transform: Transform::from_translation(Vector3::new(4.0, 0.0, 0.0)),
        };
        self.entities.spawn(cube);
        let cube = Entity {
            mesh: cube_mesh.clone(),
            material: self.assets.get_material("white").unwrap(),
            transform: Transform::from_translation(Vector3::new(-4.0, 0.0, 0.0)),
        };
        self.entities.spawn(cube);

        // let plant = Entity {
        //     mesh: cube_mesh.clone(),
        //     material: self.assets.get_material("green").unwrap(),
        //     transform: Transform::new(Vector3::new(0.0, 0.0, 0.0), Quaternion::one(), 0.25),
        // };
        // let plant_id = self.entities.spawn(plant);
        // self.entities.flora_mut().insert(plant_id, entity::flora::Flora::new_test());
        let mesh = draw_system.load_mesh(crate::lsystem::test_mesh());
        let plant = Entity {
            mesh,
            material: self.assets.get_material("green").unwrap(),
            transform: Transform::new(Vector3::new(0.0, 0.0, 0.0), Quaternion::one(), 0.25),
        };
        self.entities.spawn(plant);
    }
    pub fn handle_device_event(&mut self, event: DeviceEvent) {
        self.input.handle_device_event(event);
    }
    pub fn update(&mut self, draw_system: &TriangleDrawSystem) {
        let now = Instant::now();
        let delta_time = now.duration_since(self.time.last_frame).as_secs_f64();
        self.time.last_frame = now;
        if let Some(avatar_controls) = self.entities.avatar_mut().player_avatar_controls_mut() {
            self.input.player().update_avatar(avatar_controls);
        }
        self.terrain.update_clip_spheres(self.camera.camera_position());
        //self.terrain.update(draw_system, delta_time);
        self.entities.update(draw_system, delta_time);
    }
    pub fn camera_frame(&mut self, viewport_dimensions: [u32; 2]) -> Matrix4<f32> {
        self.camera.set_viewport_dimensions(viewport_dimensions);
        if let Some(avatar) = self.entities.avatar().player_avatar() {
            self.camera.set_camera_transform(self.entities.get_transform(avatar));
        } else {
            // TODO what should be rendered when no avatar is set?
            self.camera.set_camera_transform(Transform::identity());
        }
        self.camera.world_to_framebuffer()
    }
    pub fn render(&mut self, renderer: &mut TriangleDraw) {
        renderer.set_camera(self.camera.frame_data());
        self.terrain.render(renderer);
        self.entities.render(renderer);
    }
}
