use cgmath::{Deg, Euler, Quaternion};

use crate::{transform::QuaternionExtensions, world::input::FlyingControls};

use super::{DrawableId, DrawableComponentList, new_component_list_type};

// TODO make these configurable
const LOOK_SENSITIVITY: f32 = 0.1;
const MOVE_SPEED: f64 = 10.0;

pub enum AvatarControls {
    Flying(FlyingControls, Euler<Deg<f32>>),
}

pub struct AvatarComponent {
    parent: DrawableId,
    pub controls: AvatarControls,
}

impl AvatarComponent {
    pub fn new_flying(parent: DrawableId) -> AvatarComponent {
        AvatarComponent {
            parent,
            controls: AvatarControls::Flying(FlyingControls::default(), Euler::new(Deg(0.0), Deg(0.0), Deg(0.0))),
        }
    }

    pub fn parent(&self) -> DrawableId {
        self.parent
    }
}

new_component_list_type!(AvatarComponentList, AvatarId, AvatarComponent);

impl AvatarComponentList {
    pub fn update(&mut self, drawables: &mut DrawableComponentList, delta_time: f64) {
        for component in self.0.values_mut() {
            let transform = &mut drawables.get_mut(component.parent).unwrap().transform;
            match &mut component.controls {
                AvatarControls::Flying(controls, camera_angle) => {
                    // look_input is the total pointer delta for this frame, and doesn't need to be scaled by delta_time.
                    // TODO will need to change look_input to support gamepad input.
                    camera_angle.x += Deg(controls.look_input.y * LOOK_SENSITIVITY);
                    camera_angle.y -= Deg(controls.look_input.x * LOOK_SENSITIVITY);
                    transform.rot = Quaternion::from_euler_yxz(*camera_angle);
                    transform.disp += transform.rot * controls.move_input * (MOVE_SPEED * delta_time) as f32;
                }
            }
        }
    }
}
