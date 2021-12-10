mod camera;
mod library;
pub mod components;
pub mod input;
pub mod spellcaster;

use std::collections::HashMap;
use std::time::Instant;

use cgmath::{Matrix4, Vector3};
use winit::event::DeviceEvent;

use crate::code::VariableScope;
use crate::transform::{Transform, TransformExtensions};
use crate::triangle_draw::{TriangleDraw, TriangleDrawSystem, TriangleDrawable, TriangleMaterialHandle};
use camera::CameraSystem;
use input::InputSystem;
use components::{ComponentSystem, avatar::{AvatarComponent, AvatarId}};
use library::AssetLibrary;
use spellcaster::{Spellcaster, SpellContext};

struct WorldTime {
    last_frame: Instant,
}

pub struct Globals {
    player_avatar: Option<AvatarId>,
    default_terrain_material: TriangleMaterialHandle,
}

pub struct World {
    time: WorldTime,
    input: InputSystem,
    camera: CameraSystem,
    assets: AssetLibrary,
    components: ComponentSystem,
    spellcaster: Spellcaster,
    globals: Globals,
}

impl World {
    pub fn new(draw_system: &TriangleDrawSystem) -> World {
        let mut assets = AssetLibrary::new();
        assets.create_standard_assets(draw_system);
        let default_terrain_material = assets.get_material("green").unwrap();
        World {
            time: WorldTime { last_frame: Instant::now() },
            input: InputSystem::new(),
            camera: CameraSystem::new(draw_system.device()),
            assets,
            components: ComponentSystem::default(),
            spellcaster: Spellcaster::default(),
            globals: Globals {
                player_avatar: None,
                default_terrain_material,
            },
        }
    }
    pub fn init(&mut self, draw_system: &TriangleDrawSystem) {
        let avatar = TriangleDrawable {
            meshes: Vec::new(),
            transform: Transform::identity(),
        };
        let cube_id = self.components.drawables.add(avatar);
        self.globals.player_avatar = Some(self.components.avatars.add(AvatarComponent::new_flying(cube_id)));

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

        let global_variables = HashMap::new();
        let global_scope = VariableScope::new(&global_variables);
        let startup_code = match crate::syntax::parse_code_file("input/startup.txt", global_scope) {
            Ok(code) => code,
            Err(error) => {
                println!("{}", error);
                panic!("failed parsing code");
            }
        };
        let mut spell_context = SpellContext { components: &mut self.components, globals: &mut self.globals };
        for item in startup_code {
            self.spellcaster.apply_value(&mut spell_context, item);
        }
    }
    pub fn handle_device_event(&mut self, event: DeviceEvent) {
        self.input.handle_device_event(event);
    }
    pub fn update(&mut self, draw_system: &TriangleDrawSystem) {
        let now = Instant::now();
        let delta_time = now.duration_since(self.time.last_frame).as_secs_f64();
        self.time.last_frame = now;

        // update avatar
        if let Some(avatar) = self.globals.player_avatar.and_then(|id| self.components.avatars.get_mut(id)) {
            self.input.player().update_avatar(&mut avatar.controls);
        }

        // update world
        self.components.update(&self.globals, draw_system, delta_time);

        // cast spells
        let mut spell_context = SpellContext { components: &mut self.components, globals: &mut self.globals };
        for binding in self.input.player().spells().get_spellcasts() {
            self.spellcaster.cast_bound_spell(&mut spell_context, binding);
        }
    }
    pub fn camera_frame(&mut self, viewport_dimensions: [u32; 2]) -> Matrix4<f32> {
        self.camera.set_viewport_dimensions(viewport_dimensions);
        if let Some(avatar) = self.globals.player_avatar.and_then(|id| self.components.avatars.get(id)) {
            self.camera.set_camera_transform(self.components.drawables.get(avatar.parent()).unwrap().transform);
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
