use crate::common::TestCore;
use riddle::{core::Core, env::BoolExpr, env::to_cnf};
use std::rc::Rc;

mod common;

#[test]
fn test_cnf_distribution() {
    let solver = TestCore::new();
    let var_type = Rc::downgrade(&solver.bool_type());

    let v1 = Rc::new(BoolExpr::Term { var_type: var_type.clone(), term: solver.new_bool_var() });
    let v2 = Rc::new(BoolExpr::Term { var_type: var_type.clone(), term: solver.new_bool_var() });
    let v3 = Rc::new(BoolExpr::Term { var_type: var_type.clone(), term: solver.new_bool_var() });

    // v2_and_v3 = (v2 AND v3)
    let v2_and_v3 = Rc::new(BoolExpr::And { var_type: var_type.clone(), terms: vec![v2.clone(), v3.clone()] });

    // expr = v1 OR (v2 AND v3)
    let expr = Rc::new(BoolExpr::Or { var_type: var_type.clone(), terms: vec![v1.clone(), v2_and_v3] });

    // The CNF of (v1 OR (v2 AND v3)) should be ((v1 OR v2) AND (v1 OR v3))
    let cnf = to_cnf(expr);

    if let BoolExpr::And { terms, .. } = cnf.as_ref() {
        assert_eq!(terms.len(), 2, "CNF should have two clauses");

        for clause in terms {
            assert!(matches!(clause.as_ref(), BoolExpr::Or { .. }), "Each clause in CNF should be an OR expression");
        }
    } else {
        panic!("CNF should be an AND expression");
    }
}

#[test]
fn test_cnf_with_negation() {
    let solver = TestCore::new();
    let var_type = Rc::downgrade(&solver.bool_type());

    let v1 = Rc::new(BoolExpr::Term { var_type: var_type.clone(), term: solver.new_bool_var() });
    let v2 = Rc::new(BoolExpr::Term { var_type: var_type.clone(), term: solver.new_bool_var() });
    let v3 = Rc::new(BoolExpr::Term { var_type: var_type.clone(), term: solver.new_bool_var() });

    // (v2 OR v3)
    let v2_or_v3 = Rc::new(BoolExpr::Or { var_type: var_type.clone(), terms: vec![v2, v3] });
    // (v1 AND (v2 OR v3))
    let inner_and = Rc::new(BoolExpr::And { var_type: var_type.clone(), terms: vec![v1, v2_or_v3] });
    // NOT (v1 AND (v2 OR v3))
    let expr = Rc::new(BoolExpr::Not { var_type: var_type.clone(), term: inner_and });

    // The CNF of NOT (v1 AND (v2 OR v3)) should be (NOT v1 OR NOT v2) AND (NOT v1 OR NOT v3)
    let cnf = to_cnf(expr);

    assert!(matches!(cnf.as_ref(), BoolExpr::And { .. }));
    if let BoolExpr::And { terms, .. } = cnf.as_ref() {
        assert_eq!(terms.len(), 2);
        for clause in terms {
            assert!(matches!(clause.as_ref(), BoolExpr::Or { .. }));
        }
    }
}

#[test]
fn test_cnf_flattening() {
    let solver = TestCore::new();
    let var_type = Rc::downgrade(&solver.bool_type());

    let v1 = Rc::new(BoolExpr::Term { var_type: var_type.clone(), term: solver.new_bool_var() });
    let v2 = Rc::new(BoolExpr::Term { var_type: var_type.clone(), term: solver.new_bool_var() });
    let v3 = Rc::new(BoolExpr::Term { var_type: var_type.clone(), term: solver.new_bool_var() });

    // nested = (v2 AND v3)
    let nested = Rc::new(BoolExpr::And { var_type: var_type.clone(), terms: vec![v2, v3] });
    // expr = v1 AND (v2 AND v3)
    let expr = Rc::new(BoolExpr::And { var_type: var_type.clone(), terms: vec![v1, nested] });

    // The CNF of (v1 AND (v2 AND v3)) should be (v1 AND v2 AND v3) after flattening
    let cnf = to_cnf(expr);

    if let BoolExpr::And { terms, .. } = cnf.as_ref() {
        assert_eq!(terms.len(), 3, "CNF should have three clauses after flattening");
    } else {
        panic!("CNF should be an AND expression");
    }
}
