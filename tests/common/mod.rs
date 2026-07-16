#![allow(dead_code)]

use core::fmt;
use riddle::{
    RiddleError,
    core::{CommonCore, Core},
    env::{Atom, AtomId, BoolExpr, Env, Object, ObjectId, Slot, Var},
    language::Disjunction,
    scope::{Class, Field, Function, Predicate, Scope, Type, arith_type, get_type_by_path},
};
use std::{
    any::Any,
    cell::RefCell,
    collections::HashMap,
    rc::{Rc, Weak},
};

struct TestObject {
    var_type: Weak<dyn Type>,
}

impl TestObject {
    fn new(var_type: Rc<dyn Type>) -> Self {
        Self { var_type: Rc::downgrade(&var_type) }
    }
}

impl Var for TestObject {
    fn var_type(&self) -> Rc<dyn Type> {
        self.var_type.upgrade().expect("Type has been dropped").clone()
    }

    fn as_any(self: Rc<Self>) -> Rc<dyn Any> {
        self
    }
}

struct TestEnum {
    var_type: Weak<dyn Type>,
    variables: RefCell<HashMap<String, Slot>>,
}

impl TestEnum {
    fn new(var_type: Rc<dyn Type>) -> Self {
        Self { var_type: Rc::downgrade(&var_type), variables: RefCell::new(HashMap::new()) }
    }
}

impl Var for TestEnum {
    fn var_type(&self) -> Rc<dyn Type> {
        self.var_type.upgrade().expect("Type has been dropped").clone()
    }

    fn as_any(self: Rc<Self>) -> Rc<dyn Any> {
        self
    }

    fn as_env(self: Rc<Self>) -> Option<Rc<dyn Env>> {
        Some(self.clone())
    }
}

impl Env for TestEnum {
    fn parent(&self) -> Option<Rc<dyn Env>> {
        None
    }

    fn get(&self, name: &str) -> Option<Slot> {
        if let Some(var) = self.variables.borrow().get(name) {
            return Some(var.clone());
        } else {
            if let Some(class) = self.var_type.upgrade().expect("Type has been dropped").as_class() {
                if let Some(field) = class.get_field(name) {
                    let field_type = get_type_by_path(class.as_ref(), field.field_type()).expect("Field type should exist");
                    return Some(field_type.new_instance());
                }
            }
        }
        None
    }

    fn set(&self, name: String, value: Slot) {
        self.variables.borrow_mut().insert(name, value);
    }
}

pub(crate) struct TestCore {
    core: Rc<CommonCore>,
}

impl TestCore {
    pub(crate) fn new() -> Rc<Self> {
        let slv = Rc::new_cyclic(|core| TestCore {
            core: {
                let core: Weak<TestCore> = core.clone();
                CommonCore::new(core)
            },
        });
        slv.core.read("real origin, horizon; origin >= 0.0; origin <= horizon;").expect("Failed to read core definitions");
        slv.core.read("predicate Impulse(real at) { at >= origin; at <= horizon; }").expect("Failed to read core definitions");
        slv.core.read("predicate Interval(real start, real duration, real end) { start >= origin; end <= horizon; duration == end - start; }").expect("Failed to read core definitions");
        slv.core.read("class StateVariable { }").expect("Failed to read core definitions");
        slv.core.read("class ReusableResource { real capacity; ReusableResource(real capacity) : capacity(capacity) { } predicate Use(real amount) : Interval { amount >= 0.0; amount <= capacity; } }").expect("Failed to read core definitions");
        slv
    }

    pub(crate) fn read(&self, script: &str) -> Result<(), RiddleError> {
        self.core.read(script)
    }
}

impl Core for TestCore {
    fn new_bool(&self, _value: bool) -> Slot {
        Slot::Primitive(Rc::new(TestObject::new(self.bool_type())))
    }
    fn new_bool_var(&self) -> Slot {
        Slot::Primitive(Rc::new(TestObject::new(self.bool_type())))
    }
    fn new_int(&self, _value: &str) -> Slot {
        Slot::Primitive(Rc::new(TestObject::new(self.int_type())))
    }
    fn new_int_var(&self) -> Slot {
        Slot::Primitive(Rc::new(TestObject::new(self.int_type())))
    }
    fn new_real(&self, _value: &str) -> Slot {
        Slot::Primitive(Rc::new(TestObject::new(self.real_type())))
    }
    fn new_real_var(&self) -> Slot {
        Slot::Primitive(Rc::new(TestObject::new(self.real_type())))
    }
    fn new_string(&self, _value: &str) -> Slot {
        Slot::Primitive(Rc::new(TestObject::new(self.string_type())))
    }
    fn new_string_var(&self) -> Slot {
        Slot::Primitive(Rc::new(TestObject::new(self.string_type())))
    }

    fn sum(&self, sum: &[Slot]) -> Result<Slot, RiddleError> {
        let tp = arith_type(self, sum)?;
        Ok(Slot::Primitive(Rc::new(TestObject::new(tp))))
    }
    fn opposite(&self, term: Slot) -> Result<Slot, RiddleError> {
        let tp = match term {
            Slot::Primitive(var) => var.var_type(),
            Slot::ObjectRef(id) => self.get_object(id).expect("Object should exist").class(),
            Slot::AtomRef(id) => self.get_atom(id).expect("Atom should exist").predicate(),
        };
        Ok(Slot::Primitive(Rc::new(TestObject::new(tp))))
    }
    fn mul(&self, mul: &[Slot]) -> Result<Slot, RiddleError> {
        let tp = arith_type(self, mul)?;
        Ok(Slot::Primitive(Rc::new(TestObject::new(tp))))
    }
    fn div(&self, left: Slot, right: Slot) -> Result<Slot, RiddleError> {
        let tp = arith_type(self, &[left, right])?;
        Ok(Slot::Primitive(Rc::new(TestObject::new(tp))))
    }

    fn assert(&self, _term: Rc<BoolExpr>) -> bool {
        true
    }

    fn new_var(&self, class: Rc<dyn Class>, instances: &[ObjectId]) -> Result<Slot, RiddleError> {
        if instances.is_empty() {
            return Err(RiddleError::InconsistencyError("Cannot create variable with no instances".into()));
        }
        Ok(Slot::Primitive(Rc::new(TestEnum::new(class))))
    }
    fn new_disjunction(&self, _disjunction: Disjunction) {}

    fn new_object(&self, class: Rc<dyn Class>) -> ObjectId {
        self.core.new_object(class)
    }
    fn get_object(&self, id: ObjectId) -> Option<Rc<Object>> {
        self.core.get_object(id)
    }
    fn new_atom(&self, predicate: Rc<Predicate>, fact: bool, args: HashMap<String, Slot>) -> AtomId {
        self.core.new_atom(predicate, fact, args)
    }
    fn get_atom(&self, id: AtomId) -> Option<Rc<Atom>> {
        self.core.get_atom(id)
    }
}

impl Scope for TestCore {
    fn core(&self) -> Rc<dyn Core> {
        panic!("Core should not call scope core method")
    }

    fn scope(&self) -> Option<Rc<dyn Scope>> {
        None
    }

    fn get_fields(&self) -> Vec<Rc<Field>> {
        self.core.get_fields()
    }

    fn get_field(&self, _name: &str) -> Option<Rc<Field>> {
        self.core.get_field(_name)
    }

    fn get_function(&self, name: &str, types: &[Rc<dyn Type>]) -> Option<Rc<Function>> {
        self.core.get_function(name, types)
    }

    fn get_type(&self, name: &str) -> Option<Rc<dyn Type>> {
        self.core.get_type(name)
    }

    fn get_predicate(&self, name: &str) -> Option<Rc<Predicate>> {
        self.core.get_predicate(name)
    }
}

impl Env for TestCore {
    fn parent(&self) -> Option<Rc<dyn Env>> {
        None
    }

    fn get(&self, name: &str) -> Option<Slot> {
        self.core.get(name)
    }

    fn set(&self, name: String, value: Slot) {
        self.core.set(name, value);
    }
}

impl fmt::Debug for TestCore {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "TestCore {{ ... }}")
    }
}
