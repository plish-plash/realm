use std::collections::HashMap;

pub type Variable = String;

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

    pub fn get(&self, key: &Variable) -> f64 {
        if let Some(value) = self.variables.get(key) {
            *value
        } else if let Some(parent) = self.parent {
            parent.get(key)
        } else {
            // TODO better errors
            panic!("unknown variable `{}`", key);
        }
    }
}

pub trait Evaluable {
    type Output;
    fn evaluate(&self, scope: VariableScope) -> Self::Output;
}
