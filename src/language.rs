use crate::{
    RiddleError,
    env::{BoolExpr, CommonEnv, Env, Slot, Var, get_var_by_path, to_cnf},
    scope::{Scope, Type, get_type_by_path, is_assignable_from},
};
use std::{
    collections::{HashMap, VecDeque},
    fmt,
    rc::Rc,
};

pub struct ProblemDef {
    pub methods: Vec<MethodDef>,
    pub predicates: Vec<PredicateDef>,
    pub classes: Vec<ClassDef>,
    pub statements: Vec<Statement>,
}

pub type FieldDef = (Vec<String>, Vec<(String, Option<Expr>)>); // (type, [(name, optional initializer)])

pub struct ClassDef {
    pub name: String,
    pub parents: Vec<Vec<String>>,
    pub fields: Vec<FieldDef>,
    pub constructors: Vec<ConstructorDef>,
    pub methods: Vec<MethodDef>,
    pub predicates: Vec<PredicateDef>,
    pub classes: Vec<ClassDef>,
}

pub struct ConstructorDef {
    pub args: Vec<(Vec<String>, String)>,
    pub init: Vec<(Vec<String>, Vec<Expr>)>,
    pub statements: Vec<Statement>,
}

pub struct MethodDef {
    pub return_type: Option<Vec<String>>,
    pub name: String,
    pub args: Vec<(Vec<String>, String)>,
    pub statements: Vec<Statement>,
}

pub struct PredicateDef {
    pub name: String,
    pub args: Vec<(Vec<String>, String)>,
    pub parents: Vec<Vec<String>>,
    pub statements: Vec<Statement>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Statement {
    Expr(Expr),
    LocalField { field_type: Vec<String>, fields: Vec<(String, Option<Expr>)> },
    Assign { name: Vec<String>, value: Expr },
    ForAll { var_type: Vec<String>, var_name: String, statements: Vec<Statement> },
    Disjunction { disjuncts: Vec<(Vec<Statement>, Expr)> },
    Formula { is_fact: bool, name: String, tau: Vec<String>, predicate_name: String, args: Vec<(String, Expr)> },
    Return { value: Expr },
}

impl fmt::Display for Statement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Statement::Expr(e) => write!(f, "{};", e),
            Statement::LocalField { field_type, fields } => write!(f, "{} {};", field_type.join("."), fields.iter().map(|(n, v)| format!("{}{}", n, v.as_ref().map(|v| format!(" = {}", v)).unwrap_or_default())).collect::<Vec<_>>().join(", ")),
            Statement::Assign { name, value } => write!(f, "{} = {};", name.join("."), value),
            Statement::ForAll { var_type, var_name, statements } => write!(f, "for {} {} {{\n{}\n}}", var_type.join("."), var_name, statements.iter().map(|s| format!("    {}", s)).collect::<Vec<_>>().join("\n")),
            Statement::Disjunction { disjuncts } => write!(f, "{{\n{}\n}}", disjuncts.iter().map(|(s, e)| format!("    {{\n{}\n    }}: {}", s.iter().map(|s| format!("        {}", s)).collect::<Vec<_>>().join("\n"), e)).collect::<Vec<_>>().join(" or ")),
            Statement::Formula { is_fact, name, tau, predicate_name, args } => write!(f, "{} {} = new {}{}({});", if *is_fact { "fact" } else { "formula" }, name, if tau.is_empty() { String::new() } else { tau.join(".") + "." }, predicate_name, args.iter().map(|(n, e)| format!("{}: {}", n, e)).collect::<Vec<_>>().join(", ")),
            Statement::Return { value } => write!(f, "return {};", value),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Expr {
    Bool(bool),
    Int(i64),
    Real(i64, i64),
    String(String),
    QualifiedId { ids: Vec<String> },
    Sum { terms: Vec<Expr> },
    Opposite { term: Box<Expr> },
    Not { term: Box<Expr> },
    Mul { factors: Vec<Expr> },
    Div { left: Box<Expr>, right: Box<Expr> },
    Function { name: Vec<String>, args: Vec<Expr> },
    Eq { left: Box<Expr>, right: Box<Expr> },
    Neq { left: Box<Expr>, right: Box<Expr> },
    Lt { left: Box<Expr>, right: Box<Expr> },
    Leq { left: Box<Expr>, right: Box<Expr> },
    Gt { left: Box<Expr>, right: Box<Expr> },
    Geq { left: Box<Expr>, right: Box<Expr> },
    Or { terms: Vec<Expr> },
    And { terms: Vec<Expr> },
    NewObject { class_name: Vec<String>, args: Vec<Expr> },
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expr::Bool(b) => write!(f, "{}", b),
            Expr::Int(i) => write!(f, "{}", i),
            Expr::Real(n, d) => write!(f, "{}/{}", n, d),
            Expr::String(s) => write!(f, "\"{}\"", s),
            Expr::QualifiedId { ids } => write!(f, "{}", ids.join(".")),
            Expr::Sum { terms } => write!(f, "({})", terms.iter().map(|t| format!("{}", t)).collect::<Vec<_>>().join(" + ")),
            Expr::Opposite { term } => write!(f, "-({})", term),
            Expr::Not { term } => write!(f, "!({})", term),
            Expr::Mul { factors } => write!(f, "({})", factors.iter().map(|t| format!("{}", t)).collect::<Vec<_>>().join(" * ")),
            Expr::Div { left, right } => write!(f, "({} / {})", left, right),
            Expr::Function { name, args } => write!(f, "{}({})", name.join("."), args.iter().map(|a| format!("{}", a)).collect::<Vec<_>>().join(", ")),
            Expr::Eq { left, right } => write!(f, "({} == {})", left, right),
            Expr::Neq { left, right } => write!(f, "({} != {})", left, right),
            Expr::Lt { left, right } => write!(f, "({} < {})", left, right),
            Expr::Leq { left, right } => write!(f, "({} <= {})", left, right),
            Expr::Gt { left, right } => write!(f, "({} > {})", left, right),
            Expr::Geq { left, right } => write!(f, "({} >= {})", left, right),
            Expr::Or { terms } => write!(f, "({})", terms.iter().map(|t| format!("{}", t)).collect::<Vec<_>>().join(" || ")),
            Expr::And { terms } => write!(f, "({})", terms.iter().map(|t| format!("{}", t)).collect::<Vec<_>>().join(" && ")),
            Expr::NewObject { class_name, args } => write!(f, "new {}({})", class_name.join("."), args.iter().map(|a| format!("{}", a)).collect::<Vec<_>>().join(", ")),
        }
    }
}

pub struct Disjunction {
    pub scp: Rc<dyn Scope>,
    pub env: Rc<dyn Env>,
    pub disjuncts: Vec<(Vec<Statement>, Expr)>,
}

pub fn execute(scp: &Rc<dyn Scope>, env: Rc<dyn Env>, stmt: &Statement) -> Result<(), RiddleError> {
    match stmt {
        Statement::Expr(expr) => {
            let expr = evaluate(scp.as_ref(), env.as_ref(), expr)?;
            if let Slot::Primitive(var) = expr.clone()
                && let Ok(bool_expr) = var.as_any().downcast::<BoolExpr>()
            {
                scp.core().assert(to_cnf(bool_expr));
                Ok(())
            } else {
                Err(RiddleError::RuntimeError(format!("Expected boolean expression, got {}", expr)))
            }
        }
        Statement::LocalField { field_type, fields } => {
            let fld_tp = get_type_by_path(scp.as_ref(), field_type)?;
            for (name, default) in fields {
                if let Some(expr) = default {
                    let value = evaluate(scp.as_ref(), env.as_ref(), expr)?;
                    match &value {
                        Slot::Primitive(var) => {
                            if !is_assignable_from(&fld_tp, &var.var_type()) {
                                return Err(RiddleError::TypeError(format!("Default value for field '{}' is not assignable to field type '{}'", name, field_type.join("."))));
                            }
                        }
                        Slot::ObjectRef(obj_id) => {
                            let obj = scp.core().get_object(*obj_id).ok_or_else(|| RiddleError::NotFound(format!("Object with id {} not found", obj_id.0)))?;
                            let obj_type: Rc<dyn Type> = obj.class();
                            if !is_assignable_from(&fld_tp, &obj_type) {
                                return Err(RiddleError::TypeError(format!("Default value for field '{}' is not assignable to field type '{}'", name, field_type.join("."))));
                            }
                        }
                        Slot::AtomRef(atom_id) => {
                            let atom = scp.core().get_atom(*atom_id).ok_or_else(|| RiddleError::NotFound(format!("Atom with id {} not found", atom_id.0)))?;
                            let atom_type: Rc<dyn Type> = atom.predicate();
                            if !is_assignable_from(&fld_tp, &atom_type) {
                                return Err(RiddleError::TypeError(format!("Default value for field '{}' is not assignable to field type '{}'", name, field_type.join("."))));
                            }
                        }
                    }
                    env.set(name.clone(), value);
                } else if let Some(class) = fld_tp.clone().as_class() {
                    let instances = class.instances();
                    if instances.is_empty() {
                        return Err(RiddleError::RuntimeError(format!("No instances found for field '{}' of type '{}'", name, class.full_name())));
                    } else if instances.len() == 1 {
                        env.set(name.clone(), Slot::ObjectRef(instances[0]));
                    } else {
                        env.set(name.clone(), scp.core().new_var(class, instances.as_slice())?);
                    }
                } else {
                    env.set(name.clone(), fld_tp.clone().new_instance());
                }
            }
            Ok(())
        }
        Statement::Assign { name, value } => {
            let value = evaluate(scp.as_ref(), env.as_ref(), value)?;
            if name.len() == 1 {
                env.set(name[0].clone(), value);
                Ok(())
            } else {
                let (last, rest) = name.split_last().ok_or_else(|| RiddleError::RuntimeError("Empty assignment path".into()))?;
                let var = get_var_by_path(scp.core().as_ref(), env.as_ref(), rest)?;
                match &var {
                    Slot::Primitive(_) => return Err(RiddleError::NotAnEnvironment(format!("Variable '{}' in assignment path is a primitive variable, cannot assign to '{}'", rest.join("."), last))),
                    Slot::ObjectRef(obj_id) => {
                        let obj = scp.core().get_object(*obj_id).ok_or_else(|| RiddleError::NotFound(format!("Object with id {} not found", obj_id.0)))?;
                        obj.as_env().ok_or_else(|| RiddleError::NotAnEnvironment(format!("Object with id {} does not have an environment", obj_id.0)))?.set(last.to_string(), value);
                        Ok(())
                    }
                    Slot::AtomRef(atom_id) => {
                        let atom = scp.core().get_atom(*atom_id).ok_or_else(|| RiddleError::NotFound(format!("Atom with id {} not found", atom_id.0)))?;
                        atom.as_env().ok_or_else(|| RiddleError::NotAnEnvironment(format!("Atom with id {} does not have an environment", atom_id.0)))?.set(last.to_string(), value);
                        Ok(())
                    }
                }
            }
        }
        Statement::ForAll { var_type, var_name, statements } => {
            let class = get_type_by_path(scp.as_ref(), var_type)?.as_class().ok_or_else(|| RiddleError::NotAClass(var_type.join(".")))?;
            for instance in class.instances() {
                let loop_env = Rc::new(CommonEnv::new(Some(env.clone())));
                loop_env.set(var_name.clone(), Slot::ObjectRef(instance));
                for stmt in statements {
                    execute(scp, loop_env.clone(), stmt)?;
                }
            }
            Ok(())
        }
        Statement::Disjunction { disjuncts } => {
            let disjunction = Disjunction { scp: scp.clone(), env: env.clone(), disjuncts: disjuncts.clone() };
            scp.core().new_disjunction(disjunction);
            Ok(())
        }
        Statement::Formula { is_fact, name, tau, predicate_name, args } => {
            let tau = if tau.is_empty() { None } else { Some(get_var_by_path(scp.core().as_ref(), env.as_ref(), tau)?) };
            let predicate = if let Some(tau) = tau.as_ref() {
                let tau = match tau {
                    Slot::Primitive(var) => Err(RiddleError::NotAClass(format!("Tau variable is a primitive variable of type '{}', expected a class", var.var_type().full_name()))),
                    Slot::ObjectRef(obj_id) => scp.core().get_object(*obj_id).ok_or_else(|| RiddleError::NotFound(format!("Object with id {} not found", obj_id.0))),
                    Slot::AtomRef(atom_id) => Err(RiddleError::NotAClass(format!("Tau variable is an atom with id {}, expected a class", atom_id.0))),
                }?;
                tau.var_type().as_class().ok_or_else(|| RiddleError::NotAClass(format!("Type '{}' in tau path", tau.var_type().full_name())))?.get_predicate(predicate_name).ok_or_else(|| RiddleError::NotFound(format!("Predicate '{}' in class '{}'", predicate_name, tau.var_type().full_name())))?
            } else {
                scp.get_predicate(predicate_name).ok_or_else(|| RiddleError::NotFound(format!("Predicate '{}'", predicate_name)))?
            };
            let mut args: HashMap<String, Slot> = args
                .iter()
                .map(|(n, e)| {
                    let val = evaluate(scp.as_ref(), env.as_ref(), e)?;
                    Ok((n.clone(), val))
                })
                .collect::<Result<_, _>>()?;
            if let Some(tau) = tau {
                args.insert("tau".to_string(), tau);
            }
            let mut pred_hierarchy = VecDeque::from(vec![predicate.clone()]);
            while let Some(pred) = pred_hierarchy.pop_front() {
                for (arg_type, name) in pred.args() {
                    if !args.contains_key(name) {
                        let arg_tp = get_type_by_path(scp.as_ref(), arg_type)?;
                        if let Some(class) = arg_tp.clone().as_class() {
                            let instances = class.instances();
                            if instances.is_empty() {
                                return Err(RiddleError::RuntimeError(format!("No instances found for argument '{}' of type '{}'", name, class.full_name())));
                            } else if instances.len() == 1 {
                                args.insert(name.clone(), Slot::ObjectRef(instances[0]));
                            } else {
                                args.insert(name.clone(), scp.core().new_var(class, instances.as_slice())?);
                            }
                        } else {
                            args.insert(name.clone(), arg_tp.new_instance());
                        }
                    }
                }
                for parent_path in pred.parents() {
                    let (predicate_name, class_path) = parent_path.split_last().ok_or_else(|| RiddleError::RuntimeError("Empty parent predicate path".into()))?;
                    let parent_predicate = if class_path.is_empty() {
                        scp.get_predicate(predicate_name).ok_or_else(|| RiddleError::NotFound(format!("Predicate '{}' in parent path", predicate_name)))?
                    } else {
                        let class = get_type_by_path(scp.as_ref(), class_path)?.as_class().ok_or_else(|| RiddleError::NotAClass(format!("Type '{}' in parent path", class_path.join("."))))?;
                        class.get_predicate(predicate_name).ok_or_else(|| RiddleError::NotFound(format!("Predicate '{}' in class '{}'", predicate_name, class.full_name())))?
                    };
                    pred_hierarchy.push_back(parent_predicate);
                }
            }
            let atom = scp.core().new_atom(predicate, *is_fact, args);
            env.set(name.clone(), Slot::AtomRef(atom));
            Ok(())
        }
        Statement::Return { value } => {
            let ret = evaluate(scp.as_ref(), env.as_ref(), value)?;
            env.set("__return".to_string(), ret);
            Ok(())
        }
    }
}

pub fn evaluate(scp: &dyn Scope, env: &dyn Env, expr: &Expr) -> Result<Slot, RiddleError> {
    match expr {
        Expr::Bool(bool) => Ok(scp.core().new_bool(*bool)),
        Expr::Int(int) => Ok(scp.core().new_int(*int)),
        Expr::Real(num, den) => Ok(scp.core().new_real(*num, *den)),
        Expr::String(string) => Ok(scp.core().new_string(string)),
        Expr::QualifiedId { ids } => get_var_by_path(scp.core().as_ref(), env, ids),
        Expr::Sum { terms } => {
            let evaluated_terms: Vec<Slot> = terms.iter().map(|t| evaluate(scp, env, t)).collect::<Result<_, _>>()?;
            Ok(scp.core().sum(&evaluated_terms)?)
        }
        Expr::Opposite { term } => {
            let evaluated_term = evaluate(scp, env, term)?;
            Ok(scp.core().opposite(evaluated_term)?)
        }
        Expr::Not { term } => {
            let evaluated_term = evaluate(scp, env, term)?;
            match &evaluated_term {
                Slot::Primitive(var) => {
                    if let Ok(bool_expr) = var.clone().as_any().downcast::<BoolExpr>() {
                        Ok(Slot::Primitive(Rc::new(BoolExpr::Not { var_type: Rc::downgrade(&scp.core().bool_type()), term: bool_expr })))
                    } else {
                        Err(RiddleError::RuntimeError(format!("Expected boolean expression, got {}", evaluated_term)))
                    }
                }
                _ => Err(RiddleError::RuntimeError(format!("Expected a primitive variable for negation, got {}", evaluated_term))),
            }
        }
        Expr::Mul { factors } => {
            let evaluated_factors: Vec<Slot> = factors.iter().map(|f| evaluate(scp, env, f)).collect::<Result<_, _>>()?;
            Ok(scp.core().mul(&evaluated_factors)?)
        }
        Expr::Div { left, right } => {
            let evaluated_left = evaluate(scp, env, left)?;
            let evaluated_right = evaluate(scp, env, right)?;
            Ok(scp.core().div(evaluated_left, evaluated_right)?)
        }
        _ => unimplemented!(),
    }
}
