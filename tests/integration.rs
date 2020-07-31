const COUNTRY_FLAGS_DIR: &str = "target/country-flags";

#[test]
#[ignore]
fn is_valid_dir_path_for_country_flags() {
    let result = flock::is_valid_dir_path(COUNTRY_FLAGS_DIR.to_string());
    assert!(result.is_ok());
}

#[test]
#[ignore]
fn get_countries_for_country_flags() {
    let result = flock::get_countries(COUNTRY_FLAGS_DIR);

    assert!(result.contains_key("AD"));
    assert!(!result.contains_key("AQ"));
    assert_eq!(result.len(), 230);
}
