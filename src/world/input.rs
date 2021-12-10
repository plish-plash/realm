use cgmath::{Vector2, Vector3};
use winit::event::{DeviceEvent, ElementState, KeyboardInput};

use super::components::avatar::AvatarControls;

pub struct FlyingControls {
    pub move_input: Vector3<f32>,
    pub look_input: Vector2<f32>,
}

impl Default for FlyingControls {
    fn default() -> FlyingControls {
        FlyingControls {
            move_input: Vector3::new(0.0, 0.0, 0.0),
            look_input: Vector2::new(0.0, 0.0),
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct ButtonPair(ElementState, ElementState);

impl Default for ButtonPair {
    fn default() -> ButtonPair {
        ButtonPair(ElementState::Released, ElementState::Released)
    }
}

impl From<ButtonPair> for f32 {
    fn from(pair: ButtonPair) -> f32 {
        let neg = match pair.0 {
            ElementState::Pressed => 1.0,
            ElementState::Released => 0.0,
        };
        let pos = match pair.1 {
            ElementState::Pressed => 1.0,
            ElementState::Released => 0.0,
        };
        pos - neg
    }
}

trait ControlScheme {
    fn handle_device_event(&mut self, event: DeviceEvent);
    fn update_avatar(&mut self, avatar: &mut AvatarControls) -> bool;
}

#[derive(Default)]
struct KeyboardFlyingControlScheme {
    x_keys: ButtonPair,
    z_keys: ButtonPair,
    mouse_delta: (f64, f64),
}

impl ControlScheme for KeyboardFlyingControlScheme {
    fn handle_device_event(&mut self, event: DeviceEvent) {
        match event {
            // Q
            DeviceEvent::Key(KeyboardInput { scancode: 16, .. }) => (),
            // W
            DeviceEvent::Key(KeyboardInput { scancode: 17, state, .. }) => { self.z_keys.0 = state; }
            // E
            DeviceEvent::Key(KeyboardInput { scancode: 18, .. }) => (),
            // A
            DeviceEvent::Key(KeyboardInput { scancode: 30, state, .. }) => { self.x_keys.0 = state; }
            // S
            DeviceEvent::Key(KeyboardInput { scancode: 31, state, .. }) => { self.z_keys.1 = state; }
            // D
            DeviceEvent::Key(KeyboardInput { scancode: 32, state, .. }) => { self.x_keys.1 = state; }
            // Mouse
            DeviceEvent::MouseMotion { delta } => {
                self.mouse_delta.0 += delta.0;
                self.mouse_delta.1 += delta.1;
            }

            _ => (),
        }
    }
    fn update_avatar(&mut self, avatar: &mut AvatarControls) -> bool {
        match avatar {
            AvatarControls::Flying(controls, _) => {
                *controls = FlyingControls {
                    move_input: Vector3::new(self.x_keys.into(), 0.0, self.z_keys.into()),
                    look_input: Vector2::new(self.mouse_delta.0 as f32, self.mouse_delta.1 as f32),
                };
                self.mouse_delta = (0.0, 0.0);
                true
            }
            // _ => false,
        }
    }
}

#[derive(Default)]
pub struct SpellcastControlScheme {
    keys: [(bool, bool); 10],
}

impl SpellcastControlScheme {
    fn handle_device_event(&mut self, event: DeviceEvent) {
        match event {
            DeviceEvent::Key(KeyboardInput { scancode, state, .. }) => {
                if scancode >= 2 && scancode <= 11 {
                    let binding = if scancode == 11 { 0 } else { scancode - 1 };
                    let state = state == ElementState::Pressed;
                    let key_state = &mut self.keys[binding as usize];
                    key_state.0 |= state && !key_state.1;
                    key_state.1 = state;
                }
            }
            _ => (),
        }
    }
    pub fn get_spellcasts<'a>(&'a mut self) -> impl std::iter::Iterator<Item=u8> + 'a {
        self.keys.iter_mut().enumerate().filter_map(|(idx, (trigger, _))| {
            if *trigger {
                *trigger = false;
                Some(idx as u8)
            } else { None }
        })
    }
}

#[derive(Default)]
pub struct PlayerInput {
    active_controls: Option<Box<dyn ControlScheme>>,
    spellcast_controls: SpellcastControlScheme,
}

impl PlayerInput {
    fn handle_device_event(&mut self, event: DeviceEvent) {
        self.spellcast_controls.handle_device_event(event.clone());
        if let Some(controls) = &mut self.active_controls {
            controls.handle_device_event(event);
        }
    }
    pub fn update_avatar(&mut self, avatar: &mut AvatarControls) {
        if let Some(controls) = &mut self.active_controls {
            if !controls.update_avatar(avatar) {
                self.active_controls = None;
            }
        }
        if self.active_controls.is_none() {
            match avatar {
                AvatarControls::Flying(_, _) => self.active_controls = Some(Box::new(KeyboardFlyingControlScheme::default())),
            }
        }
    }
    pub fn spells(&mut self) -> &mut SpellcastControlScheme {
        &mut self.spellcast_controls
    }
}

pub struct InputSystem {
    player: PlayerInput,
}

impl InputSystem {
    pub fn new() -> InputSystem {
        InputSystem { player: PlayerInput::default() }
    }

    pub fn player(&mut self) -> &mut PlayerInput { &mut self.player }

    pub fn handle_device_event(&mut self, event: DeviceEvent) {
        self.player.handle_device_event(event);
    }
}
