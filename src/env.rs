use crate::{
    RiddleError,
    core::Core,
    scope::{BoolType, Class, Predicate, Scope, Type},
};
use core::fmt;
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

impl fmt::Display for Slot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Slot::Primitive(var) => write!(f, "{}", var.var_type().name()),
            Slot::ObjectRef(obj_id) => write!(f, "Object({})", obj_id.0),
            Slot::AtomRef(atom_id) => write!(f, "Atom({})", atom_id.0),
        }
    }
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
    Or { var_type: Weak<BoolType>, terms: Vec<Rc<BoolExpr>> },
    And { var_type: Weak<BoolType>, terms: Vec<Rc<BoolExpr>> },
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
    class: Weak<dyn Class>,
    env: CommonEnv,
}

impl Object {
    pub(super) fn new(id: ObjectId, class: Rc<dyn Class>) -> Self {
        Self { id, class: Rc::downgrade(&class), env: CommonEnv::new(None) }
    }

    pub fn id(&self) -> ObjectId {
        self.id
    }

    pub fn class(&self) -> Rc<dyn Class> {
        self.class.upgrade().unwrap()
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

impl Atom {
    pub fn new(id: AtomId, predicate: Rc<Predicate>, fact: bool, args: HashMap<String, Slot>) -> Self {
        let env = match args.get("tau") {
            Some(tau) => match tau {
                Slot::Primitive(var) => var.clone().as_env().expect("Tau variable does not have an environment").clone(),
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

    pub fn id(&self) -> AtomId {
        self.id
    }

    pub fn predicate(&self) -> Rc<Predicate> {
        self.predicate.upgrade().unwrap()
    }

    pub fn is_fact(&self) -> bool {
        self.fact
    }
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

fn push_negations(expr: Rc<BoolExpr>) -> Rc<BoolExpr> {
    match expr.as_ref() {
        BoolExpr::Not { var_type, term } => {
            let inner_expr = push_negations(term.clone());
            match inner_expr.as_any().downcast_ref::<BoolExpr>() {
                Some(BoolExpr::Not { var_type: inner_var_type, term: inner_term }) => Rc::new(BoolExpr::Term { var_type: inner_var_type.clone(), term: Slot::Primitive(inner_term.clone()) }),
                Some(BoolExpr::And { var_type: inner_var_type, terms }) => Rc::new(BoolExpr::Or {
                    var_type: inner_var_type.clone(),
                    terms: terms.iter().map(|t| push_negations(t.clone())).collect(),
                }),
                Some(BoolExpr::Or { var_type: inner_var_type, terms }) => Rc::new(BoolExpr::And {
                    var_type: inner_var_type.clone(),
                    terms: terms.iter().map(|t| push_negations(t.clone())).collect(),
                }),
                _ => Rc::new(BoolExpr::Not { var_type: var_type.clone(), term: push_negations(term.clone()) }),
            }
        }
        BoolExpr::And { var_type, terms } => Rc::new(BoolExpr::And {
            var_type: var_type.clone(),
            terms: terms.iter().map(|t| push_negations(t.clone())).collect(),
        }),
        BoolExpr::Or { var_type, terms } => Rc::new(BoolExpr::Or {
            var_type: var_type.clone(),
            terms: terms.iter().map(|t| push_negations(t.clone())).collect(),
        }),
        _ => expr.clone(),
    }
}

fn distribute(expr: Rc<BoolExpr>) -> Rc<BoolExpr> {
    match expr.as_ref() {
        BoolExpr::Or { var_type, terms } => {
            let distributed_terms: Vec<Rc<BoolExpr>> = terms.iter().map(|t| distribute(t.clone())).collect();
            let mut result_terms: Vec<Rc<BoolExpr>> = vec![Rc::new(BoolExpr::Term {
                var_type: var_type.clone(),
                term: Slot::Primitive(Rc::new(BoolExpr::And { var_type: var_type.clone(), terms: vec![] })),
            })];
            for term in distributed_terms {
                if let BoolExpr::Or { terms: or_terms, .. } = term.as_ref() {
                    let mut new_result_terms: Vec<Rc<BoolExpr>> = Vec::new();
                    for res_term in &result_terms {
                        for or_term in or_terms {
                            new_result_terms.push(Rc::new(BoolExpr::And { var_type: var_type.clone(), terms: vec![res_term.clone(), or_term.clone()] }));
                        }
                    }
                    result_terms = new_result_terms;
                } else {
                    result_terms = result_terms.into_iter().map(|res_term| Rc::new(BoolExpr::And { var_type: var_type.clone(), terms: vec![res_term, term.clone()] })).collect();
                }
            }
            Rc::new(BoolExpr::Or { var_type: var_type.clone(), terms: result_terms })
        }
        BoolExpr::And { var_type, terms } => Rc::new(BoolExpr::Or { var_type: var_type.clone(), terms: terms.iter().map(|t| distribute(t.clone())).collect() }),
        _ => expr.clone(),
    }
}

pub(crate) fn to_cnf(expr: Rc<BoolExpr>) -> Rc<BoolExpr> {
    distribute(push_negations(expr))
}

pub fn get_var_by_path(core: &dyn Core, env: &dyn Env, path: &[String]) -> Result<Slot, RiddleError> {
    let (first, rest) = path.split_first().ok_or_else(|| RiddleError::RuntimeError("Empty variable path".into()))?;
    rest.iter().try_fold(env.get(first).ok_or_else(|| RiddleError::NotFound(first.to_string()))?, |acc, id| match acc {
        Slot::Primitive(var) => var.clone().as_env().ok_or_else(|| RiddleError::NotAnEnvironment(format!("Variable '{}' in path does not have an environment", first)))?.get(id).ok_or_else(|| RiddleError::NotFound(format!("Variable '{}' in path not found in variable '{}'", id, first))),
        Slot::ObjectRef(obj_id) => {
            let obj = core.get_object(obj_id).ok_or_else(|| RiddleError::NotFound(format!("Object with id {} not found", obj_id.0)))?;
            obj.as_env().ok_or_else(|| RiddleError::NotAnEnvironment(format!("Object with id {} does not have an environment", obj_id.0)))?.get(id).ok_or_else(|| RiddleError::NotFound(format!("Variable '{}' in path not found in object with id {}", id, obj_id.0)))
        }
        Slot::AtomRef(atom_id) => {
            let atom = core.get_atom(atom_id).ok_or_else(|| RiddleError::NotFound(format!("Atom with id {} not found", atom_id.0)))?;
            atom.as_env().ok_or_else(|| RiddleError::NotAnEnvironment(format!("Atom with id {} does not have an environment", atom_id.0)))?.get(id).ok_or_else(|| RiddleError::NotFound(format!("Variable '{}' in path not found in atom with id {}", id, atom_id.0)))
        }
    })
}
