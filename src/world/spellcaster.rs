use std::sync::Arc;
use cgmath::EuclideanSpace;

use crate::{code::{Value, spell::*}, transform::{Vector3f, Point3f}};

use super::{Globals, components::ComponentSystem};

#[derive(Default)]
pub struct Spellcaster {
    bindings: [Option<Arc<Spell>>; 10],
}

pub struct SpellContext<'a> {
    pub components: &'a mut ComponentSystem,
    pub globals: &'a mut Globals,
}

impl Spellcaster {
    fn resolve_target(&self, context: &mut SpellContext, spell_target: &SpellTarget) -> ResolvedTarget {
        match spell_target {
            SpellTarget::Myself => {
                let avatar = context.globals.player_avatar.expect("casting target_self spell without an avatar set");
                let avatar_entity = context.components.avatars.get(avatar).unwrap().parent();
                ResolvedTarget {
                    entity: avatar_entity,
                    position: Point3f::from_vec(context.components.drawables.get(avatar_entity).unwrap().transform.disp + Vector3f::unit_y()),
                }
            }
        }
    }
    fn cast_spell(&self, context: &mut SpellContext, spell: &Spell) {
        let target = self.resolve_target(context, &spell.target);
        let targets = [target];
        spell.effect.apply(context, &targets);
    }
    pub fn cast_bound_spell(&self, context: &mut SpellContext, binding: u8) {
        if let Some(spell) = self.bindings[binding as usize].as_ref() {
            self.cast_spell(context, spell);
        }
    }

    pub fn apply_value(&mut self, context: &mut SpellContext, value: Value) {
        match value {
            Value::Spell(spell) => {
                self.cast_spell(context, &spell);
            }
            Value::SpellBinding(binding, spell) => {
                self.bindings[binding as usize] = Some(spell);
            }
            _ => println!("{:?}", value),
        }
    }
}