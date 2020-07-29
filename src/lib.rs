use rand::seq::SliceRandom;
use rand::Rng;
use serde::Serialize;
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};

const COUNTRIES_JSON: &str = "countries.json";
const PNG_DIR: &str = "png250px";
const PNG_EXTENSION: &str = "png";
const NUMBER_OF_OPTIONS: u8 = 4;

#[derive(Serialize, Debug)]
pub struct Question {
    country: Country,
    options: Vec<Country>,
}

#[derive(Serialize, Debug)]
pub struct Country {
    cca2: String,
    name: String,
    flag: String,
}

pub fn is_valid_dir_path(val: String) -> Result<(), String> {
    // check whether the directory is a copy of country-flags git repo
    let mut json_path_buf = PathBuf::from(&val);
    json_path_buf.push(COUNTRIES_JSON);
    let mut png_dir_path_buf = PathBuf::from(&val);
    png_dir_path_buf.push(PNG_DIR);
    if Path::new(&val).is_dir()
        && json_path_buf.as_path().is_file()
        && png_dir_path_buf.as_path().is_dir()
    {
        Ok(())
    } else {
        Err("Dir path is not valid".to_string())
    }
}

pub fn get_countries(flag_dir_path: &str) -> HashMap<String, String> {
    // read countries.json
    let mut path_buf = PathBuf::from(&flag_dir_path);
    path_buf.push(COUNTRIES_JSON);
    let result = read_from_json_file(path_buf.as_path());
    filter_countries(result.unwrap())
}

fn read_from_json_file(path: &Path) -> Result<HashMap<String, String>, Box<dyn Error>> {
    let file = File::open(path)?;
    let buf_reader = BufReader::new(file);
    let countries = serde_json::from_reader(buf_reader)?;
    Ok(countries)
}

fn filter_countries(mut countries: HashMap<String, String>) -> HashMap<String, String> {
    // remove any non 2-char values - e.g. "GB-ENG"
    countries.retain(|k, _| k.len() == 2);
    // remove any regions OR territories with similar flags
    let exclusion_list = vec![
        "AQ", "BQ", "BV", "EU", "GF", "GP", "HM", "PM", "RE", "SH", "SJ", "UM", "XK", "YT",
    ];
    for cca2 in exclusion_list {
        countries.remove(cca2);
    }
    countries
}

pub fn get_question(countries: &HashMap<String, String>, flag_dir_path: &str) -> Question {
    let mut rng = rand::thread_rng();
    let country_codes: Vec<String> = countries.keys().cloned().collect();
    let index = rng.gen_range(0, country_codes.len());
    let cca2 = &country_codes[index];
    let name = &countries[cca2];
    let country = get_country_with_flag(cca2, name, flag_dir_path);
    let mut country_code_options: Vec<String> = country_codes
        .choose_multiple(&mut rng, NUMBER_OF_OPTIONS as usize - 1)
        .cloned()
        .collect();
    country_code_options.push(cca2.to_string());
    let options = get_options(countries, country_code_options);
    Question { country, options }
}

fn get_options(
    countries: &HashMap<String, String>,
    country_code_options: Vec<String>,
) -> Vec<Country> {
    let mut options = Vec::new();
    for cca2 in country_code_options {
        let country = Country {
            cca2: cca2.to_string(),
            name: countries[&cca2].to_string(),
            flag: "".to_string(),
        };
        options.push(country);
    }
    options
}

fn get_country_with_flag(cca2: &str, name: &str, flag_dir_path: &str) -> Country {
    let flag_base64 = get_flag_base64_encoded(cca2, flag_dir_path);
    Country {
        cca2: cca2.to_string(),
        name: name.to_string(),
        flag: flag_base64,
    }
}

fn get_flag_base64_encoded(cca2: &str, flag_dir_path: &str) -> String {
    let mut path_buf = PathBuf::from(&flag_dir_path);
    path_buf.push(PNG_DIR);
    path_buf.push(cca2.to_ascii_lowercase());
    path_buf.set_extension(PNG_EXTENSION);
    let result = std::fs::read(path_buf.as_path());
    base64::encode(result.unwrap())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_valid_dir_path_for_target() {
        let result = is_valid_dir_path("target".to_string());
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Dir path is not valid");
    }

    #[test]
    fn filter_countries_for_gb_countries() {
        let mut countries = HashMap::new();
        countries.insert("GB-ENG".to_string(), "England".to_string());
        countries.insert("GB-SCT".to_string(), "Scotland".to_string());
        countries.insert("GB-WLS".to_string(), "Wales".to_string());
        countries.insert("GB".to_string(), "United Kingdom".to_string());
        let result = filter_countries(countries);
        assert!(result.contains_key("GB"));
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn filter_countries_for_excluded_countries() {
        let mut countries = HashMap::new();
        countries.insert("AD".to_string(), "Andorra".to_string());
        countries.insert("AQ".to_string(), "Antarctica".to_string());
        countries.insert("EU".to_string(), "Europe".to_string());
        countries.insert("GB".to_string(), "United Kingdom".to_string());
        countries.insert("YT".to_string(), "Mayotte".to_string());
        countries.insert("ZW".to_string(), "Zimbabwe".to_string());
        let result = filter_countries(countries);
        assert!(result.contains_key("AD"));
        assert!(!result.contains_key("AQ"));
        assert_eq!(result.len(), 3);
    }
}
