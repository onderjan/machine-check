#[test]
fn machine_description() {
    let t = trybuild::TestCases::new();
    t.pass("tests/cases/01-items.rs");
    t.compile_fail("tests/cases/02-unsupported-item.rs");
}
