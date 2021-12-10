mod function;
pub mod spell;

use std::{collections::HashMap, sync::Arc};

use crate::{transform::{Vector3f, Transform, TransformExtensions}, world::components::DrawableId, syntax::code::{ListTerm, SourceListTerm, SourceListArgument, SourceError}};
use spell::*;

pub type Variable = String;
pub type EntityId = DrawableId;

pub type VariableMap = HashMap<Variable, f64>;

#[derive(Copy, Clone)]
pub struct VariableScope<'a> {
    parent: Option<&'a VariableScope<'a>>,
    variables: &'a VariableMap,
}

impl<'a> VariableScope<'a> {
    pub fn new(variables: &'a VariableMap) -> VariableScope<'a> {
        VariableScope {
            parent: None,
            variables,
        }
    }
    pub fn inner_scope(&'a self, variables: &'a VariableMap) -> VariableScope<'a> {
        VariableScope {
            parent: Some(self),
            variables,
        }
    }

    pub fn get(&self, key: &Variable) -> Option<f64> {
        if let Some(value) = self.variables.get(key) {
            Some(*value)
        } else if let Some(parent) = self.parent {
            parent.get(key)
        } else {
            None
        }
    }
}

#[derive(Debug)]
pub enum Value {
    Number(f64),
    Position(Vector3f),
    Transform(Transform),
    Entity(EntityId),
    SpellTarget(SpellTarget),
    SpellEffect(Box<dyn SpellEffect>),
    Spell(Arc<Spell>),
    SpellBinding(u8, Arc<Spell>),
}

impl Value {
    pub fn kind(&self) -> &'static str {
        match self {
            Value::Number(_) => "Number",
            Value::Position(_) => "Position",
            Value::Transform(_) => "Transform",
            Value::Entity(_) => "Entity",
            Value::SpellTarget(_) => "SpellTarget",
            Value::SpellEffect(_) => "SpellEffect",
            Value::Spell(_) => "Spell",
            Value::SpellBinding(_, _) => "SpellBinding",
        }
    }
    fn into_number(&self) -> Option<f64> {
        match self {
            Value::Number(num) => Some(*num),
            _ => None,
        }
    }
    fn into_position(&self) -> Option<Vector3f> {
        match self {
            Value::Position(pos) => Some(*pos),
            Value::Transform(transform) => Some(transform.disp),
            _ => None,
        }
    }
    fn into_transform(&self) -> Option<Transform> {
        match self {
            Value::Position(pos) => Some(Transform::from_translation(*pos)),
            Value::Transform(transform) => Some(*transform),
            _ => None,
        }
    }
}

type ValueResult = Result<Value, SourceError>;

pub trait Evaluable {
    type Output;
    fn evaluate(&self, scope: VariableScope) -> Self::Output;
}

impl Evaluable for SourceListTerm {
    type Output = ValueResult;
    fn evaluate(&self, scope: VariableScope) -> ValueResult {
        match &self.term {
            ListTerm::Identifier(ident) => scope.get(ident).map(Value::Number).ok_or_else(|| SourceError::unknown_variable(self.source_position(), ident)),
            ListTerm::Number(num) => Ok(Value::Number(*num)),
            ListTerm::List(list) => function::call_function(scope, list, self.source_position()),
        }
    }
}

impl<'a> Evaluable for SourceListArgument<'a> {
    type Output = Result<(SourceListArgument<'a>, Value), SourceError>;
    fn evaluate(&self, scope: VariableScope) -> Self::Output {
        self.term().evaluate(scope).map(|val| (self.clone(), val))
    }
}