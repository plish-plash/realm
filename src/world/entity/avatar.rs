use cgmath::{Deg, Euler, Quaternion};

use crate::{transform::QuaternionExtensions, triangle_draw::TriangleDrawSystem, world::input::FlyingControls};

use super::{Component, Entities, EntityComponents, EntityId};

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

#[derive(Default)]
pub struct AvatarComponent {
    components: EntityComponents<AvatarControls>,
    player_avatar: Option<EntityId>,
}

impl AvatarComponent {
    pub fn player_avatar(&self) -> Option<EntityId> {
        self.player_avatar
    }
    pub fn player_avatar_controls_mut(&mut self) -> Option<&mut AvatarControls> {
        self.player_avatar.and_then(|entity| self.components.get_mut(entity))
    }
    pub fn set_player_avatar(&mut self, player_avatar: Option<EntityId>) {
        self.player_avatar = player_avatar;
    }

    pub fn insert(&mut self, entity: EntityId, component: AvatarControls) -> Option<AvatarControls> {
        self.components.insert(entity, component)
    }
}

impl Component for AvatarComponent {
    fn update(&mut self, entities: &mut Entities, _draw_system: &TriangleDrawSystem, delta_time: f64) {
        for (entity, component) in self.components.iter_mut() {
            let transform = &mut entities.get_mut(entity).unwrap().transform;
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
