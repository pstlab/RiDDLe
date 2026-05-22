use crate::{
    ToJson,
    env::{Atom, BoolExpr, CommonEnv, Env, Var},
    language::{Disjunction, RiddleError, execute},
    parse_problem,
    scope::{BoolType, CommonScope, Field, IntType, Method, Predicate, RealType, Scope, StringType, Type},
};
use serde_json::{Value, json};
use std::{
    collections::{HashMap, VecDeque},
    rc::{Rc, Weak},
};

pub trait Core: Scope + Env {
    fn new_bool(&self, value: bool) -> Rc<dyn Var>;
    fn new_bool_var(&self) -> Rc<dyn Var>;
    fn new_int(&self, value: i64) -> Rc<dyn Var>;
    fn new_int_var(&self) -> Rc<dyn Var>;
    fn new_real(&self, num: i64, den: i64) -> Rc<dyn Var>;
    fn new_real_var(&self) -> Rc<dyn Var>;
    fn new_string(&self, value: &str) -> Rc<dyn Var>;
    fn new_string_var(&self) -> Rc<dyn Var>;

    fn sum(&self, sum: &[Rc<dyn Var>]) -> Result<Rc<dyn Var>, RiddleError>;
    fn opposite(&self, term: Rc<dyn Var>) -> Result<Rc<dyn Var>, RiddleError>;
    fn mul(&self, mul: &[Rc<dyn Var>]) -> Result<Rc<dyn Var>, RiddleError>;
    fn div(&self, left: Rc<dyn Var>, right: Rc<dyn Var>) -> Result<Rc<dyn Var>, RiddleError>;

    fn assert(&self, term: Rc<BoolExpr>) -> bool;
    fn new_var(&self, class: Rc<dyn Type>, instances: &[Rc<dyn Var>]) -> Result<Rc<dyn Var>, RiddleError>;
    fn new_disjunction(&self, disjunction: Disjunction);
    fn new_atom(&self, atom: Rc<Atom>);

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

#[derive(Debug)]
pub struct CommonCore {
    scope: Rc<CommonScope>,
    env: Rc<CommonEnv>,
}

impl CommonCore {
    /// Creates a new shared core with a root scope and environment.
    ///
    /// The core is initialized with the builtin primitive types:
    /// `bool`, `int`, `real`, and `string`.
    pub fn new(core: Weak<dyn Core>) -> Rc<Self> {
        let c_core = Rc::new(CommonCore { scope: Rc::new(CommonScope::new(core.clone(), None)), env: Rc::new(CommonEnv::new(None)) });
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
        self.scope.add_problem(problem);
        for stmt in statments {
            execute(self.scope.clone(), self.env.clone(), &stmt)?;
        }
        Ok(())
    }

    /// Registers a type in the core type table under its declared name.
    pub fn add_type(&self, class: Rc<dyn Type>) {
        self.scope.classes.borrow_mut().insert(class.name().to_string(), class);
    }
}

impl ToJson for CommonCore {
    fn to_json(&self) -> Value {
        let mut terms = HashMap::new();
        let mut atoms = HashMap::new();
        let mut q = VecDeque::new();
        for tp in self.scope.classes.borrow().values() {
            if let Some(class) = tp.clone().as_class() {
                q.push_back(class.clone());
            }
        }
        while let Some(tp) = q.pop_front() {
            for instance in tp.instances() {
                let id = Rc::as_ptr(&instance) as *const () as usize;
                terms.insert(id, instance.var_type().name().to_string());
            }
            for pred in tp.predicates() {
                for atom in pred.atoms() {
                    let id = Rc::as_ptr(&atom) as *const () as usize;
                    atoms.insert(id, pred.name().to_string());
                }
            }
            for class in tp.classes() {
                q.push_back(class);
            }
        }
        json!({ "terms": terms, "atoms": atoms })
    }
}

impl Scope for CommonCore {
    fn core(self: Rc<Self>) -> Rc<dyn Core> {
        self.scope.clone().core()
    }

    fn scope(&self) -> Option<Rc<dyn Scope>> {
        None
    }

    fn get_fields(&self) -> Vec<Rc<Field>> {
        self.scope.get_fields()
    }

    fn get_field(&self, _name: &str) -> Option<Rc<Field>> {
        self.scope.get_field(_name)
    }

    fn get_method(&self, _name: &str, _classes: &[Rc<dyn Type>]) -> Option<Rc<Method>> {
        self.scope.get_method(_name, _classes)
    }

    fn get_type(&self, name: &str) -> Option<Rc<dyn Type>> {
        self.scope.get_type(name)
    }

    fn get_predicate(&self, _name: &str) -> Option<Rc<Predicate>> {
        self.scope.get_predicate(_name)
    }
}

impl Env for CommonCore {
    fn parent(&self) -> Option<Rc<dyn Env>> {
        None
    }

    fn get(&self, name: &str) -> Option<Rc<dyn Var>> {
        self.env.get(name)
    }

    fn set(&self, name: String, value: Rc<dyn Var>) {
        self.env.set(name, value);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{any::Any, cell::RefCell};

    #[derive(Debug)]
    struct TestObject {
        class: Weak<dyn Type>,
        _id: usize,
    }

    impl Var for TestObject {
        fn var_type(&self) -> Rc<dyn Type> {
            self.class.upgrade().expect("Class should still exist")
        }

        fn as_any(self: Rc<Self>) -> Rc<dyn Any> {
            self
        }
    }

    #[derive(Debug)]
    struct TestCore {
        core: Rc<CommonCore>,
        id_counter: RefCell<usize>,
    }

    impl TestCore {
        fn new() -> Rc<Self> {
            Rc::new_cyclic(|core| Self {
                core: {
                    let core: Weak<TestCore> = core.clone();
                    CommonCore::new(core)
                },
                id_counter: RefCell::new(0),
            })
        }

        fn read(&self, riddle: &str) -> Result<(), RiddleError> {
            self.core.read(riddle)
        }

        fn next_id(&self) -> usize {
            let mut counter = self.id_counter.borrow_mut();
            let id = *counter;
            *counter += 1;
            id
        }
    }

    impl Core for TestCore {
        fn new_bool(&self, _value: bool) -> Rc<dyn Var> {
            Rc::new(TestObject { class: Rc::downgrade(&(self.bool_type() as Rc<dyn Type>)), _id: self.next_id() })
        }

        fn new_bool_var(&self) -> Rc<dyn Var> {
            Rc::new(TestObject { class: Rc::downgrade(&(self.bool_type() as Rc<dyn Type>)), _id: self.next_id() })
        }

        fn new_int(&self, _value: i64) -> Rc<dyn Var> {
            Rc::new(TestObject { class: Rc::downgrade(&(self.int_type() as Rc<dyn Type>)), _id: self.next_id() })
        }

        fn new_int_var(&self) -> Rc<dyn Var> {
            Rc::new(TestObject { class: Rc::downgrade(&(self.int_type() as Rc<dyn Type>)), _id: self.next_id() })
        }

        fn new_real(&self, _num: i64, _den: i64) -> Rc<dyn Var> {
            Rc::new(TestObject { class: Rc::downgrade(&(self.real_type() as Rc<dyn Type>)), _id: self.next_id() })
        }

        fn new_real_var(&self) -> Rc<dyn Var> {
            Rc::new(TestObject { class: Rc::downgrade(&(self.real_type() as Rc<dyn Type>)), _id: self.next_id() })
        }

        fn new_string(&self, _value: &str) -> Rc<dyn Var> {
            Rc::new(TestObject { class: Rc::downgrade(&(self.string_type() as Rc<dyn Type>)), _id: self.next_id() })
        }

        fn new_string_var(&self) -> Rc<dyn Var> {
            Rc::new(TestObject { class: Rc::downgrade(&(self.string_type() as Rc<dyn Type>)), _id: self.next_id() })
        }

        fn sum(&self, _sum: &[Rc<dyn Var>]) -> Result<Rc<dyn Var>, RiddleError> {
            Ok(Rc::new(TestObject { class: Rc::downgrade(&(self.int_type() as Rc<dyn Type>)), _id: self.next_id() }))
        }

        fn opposite(&self, _term: Rc<dyn Var>) -> Result<Rc<dyn Var>, RiddleError> {
            Ok(Rc::new(TestObject { class: Rc::downgrade(&(self.int_type() as Rc<dyn Type>)), _id: self.next_id() }))
        }

        fn mul(&self, _mul: &[Rc<dyn Var>]) -> Result<Rc<dyn Var>, RiddleError> {
            Ok(Rc::new(TestObject { class: Rc::downgrade(&(self.int_type() as Rc<dyn Type>)), _id: self.next_id() }))
        }

        fn div(&self, _left: Rc<dyn Var>, _right: Rc<dyn Var>) -> Result<Rc<dyn Var>, RiddleError> {
            Ok(Rc::new(TestObject { class: Rc::downgrade(&(self.int_type() as Rc<dyn Type>)), _id: self.next_id() }))
        }

        fn assert(&self, _term: Rc<BoolExpr>) -> bool {
            true
        }

        fn new_var(&self, class: Rc<dyn Type>, instances: &[Rc<dyn Var>]) -> Result<Rc<dyn Var>, RiddleError> {
            if instances.is_empty() {
                return Err(RiddleError::InconsistencyError("Cannot create variable with no instances".into()));
            }
            Ok(Rc::new(TestObject { class: Rc::downgrade(&class), _id: self.next_id() }))
        }

        fn new_disjunction(&self, _disjunction: Disjunction) {}

        fn new_atom(&self, _atom: Rc<Atom>) {}
    }

    impl Scope for TestCore {
        fn core(self: Rc<Self>) -> Rc<dyn Core> {
            self
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

        fn get_method(&self, name: &str, classes: &[Rc<dyn Type>]) -> Option<Rc<Method>> {
            self.core.get_method(name, classes)
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

        fn get(&self, name: &str) -> Option<Rc<dyn Var>> {
            self.core.get(name)
        }

        fn set(&self, name: String, value: Rc<dyn Var>) {
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

    #[test]
    fn nested_classes_are_registered_in_class_scope() {
        let core = TestCore::new();
        core.read(
            r#"
            class Outer {
                class Inner {}
            }
            "#,
        )
        .expect("Failed to read problem with nested classes");

        let outer = core.get_type("Outer").expect("Outer class should be registered").as_class().expect("Outer should be a class");
        let inner = outer.get_type("Inner").expect("Inner class should be registered in the enclosing class scope");

        assert_eq!(inner.full_name(), "Outer.Inner");
    }
}
