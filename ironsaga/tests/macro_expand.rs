#[test]
fn expansion_tests() {
    if std::env::var("CI").is_ok() {
        macrotest::expand_without_refresh("tests/expand/*.rs");
    } else {
        macrotest::expand("tests/expand/*.rs");
    }
}
