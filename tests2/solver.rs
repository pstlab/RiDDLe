use riddle::{
    core::{CommonCore, Core},
    env::{Atom, BoolExpr, Env, Var},
    language::{Disjunction, RiddleError},
    scope::{Field, Method, Predicate, Scope, Type, arith_class},
};
use std::{
    any::Any,
    fmt,
    fs::read_to_string,
    path::PathBuf,
    rc::{Rc, Weak},
};

#[derive(Debug)]
struct TestVar {
    var_type: Weak<dyn Type>,
}

impl TestVar {
    fn new(var_type: Rc<dyn Type>) -> Self {
        Self { var_type: Rc::downgrade(&var_type) }
    }
}

impl Var for TestVar {
    fn var_type(&self) -> Rc<dyn Type> {
        self.var_type.upgrade().expect("Type has been dropped").clone()
    }

    fn as_any(self: Rc<Self>) -> Rc<dyn Any> {
        self
    }
}

pub struct Solver {
    core: Rc<CommonCore>,
}

impl Solver {
    pub fn new() -> Rc<Self> {
        let slv = Rc::new_cyclic(|core| Solver {
            core: {
                let core: Weak<Solver> = core.clone();
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

impl Scope for Solver {
    fn core(self: Rc<Self>) -> Rc<dyn Core> {
        self
    }
    fn scope(&self) -> Option<Rc<dyn Scope>> {
        None
    }

    fn get_fields(&self) -> Vec<Rc<Field>> {
        self.core.get_fields()
    }
    fn get_field(&self, name: &str) -> Option<Rc<Field>> {
        self.core.get_field(name)
    }
    fn get_method(&self, name: &str, types: &[Rc<dyn Type>]) -> Option<Rc<Method>> {
        self.core.get_method(name, types)
    }
    fn get_type(&self, name: &str) -> Option<Rc<dyn Type>> {
        self.core.get_type(name)
    }
    fn get_predicate(&self, name: &str) -> Option<Rc<Predicate>> {
        self.core.get_predicate(name)
    }
}

impl Env for Solver {
    fn parent(&self) -> Option<Rc<dyn Env>> {
        None
    }
    fn get(&self, name: &str) -> Option<Rc<dyn Var>> {
        self.core.get(name)
    }
    fn set(&self, name: String, value: Rc<dyn Var>) {
        self.core.set(name, value)
    }
}

impl Core for Solver {
    fn new_bool(&self, _value: bool) -> Rc<dyn Var> {
        Rc::new(TestVar::new(self.bool_type()))
    }
    fn new_bool_var(&self) -> Rc<dyn Var> {
        Rc::new(TestVar::new(self.bool_type()))
    }
    fn new_int(&self, _value: i64) -> Rc<dyn Var> {
        Rc::new(TestVar::new(self.int_type()))
    }
    fn new_int_var(&self) -> Rc<dyn Var> {
        Rc::new(TestVar::new(self.int_type()))
    }
    fn new_real(&self, _num: i64, _den: i64) -> Rc<dyn Var> {
        Rc::new(TestVar::new(self.real_type()))
    }
    fn new_real_var(&self) -> Rc<dyn Var> {
        Rc::new(TestVar::new(self.real_type()))
    }
    fn new_string(&self, _value: &str) -> Rc<dyn Var> {
        Rc::new(TestVar::new(self.string_type()))
    }
    fn new_string_var(&self) -> Rc<dyn Var> {
        Rc::new(TestVar::new(self.string_type()))
    }

    fn sum(&self, sum: &[Rc<dyn Var>]) -> Result<Rc<dyn Var>, RiddleError> {
        let tp = arith_class(self, sum)?;
        if tp.name() == "int" {
            Ok(Rc::new(TestVar::new(self.int_type())))
        } else if tp.name() == "real" {
            Ok(Rc::new(TestVar::new(self.real_type())))
        } else {
            Err(RiddleError::TypeError(format!("Cannot sum variables of type {}", tp.name())))
        }
    }
    fn opposite(&self, term: Rc<dyn Var>) -> Result<Rc<dyn Var>, RiddleError> {
        let tp = term.var_type();
        if tp.name() == "int" {
            Ok(Rc::new(TestVar::new(self.int_type())))
        } else if tp.name() == "real" {
            Ok(Rc::new(TestVar::new(self.real_type())))
        } else {
            Err(RiddleError::TypeError(format!("Cannot negate variable of type {}", tp.name())))
        }
    }
    fn mul(&self, mul: &[Rc<dyn Var>]) -> Result<Rc<dyn Var>, RiddleError> {
        let tp = arith_class(self, mul)?;
        if tp.name() == "int" {
            Ok(Rc::new(TestVar::new(self.int_type())))
        } else if tp.name() == "real" {
            Ok(Rc::new(TestVar::new(self.real_type())))
        } else {
            Err(RiddleError::TypeError(format!("Cannot multiply variables of type {}", tp.name())))
        }
    }
    fn div(&self, left: Rc<dyn Var>, right: Rc<dyn Var>) -> Result<Rc<dyn Var>, RiddleError> {
        let tp = arith_class(self, &[left.clone(), right.clone()])?;
        if tp.name() == "int" {
            Ok(Rc::new(TestVar::new(self.int_type())))
        } else if tp.name() == "real" {
            Ok(Rc::new(TestVar::new(self.real_type())))
        } else {
            Err(RiddleError::TypeError(format!("Cannot divide variables of type {}", tp.name())))
        }
    }

    fn assert(&self, _term: Rc<BoolExpr>) -> bool {
        // For testing purposes, we can just return true for any assertion.
        // In a real solver, this would involve more complex logic to evaluate the expression.
        true
    }

    fn new_var(&self, class: Rc<dyn Type>, _instances: &[Rc<dyn Var>]) -> Result<Rc<dyn Var>, RiddleError> {
        // For testing purposes, we can ignore the instances and just create a new variable of the given class.
        Ok(Rc::new(TestVar::new(class)))
    }

    fn new_disjunction(&self, _disjunction: Disjunction) {}

    fn new_atom(&self, _atom: Rc<Atom>) {}
}

impl fmt::Debug for Solver {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Solver {{ ... }}")
    }
}

macro_rules! test_riddle {
    ($name:ident, $($path:expr),+) => {
        #[test]
        fn $name() {
            let solver = Solver::new();
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
