use crate::scope::{BoolType, Predicate, Scope, Type};
use std::{
    any::Any,
    cell::RefCell,
    collections::HashMap,
    rc::{Rc, Weak},
};

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct ObjectId(pub(super) usize);

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct AtomId(pub(super) usize);

#[derive(Clone)]
pub enum Slot {
    Primitive(Rc<dyn Var>),
    ObjectRef(ObjectId),
    AtomRef(AtomId),
}

pub trait Var {
    fn var_type(&self) -> Rc<dyn Type>;
    fn as_any(self: Rc<Self>) -> Rc<dyn Any>;
    fn as_env(self: Rc<Self>) -> Option<Rc<dyn Env>> {
        None
    }
}

pub trait Env {
    fn parent(&self) -> Option<Rc<dyn Env>>;
    fn get(&self, name: &str) -> Option<Slot>;
    fn set(&self, name: String, value: Slot);
}

pub struct CommonEnv {
    parent: Option<Rc<dyn Env>>,
    variables: RefCell<HashMap<String, Slot>>,
}

impl CommonEnv {
    pub fn new(parent: Option<Rc<dyn Env>>) -> Self {
        Self { parent, variables: RefCell::new(HashMap::new()) }
    }
}

impl Env for CommonEnv {
    fn parent(&self) -> Option<Rc<dyn Env>> {
        self.parent.clone()
    }

    fn get(&self, name: &str) -> Option<Slot> {
        self.variables.borrow().get(name).cloned().or_else(|| self.parent.as_ref()?.get(name))
    }

    fn set(&self, name: String, value: Slot) {
        self.variables.borrow_mut().insert(name, value);
    }
}

pub enum BoolExpr {
    Term { var_type: Weak<BoolType>, term: Slot },
    Not { var_type: Weak<BoolType>, term: Rc<BoolExpr> },
    Eq { var_type: Weak<BoolType>, left: Slot, right: Slot },
    Lt { var_type: Weak<BoolType>, left: Slot, right: Slot },
    Leq { var_type: Weak<BoolType>, left: Slot, right: Slot },
    Or { var_type: Weak<BoolType>, terms: Vec<BoolExpr> },
    And { var_type: Weak<BoolType>, terms: Vec<BoolExpr> },
}

impl Var for BoolExpr {
    fn var_type(&self) -> Rc<dyn Type> {
        match self {
            BoolExpr::Term { var_type: var_tp, .. } | BoolExpr::Not { var_type: var_tp, .. } | BoolExpr::Eq { var_type: var_tp, .. } | BoolExpr::Lt { var_type: var_tp, .. } | BoolExpr::Leq { var_type: var_tp, .. } | BoolExpr::Or { var_type: var_tp, .. } | BoolExpr::And { var_type: var_tp, .. } => var_tp.upgrade().unwrap(),
        }
    }

    fn as_any(self: Rc<Self>) -> Rc<dyn Any> {
        self
    }
}

pub struct Object {
    id: ObjectId,
    class: Weak<dyn Type>,
    env: CommonEnv,
}

impl Object {
    pub(super) fn new(id: ObjectId, class: Rc<dyn Type>, parent_env: Rc<dyn Env>) -> Self {
        Self { id, class: Rc::downgrade(&class), env: CommonEnv::new(Some(parent_env)) }
    }
}

impl Var for Object {
    fn var_type(&self) -> Rc<dyn Type> {
        self.class.upgrade().unwrap()
    }

    fn as_any(self: Rc<Self>) -> Rc<dyn Any> {
        self
    }

    fn as_env(self: Rc<Self>) -> Option<Rc<dyn Env>> {
        Some(self.clone())
    }
}

impl Env for Object {
    fn parent(&self) -> Option<Rc<dyn Env>> {
        self.env.parent.clone()
    }

    fn get(&self, name: &str) -> Option<Slot> {
        self.env.get(name)
    }

    fn set(&self, name: String, value: Slot) {
        self.env.set(name, value);
    }
}

pub struct Atom {
    id: AtomId,
    predicate: Weak<Predicate>,
    fact: bool,
    env: CommonEnv,
}

impl Var for Atom {
    fn var_type(&self) -> Rc<dyn Type> {
        self.predicate.upgrade().unwrap()
    }

    fn as_any(self: Rc<Self>) -> Rc<dyn Any> {
        self
    }

    fn as_env(self: Rc<Self>) -> Option<Rc<dyn Env>> {
        Some(self.clone())
    }
}

impl Env for Atom {
    fn parent(&self) -> Option<Rc<dyn Env>> {
        self.env.parent.clone()
    }

    fn get(&self, name: &str) -> Option<Slot> {
        self.env.get(name)
    }

    fn set(&self, name: String, value: Slot) {
        self.env.set(name, value);
    }
}

impl Atom {
    pub fn new(id: AtomId, predicate: Rc<Predicate>, fact: bool, args: HashMap<String, Slot>) -> Self {
        let env = match args.get("tau") {
            Some(tau) => match tau {
                Slot::Primitive(_var) => panic!("Tau cannot be a primitive variable"),
                Slot::ObjectRef(obj_id) => predicate.clone().core().get_object(*obj_id).expect("Object ID in tau does not exist").as_env().expect("Object in tau does not have an environment").clone(),
                Slot::AtomRef(atom_id) => predicate.clone().core().get_atom(*atom_id).expect("Atom ID in tau does not exist").as_env().expect("Atom in tau does not have an environment").clone(),
            },
            None => predicate.clone().core(),
        };
        let env = CommonEnv::new(Some(env));
        for (name, value) in args {
            env.set(name, value);
        }
        Self { id, predicate: Rc::downgrade(&predicate), fact, env }
    }
}
