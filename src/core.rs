use std::{
    cell::RefCell,
    collections::HashMap,
    rc::{Rc, Weak},
};

use crate::{
    env::{Atom, AtomId, CommonEnv, Env, Object, ObjectId, Slot},
    scope::{BoolType, CommonScope, Predicate, Scope, Type},
};

pub trait Core: Scope + Env {
    fn new_bool(&self, value: bool) -> Slot;
    fn new_bool_var(&self) -> Slot;

    fn new_object(&self, class: Rc<dyn Type>, parent_env: Option<Rc<dyn Env>>) -> ObjectId;
    fn get_object(&self, id: ObjectId) -> Option<Rc<Object>>;
    fn new_atom(&self, predicate: Rc<Predicate>, fact: bool, args: HashMap<String, Slot>) -> AtomId;
    fn get_atom(&self, id: AtomId) -> Option<Rc<Atom>>;
}

pub struct CommonCore {
    scope: Rc<CommonScope>,
    env: Rc<CommonEnv>,
    objects: RefCell<Vec<Rc<Object>>>,
    atoms: RefCell<Vec<Rc<Atom>>>,
}

impl CommonCore {
    pub fn new(core: Weak<dyn Core>) -> Rc<Self> {
        let c_core = Rc::new(CommonCore {
            scope: Rc::new(CommonScope::new(core.clone(), None)),
            env: Rc::new(CommonEnv::new(None)),
            objects: RefCell::new(Vec::new()),
            atoms: RefCell::new(Vec::new()),
        });
        c_core.add_type(Rc::new(BoolType::new(core.clone())));
        c_core
    }

    /// Registers a type in the core type table under its declared name.
    pub fn add_type(&self, class: Rc<dyn Type>) {
        self.scope.types.borrow_mut().insert(class.name().to_string(), class);
    }

    pub fn new_object(&self, class: Rc<dyn Type>, parent_env: Option<Rc<dyn Env>>) -> ObjectId {
        let id = ObjectId(self.objects.borrow().len());
        self.objects.borrow_mut().push(Rc::new(Object::new(id, class, parent_env)));
        id
    }

    pub fn new_atom(&self, predicate: Rc<Predicate>, fact: bool, args: HashMap<String, Slot>) -> AtomId {
        let id = AtomId(self.atoms.borrow().len());
        self.atoms.borrow_mut().push(Rc::new(Atom::new(id, predicate, fact, args)));
        id
    }
}

impl Scope for CommonCore {
    fn core(self: Rc<Self>) -> Rc<dyn Core> {
        self.scope.clone().core()
    }

    fn scope(&self) -> Option<Rc<dyn Scope>> {
        None
    }

    fn get_type(&self, name: &str) -> Option<Rc<dyn Type>> {
        self.scope.types.borrow().get(name).cloned()
    }
}

impl Env for CommonCore {
    fn parent(&self) -> Option<Rc<dyn Env>> {
        None
    }

    fn get(&self, name: &str) -> Option<Slot> {
        self.env.get(name)
    }

    fn set(&self, name: String, value: Slot) {
        self.env.set(name, value);
    }
}
