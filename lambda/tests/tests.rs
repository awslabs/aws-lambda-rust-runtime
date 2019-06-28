use trybuild::TestCases;

#[test]
fn ui_tests() {
    let t = TestCases::new();
    t.pass("tests/ui/arg.rs");
    t.pass("tests/ui/arg-plus-ctx.rs");
    t.compile_fail("tests/compile-fail/no-args.rs");
    t.compile_fail("tests/compile-fail/non-async-main.rs");
}
