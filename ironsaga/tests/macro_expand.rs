#[test]
fn expansion_tests() {
    macrotest::expand("tests/expand/*.rs");
    // or for CI, use:
    // macrotest::expand_without_refresh("tests/expand/*.rs");
}
