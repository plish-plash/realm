use cgmath::{Deg, Euler, Quaternion};
use slotmap::{DenseSlotMap, new_key_type};

use crate::{transform::QuaternionExtensions, world::input::FlyingControls};

use super::{DrawableId, DrawableComponentList};

// TODO make these configurable
const LOOK_SENSITIVITY: f32 = 0.1;
const MOVE_SPEED: f64 = 10.0;

pub enum AvatarControls {
    Flying(FlyingControls, Euler<Deg<f32>>),
}

impl AvatarControls {
    pub fn new_flying() -> AvatarControls {
        AvatarControls::Flying(FlyingControls::default(), Euler::new(Deg(0.0), Deg(0.0), Deg(0.0)))
    }
}

new_key_type! { pub struct AvatarId; }

#[derive(Default)]
pub struct AvatarComponentList {
    components: DenseSlotMap<AvatarId, (DrawableId, AvatarControls)>,
    player_avatar: Option<AvatarId>,
}

impl AvatarComponentList {
    pub fn add(&mut self, parent: DrawableId, component: AvatarControls) -> AvatarId {
        self.components.insert((parent, component))
    }
    pub fn remove(&mut self, id: AvatarId) {
        self.components.remove(id);
    }
    pub fn get(&self, id: AvatarId) -> Option<&AvatarControls> {
        self.components.get(id).map(|(_, c)| c)
    }
    pub fn get_mut(&mut self, id: AvatarId) -> Option<&mut AvatarControls> {
        self.components.get_mut(id).map(|(_, c)| c)
    }
    pub fn get_parent(&self, id: AvatarId) -> Option<DrawableId> {
        self.components.get(id).map(|(p, _)| *p)
    }
}

impl AvatarComponentList {
    pub fn player_avatar(&self) -> Option<AvatarId> {
        self.player_avatar
    }
    pub fn set_player_avatar(&mut self, player_avatar: Option<AvatarId>) {
        self.player_avatar = player_avatar;
    }
    
    pub fn update(&mut self, drawables: &mut DrawableComponentList, delta_time: f64) {
        for (parent, component) in self.components.values_mut() {
            let transform = &mut drawables.get_mut(*parent).unwrap().transform;
            match component {
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
