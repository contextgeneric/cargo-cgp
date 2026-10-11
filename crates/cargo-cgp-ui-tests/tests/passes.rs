//! Tests for the checks that compare passes ([`cargo_cgp_ui_tests::passes`]) and for the JSON
//! reduction.

use cargo_cgp_ui_tests::passes::{pass_through_check, reduce_json, status_check};
use cargo_cgp_ui_tests::snapshot::Outcome;

fn is_ok(outcome: &Outcome) -> bool {
    matches!(outcome, Outcome::Ok)
}

fn mismatch(outcome: Outcome) -> String {
    match outcome {
        Outcome::Mismatch(text) => text,
        _ => panic!("expected a mismatch"),
    }
}

#[test]
fn agreeing_exit_statuses_pass() {
    assert!(is_ok(&status_check(true, true)));
    assert!(is_ok(&status_check(false, false)));
}

#[test]
fn disagreeing_exit_statuses_fail() {
    let text = mismatch(status_check(false, true));
    assert!(
        text.contains("plain `cargo check` fails but `cargo cgp check` compiles"),
        "{text}"
    );
    assert!(!is_ok(&status_check(true, false)));
}

#[test]
fn pass_through_requires_equal_output() {
    assert!(is_ok(&pass_through_check("error: a\n", "error: a", None)));
    let text = mismatch(pass_through_check(
        "expected `a::Foo`",
        "expected `Foo`",
        None,
    ));
    assert!(
        text.contains("- expected `a::Foo`") && text.contains("+ expected `Foo`"),
        "{text}"
    );
}

#[test]
fn a_rewrites_falsely_marker_expects_a_difference() {
    assert!(is_ok(&pass_through_check("a", "b", Some("strips paths"))));
    let text = mismatch(pass_through_check("a", "a", Some("strips paths")));
    assert!(
        text.contains("remove the `//@rewrites-falsely: strips paths` marker"),
        "{text}"
    );
}

#[test]
fn reduces_cargo_json_to_compiler_messages() {
    let stream = r#"{"reason":"compiler-artifact","package_id":"cgp"}
{"reason":"compiler-message","message":{"level":"error","code":{"code":"E0277","explanation":"long"},"message":"[CGP-E001] headline","rendered":"full text","spans":[{"file_name":"src/main.rs","line_start":3,"column_start":5,"is_primary":true,"label":"here"}],"children":[{"level":"note","message":"root cause: x","spans":[]}]}}
{"reason":"build-finished","success":false}"#;
    let reduced = reduce_json(stream);
    let value: serde_json::Value = serde_json::from_str(&reduced).unwrap();
    assert_eq!(
        value,
        serde_json::json!([{
            "level": "error",
            "code": "E0277",
            "message": "[CGP-E001] headline",
            "spans": [{"file": "src/main.rs", "line": 3, "column": 5, "primary": true, "label": "here"}],
            "children": [{"level": "note", "message": "root cause: x", "spans": []}],
        }])
    );
}
