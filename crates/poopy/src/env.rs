use std::collections::HashMap;

use crate::val::Val;

pub struct Env<'parent> {
    bindings: HashMap<String, Val>,
    parent: Option<&'parent Self>,
}

impl<'parent> Env<'parent> {
    pub(crate) fn store(&mut self, name: String, val: Val) {
        self.bindings.insert(name, val);
    }

    pub(crate) fn get_binding_val(&self, name: &str) -> Result<Val, String> {
        self.get_bidning_val_without_err_msg(name)
            .ok_or_else(|| format!("binding with name '{}' does not exist", name))
    }

    fn get_bidning_val_without_err_msg(&self, name: &str) -> Option<Val> {
        self.bindings.get(name).cloned().or_else(|| {
            self.parent
                .and_then(|parent| parent.get_bidning_val_without_err_msg(name))
        })
    }

    pub(crate) fn create_child(&'parent self) -> Self {
        Self {
            bindings: HashMap::new(),
            parent: Some(self),
        }
    }

    pub fn default() -> Self {
        Self {
            bindings: HashMap::new(),
            parent: None,
        }
    }
}
