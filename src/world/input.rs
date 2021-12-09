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
pub struct PlayerInput {
    active_controls: Option<Box<dyn ControlScheme>>,
}

impl PlayerInput {
    fn handle_device_event(&mut self, event: DeviceEvent) {
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
