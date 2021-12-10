use std::sync::Arc;

use combine::stream::position::SourcePosition;

use crate::{syntax::code::{List, SourceError}, code::spell::SpellTarget};
use super::{Value, ValueResult, spell::*, VariableScope, Evaluable};

pub fn call_function(scope: VariableScope, list: &List, position: SourcePosition) -> ValueResult {
    if list.len() == 0 {
        return Err(SourceError::empty_list(position));
    }
    let function = list[0].into_literal().ok_or_else(|| SourceError::invalid_function_name(list[0].source_position()))?;
    match function {
        "spell" => spell(scope, list, position),
        "bind" => bind(scope, list, position),
        "target_self" => target_self(scope, list, position),
        "create_terrain" => create_terrain(scope, list, position),
        _ => Err(SourceError::unknown_function(list[0].source_position(), function)),
    }
}

fn spell(scope: VariableScope, list: &List, position: SourcePosition) -> ValueResult {
    if list.len() < 3 {
        return Err(SourceError::not_enough_arguments(position, list, 3));
    }
    let target = list.argument(1).evaluate(scope).and_then(|(arg, val)| match val {
        Value::SpellTarget(t) => Ok(t),
        _ => Err(SourceError::unexpected_value(&arg, "SpellTarget", &val)),
    })?;
    let effect = list.argument(2).evaluate(scope).and_then(|(arg, val)| match val {
        Value::SpellEffect(e) => Ok(e),
        _ => Err(SourceError::unexpected_value(&arg, "SpellEffect", &val)),
    })?;
    Ok(Value::Spell(Arc::new(Spell { target, effect })))
}

fn bind(scope: VariableScope, list: &List, position: SourcePosition) -> ValueResult {
    if list.len() < 3 {
        return Err(SourceError::not_enough_arguments(position, list, 3));
    }
    let binding = list.argument(1).into_number()?;
    let spell = list.argument(2).evaluate(scope).and_then(|(arg, val)| match val {
        Value::Spell(s) => Ok(s),
        _ => Err(SourceError::unexpected_value(&arg, "Spell", &val)),
    })?;
    Ok(Value::SpellBinding(binding as u8, spell))
}

fn target_self(_scope: VariableScope, _list: &List, _position: SourcePosition) -> ValueResult {
    Ok(Value::SpellTarget(SpellTarget::Myself))
}

fn create_terrain(_scope: VariableScope, list: &List, position: SourcePosition) -> ValueResult {
    if list.len() < 3 {
        return Err(SourceError::not_enough_arguments(position, list, 3));
    }
    let w = list.argument(1).into_number()?;
    let h = list.argument(2).into_number()?;
    Ok(Value::SpellEffect(Box::new(CreateTerrainEffect(w as u32, h as u32))))
}
