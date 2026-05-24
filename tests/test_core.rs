use riddle::{
    RiddleError,
    core::{CommonCore, Core},
    env::{Atom, AtomId, BoolExpr, Env, Object, ObjectId, Slot, Var},
    language::Disjunction,
    scope::{Class, Field, Function, Predicate, Scope, Type, arith_class},
};
use std::{
    any::Any,
    collections::HashMap,
    fmt,
    fs::read_to_string,
    path::PathBuf,
    rc::{Rc, Weak},
};

#[derive(Debug)]
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

pub struct TestCore {
    core: Rc<CommonCore>,
}

impl TestCore {
    pub fn new() -> Rc<Self> {
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
        let tp = arith_class(self, sum)?;
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
        let tp = arith_class(self, mul)?;
        Ok(Slot::Primitive(Rc::new(TestObject::new(tp))))
    }
    fn div(&self, left: Slot, right: Slot) -> Result<Slot, RiddleError> {
        let tp = arith_class(self, &[left, right])?;
        Ok(Slot::Primitive(Rc::new(TestObject::new(tp))))
    }

    fn assert(&self, _term: Rc<BoolExpr>) -> bool {
        true
    }

    fn new_var(&self, class: Rc<dyn Type>, instances: &[ObjectId]) -> Result<Slot, RiddleError> {
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

    fn get_function(&self, name: &str, classes: &[Rc<dyn Type>]) -> Option<Rc<Function>> {
        self.core.get_function(name, classes)
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

macro_rules! test_riddle {
    ($name:ident, $($path:expr),+) => {
        #[test]
        fn $name() {
            let solver = TestCore::new();
            $(
                let mut full_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
                full_path.push($path);
                let content = read_to_string(&full_path).expect(&format!("Failed to read file: {}", $path));
                solver.core.read(&content).expect("Failed to read problem");
            )+
        }
    };
}

test_riddle!(test_core_00, "examples/core/example_00.rddl");
test_riddle!(test_core_01, "examples/core/example_01.rddl");
test_riddle!(test_core_02, "examples/core/example_02.rddl");
test_riddle!(test_core_03, "examples/core/example_03.rddl");
test_riddle!(test_core_04, "examples/core/example_04.rddl");
test_riddle!(test_core_05, "examples/core/example_05.rddl");
test_riddle!(test_core_06, "examples/core/example_06.rddl");
test_riddle!(test_core_07, "examples/core/example_07.rddl");
test_riddle!(test_core_08, "examples/core/example_08.rddl");
test_riddle!(test_core_09, "examples/core/example_09.rddl");
test_riddle!(test_core_10, "examples/core/example_10.rddl");
test_riddle!(test_core_11, "examples/core/example_11.rddl");
test_riddle!(test_core_12, "examples/core/example_12.rddl");
test_riddle!(test_core_13, "examples/core/example_13.rddl");

test_riddle!(blocks_domain, "examples/blocks/blocks_domain.rddl");
test_riddle!(blocks_01, "examples/blocks/blocks_domain.rddl", "examples/blocks/blocks_01.rddl");
test_riddle!(blocks_02, "examples/blocks/blocks_domain.rddl", "examples/blocks/blocks_02.rddl");
test_riddle!(blocks_03, "examples/blocks/blocks_domain.rddl", "examples/blocks/blocks_03.rddl");

test_riddle!(types_rr_rr0, "examples/types/rr/rr_0.rddl");
test_riddle!(types_rr_rr1, "examples/types/rr/rr_1.rddl");
test_riddle!(types_rr_rr2, "examples/types/rr/rr_2.rddl");
test_riddle!(types_rr_rr3, "examples/types/rr/rr_3.rddl");
test_riddle!(types_sv_sv0, "examples/types/sv/sv_0.rddl");
test_riddle!(types_sv_sv1, "examples/types/sv/sv_1.rddl");
test_riddle!(types_sv_sv2, "examples/types/sv/sv_2.rddl");
test_riddle!(types_sv_sv3, "examples/types/sv/sv_3.rddl");

test_riddle!(ui_domain, "examples/urban_intelligence/urban_intelligence_domain.rddl");
test_riddle!(ui_01_03, "examples/urban_intelligence/urban_intelligence_domain.rddl", "examples/urban_intelligence/urban_intelligence_01_03.rddl");
test_riddle!(ui_01_06, "examples/urban_intelligence/urban_intelligence_domain.rddl", "examples/urban_intelligence/urban_intelligence_01_06.rddl");
test_riddle!(ui_01_09, "examples/urban_intelligence/urban_intelligence_domain.rddl", "examples/urban_intelligence/urban_intelligence_01_09.rddl");
test_riddle!(ui_01_12, "examples/urban_intelligence/urban_intelligence_domain.rddl", "examples/urban_intelligence/urban_intelligence_01_12.rddl");
test_riddle!(ui_01_15, "examples/urban_intelligence/urban_intelligence_domain.rddl", "examples/urban_intelligence/urban_intelligence_01_15.rddl");
test_riddle!(ui_01_18, "examples/urban_intelligence/urban_intelligence_domain.rddl", "examples/urban_intelligence/urban_intelligence_01_18.rddl");
test_riddle!(ui_01_21, "examples/urban_intelligence/urban_intelligence_domain.rddl", "examples/urban_intelligence/urban_intelligence_01_21.rddl");
test_riddle!(ui_01_24, "examples/urban_intelligence/urban_intelligence_domain.rddl", "examples/urban_intelligence/urban_intelligence_01_24.rddl");
test_riddle!(ui_01_27, "examples/urban_intelligence/urban_intelligence_domain.rddl", "examples/urban_intelligence/urban_intelligence_01_27.rddl");
test_riddle!(ui_01_30, "examples/urban_intelligence/urban_intelligence_domain.rddl", "examples/urban_intelligence/urban_intelligence_01_30.rddl");
test_riddle!(ui_02_03, "examples/urban_intelligence/urban_intelligence_domain.rddl", "examples/urban_intelligence/urban_intelligence_02_03.rddl");
test_riddle!(ui_02_06, "examples/urban_intelligence/urban_intelligence_domain.rddl", "examples/urban_intelligence/urban_intelligence_02_06.rddl");
test_riddle!(ui_02_09, "examples/urban_intelligence/urban_intelligence_domain.rddl", "examples/urban_intelligence/urban_intelligence_02_09.rddl");
test_riddle!(ui_02_12, "examples/urban_intelligence/urban_intelligence_domain.rddl", "examples/urban_intelligence/urban_intelligence_02_12.rddl");
test_riddle!(ui_02_15, "examples/urban_intelligence/urban_intelligence_domain.rddl", "examples/urban_intelligence/urban_intelligence_02_15.rddl");
test_riddle!(ui_02_18, "examples/urban_intelligence/urban_intelligence_domain.rddl", "examples/urban_intelligence/urban_intelligence_02_18.rddl");
test_riddle!(ui_02_21, "examples/urban_intelligence/urban_intelligence_domain.rddl", "examples/urban_intelligence/urban_intelligence_02_21.rddl");
test_riddle!(ui_02_24, "examples/urban_intelligence/urban_intelligence_domain.rddl", "examples/urban_intelligence/urban_intelligence_02_24.rddl");
test_riddle!(ui_02_27, "examples/urban_intelligence/urban_intelligence_domain.rddl", "examples/urban_intelligence/urban_intelligence_02_27.rddl");
test_riddle!(ui_02_30, "examples/urban_intelligence/urban_intelligence_domain.rddl", "examples/urban_intelligence/urban_intelligence_02_30.rddl");
test_riddle!(ui_03_03, "examples/urban_intelligence/urban_intelligence_domain.rddl", "examples/urban_intelligence/urban_intelligence_03_03.rddl");
test_riddle!(ui_03_06, "examples/urban_intelligence/urban_intelligence_domain.rddl", "examples/urban_intelligence/urban_intelligence_03_06.rddl");
test_riddle!(ui_03_09, "examples/urban_intelligence/urban_intelligence_domain.rddl", "examples/urban_intelligence/urban_intelligence_03_09.rddl");
test_riddle!(ui_03_12, "examples/urban_intelligence/urban_intelligence_domain.rddl", "examples/urban_intelligence/urban_intelligence_03_12.rddl");
test_riddle!(ui_03_15, "examples/urban_intelligence/urban_intelligence_domain.rddl", "examples/urban_intelligence/urban_intelligence_03_15.rddl");
test_riddle!(ui_03_18, "examples/urban_intelligence/urban_intelligence_domain.rddl", "examples/urban_intelligence/urban_intelligence_03_18.rddl");
test_riddle!(ui_03_21, "examples/urban_intelligence/urban_intelligence_domain.rddl", "examples/urban_intelligence/urban_intelligence_03_21.rddl");
test_riddle!(ui_03_24, "examples/urban_intelligence/urban_intelligence_domain.rddl", "examples/urban_intelligence/urban_intelligence_03_24.rddl");
test_riddle!(ui_03_27, "examples/urban_intelligence/urban_intelligence_domain.rddl", "examples/urban_intelligence/urban_intelligence_03_27.rddl");
test_riddle!(ui_03_30, "examples/urban_intelligence/urban_intelligence_domain.rddl", "examples/urban_intelligence/urban_intelligence_03_30.rddl");
