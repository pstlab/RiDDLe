use crate::{
    core::Core,
    env::{BoolExpr, Slot},
};
use std::{
    any::Any,
    cell::RefCell,
    collections::HashMap,
    rc::{Rc, Weak},
};

pub trait Type {
    fn name(&self) -> &str;
    fn full_name(&self) -> String {
        self.name().to_string()
    }
    fn as_any(self: Rc<Self>) -> Rc<dyn Any>;

    fn new_instance(self: Rc<Self>) -> Slot;
}

pub struct BoolType {
    core: Weak<dyn Core>,
}

impl BoolType {
    /// Creates the built-in boolean type.
    pub fn new(core: Weak<dyn Core>) -> Self {
        Self { core }
    }
}

impl Type for BoolType {
    fn name(&self) -> &str {
        "bool"
    }

    fn as_any(self: Rc<Self>) -> Rc<dyn Any> {
        self
    }

    fn new_instance(self: Rc<Self>) -> Slot {
        Slot::Primitive(Rc::new(BoolExpr::Term { var_type: Rc::downgrade(&self), term: self.core.upgrade().unwrap().new_bool_var() }))
    }
}

pub trait Scope {
    fn core(self: Rc<Self>) -> Rc<dyn Core>;
    fn scope(&self) -> Option<Rc<dyn Scope>>;

    fn get_type(&self, name: &str) -> Option<Rc<dyn Type>>;
}

pub struct CommonScope {
    core: Weak<dyn Core>,
    scope: Option<Weak<dyn Scope>>,
    pub(crate) types: RefCell<HashMap<String, Rc<dyn Type>>>,
}

impl CommonScope {
    /// Creates an empty scope with an optional parent scope.
    pub fn new(core: Weak<dyn Core>, scope: Option<Weak<dyn Scope>>) -> Self {
        Self { core, scope, types: RefCell::new(HashMap::new()) }
    }
}

impl Scope for CommonScope {
    fn core(self: Rc<Self>) -> Rc<dyn Core> {
        self.core.upgrade().expect("Core should never be dropped while scopes exist")
    }

    fn scope(&self) -> Option<Rc<dyn Scope>> {
        self.scope.as_ref()?.upgrade()
    }

    fn get_type(&self, name: &str) -> Option<Rc<dyn Type>> {
        self.types.borrow().get(name).cloned().or_else(|| self.scope()?.get_type(name))
    }
}

pub struct Predicate {
    core: Weak<dyn Core>,
    scope: CommonScope,
    name: String,
}

impl Type for Predicate {
    fn name(&self) -> &str {
        &self.name
    }

    fn as_any(self: Rc<Self>) -> Rc<dyn Any> {
        self
    }

    fn new_instance(self: Rc<Self>) -> Slot {
        panic!("Cannot create instance of a predicate")
    }
}

impl Scope for Predicate {
    fn core(self: Rc<Self>) -> Rc<dyn Core> {
        self.core.upgrade().expect("Core should never be dropped while predicates exist")
    }

    fn scope(&self) -> Option<Rc<dyn Scope>> {
        self.scope.scope()
    }

    fn get_type(&self, name: &str) -> Option<Rc<dyn Type>> {
        self.scope.get_type(name)
    }
}
