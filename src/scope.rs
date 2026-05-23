use crate::{
    RiddleError,
    core::Core,
    env::{BoolExpr, ObjectId, Slot},
    language::{Expr, Statement},
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
    fn as_class(self: Rc<Self>) -> Option<Rc<dyn Class>> {
        None
    }

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

pub struct IntType {
    core: Weak<dyn Core>,
}

impl IntType {
    /// Creates the built-in integer type.
    pub fn new(core: Weak<dyn Core>) -> Self {
        Self { core }
    }
}

impl Type for IntType {
    fn name(&self) -> &str {
        "int"
    }

    fn as_any(self: Rc<Self>) -> Rc<dyn Any> {
        self
    }

    fn new_instance(self: Rc<Self>) -> Slot {
        self.core.upgrade().unwrap().new_int_var()
    }
}

pub struct RealType {
    core: Weak<dyn Core>,
}

impl RealType {
    /// Creates the built-in real (floating-point) type.
    pub fn new(core: Weak<dyn Core>) -> Self {
        Self { core }
    }
}

impl Type for RealType {
    fn name(&self) -> &str {
        "real"
    }

    fn as_any(self: Rc<Self>) -> Rc<dyn Any> {
        self
    }

    fn new_instance(self: Rc<Self>) -> Slot {
        self.core.upgrade().unwrap().new_real_var()
    }
}

pub struct StringType {
    core: Weak<dyn Core>,
}

impl StringType {
    /// Creates the built-in string type.
    pub fn new(core: Weak<dyn Core>) -> Self {
        Self { core }
    }
}

impl Type for StringType {
    fn name(&self) -> &str {
        "string"
    }

    fn as_any(self: Rc<Self>) -> Rc<dyn Any> {
        self
    }

    fn new_instance(self: Rc<Self>) -> Slot {
        self.core.upgrade().unwrap().new_string_var()
    }
}

pub trait Scope {
    fn core(&self) -> Rc<dyn Core>;
    fn scope(&self) -> Option<Rc<dyn Scope>>;

    fn get_type(&self, name: &str) -> Option<Rc<dyn Type>>;
    fn get_predicate(&self, name: &str) -> Option<Rc<Predicate>>;
}

pub struct CommonScope {
    core: Weak<dyn Core>,
    scope: Option<Weak<dyn Scope>>,
    pub(crate) types: RefCell<HashMap<String, Rc<dyn Type>>>,
    predicates: RefCell<HashMap<String, Rc<Predicate>>>,
}

impl CommonScope {
    /// Creates an empty scope with an optional parent scope.
    pub fn new(core: Weak<dyn Core>, scope: Option<Weak<dyn Scope>>) -> Self {
        Self { core, scope, types: RefCell::new(HashMap::new()), predicates: RefCell::new(HashMap::new()) }
    }
}

impl Scope for CommonScope {
    fn core(&self) -> Rc<dyn Core> {
        self.core.upgrade().expect("Core should never be dropped while scopes exist")
    }

    fn scope(&self) -> Option<Rc<dyn Scope>> {
        self.scope.as_ref()?.upgrade()
    }

    fn get_type(&self, name: &str) -> Option<Rc<dyn Type>> {
        self.types.borrow().get(name).cloned().or_else(|| self.scope()?.get_type(name))
    }

    fn get_predicate(&self, name: &str) -> Option<Rc<Predicate>> {
        self.predicates.borrow().get(name).cloned().or_else(|| self.scope()?.get_predicate(name))
    }
}

/// Executable constructor declaration.
pub struct Constructor {
    core: Weak<dyn Core>,
    scope: Rc<CommonScope>,
    args: Vec<(Vec<String>, String)>,
    init: Vec<(Vec<String>, Vec<Expr>)>,
    statements: Vec<Statement>,
}

/// Class-specific API surface layered on top of type and scope behavior.
pub trait Class: Type + Scope {
    fn parents(&self) -> &[Vec<String>];
    fn constructors(&self) -> &[Constructor];
    fn constructor(&self, args: &[Rc<dyn Type>]) -> Option<&Constructor>;
    fn predicates(&self) -> Vec<Rc<Predicate>>;
    fn classes(&self) -> Vec<Rc<dyn Class>>;
    fn instances(&self) -> Vec<ObjectId>;
}

/// Returns whether a value of source type can be assigned to target.
///
/// The check accepts exact type matches and direct parent/child relationships
/// between class types.
pub fn is_assignable_from(target: &Rc<dyn Type>, source: &Rc<dyn Type>) -> bool {
    if Rc::ptr_eq(target, source) {
        return true;
    }
    if let Some(target_class) = target.clone().as_class()
        && let Some(source_class) = source.clone().as_class()
    {
        for parent in source_class.parents() {
            if parent.iter().map(|s| s.as_str()).eq(target_class.full_name().split('.')) {
                return true;
            }
        }
        for parent in target_class.parents() {
            if parent.iter().map(|s| s.as_str()).eq(source_class.full_name().split('.')) {
                return true;
            }
        }
    }
    false
}

pub struct Predicate {
    core: Weak<dyn Core>,
    scope: CommonScope,
    name: String,
    parents: Vec<Vec<String>>,
    args: Vec<(Vec<String>, String)>,
}

impl Predicate {
    pub fn new(core: Weak<dyn Core>, scope: CommonScope, name: String) -> Self {
        Self { core, scope, name, parents: Vec::new(), args: Vec::new() }
    }

    pub fn parents(&self) -> &[Vec<String>] {
        &self.parents
    }

    pub fn args(&self) -> &[(Vec<String>, String)] {
        &self.args
    }
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
    fn core(&self) -> Rc<dyn Core> {
        self.core.upgrade().expect("Core should never be dropped while predicates exist")
    }

    fn scope(&self) -> Option<Rc<dyn Scope>> {
        self.scope.scope()
    }

    fn get_type(&self, name: &str) -> Option<Rc<dyn Type>> {
        self.scope.get_type(name)
    }

    fn get_predicate(&self, name: &str) -> Option<Rc<Predicate>> {
        self.scope.get_predicate(name)
    }
}

pub fn get_type_by_path(scope: &dyn Scope, path: &[String]) -> Result<Rc<dyn Type>, RiddleError> {
    let (first, rest) = path.split_first().ok_or_else(|| RiddleError::RuntimeError("Empty type path".into()))?;
    rest.iter().try_fold(scope.get_type(first).ok_or_else(|| RiddleError::NotFound(first.clone()))?, |current, part| current.as_class().ok_or_else(|| RiddleError::NotAClass(first.clone()))?.get_type(part).ok_or_else(|| RiddleError::NotFound(format!("Class '{}' in path", part))))
}
