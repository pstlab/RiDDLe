use crate::{
    env::{BoolExpr, CommonEnv, Env, Var, to_cnf},
    scope::{Scope, is_assignable_from},
};
use std::{
    collections::{HashMap, VecDeque},
    fmt,
    rc::Rc,
};

#[derive(Debug)]
pub enum RiddleError {
    NotAnEnvironment(String),
    NotAClass(String),
    NotAPredicate(String),
    TypeError(String),
    NotFound(String),
    InconsistencyError(String),
    RuntimeError(String),
}

impl fmt::Display for RiddleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RiddleError::NotAnEnvironment(name) => write!(f, "Variable '{}' is not an environment", name),
            RiddleError::NotAClass(name) => write!(f, "Type '{}' is not a class", name),
            RiddleError::NotAPredicate(name) => write!(f, "Predicate '{}' not found", name),
            RiddleError::TypeError(msg) => write!(f, "Type error: {}", msg),
            RiddleError::NotFound(name) => write!(f, "'{}' not found", name),
            RiddleError::InconsistencyError(msg) => write!(f, "Inconsistency error: {}", msg),
            RiddleError::RuntimeError(msg) => write!(f, "Runtime error: {}", msg),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct ProblemDef {
    pub methods: Vec<MethodDef>,
    pub predicates: Vec<PredicateDef>,
    pub classes: Vec<ClassDef>,
    pub statements: Vec<Statement>,
}

pub type FieldDef = (Vec<String>, Vec<(String, Option<Expr>)>); // (type, [(name, optional initializer)])

#[derive(Debug, PartialEq)]
pub struct ClassDef {
    pub name: String,
    pub parents: Vec<Vec<String>>,
    pub fields: Vec<FieldDef>,
    pub constructors: Vec<ConstructorDef>,
    pub methods: Vec<MethodDef>,
    pub predicates: Vec<PredicateDef>,
    pub classes: Vec<ClassDef>,
}

#[derive(Debug, PartialEq)]
pub struct ConstructorDef {
    pub args: Vec<(Vec<String>, String)>,
    pub init: Vec<(String, Vec<Expr>)>,
    pub statements: Vec<Statement>,
}

#[derive(Debug, PartialEq)]
pub struct MethodDef {
    pub return_type: Option<Vec<String>>,
    pub name: String,
    pub args: Vec<(Vec<String>, String)>,
    pub statements: Vec<Statement>,
}

#[derive(Debug, PartialEq)]
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

impl fmt::Display for ProblemDef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for method in &self.methods {
            writeln!(f, "{}", method)?;
        }
        for predicate in &self.predicates {
            writeln!(f, "{}", predicate)?;
        }
        for class in &self.classes {
            writeln!(f, "{}", class)?;
        }
        for statement in &self.statements {
            writeln!(f, "{}", statement)?;
        }
        Ok(())
    }
}

impl fmt::Display for ClassDef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "class {}{} {{", self.name, if !self.parents.is_empty() { format!(" extends {}", self.parents.iter().map(|p| p.join(".")).collect::<Vec<_>>().join(", ")) } else { String::new() })?;
        for (field_type, fields) in &self.fields {
            writeln!(f, "    {} {};", field_type.join("."), fields.iter().map(|(n, v)| format!("{}{}", n, v.as_ref().map(|v| format!(" = {}", v)).unwrap_or_default())).collect::<Vec<_>>().join(", "))?;
        }
        for constructor in &self.constructors {
            writeln!(f, "    {}", constructor)?;
        }
        for method in &self.methods {
            writeln!(f, "    {}", method)?;
        }
        for predicate in &self.predicates {
            writeln!(f, "    {}", predicate)?;
        }
        write!(f, "}}")
    }
}

impl fmt::Display for ConstructorDef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "constructor({}) {{\n{}\n}}", self.args.iter().map(|(t, n)| format!("{} {}", t.join("."), n)).collect::<Vec<_>>().join(", "), self.statements.iter().map(|s| format!("    {}", s)).collect::<Vec<_>>().join("\n"))
    }
}

impl fmt::Display for MethodDef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {}({}) {{\n{}\n}}",
            self.return_type.as_ref().map(|t| t.join(".")).unwrap_or_else(|| "void".to_string()),
            self.name,
            self.args.iter().map(|(t, n)| format!("{} {}", t.join("."), n)).collect::<Vec<_>>().join(", "),
            self.statements.iter().map(|s| format!("    {}", s)).collect::<Vec<_>>().join("\n")
        )
    }
}

impl fmt::Display for PredicateDef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "predicate {}({}) {{\n{}\n}}", self.name, self.args.iter().map(|(t, n)| format!("{} {}", t.join("."), n)).collect::<Vec<_>>().join(", "), self.statements.iter().map(|s| format!("    {}", s)).collect::<Vec<_>>().join("\n"))
    }
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

pub fn execute(scp: Rc<dyn Scope>, env: Rc<dyn Env>, stmt: &Statement) -> Result<(), RiddleError> {
    match stmt {
        Statement::Expr(expr) => {
            if scp.clone().core().assert(to_cnf(evaluate(scp.clone(), env.clone(), expr)?)) {
                Ok(())
            } else {
                Err(RiddleError::InconsistencyError(format!("Expression '{}' evaluated to false", expr)))
            }
        }
        Statement::LocalField { field_type, fields } => {
            let (first, rest) = field_type.split_first().ok_or_else(|| RiddleError::RuntimeError("Empty field type path".into()))?;
            let fld_tp = scp.get_type(first).ok_or_else(|| RiddleError::NotFound(first.to_string()))?;
            rest.iter().try_fold(fld_tp.clone(), |acc, id| acc.as_class().ok_or_else(|| RiddleError::NotAClass(first.to_string()))?.get_type(id).ok_or_else(|| RiddleError::NotFound(format!("Class '{}' in path", id))))?;
            for (name, default) in fields {
                if let Some(expr) = default {
                    let value = evaluate(scp.clone(), env.clone(), expr)?;
                    if !is_assignable_from(&fld_tp, &value.var_type()) {
                        return Err(RiddleError::TypeError(format!("Default value for field '{}' is not assignable to field type '{}'", name, field_type.join("."))));
                    }
                    env.set(name.clone(), value);
                } else if let Some(class) = fld_tp.clone().as_class() {
                    let instances = class.instances().into_iter().map(|obj| obj as Rc<dyn Var>).collect::<Vec<_>>();
                    if instances.is_empty() {
                        return Err(RiddleError::RuntimeError(format!("No instances found for field '{}' of type '{}'", name, class.full_name())));
                    } else if instances.len() == 1 {
                        env.set(name.clone(), instances[0].clone());
                    } else {
                        env.set(name.clone(), scp.clone().core().new_var(class, instances.as_slice())?);
                    }
                } else {
                    env.set(name.clone(), fld_tp.clone().new_instance());
                }
            }
            Ok(())
        }
        Statement::Assign { name, value } => {
            let value = evaluate(scp.clone(), env.clone(), value)?;
            if name.len() == 1 {
                env.set(name[0].clone(), value);
                Ok(())
            } else {
                let (first, rest) = name.split_first().ok_or_else(|| RiddleError::RuntimeError("Empty assignment path".into()))?;
                let root = env.get(first).ok_or_else(|| RiddleError::NotFound(first.to_string()))?;
                let (last, rest) = rest.split_last().ok_or_else(|| RiddleError::RuntimeError("Empty assignment path".into()))?;
                rest.iter().try_fold(root, |acc, id| acc.as_env().ok_or_else(|| RiddleError::NotAnEnvironment(id.to_string()))?.get(id).ok_or_else(|| RiddleError::NotFound(format!("Member '{}' in path", id))))?.as_env().ok_or_else(|| RiddleError::NotAnEnvironment(last.to_string()))?.set(last.to_string(), value);
                Ok(())
            }
        }
        Statement::ForAll { var_type, var_name, statements } => {
            let (first, rest) = var_type.split_first().ok_or_else(|| RiddleError::RuntimeError("Empty variable type path".into()))?;
            let class = scp.get_type(first).ok_or_else(|| RiddleError::NotFound(first.to_string()))?.as_class().ok_or_else(|| RiddleError::NotAClass(first.to_string()))?;
            rest.iter().try_fold(class.clone(), |acc, id| acc.get_type(id).ok_or_else(|| RiddleError::NotFound(format!("Class '{}' in path", id)))?.as_class().ok_or_else(|| RiddleError::NotAClass(id.to_string())))?;
            for instance in class.instances() {
                let loop_env = Rc::new(CommonEnv::new(Some(env.clone())));
                loop_env.set(var_name.clone(), instance);
                for stmt in statements {
                    execute(scp.clone(), loop_env.clone(), stmt)?;
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
            let tau = if tau.is_empty() {
                None
            } else {
                let (first, rest) = tau.split_first().ok_or_else(|| RiddleError::RuntimeError("Empty identifier path".into()))?;
                let root = env.get(first).ok_or_else(|| RiddleError::NotFound(first.to_string()))?;
                let root = rest.iter().try_fold(root, |acc, id| acc.as_env().ok_or_else(|| RiddleError::NotAnEnvironment(id.to_string()))?.get(id).ok_or_else(|| RiddleError::NotFound(format!("Member '{}' in path", id))))?;
                Some(root)
            };
            let predicate = if let Some(tau) = tau.clone() {
                tau.as_ref().var_type().as_class().ok_or_else(|| RiddleError::NotAClass(format!("Type '{}' in tau path", tau.var_type().full_name())))?.get_predicate(predicate_name).ok_or_else(|| RiddleError::NotFound(format!("Predicate '{}' in class '{}'", predicate_name, tau.var_type().full_name())))?
            } else {
                scp.get_predicate(predicate_name).ok_or_else(|| RiddleError::NotFound(format!("Predicate '{}'", predicate_name)))?
            };
            let mut args: HashMap<String, Rc<dyn Var>> = args
                .iter()
                .map(|(n, e)| {
                    let val = evaluate(scp.clone(), env.clone(), e)?;
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
                        let (first, rest) = arg_type.split_first().ok_or_else(|| RiddleError::RuntimeError("Empty argument type path".into()))?;
                        let arg_tp = scp.get_type(first).ok_or_else(|| RiddleError::NotFound(format!("Class '{}' in argument type path", first)))?;
                        let arg_tp = rest.iter().try_fold(arg_tp, |acc, id| acc.as_class().ok_or_else(|| RiddleError::NotAClass(format!("Class '{}' in argument type path", first)))?.get_type(id).ok_or_else(|| RiddleError::NotFound(format!("Class '{}' in argument type path", id))))?;
                        if let Some(class) = arg_tp.clone().as_class() {
                            let instances = class.instances().into_iter().map(|obj| obj as Rc<dyn Var>).collect::<Vec<_>>();
                            if instances.is_empty() {
                                return Err(RiddleError::RuntimeError(format!("No instances found for argument '{}' of type '{}'", name, class.full_name())));
                            } else if instances.len() == 1 {
                                args.insert(name.clone(), instances[0].clone());
                            } else {
                                args.insert(name.clone(), scp.clone().core().new_var(class, instances.as_slice())?);
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
                        let (first_class, nested_classes) = class_path.split_first().ok_or_else(|| RiddleError::RuntimeError("Empty parent predicate path".into()))?;
                        let class_type = scp.get_type(first_class).ok_or_else(|| RiddleError::NotFound(format!("Class '{}' in parent path", first_class)))?;
                        let class_type = nested_classes.iter().try_fold(class_type, |acc, class_name| {
                            let acc_name = acc.full_name();
                            acc.clone().as_class().ok_or_else(|| RiddleError::NotAClass(format!("Type '{}' in parent path is not a class", acc_name)))?.get_type(class_name).ok_or_else(|| RiddleError::NotFound(format!("Class '{}' in parent path", class_name)))
                        })?;
                        let class_type_name = class_type.full_name();
                        class_type.clone().as_class().ok_or_else(|| RiddleError::NotAClass(format!("Type '{}' in parent path is not a class", class_type_name)))?.get_predicate(predicate_name).ok_or_else(|| RiddleError::NotFound(format!("Predicate '{}' in parent path", predicate_name)))?
                    };
                    pred_hierarchy.push_back(parent_predicate);
                }
            }
            let atom = predicate.new_atom(*is_fact, args);
            env.set(name.clone(), atom);
            Ok(())
        }
        Statement::Return { value } => {
            let ret = evaluate(scp.clone(), env.clone(), value)?;
            env.set("__return".to_string(), ret);
            Ok(())
        }
    }
}

pub fn evaluate(scp: Rc<dyn Scope>, env: Rc<dyn Env>, expr: &Expr) -> Result<Rc<dyn Var>, RiddleError> {
    match expr {
        Expr::Bool(bool) => {
            let evaluated_term = scp.clone().core().new_bool(*bool);
            Ok(Rc::new(BoolExpr::Term { var_type: Rc::downgrade(&scp.core().bool_type()), term: evaluated_term }))
        }
        Expr::Int(int) => Ok(scp.core().new_int(*int)),
        Expr::Real(num, den) => Ok(scp.core().new_real(*num, *den)),
        Expr::String(string) => Ok(scp.core().new_string(string)),
        Expr::QualifiedId { ids } => {
            let (first, rest) = ids.split_first().ok_or_else(|| RiddleError::RuntimeError("Empty identifier path".into()))?;
            let root = env.get(first).ok_or_else(|| RiddleError::NotFound(first.to_string()))?;
            rest.iter().try_fold(root, |acc, id| acc.as_env().ok_or_else(|| RiddleError::NotAnEnvironment(id.to_string()))?.get(id).ok_or_else(|| RiddleError::NotFound(format!("Member '{}' in path", id))))
        }
        Expr::Sum { terms } => {
            let evaluated_terms: Vec<Rc<dyn Var>> = terms.iter().map(|t| evaluate(scp.clone(), env.clone(), t)).collect::<Result<_, _>>()?;
            Ok(scp.core().sum(&evaluated_terms)?)
        }
        Expr::Opposite { term } => {
            let evaluated_term = evaluate(scp.clone(), env, term)?;
            Ok(scp.core().opposite(evaluated_term)?)
        }
        Expr::Not { term } => {
            let evaluated_term = evaluate(scp.clone(), env, term)?.as_any().downcast::<BoolExpr>().map_err(|_| RiddleError::TypeError("Expected a boolean expression in 'not' operator".into()))?;
            Ok(Rc::new(BoolExpr::Not { var_type: Rc::downgrade(&scp.core().bool_type()), term: evaluated_term }))
        }
        Expr::Mul { factors } => {
            let evaluated_factors: Vec<Rc<dyn Var>> = factors.iter().map(|f| evaluate(scp.clone(), env.clone(), f)).collect::<Result<_, _>>()?;
            Ok(scp.core().mul(&evaluated_factors)?)
        }
        Expr::Div { left, right } => {
            let evaluated_left = evaluate(scp.clone(), env.clone(), left)?;
            let evaluated_right = evaluate(scp.clone(), env, right)?;
            Ok(scp.core().div(evaluated_left, evaluated_right)?)
        }
        Expr::Function { name, args } => {
            let evaluated_args: Vec<Rc<dyn Var>> = args.iter().map(|a| evaluate(scp.clone(), env.clone(), a)).collect::<Result<_, _>>()?;
            let method = scp.get_method(name.last().unwrap(), &evaluated_args.iter().map(|arg| arg.var_type()).collect::<Vec<_>>()).ok_or_else(|| RiddleError::NotFound(format!("Method '{}' with specified argument types", name.join("."))))?;
            method.call(env, evaluated_args)?.ok_or_else(|| RiddleError::RuntimeError(format!("Method '{}' did not return a value", name.join("."))))
        }
        Expr::Eq { left, right } => {
            let evaluated_left = evaluate(scp.clone(), env.clone(), left)?;
            let evaluated_right = evaluate(scp.clone(), env, right)?;
            Ok(Rc::new(BoolExpr::Eq {
                var_type: Rc::downgrade(&scp.core().bool_type()),
                left: evaluated_left,
                right: evaluated_right,
            }))
        }
        Expr::Neq { left, right } => {
            let evaluated_left = evaluate(scp.clone(), env.clone(), left)?;
            let evaluated_right = evaluate(scp.clone(), env, right)?;
            Ok(Rc::new(BoolExpr::Not {
                var_type: Rc::downgrade(&scp.clone().core().bool_type()),
                term: Rc::new(BoolExpr::Eq {
                    var_type: Rc::downgrade(&scp.core().bool_type()),
                    left: evaluated_left,
                    right: evaluated_right,
                }),
            }))
        }
        Expr::Lt { left, right } => {
            let evaluated_left = evaluate(scp.clone(), env.clone(), left)?;
            let evaluated_right = evaluate(scp.clone(), env, right)?;
            Ok(Rc::new(BoolExpr::Lt {
                var_type: Rc::downgrade(&scp.core().bool_type()),
                left: evaluated_left,
                right: evaluated_right,
            }))
        }
        Expr::Leq { left, right } => {
            let evaluated_left = evaluate(scp.clone(), env.clone(), left)?;
            let evaluated_right = evaluate(scp.clone(), env, right)?;
            Ok(Rc::new(BoolExpr::Leq {
                var_type: Rc::downgrade(&scp.core().bool_type()),
                left: evaluated_left,
                right: evaluated_right,
            }))
        }
        Expr::Geq { left, right } => {
            let evaluated_left = evaluate(scp.clone(), env.clone(), left)?;
            let evaluated_right = evaluate(scp.clone(), env, right)?;
            Ok(Rc::new(BoolExpr::Leq {
                var_type: Rc::downgrade(&scp.core().bool_type()),
                left: evaluated_right,
                right: evaluated_left,
            }))
        }
        Expr::Gt { left, right } => {
            let evaluated_left = evaluate(scp.clone(), env.clone(), left)?;
            let evaluated_right = evaluate(scp.clone(), env, right)?;
            Ok(Rc::new(BoolExpr::Lt {
                var_type: Rc::downgrade(&scp.core().bool_type()),
                left: evaluated_right,
                right: evaluated_left,
            }))
        }
        Expr::Or { terms } => {
            let evaluated_terms: Vec<Rc<BoolExpr>> = terms.iter().map(|t| evaluate(scp.clone(), env.clone(), t).expect("Expected a boolean expression in 'or' operator").as_any().downcast::<BoolExpr>().expect("Expected a boolean expression in 'or' operator")).collect();
            Ok(Rc::new(BoolExpr::Or { var_type: Rc::downgrade(&scp.core().bool_type()), terms: evaluated_terms }))
        }
        Expr::And { terms } => {
            let evaluated_terms: Vec<Rc<BoolExpr>> = terms.iter().map(|t| evaluate(scp.clone(), env.clone(), t).expect("Expected a boolean expression in 'and' operator").as_any().downcast::<BoolExpr>().expect("Expected a boolean expression in 'and' operator")).collect();
            Ok(Rc::new(BoolExpr::And { var_type: Rc::downgrade(&scp.core().bool_type()), terms: evaluated_terms }))
        }
        Expr::NewObject { class_name, args } => {
            let (first, rest) = class_name.split_first().ok_or_else(|| RiddleError::RuntimeError("Empty class name".into()))?;
            let class = scp.get_type(first).ok_or_else(|| RiddleError::NotFound(first.to_string()))?.as_class().ok_or_else(|| RiddleError::NotAClass(first.to_string()))?;
            let class = rest.iter().try_fold(class.clone(), |acc, id| acc.get_type(id).ok_or_else(|| RiddleError::NotFound(format!("Class '{}' in path", id)))?.as_class().ok_or_else(|| RiddleError::NotAClass(id.to_string())))?;
            let evaluated_args: Vec<Rc<dyn Var>> = args.iter().map(|a| evaluate(scp.clone(), env.clone(), a)).collect::<Result<_, _>>()?;
            let constructor = class.constructor(&evaluated_args.iter().map(|arg| arg.var_type()).collect::<Vec<_>>()).ok_or_else(|| RiddleError::NotFound(format!("Constructor for class '{}' with specified argument types", class_name.join("."))))?;
            let object = class.clone().new_instance();
            constructor.call(object.clone(), evaluated_args)
        }
    }
}
