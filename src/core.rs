use crate::{
    RiddleError,
    env::{Atom, AtomId, BoolExpr, CommonEnv, Env, Object, ObjectId, Slot},
    language::{Disjunction, execute},
    parse_problem,
    scope::{BoolType, Class, CommonScope, Field, Function, IntType, Predicate, RealType, Scope, StringType, Type},
};
use std::{
    cell::RefCell,
    collections::HashMap,
    rc::{Rc, Weak},
};

pub trait Core: Scope + Env {
    fn new_bool(&self, value: bool) -> Slot;
    fn new_bool_var(&self) -> Slot;
    fn new_int(&self, value: i64) -> Slot;
    fn new_int_var(&self) -> Slot;
    fn new_real(&self, num: i64, den: i64) -> Slot;
    fn new_real_var(&self) -> Slot;
    fn new_string(&self, value: &str) -> Slot;
    fn new_string_var(&self) -> Slot;

    fn sum(&self, sum: &[Slot]) -> Result<Slot, RiddleError>;
    fn opposite(&self, term: Slot) -> Result<Slot, RiddleError>;
    fn mul(&self, mul: &[Slot]) -> Result<Slot, RiddleError>;
    fn div(&self, left: Slot, right: Slot) -> Result<Slot, RiddleError>;

    fn assert(&self, term: Rc<BoolExpr>) -> bool;
    fn new_var(&self, tp: Rc<dyn Class>, instances: &[ObjectId]) -> Result<Slot, RiddleError>;
    fn new_disjunction(&self, disjunction: Disjunction);

    fn new_object(&self, class: Rc<dyn Class>) -> ObjectId;
    fn get_object(&self, id: ObjectId) -> Option<Rc<Object>>;
    fn new_atom(&self, predicate: Rc<Predicate>, fact: bool, args: HashMap<String, Slot>) -> AtomId;
    fn get_atom(&self, id: AtomId) -> Option<Rc<Atom>>;

    fn bool_type(&self) -> Rc<BoolType> {
        self.get_type("bool").expect("Core should have bool type").as_any().downcast::<BoolType>().expect("Core bool type should be BoolType")
    }

    fn int_type(&self) -> Rc<IntType> {
        self.get_type("int").expect("Core should have int type").as_any().downcast::<IntType>().expect("Core int type should be IntType")
    }

    fn real_type(&self) -> Rc<RealType> {
        self.get_type("real").expect("Core should have real type").as_any().downcast::<RealType>().expect("Core real type should be RealType")
    }

    fn string_type(&self) -> Rc<StringType> {
        self.get_type("string").expect("Core should have string type").as_any().downcast::<StringType>().expect("Core string type should be StringType")
    }
}

pub struct CommonCore {
    scope: Rc<CommonScope>,
    env: Rc<CommonEnv>,
    objects: RefCell<Vec<Rc<Object>>>,
    atoms: RefCell<Vec<Rc<Atom>>>,
}

impl CommonCore {
    pub fn new(core: Weak<dyn Core>) -> Rc<Self> {
        let c_core = Rc::new(CommonCore {
            scope: Rc::new(CommonScope::new(core.clone(), None)),
            env: Rc::new(CommonEnv::new(None)),
            objects: RefCell::new(Vec::new()),
            atoms: RefCell::new(Vec::new()),
        });
        c_core.add_type(Rc::new(BoolType::new(core.clone())));
        c_core.add_type(Rc::new(IntType::new(core.clone())));
        c_core.add_type(Rc::new(RealType::new(core.clone())));
        c_core.add_type(Rc::new(StringType::new(core.clone())));
        c_core
    }

    /// Parses and executes a RiDDLe problem in this core context.
    ///
    /// The parsed problem metadata is registered in the current scope,
    /// then each statement is executed in order using this core scope
    /// and environment.
    ///
    /// Returns an error if parsing or execution fails.
    pub fn read(&self, riddle: &str) -> Result<(), RiddleError> {
        let mut problem = parse_problem(riddle)?;
        let statments = std::mem::take(&mut problem.statements);
        self.scope.clone().add_problem(problem);
        let scope: Rc<dyn Scope> = self.scope.clone();
        for stmt in statments {
            execute(&scope, self.env.clone(), &stmt)?;
        }
        Ok(())
    }

    /// Registers a type in the core type table under its declared name.
    pub fn add_type(&self, tp: Rc<dyn Type>) {
        self.scope.types.borrow_mut().insert(tp.name().to_string(), tp);
    }

    pub fn get_objects(&self) -> Vec<Rc<Object>> {
        self.objects.borrow().clone()
    }

    pub fn get_object(&self, id: ObjectId) -> Option<Rc<Object>> {
        self.objects.borrow().get(*id).cloned()
    }

    pub fn new_object(&self, class: Rc<dyn Class>) -> ObjectId {
        let id = ObjectId(self.objects.borrow().len());
        self.objects.borrow_mut().push(Rc::new(Object::new(id, class)));
        id
    }

    pub fn get_atoms(&self) -> Vec<Rc<Atom>> {
        self.atoms.borrow().clone()
    }

    pub fn get_atom(&self, id: AtomId) -> Option<Rc<Atom>> {
        self.atoms.borrow().get(*id).cloned()
    }

    pub fn new_atom(&self, predicate: Rc<Predicate>, fact: bool, args: HashMap<String, Slot>) -> AtomId {
        let id = AtomId(self.atoms.borrow().len());
        self.atoms.borrow_mut().push(Rc::new(Atom::new(id, predicate, fact, args)));
        id
    }
}

impl Scope for CommonCore {
    fn core(&self) -> Rc<dyn Core> {
        self.scope.clone().core()
    }

    fn scope(&self) -> Option<Rc<dyn Scope>> {
        None
    }

    fn get_fields(&self) -> Vec<Rc<Field>> {
        self.scope.get_fields()
    }

    fn get_field(&self, name: &str) -> Option<Rc<Field>> {
        self.scope.get_field(name)
    }

    fn get_function(&self, name: &str, types: &[Rc<dyn Type>]) -> Option<Rc<Function>> {
        self.scope.get_function(name, types)
    }

    fn get_type(&self, name: &str) -> Option<Rc<dyn Type>> {
        self.scope.get_type(name)
    }

    fn get_predicate(&self, name: &str) -> Option<Rc<Predicate>> {
        self.scope.get_predicate(name)
    }
}

impl Env for CommonCore {
    fn parent(&self) -> Option<Rc<dyn Env>> {
        None
    }

    fn get(&self, name: &str) -> Option<Slot> {
        self.env.get(name)
    }

    fn set(&self, name: String, value: Slot) {
        self.env.set(name, value);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{env::Var, scope::arith_type};
    use std::any::Any;

    struct TestObject {
        tp: Weak<dyn Type>,
    }

    impl TestObject {
        fn new(var_type: Rc<dyn Type>) -> Self {
            Self { tp: Rc::downgrade(&var_type) }
        }
    }

    impl Var for TestObject {
        fn var_type(&self) -> Rc<dyn Type> {
            self.tp.upgrade().expect("Type should still exist")
        }

        fn as_any(self: Rc<Self>) -> Rc<dyn Any> {
            self
        }
    }

    struct TestCore {
        core: Rc<CommonCore>,
    }

    impl TestCore {
        fn new() -> Rc<Self> {
            Rc::new_cyclic(|core| Self {
                core: {
                    let core: Weak<TestCore> = core.clone();
                    CommonCore::new(core)
                },
            })
        }

        fn read(&self, riddle: &str) -> Result<(), RiddleError> {
            self.core.read(riddle)
        }
    }

    impl Core for TestCore {
        fn new_bool(&self, _value: bool) -> Slot {
            Slot::Primitive(Rc::new(TestObject::new(self.bool_type())))
        }
        fn new_bool_var(&self) -> Slot {
            Slot::Primitive(Rc::new(TestObject::new(self.bool_type())))
        }
        fn new_int(&self, _value: i64) -> Slot {
            Slot::Primitive(Rc::new(TestObject::new(self.int_type())))
        }
        fn new_int_var(&self) -> Slot {
            Slot::Primitive(Rc::new(TestObject::new(self.int_type())))
        }
        fn new_real(&self, _num: i64, _den: i64) -> Slot {
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
            Ok(Slot::Primitive(Rc::new(TestObject::new(class))))
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
            panic!("Core should not call scope core function")
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

    #[test]
    fn create_core() {
        let core = TestCore::new();
        assert!(core.get_type("bool").is_some());
        assert!(core.get_type("int").is_some());
        assert!(core.get_type("real").is_some());
        assert!(core.get_type("string").is_some());
    }

    #[test]
    fn read_problem() {
        let core = TestCore::new();
        core.read("bool a, b, c; (a & b) | c;").expect("Failed to read problem with boolean variables and expression");
    }
}
