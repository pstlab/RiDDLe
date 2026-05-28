use crate::common::TestCore;
use std::{fs::read_to_string, path::PathBuf};

mod common;

macro_rules! test_riddle {
    ($name:ident, $($path:expr),+) => {
        #[test]
        fn $name() {
            let solver = TestCore::new();
            $(
                let mut full_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
                full_path.push($path);
                let content = read_to_string(&full_path).expect(&format!("Failed to read file: {}", $path));
                solver.read(&content).expect("Failed to read problem");
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
