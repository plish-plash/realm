pub mod avatar;
pub mod terrain;

use crate::triangle_draw::{TriangleDraw, TriangleDrawable, TriangleDrawSystem};
use avatar::AvatarComponentList;
use terrain::TerrainComponentList;

use super::Globals;

macro_rules! new_component_list_type {
    ($list:ident, $id:ident, $comp:ident) => {
        use slotmap::{DenseSlotMap, new_key_type};
        new_key_type! { pub struct $id; }
        #[derive(Default)]
        pub struct $list(DenseSlotMap<$id, $comp>);
        impl $list {
            pub fn add(&mut self, component: $comp) -> $id {
                self.0.insert(component)
            }
            pub fn remove(&mut self, id: $id) {
                self.0.remove(id);
            }
            pub fn get(&self, id: $id) -> Option<&$comp> {
                self.0.get(id)
            }
            pub fn get_mut(&mut self, id: $id) -> Option<&mut $comp> {
                self.0.get_mut(id)
            }
        }
    };
}
pub(crate) use new_component_list_type;

new_component_list_type!(DrawableComponentList, DrawableId, TriangleDrawable);

#[derive(Default)]
pub struct ComponentSystem {
    pub drawables: DrawableComponentList,
    pub avatars: AvatarComponentList,
    pub terrain: TerrainComponentList,
}

impl ComponentSystem {
    pub fn update(&mut self, globals: &Globals, draw_system: &TriangleDrawSystem, delta_time: f64) {
        self.avatars.update(&mut self.drawables, delta_time);
        self.terrain.update(globals, draw_system, &mut self.drawables);
    }
    pub fn render(&self, renderer: &mut TriangleDraw) {
        for entity in self.drawables.0.values() {
            renderer.draw(entity);
        }
    }
}
