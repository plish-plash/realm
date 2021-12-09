pub mod avatar;

use slotmap::{DenseSlotMap, new_key_type};

use crate::triangle_draw::{TriangleDraw, TriangleDrawable, TriangleDrawSystem};
use avatar::AvatarComponentList;

use super::WorldSystem;

new_key_type! { pub struct DrawableId; }

#[derive(Default)]
pub struct DrawableComponentList(DenseSlotMap<DrawableId, TriangleDrawable>);

impl DrawableComponentList {
    pub fn add(&mut self, component: TriangleDrawable) -> DrawableId {
        self.0.insert(component)
    }
    pub fn remove(&mut self, id: DrawableId) {
        self.0.remove(id);
    }
    pub fn get(&self, id: DrawableId) -> Option<&TriangleDrawable> {
        self.0.get(id)
    }
    pub fn get_mut(&mut self, id: DrawableId) -> Option<&mut TriangleDrawable> {
        self.0.get_mut(id)
    }
}

#[derive(Default)]
pub struct ComponentSystem {
    pub drawables: DrawableComponentList,
    pub avatars: AvatarComponentList,
}

impl WorldSystem for ComponentSystem {
    fn update(&mut self, draw_system: &TriangleDrawSystem, delta_time: f64) {
        self.avatars.update(&mut self.drawables, delta_time);
    }
    fn render(&self, renderer: &mut TriangleDraw) {
        for entity in self.drawables.0.values() {
            renderer.draw(entity);
        }
    }
}
