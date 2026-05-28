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
    ops::Deref,
    rc::{Rc, Weak},
};

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct ObjectId(pub(super) usize);

impl Deref for ObjectId {
    type Target = usize;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl fmt::Display for ObjectId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "obj-{}", self.0)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct AtomId(pub(super) usize);

impl Deref for AtomId {
    type Target = usize;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl fmt::Display for AtomId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "atm-{}", self.0)
    }
}

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
            Slot::ObjectRef(obj_id) => write!(f, "Object({})", *obj_id),
            Slot::AtomRef(atom_id) => write!(f, "Atom({})", *atom_id),
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
        BoolExpr::Not { term, .. } => push_inverted(term.clone()),
        BoolExpr::And { var_type, terms } => Rc::new(BoolExpr::And {
            var_type: var_type.clone(),
            terms: terms.iter().map(|t| push_negations(t.clone())).collect(),
        }),
        BoolExpr::Or { var_type, terms } => Rc::new(BoolExpr::Or {
            var_type: var_type.clone(),
            terms: terms.iter().map(|t| push_negations(t.clone())).collect(),
        }),
        _ => expr,
    }
}

/// Processes an expression as if a `Not` wrapper were applied to it.
fn push_inverted(expr: Rc<BoolExpr>) -> Rc<BoolExpr> {
    match expr.as_ref() {
        // Double Negation: Not(Not(term)) => term
        BoolExpr::Not { term, .. } => push_negations(term.clone()),

        // De Morgan: Not(And(A, B)) => Or(Not(A), Not(B))
        BoolExpr::And { var_type, terms } => Rc::new(BoolExpr::Or {
            var_type: var_type.clone(),
            terms: terms.iter().map(|t| push_inverted(t.clone())).collect(),
        }),

        // De Morgan: Not(Or(A, B)) => And(Not(A), Not(B))
        BoolExpr::Or { var_type, terms } => Rc::new(BoolExpr::And {
            var_type: var_type.clone(),
            terms: terms.iter().map(|t| push_inverted(t.clone())).collect(),
        }),

        BoolExpr::Leq { var_type, left, right } => Rc::new(BoolExpr::Lt { var_type: var_type.clone(), left: right.clone(), right: left.clone() }),
        BoolExpr::Lt { var_type, left, right } => Rc::new(BoolExpr::Leq { var_type: var_type.clone(), left: right.clone(), right: left.clone() }),

        BoolExpr::Term { var_type: var_tp, .. } | BoolExpr::Eq { var_type: var_tp, .. } => Rc::new(BoolExpr::Not { var_type: var_tp.clone(), term: expr }),
    }
}

fn distribute(expr: Rc<BoolExpr>) -> Rc<BoolExpr> {
    match expr.as_ref() {
        BoolExpr::Or { var_type, terms } => {
            // Step 1: Recursively distribute child nodes and flatten any nested Ors
            let mut distributed_terms = Vec::new();
            for t in terms {
                let dist = distribute(t.clone());
                if let BoolExpr::Or { terms: inner_terms, .. } = dist.as_ref() {
                    distributed_terms.extend(inner_terms.clone());
                } else {
                    distributed_terms.push(dist);
                }
            }

            // Step 2: Build the Cartesian product of terms over And boundaries
            // Start with a pool containing a single empty clause
            let mut result_ands: Vec<Vec<Rc<BoolExpr>>> = vec![vec![]];

            for term in distributed_terms {
                if let BoolExpr::And { terms: and_terms, .. } = term.as_ref() {
                    // Split all existing combinations across the newly encountered And choices
                    let mut next_ands = Vec::new();
                    for existing_and in &result_ands {
                        for and_term in and_terms {
                            let mut combo = existing_and.clone();
                            combo.push(and_term.clone());
                            next_ands.push(combo);
                        }
                    }
                    result_ands = next_ands;
                } else {
                    // Leaf nodes or Or nodes get appended to all current paths
                    for existing_and in &mut result_ands {
                        existing_and.push(term.clone());
                    }
                }
            }

            // Step 3: Map our combinations back into Or nodes inside a master And node
            let cnf_or_nodes: Vec<Rc<BoolExpr>> = result_ands.into_iter().map(|sub_terms| Rc::new(BoolExpr::Or { var_type: var_type.clone(), terms: sub_terms })).collect();

            // Optimization: If no distribution happened, don't wrap in a redundant And
            if cnf_or_nodes.len() == 1 { cnf_or_nodes[0].clone() } else { Rc::new(BoolExpr::And { var_type: var_type.clone(), terms: cnf_or_nodes }) }
        }

        BoolExpr::And { var_type, terms } => {
            // Flatten nested Ands to keep the AST compact
            let mut distributed_terms = Vec::new();
            for t in terms {
                let dist = distribute(t.clone());
                if let BoolExpr::And { terms: inner_terms, .. } = dist.as_ref() {
                    distributed_terms.extend(inner_terms.clone());
                } else {
                    distributed_terms.push(dist);
                }
            }
            Rc::new(BoolExpr::And { var_type: var_type.clone(), terms: distributed_terms })
        }

        _ => expr,
    }
}

pub fn to_cnf(expr: Rc<BoolExpr>) -> Rc<BoolExpr> {
    distribute(push_negations(expr))
}

pub fn get_var_by_path(core: &dyn Core, env: &dyn Env, path: &[String]) -> Result<Slot, RiddleError> {
    let (first, rest) = path.split_first().ok_or_else(|| RiddleError::RuntimeError("Empty variable path".into()))?;
    rest.iter().try_fold(env.get(first).ok_or_else(|| RiddleError::NotFound(first.to_string()))?, |acc, id| match acc {
        Slot::Primitive(var) => var.clone().as_env().ok_or_else(|| RiddleError::NotAnEnvironment(format!("Variable '{}' in path does not have an environment", first)))?.get(id).ok_or_else(|| RiddleError::NotFound(format!("Variable '{}' in path not found in variable '{}'", id, first))),
        Slot::ObjectRef(obj_id) => {
            let obj = core.get_object(obj_id).ok_or_else(|| RiddleError::NotFound(format!("Object {} not found", *obj_id)))?;
            obj.as_env().ok_or_else(|| RiddleError::NotAnEnvironment(format!("Object {} does not have an environment", *obj_id)))?.get(id).ok_or_else(|| RiddleError::NotFound(format!("Variable '{}' in path not found in object {}", id, *obj_id)))
        }
        Slot::AtomRef(atom_id) => {
            let atom = core.get_atom(atom_id).ok_or_else(|| RiddleError::NotFound(format!("Atom {} not found", *atom_id)))?;
            atom.as_env().ok_or_else(|| RiddleError::NotAnEnvironment(format!("Atom {} does not have an environment", *atom_id)))?.get(id).ok_or_else(|| RiddleError::NotFound(format!("Variable '{}' in path not found in atom {}", id, *atom_id)))
        }
    })
}
