use crate::{
    RiddleError,
    env::{BoolExpr, Env, ObjectId, Slot, Var, get_var_by_path, to_cnf},
    scope::{Scope, Type, get_type_by_path, is_assignable_from},
};
use std::{fmt, rc::Rc};

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

pub fn execute(scp: &dyn Scope, env: &dyn Env, stmt: &Statement) -> Result<(), RiddleError> {
    match stmt {
        Statement::Expr(expr) => {
            let expr = evaluate(scp, env, expr)?;
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
            let fld_tp = get_type_by_path(scp, field_type)?;
            for (name, default) in fields {
                if let Some(expr) = default {
                    let value = evaluate(scp, env, expr)?;
                    match &value {
                        Slot::Primitive(var) => {
                            if !is_assignable_from(&fld_tp, &var.var_type()) {
                                return Err(RiddleError::TypeError(format!("Default value for field '{}' is not assignable to field type '{}'", name, field_type.join("."))));
                            }
                        }
                        Slot::ObjectRef(obj_id) => {
                            let obj = scp.core().get_object(*obj_id).ok_or_else(|| RiddleError::NotFound(format!("Object with id {} not found", obj_id.0)))?;
                            if !is_assignable_from(&fld_tp, &obj.class()) {
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
                        env.set(name.clone(), instances[0].clone());
                    } else {
                        let instances: Vec<ObjectId> = instances
                            .iter()
                            .map(|instance| match instance {
                                Slot::Primitive(_var) => Err(RiddleError::RuntimeError(format!("Field '{}' has multiple instances, but one of them is a primitive variable", name))),
                                Slot::ObjectRef(obj_id) => Ok(*obj_id),
                                Slot::AtomRef(_atom_id) => Err(RiddleError::RuntimeError(format!("Field '{}' has multiple instances, but one of them is an atom", name))),
                            })
                            .collect::<Result<Vec<_>, _>>()?;
                        env.set(name.clone(), scp.core().new_var(class, instances.as_slice())?);
                    }
                } else {
                    env.set(name.clone(), fld_tp.clone().new_instance());
                }
            }
            Ok(())
        }
        Statement::Assign { name, value } => {
            let value = evaluate(scp, env, value)?;
            if name.len() == 1 {
                env.set(name[0].clone(), value);
                Ok(())
            } else {
                let (last, rest) = name.split_last().ok_or_else(|| RiddleError::RuntimeError("Empty assignment path".into()))?;
                let var = get_var_by_path(scp.core().as_ref(), env, rest)?;
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
        _ => unimplemented!(),
    }
}

pub fn evaluate(scp: &dyn Scope, _env: &dyn Env, expr: &Expr) -> Result<Slot, RiddleError> {
    match expr {
        Expr::Bool(bool) => Ok(scp.core().new_bool(*bool)),
        _ => unimplemented!(),
    }
}
