use crate::{
    RiddleError,
    core::Core,
    env::{Atom, AtomId, BoolExpr, CommonEnv, Env, ObjectId, Slot, Var},
    language::{ClassDef, ConstructorDef, Expr, MethodDef, PredicateDef, ProblemDef, Statement, evaluate, execute},
};
use std::{
    any::Any,
    cell::RefCell,
    collections::HashMap,
    fmt,
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

pub struct Field {
    name: String,
    field_type: Vec<String>,
    default: Option<Expr>,
}

impl Field {
    /// Creates a new field descriptor.
    pub fn new(name: String, field_type: Vec<String>, default: Option<Expr>) -> Self {
        Self { name, field_type, default }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn field_type(&self) -> &[String] {
        &self.field_type
    }

    pub fn default(&self) -> Option<&Expr> {
        self.default.as_ref()
    }
}

pub trait Scope {
    fn core(&self) -> Rc<dyn Core>;
    fn scope(&self) -> Option<Rc<dyn Scope>>;
    fn as_class(self: Rc<Self>) -> Option<Rc<dyn Class>> {
        None
    }

    fn get_fields(&self) -> Vec<Rc<Field>>;
    fn get_field(&self, name: &str) -> Option<Rc<Field>>;
    fn get_method(&self, name: &str, types: &[Rc<dyn Type>]) -> Option<Rc<Method>>;
    fn get_type(&self, name: &str) -> Option<Rc<dyn Type>>;
    fn get_predicate(&self, name: &str) -> Option<Rc<Predicate>>;
}

pub struct CommonScope {
    core: Weak<dyn Core>,
    scope: Option<Weak<dyn Scope>>,
    fields: RefCell<HashMap<String, Rc<Field>>>,
    methods: RefCell<HashMap<String, Vec<Rc<Method>>>>,
    pub(crate) types: RefCell<HashMap<String, Rc<dyn Type>>>,
    predicates: RefCell<HashMap<String, Rc<Predicate>>>,
}

impl CommonScope {
    /// Creates an empty scope with an optional parent scope.
    pub fn new(core: Weak<dyn Core>, scope: Option<Weak<dyn Scope>>) -> Self {
        Self {
            core,
            scope,
            fields: RefCell::new(HashMap::new()),
            methods: RefCell::new(HashMap::new()),
            types: RefCell::new(HashMap::new()),
            predicates: RefCell::new(HashMap::new()),
        }
    }

    /// Builds a scope populated from a class definition.
    pub fn from_class(core: Weak<dyn Core>, scope: Option<Weak<dyn Scope>>, class_scope: Weak<dyn Scope>, class: ClassDef) -> Rc<Self> {
        let scope = Rc::new(Self::new(core.clone(), scope));
        for (field_type, fields) in class.fields {
            for (name, default) in fields {
                scope.fields.borrow_mut().insert(name.clone(), Rc::new(Field { name, field_type: field_type.clone(), default }));
            }
        }
        for method_def in class.methods {
            scope.methods.borrow_mut().entry(method_def.name.clone()).or_default().push(Method::new(core.clone(), Some(class_scope.clone()), method_def));
        }
        for class_def in class.classes {
            let class_name = class_def.name.clone();
            scope.types.borrow_mut().insert(class_name, CommonClass::new(core.clone(), Some(class_scope.clone()), class_def));
        }
        for predicate_def in class.predicates {
            scope.predicates.borrow_mut().insert(predicate_def.name.clone(), Predicate::new(core.clone(), Some(class_scope.clone()), predicate_def));
        }
        scope
    }

    /// Builds a local scope for constructor arguments.
    pub fn from_constructor(core: Weak<dyn Core>, scope: Weak<dyn Class>, constructor: ConstructorDef) -> Self {
        let scope = Self::new(core, Some(scope));
        for (arg_type, arg_name) in constructor.args {
            scope.fields.borrow_mut().insert(arg_name.clone(), Rc::new(Field { name: arg_name, field_type: arg_type, default: None }));
        }
        scope
    }

    /// Builds a local scope for method arguments.
    pub fn from_method(core: Weak<dyn Core>, scope: Option<Weak<dyn Scope>>, method: MethodDef) -> Self {
        let scope = Self::new(core, scope);
        for (arg_type, arg_name) in method.args {
            scope.fields.borrow_mut().insert(arg_name.clone(), Rc::new(Field { name: arg_name, field_type: arg_type, default: None }));
        }
        scope
    }

    /// Builds a local scope for predicate arguments.
    pub fn from_predicate(core: Weak<dyn Core>, scope: Option<Weak<dyn Scope>>, predicate: PredicateDef) -> Self {
        let scope = Self::new(core, scope);
        for (arg_type, arg_name) in predicate.args {
            scope.fields.borrow_mut().insert(arg_name.clone(), Rc::new(Field { name: arg_name, field_type: arg_type, default: None }));
        }
        scope
    }

    /// Merges problem-level declarations into this scope.
    pub fn add_problem(&self, problem: ProblemDef) {
        for method_def in problem.methods {
            self.methods.borrow_mut().entry(method_def.name.clone()).or_default().push(Method::new(self.core.clone(), Some(self.core.clone()), method_def));
        }
        for class_def in problem.classes {
            self.types.borrow_mut().insert(class_def.name.clone(), CommonClass::new(self.core.clone(), Some(self.core.clone()), class_def));
        }
        for predicate_def in problem.predicates {
            self.predicates.borrow_mut().insert(predicate_def.name.clone(), Predicate::new(self.core.clone(), Some(self.core.clone()), predicate_def));
        }
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

    fn get_fields(&self) -> Vec<Rc<Field>> {
        self.fields.borrow().values().cloned().collect()
    }

    fn get_field(&self, name: &str) -> Option<Rc<Field>> {
        self.fields.borrow().get(name).cloned().or_else(|| self.scope()?.get_field(name))
    }

    fn get_method(&self, name: &str, types: &[Rc<dyn Type>]) -> Option<Rc<Method>> {
        self.methods
            .borrow()
            .get(name)
            .and_then(|methods| {
                methods
                    .iter()
                    .find(|m| {
                        if m.args().len() != types.len() {
                            return false;
                        }
                        for (class, arg_type) in types.iter().zip(m.args().iter().map(|(t, _)| t)) {
                            if !get_type_by_path(self, arg_type).ok().is_some_and(|t| is_assignable_from(&t, class)) {
                                return false;
                            }
                        }
                        true
                    })
                    .cloned()
            })
            .or_else(|| self.scope.as_ref()?.upgrade()?.get_method(name, types))
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

impl Constructor {
    /// Creates a constructor from its parsed definition.
    pub fn new(core: Weak<dyn Core>, scope: Weak<dyn Class>, mut constructor: ConstructorDef) -> Self {
        Self {
            core: core.clone(),
            args: std::mem::take(&mut constructor.args),
            statements: std::mem::take(&mut constructor.statements),
            init: std::mem::take(&mut constructor.init),
            scope: Rc::new(CommonScope::from_constructor(core, scope, constructor)),
        }
    }

    pub fn args(&self) -> &[(Vec<String>, String)] {
        &self.args
    }

    pub fn statements(&self) -> &[Statement] {
        &self.statements
    }

    /// Creates a new object instance and runs constructor statements.
    pub fn call(&self, object: ObjectId, args: Vec<Slot>) -> Result<(), RiddleError> {
        if args.len() != self.args.len() {
            return Err(RiddleError::RuntimeError(format!("Expected {} arguments, got {}", self.args.len(), args.len())));
        }
        let obj_env = self.core().get_object(object).ok_or_else(|| RiddleError::NotFound(format!("Object with ID {} not found", object.0)))?.as_env().ok_or_else(|| RiddleError::RuntimeError("Object environment not found".into()))?;
        // the context in which the constructor is invoked..
        let constructor_env = Rc::new(CommonEnv::new(Some(obj_env.clone())));
        constructor_env.set("this".to_string(), Slot::ObjectRef(object));
        for ((arg_type, arg_name), arg_value) in self.args.iter().zip(args) {
            let expected_type = get_type_by_path(self.scope.as_ref(), arg_type)?;
            let arg_value_type = match &arg_value {
                Slot::Primitive(p) => p.var_type(),
                Slot::ObjectRef(obj_id) => self.scope.core().get_object(*obj_id).ok_or_else(|| RiddleError::NotFound(format!("Object with ID {}", obj_id.0)))?.var_type(),
                Slot::AtomRef(atom_id) => self.scope.core().get_atom(*atom_id).ok_or_else(|| RiddleError::NotFound(format!("Atom with ID {}", atom_id.0)))?.var_type(),
            };
            if !is_assignable_from(&expected_type, &arg_value_type) {
                return Err(RiddleError::TypeError(format!("Argument '{}' expected to be of type '{}', got '{}'", arg_name, expected_type.full_name(), arg_value_type.full_name())));
            }
            constructor_env.set(arg_name.clone(), arg_value);
        }

        let class = self.scope.scope.as_ref().and_then(|s| s.upgrade()).and_then(|s| s.as_class()).ok_or_else(|| RiddleError::RuntimeError("Constructor is not defined within a class".into()))?;
        // we first execute parent constructors in declaration order, passing specified arguments or defaults if provided..
        for parent in class.parents() {
            let parent_class = get_type_by_path(self.scope.as_ref(), parent)?.as_class().ok_or_else(|| RiddleError::NotAClass(parent.join(".")))?;
            if let Some((_, init_exprs)) = self.init.iter().find(|(init_field, _)| init_field.iter().map(|s| s.as_str()).eq(parent.iter().map(|s| s.as_str()))) {
                let exprs = init_exprs.iter().map(|e| evaluate(self.scope.as_ref(), constructor_env.clone(), e)).collect::<Result<Vec<_>, _>>()?;
                let types = exprs
                    .iter()
                    .map(|e| match e {
                        Slot::Primitive(p) => Ok(p.var_type()),
                        Slot::ObjectRef(obj_id) => Ok(self.scope.core().get_object(*obj_id).ok_or_else(|| RiddleError::NotFound(format!("Object with ID {}", obj_id.0)))?.var_type()),
                        Slot::AtomRef(atom_id) => Ok(self.scope.core().get_atom(*atom_id).ok_or_else(|| RiddleError::NotFound(format!("Atom with ID {}", atom_id.0)))?.var_type()),
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                let constructor = parent_class.constructor(&types).ok_or_else(|| RiddleError::NotFound(format!("Constructor for parent class '{}' with specified argument types", parent_class.full_name())))?;
                constructor.call(object, exprs)?;
            } else {
                let constructor = parent_class.constructor(&[]).ok_or_else(|| RiddleError::NotFound(format!("No-arg constructor for parent class '{}'", parent_class.full_name())))?;
                constructor.call(object, vec![])?;
            }
        }

        // we then populate fields declared in this class..
        for field in class.get_fields() {
            let fld_tp = get_type_by_path(self.scope.as_ref(), field.field_type())?;
            if obj_env.get(field.name()).is_none() {
                if let Some(default_expr) = field.default() {
                    let value = evaluate(self.scope.as_ref(), constructor_env.clone(), default_expr)?;
                    let value_type = match &value {
                        Slot::Primitive(p) => p.var_type(),
                        Slot::ObjectRef(obj_id) => self.scope.core().get_object(*obj_id).ok_or_else(|| RiddleError::NotFound(format!("Object with ID {}", obj_id.0)))?.var_type(),
                        Slot::AtomRef(atom_id) => self.scope.core().get_atom(*atom_id).ok_or_else(|| RiddleError::NotFound(format!("Atom with ID {}", atom_id.0)))?.var_type(),
                    };
                    if !is_assignable_from(&fld_tp, &value_type) {
                        return Err(RiddleError::TypeError(format!("Field '{}' expected to be of type '{}', got '{}'", field.name(), fld_tp.full_name(), value_type.full_name())));
                    }
                    obj_env.set(field.name().to_string(), value);
                } else if let Some(class) = fld_tp.clone().as_class() {
                    let instances = class.instances();
                    if instances.is_empty() {
                        return Err(RiddleError::RuntimeError(format!("No instances found for field '{}' of type '{}'", field.name(), class.full_name())));
                    } else if instances.len() == 1 {
                        obj_env.set(field.name().to_string(), Slot::ObjectRef(instances[0]));
                    } else {
                        obj_env.set(field.name().to_string(), self.scope.clone().core().new_var(class, &instances)?);
                    }
                } else {
                    obj_env.set(field.name().to_string(), fld_tp.clone().new_instance());
                }
            }
        }

        // finally, we execute constructor statements in the context of the new object..
        let scope: Rc<dyn Scope> = self.scope.clone();
        for stmt in &self.statements {
            execute(&scope, constructor_env.clone(), stmt)?;
        }
        Ok(())
    }
}

impl Scope for Constructor {
    fn core(&self) -> Rc<dyn Core> {
        self.core.upgrade().expect("Core should never be dropped while scopes exist")
    }

    fn scope(&self) -> Option<Rc<dyn Scope>> {
        self.scope.scope.as_ref()?.upgrade()
    }

    fn get_fields(&self) -> Vec<Rc<Field>> {
        self.scope.get_fields()
    }

    fn get_field(&self, name: &str) -> Option<Rc<Field>> {
        self.scope.get_field(name)
    }

    fn get_method(&self, name: &str, classes: &[Rc<dyn Type>]) -> Option<Rc<Method>> {
        self.scope.get_method(name, classes)
    }

    fn get_type(&self, name: &str) -> Option<Rc<dyn Type>> {
        self.scope.get_type(name)
    }

    fn get_predicate(&self, name: &str) -> Option<Rc<Predicate>> {
        self.scope.get_predicate(name)
    }
}

impl fmt::Display for Constructor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let class_name = self.scope.scope.as_ref().and_then(|s| s.upgrade()).and_then(|s| s.as_class()).map(|c| c.full_name()).unwrap_or_else(|| "<unknown class>".to_string());
        let args = self.args.iter().map(|(t, n)| format!("{} {}", t.join("."), n)).collect::<Vec<_>>().join(", ");
        write!(f, "{}({})", class_name, args)
    }
}

pub struct Method {
    core: Weak<dyn Core>,
    scope: Rc<CommonScope>,
    name: String,
    return_type: Option<Vec<String>>,
    args: Vec<(Vec<String>, String)>,
    statements: Vec<Statement>,
}

impl Method {
    /// Creates a method from its parsed definition.
    pub fn new(core: Weak<dyn Core>, scope: Option<Weak<dyn Scope>>, mut method: MethodDef) -> Rc<Self> {
        Rc::new(Self {
            core: core.clone(),
            name: std::mem::take(&mut method.name),
            return_type: std::mem::take(&mut method.return_type),
            args: std::mem::take(&mut method.args),
            statements: std::mem::take(&mut method.statements),
            scope: Rc::new(CommonScope::from_method(core, scope, method)),
        })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn return_type(&self) -> Option<&[String]> {
        self.return_type.as_deref()
    }

    pub fn args(&self) -> &[(Vec<String>, String)] {
        &self.args
    }

    pub fn statements(&self) -> &[Statement] {
        &self.statements
    }

    /// Invokes the method in a fresh local environment.
    ///
    /// The call validates argument count and type compatibility, executes all
    /// method statements, and checks the declared return type (if any).
    pub fn call(&self, env: Rc<dyn Env>, args: Vec<Slot>) -> Result<Option<Slot>, RiddleError> {
        if args.len() != self.args.len() {
            return Err(RiddleError::RuntimeError(format!("Expected {} arguments, got {}", self.args.len(), args.len())));
        }
        let method_env = Rc::new(CommonEnv::new(Some(env)));
        for ((arg_type, arg_name), arg_value) in self.args.iter().zip(args) {
            let expected_type = get_type_by_path(self.scope.as_ref(), arg_type)?;
            let arg_value_type = match &arg_value {
                Slot::Primitive(p) => p.var_type(),
                Slot::ObjectRef(obj_id) => self.scope.core().get_object(*obj_id).ok_or_else(|| RiddleError::NotFound(format!("Object with ID {}", obj_id.0)))?.var_type(),
                Slot::AtomRef(atom_id) => self.scope.core().get_atom(*atom_id).ok_or_else(|| RiddleError::NotFound(format!("Atom with ID {}", atom_id.0)))?.var_type(),
            };
            if !is_assignable_from(&expected_type, &arg_value_type) {
                return Err(RiddleError::TypeError(format!("Argument '{}' expected to be of type '{}', got '{}'", arg_name, expected_type.full_name(), arg_value_type.full_name())));
            }
            method_env.set(arg_name.clone(), arg_value);
        }
        let scope: Rc<dyn Scope> = self.scope.clone();
        for stmt in &self.statements {
            execute(&scope, method_env.clone(), stmt)?;
        }
        if let Some(return_type) = &self.return_type {
            method_env.get("__return").ok_or_else(|| RiddleError::RuntimeError("Method did not set return value".into())).and_then(|ret| {
                let expected_type = get_type_by_path(self.scope.as_ref(), return_type)?;
                let ret_type = match &ret {
                    Slot::Primitive(p) => p.var_type(),
                    Slot::ObjectRef(obj_id) => self.scope.core().get_object(*obj_id).ok_or_else(|| RiddleError::NotFound(format!("Object with ID {}", obj_id.0)))?.var_type(),
                    Slot::AtomRef(atom_id) => self.scope.core().get_atom(*atom_id).ok_or_else(|| RiddleError::NotFound(format!("Atom with ID {}", atom_id.0)))?.var_type(),
                };
                if !is_assignable_from(&expected_type, &ret_type) { Err(RiddleError::TypeError(format!("Return value expected to be of type '{}', got '{}'", expected_type.full_name(), ret_type.full_name()))) } else { Ok(Some(ret)) }
            })
        } else {
            Ok(None)
        }
    }
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

pub struct CommonClass {
    core: Weak<dyn Core>,
    scope: Rc<CommonScope>,
    name: String,
    parents: Vec<Vec<String>>,
    constructors: Vec<Constructor>,
    instances: RefCell<Vec<ObjectId>>,
}

impl CommonClass {
    /// Creates a class type from its parsed definition, including nested members.
    pub fn new(core: Weak<dyn Core>, scope: Option<Weak<dyn Scope>>, mut class: ClassDef) -> Rc<Self> {
        let name = std::mem::take(&mut class.name);
        let parents = std::mem::take(&mut class.parents);
        let constructors_def = if class.constructors.is_empty() { vec![ConstructorDef { args: Vec::new(), init: Vec::new(), statements: Vec::new() }] } else { std::mem::take(&mut class.constructors) };
        Rc::new_cyclic(move |weak_self: &Weak<CommonClass>| Self {
            core: core.clone(),
            name,
            parents,
            constructors: constructors_def.into_iter().map(|c| Constructor::new(core.clone(), weak_self.clone(), c)).collect(),
            scope: CommonScope::from_class(core.clone(), scope, weak_self.clone(), class),
            instances: RefCell::new(Vec::new()),
        })
    }
}

impl Type for CommonClass {
    fn name(&self) -> &str {
        &self.name
    }

    fn full_name(&self) -> String {
        if let Some(scope) = self.scope.scope.as_ref().and_then(|scope| scope.upgrade())
            && let Some(class) = scope.as_class()
        {
            format!("{}.{}", class.full_name(), self.name)
        } else {
            self.name.clone()
        }
    }

    fn as_any(self: Rc<Self>) -> Rc<dyn Any> {
        self
    }

    fn as_class(self: Rc<Self>) -> Option<Rc<dyn Class>> {
        Some(self)
    }

    fn new_instance(self: Rc<Self>) -> Slot {
        let instance = self.core().new_object(self.clone());
        self.instances.borrow_mut().push(instance);
        for parent in &self.parents {
            let parent_class = get_type_by_path(self.as_ref(), parent).expect("Parent class should exist").as_class().expect("Parent class should be a class");
            parent_class.as_any().downcast_ref::<CommonClass>().expect("Parent class should be a CommonClass").instances.borrow_mut().push(instance);
        }
        Slot::ObjectRef(instance)
    }
}

impl Scope for CommonClass {
    fn core(&self) -> Rc<dyn Core> {
        self.core.upgrade().expect("Core should never be dropped while scopes exist")
    }

    fn scope(&self) -> Option<Rc<dyn Scope>> {
        self.scope.scope.as_ref()?.upgrade()
    }

    fn as_class(self: Rc<Self>) -> Option<Rc<dyn Class>> {
        Some(self)
    }

    fn get_fields(&self) -> Vec<Rc<Field>> {
        self.scope.get_fields()
    }

    fn get_field(&self, name: &str) -> Option<Rc<Field>> {
        self.scope.get_field(name)
    }

    fn get_method(&self, name: &str, classes: &[Rc<dyn Type>]) -> Option<Rc<Method>> {
        self.scope.get_method(name, classes)
    }

    fn get_type(&self, name: &str) -> Option<Rc<dyn Type>> {
        self.scope.get_type(name)
    }

    fn get_predicate(&self, name: &str) -> Option<Rc<Predicate>> {
        self.scope.get_predicate(name)
    }
}

impl Class for CommonClass {
    fn parents(&self) -> &[Vec<String>] {
        &self.parents
    }

    fn constructors(&self) -> &[Constructor] {
        &self.constructors
    }

    fn constructor(&self, args: &[Rc<dyn Type>]) -> Option<&Constructor> {
        self.constructors.iter().find(|c| {
            if c.args().len() != args.len() {
                return false;
            }
            for ((arg_type, _), class) in c.args().iter().zip(args.iter()) {
                if !class.full_name().split('.').eq(arg_type.iter().map(|s| s.as_str())) {
                    return false;
                }
            }
            true
        })
    }

    fn predicates(&self) -> Vec<Rc<Predicate>> {
        self.scope.predicates.borrow().values().cloned().collect()
    }

    fn classes(&self) -> Vec<Rc<dyn Class>> {
        self.scope.types.borrow().values().filter_map(|t| t.clone().as_class()).collect()
    }

    fn instances(&self) -> Vec<ObjectId> {
        let mut instances = self.instances.borrow().clone();
        for parent in &self.parents {
            if let Some(parent_class) = self.core.upgrade().unwrap().get_type(&parent.join("."))
                && let Some(parent_class) = parent_class.as_class()
            {
                instances.extend(parent_class.instances());
            }
        }
        instances
    }
}

/// Returns the resulting numeric type for arithmetic terms.
///
/// If all terms are int the result is int, otherwise mixed int/real terms yield
/// real. Any other type combination results in a type error.
pub fn arith_class(cr: &dyn Core, terms: &[Slot]) -> Result<Rc<dyn Type>, RiddleError> {
    let types = terms
        .iter()
        .map(|t| match t {
            Slot::Primitive(p) => Ok(p.var_type()),
            Slot::ObjectRef(obj_id) => Err(RiddleError::TypeError(format!("Expected numeric type, got object reference to object with ID {}", obj_id.0))),
            Slot::AtomRef(atom_id) => Err(RiddleError::TypeError(format!("Expected numeric type, got atom reference to atom with ID {}", atom_id.0))),
        })
        .collect::<Result<Vec<_>, _>>()?;
    if types.iter().all(|t| t.name() == "int") {
        Ok(cr.get_type("int").expect("int class not found"))
    } else if types.iter().all(|t| t.name() == "int" || t.name() == "real") {
        Ok(cr.get_type("real").expect("real class not found"))
    } else {
        Err(RiddleError::TypeError("Invalid types for arithmetic operation".into()))
    }
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
    statements: Vec<Statement>,
    atoms: RefCell<Vec<AtomId>>,
}

impl Predicate {
    /// Creates a predicate from its parsed definition.
    pub fn new(core: Weak<dyn Core>, scope: Option<Weak<dyn Scope>>, mut predicate: PredicateDef) -> Rc<Self> {
        Rc::new(Self {
            core: core.clone(),
            name: std::mem::take(&mut predicate.name),
            parents: std::mem::take(&mut predicate.parents),
            args: std::mem::take(&mut predicate.args),
            statements: std::mem::take(&mut predicate.statements),
            scope: CommonScope::from_predicate(core, scope, predicate),
            atoms: RefCell::new(Vec::new()),
        })
    }

    pub fn parents(&self) -> &[Vec<String>] {
        &self.parents
    }

    pub fn args(&self) -> &[(Vec<String>, String)] {
        &self.args
    }

    pub fn statements(&self) -> &[Statement] {
        &self.statements
    }

    /// Executes predicate statements against a concrete atom.
    pub fn call(self: Rc<Self>, atom: Rc<Atom>) -> Result<(), RiddleError> {
        let scope: Rc<dyn Scope> = self.clone();
        for stmt in &self.statements {
            execute(&scope, atom.clone(), stmt)?;
        }
        Ok(())
    }

    pub fn atoms(&self) -> Vec<AtomId> {
        self.atoms.borrow().clone()
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

    fn get_fields(&self) -> Vec<Rc<Field>> {
        self.scope.get_fields()
    }

    fn get_field(&self, name: &str) -> Option<Rc<Field>> {
        self.scope.get_field(name)
    }

    fn get_method(&self, name: &str, classes: &[Rc<dyn Type>]) -> Option<Rc<Method>> {
        self.scope.get_method(name, classes)
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
