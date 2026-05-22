use crate::{
    language::RiddleError,
    scope::{BoolType, Predicate, Scope, Type},
};
use std::{
    any::Any,
    cell::RefCell,
    collections::HashMap,
    fmt,
    rc::{Rc, Weak},
};

pub trait Var: fmt::Debug {
    fn var_type(&self) -> Rc<dyn Type>;
    fn as_any(self: Rc<Self>) -> Rc<dyn Any>;
    fn as_env(self: Rc<Self>) -> Option<Rc<dyn Env>> {
        None
    }
}

pub trait Env: fmt::Debug {
    fn parent(&self) -> Option<Rc<dyn Env>>;
    fn get(&self, name: &str) -> Option<Rc<dyn Var>>;
    fn set(&self, name: String, value: Rc<dyn Var>);
}

pub trait EnvExt {
    fn get_as<T: Var + 'static>(&self, name: &str) -> Option<Rc<T>>;
}

impl<E: Env + ?Sized> EnvExt for E {
    fn get_as<T: Var + 'static>(&self, name: &str) -> Option<Rc<T>> {
        self.get(name)?.as_any().downcast::<T>().ok()
    }
}

#[derive(Debug)]
pub struct CommonEnv {
    parent: Option<Rc<dyn Env>>,
    variables: RefCell<HashMap<String, Rc<dyn Var>>>,
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

    fn get(&self, name: &str) -> Option<Rc<dyn Var>> {
        self.variables.borrow().get(name).cloned().or_else(|| self.parent.as_ref()?.get(name))
    }

    fn set(&self, name: String, value: Rc<dyn Var>) {
        self.variables.borrow_mut().insert(name, value);
    }
}

#[derive(Debug)]
pub struct Atom {
    predicate: Weak<Predicate>,
    fact: bool,
    env: CommonEnv,
}

impl Atom {
    pub fn new(predicate: Rc<Predicate>, fact: bool, args: HashMap<String, Rc<dyn Var>>) -> Self {
        let env = args.get("tau").and_then(|tau| tau.clone().as_env()).unwrap_or_else(|| predicate.clone().core());
        let env = CommonEnv::new(Some(env));
        for (name, value) in args {
            env.set(name, value);
        }
        Self { predicate: Rc::downgrade(&predicate), fact, env }
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
        self.predicate()
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

    fn get(&self, name: &str) -> Option<Rc<dyn Var>> {
        self.env.get(name)
    }

    fn set(&self, name: String, value: Rc<dyn Var>) {
        self.env.set(name, value);
    }
}

#[derive(Debug)]
pub struct Object {
    class: Weak<dyn Type>,
    env: CommonEnv,
}

impl Object {
    pub fn new(class: Rc<dyn Type>, parent_env: Option<Rc<dyn Env>>) -> Self {
        Self { class: Rc::downgrade(&class), env: CommonEnv::new(parent_env) }
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

    fn get(&self, name: &str) -> Option<Rc<dyn Var>> {
        self.env.get(name)
    }

    fn set(&self, name: String, value: Rc<dyn Var>) {
        self.env.set(name, value);
    }
}

#[derive(Debug)]
pub enum BoolExpr {
    Term { var_type: Weak<BoolType>, term: Rc<dyn Var> },
    Not { var_type: Weak<BoolType>, term: Rc<BoolExpr> },
    Eq { var_type: Weak<BoolType>, left: Rc<dyn Var>, right: Rc<dyn Var> },
    Lt { var_type: Weak<BoolType>, left: Rc<dyn Var>, right: Rc<dyn Var> },
    Leq { var_type: Weak<BoolType>, left: Rc<dyn Var>, right: Rc<dyn Var> },
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

fn push_negations(expr: Rc<dyn Var>) -> Rc<BoolExpr> {
    if let Ok(bool_expr) = expr.as_any().downcast::<BoolExpr>() {
        match bool_expr.as_ref() {
            BoolExpr::Not { var_type, term } => {
                let inner_expr = push_negations(term.clone());
                match inner_expr.as_any().downcast_ref::<BoolExpr>() {
                    Some(BoolExpr::Not { var_type: inner_var_type, term: inner_term }) => Rc::new(BoolExpr::Term { var_type: inner_var_type.clone(), term: inner_term.clone() }),
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
            _ => bool_expr.clone(),
        }
    } else {
        panic!("Expected a BoolExpr");
    }
}

fn distribute(expr: Rc<dyn Var>) -> Rc<BoolExpr> {
    if let Ok(bool_expr) = expr.as_any().downcast::<BoolExpr>() {
        match bool_expr.as_ref() {
            BoolExpr::Or { var_type, terms } => {
                let distributed_terms: Vec<Rc<BoolExpr>> = terms.iter().map(|t| distribute(t.clone())).collect();
                let mut result_terms: Vec<Rc<BoolExpr>> = vec![Rc::new(BoolExpr::Term {
                    var_type: var_type.clone(),
                    term: Rc::new(BoolExpr::And { var_type: var_type.clone(), terms: vec![] }),
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
            _ => bool_expr.clone(),
        }
    } else {
        panic!("Expected a BoolExpr");
    }
}

pub(crate) fn to_cnf(expr: Rc<dyn Var>) -> Rc<BoolExpr> {
    distribute(push_negations(expr))
}

pub fn get_var_by_path(env: &dyn Env, path: &[String]) -> Result<Rc<dyn Var>, RiddleError> {
    let (first, rest) = path.split_first().ok_or_else(|| RiddleError::RuntimeError("Empty variable path".into()))?;
    rest.iter().try_fold(env.get(first).ok_or_else(|| RiddleError::NotFound(first.to_string()))?, |acc, id| acc.as_env().ok_or_else(|| RiddleError::NotAnEnvironment(first.to_string()))?.get(id).ok_or_else(|| RiddleError::NotFound(format!("Variable '{}' in path", id))))
}
