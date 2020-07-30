#[test]
#[ignore]
fn get_countries_for_country_flags() {
    let flag_dir_path = "target/country-flags";
    let result = flock::get_countries(flag_dir_path);

    assert!(result.contains_key("AD"));
    assert!(!result.contains_key("AQ"));
    assert_eq!(result.len(), 230);
}
