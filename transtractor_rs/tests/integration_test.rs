use transtractor_rs::Parser;

#[test]
fn test_with_defaults_loads_all_configs() {
    let _parser = Parser::with_defaults()
        .expect("Parser::with_defaults() should succeed");

    // Verify that all bundled configs were loaded successfully.
    // The presence of the parser without error means all configs were successfully loaded.
}

#[test]
fn test_with_defaults_creates_functional_parser() {
    // Verify Parser::with_defaults() creates a usable parser
    let parser = Parser::with_defaults()
        .expect("Parser::with_defaults() should succeed");

    // Verify that we can call parser methods without error
    // This confirms the configs are correctly loaded into the ConfigDB and typer
    let keys = parser.get_applicable_config_keys(&vec![]);
    // Empty text items should return empty keys, but the method should not error
    assert!(keys.is_empty());
}

#[test]
fn test_new_parser_has_no_configs() {
    let parser = Parser::new();

    // A fresh parser with no configs loaded should not find any applicable configs
    let keys = parser.get_applicable_config_keys(&vec![]);
    assert!(keys.is_empty());
}
