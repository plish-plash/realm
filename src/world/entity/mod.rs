pub mod avatar;
pub mod flora;

use slotmap::{DenseSlotMap, SparseSecondaryMap, new_key_type};

use crate::{transform::Transform, triangle_draw::{TriangleDraw, TriangleDrawable, TriangleDrawSystem}};
use avatar::AvatarComponent;
use flora::FloraComponent;

use super::WorldSystem;

pub type Entity = TriangleDrawable;

new_key_type! { pub struct EntityId; }

pub type Entities = DenseSlotMap<EntityId, Entity>;
pub type EntityComponents<T> = SparseSecondaryMap<EntityId, T>;

pub trait Component: 'static {
    fn update(&mut self, entities: &mut Entities, draw_system: &TriangleDrawSystem, delta_time: f64);
}

pub struct EntitySystem {
    entities: Entities,
    avatar: AvatarComponent,
    flora: FloraComponent,
}

impl EntitySystem {
    pub fn new() -> EntitySystem {
        EntitySystem {
            entities: Entities::with_key(),
            avatar: AvatarComponent::default(),
            flora: FloraComponent::new(),
        }
    }

    pub fn spawn(&mut self, entity: Entity) -> EntityId {
        self.entities.insert(entity)
    }

    pub fn get_transform(&self, entity: EntityId) -> Transform {
        self.entities.get(entity).unwrap().transform
    }
    pub fn avatar(&self) -> &AvatarComponent {
        &self.avatar
    }
    pub fn avatar_mut(&mut self) -> &mut AvatarComponent {
        &mut self.avatar
    }
    pub fn flora_mut(&mut self) -> &mut FloraComponent {
        &mut self.flora
    }
}

impl WorldSystem for EntitySystem {
    fn update(&mut self, draw_system: &TriangleDrawSystem, delta_time: f64) {
        self.avatar.update(&mut self.entities, draw_system, delta_time);
        self.flora.update(&mut self.entities, draw_system, delta_time);
    }
    fn render(&self, renderer: &mut TriangleDraw) {
        let player_avatar = self.avatar.player_avatar();
        for (id, entity) in self.entities.iter() {
            // Don't draw the avatar (TODO better)
            if player_avatar != Some(id) {
                renderer.draw(entity);
            }
        }
    }
}
