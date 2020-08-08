// tests expects "country-flags" sources at "target/country-flags"
const COUNTRY_FLAGS_DIR: &str = "target/country-flags";
const COUNTRIES_COUNT: usize = 229;

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

    assert_eq!(result.len(), COUNTRIES_COUNT);
    assert!(result.contains_key("AD"));
    assert!(!result.contains_key("AQ"));
}

#[test]
#[ignore]
fn get_country_codes_for_country_flags() {
    let countries = flock::get_countries(COUNTRY_FLAGS_DIR);
    let result = flock::get_country_codes(&countries);

    assert_eq!(result.len(), COUNTRIES_COUNT);
    assert_eq!(result[0], "AD");
    assert_eq!(result[1], "AE");
}
