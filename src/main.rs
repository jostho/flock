#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;

use clap::{App, Arg};
use rocket::response::Redirect;
use rocket::{Rocket, State};
use rocket_contrib::templates::Template;
use std::collections::HashMap;

const ARG_DIR_PATH: &str = "dir-path";

struct Config {
    flag_dir_path: String,
    countries: HashMap<String, String>,
}

#[get("/")]
fn index() -> Redirect {
    Redirect::to("/quiz")
}

#[get("/healthcheck")]
fn healthcheck() -> &'static str {
    "Ok"
}

#[get("/version")]
fn version() -> &'static str {
    clap::crate_version!()
}

#[get("/list")]
fn list(config: State<Config>) -> String {
    format!("{:?}", flock::get_country_codes(&config.countries))
}

#[get("/quiz")]
fn quiz(config: State<Config>) -> Template {
    let question = flock::get_question(&config.countries, &config.flag_dir_path);
    Template::render("quiz", &question)
}

fn rocket(config: Config) -> Rocket {
    rocket::ignite()
        .mount("/", routes![index, healthcheck, version, list, quiz])
        .attach(Template::fairing())
        .manage(config)
}

fn main() {
    let args = App::new(clap::crate_name!())
        .about(clap::crate_description!())
        .version(clap::crate_version!())
        .arg(
            Arg::with_name(ARG_DIR_PATH)
                .short("d")
                .long(ARG_DIR_PATH)
                .help("Flag dir path")
                .takes_value(true)
                .validator(flock::is_valid_dir_path)
                .required(true),
        )
        .get_matches();

    let flag_dir_path = args.value_of(ARG_DIR_PATH).unwrap();

    let countries = flock::get_countries(flag_dir_path);

    println!(
        "Using flag dir: {} , countries: {}",
        flag_dir_path,
        countries.len()
    );
    let config = Config {
        flag_dir_path: flag_dir_path.to_string(),
        countries,
    };

    rocket(config).launch();
}

#[cfg(test)]
mod tests {
    use super::*;
    use rocket::http::Status;
    use rocket::local::Client;

    const COUNTRY_FLAGS_DIR: &str = "target/country-flags";

    #[test]
    fn get_index() {
        let client = Client::new(rocket(dummy_config())).unwrap();
        let response = client.get("/").dispatch();
        assert_eq!(response.status(), Status::SeeOther);
        assert_eq!(response.headers().get_one("Location"), Some("/quiz"));
    }

    #[test]
    fn get_healthcheck() {
        let client = Client::new(rocket(dummy_config())).unwrap();
        let mut response = client.get("/healthcheck").dispatch();
        assert_eq!(response.status(), Status::Ok);
        assert_eq!(response.body_string(), Some("Ok".into()));
    }

    #[test]
    fn get_version() {
        let client = Client::new(rocket(dummy_config())).unwrap();
        let mut response = client.get("/version").dispatch();
        assert_eq!(response.status(), Status::Ok);
        assert_eq!(response.body_string(), Some(clap::crate_version!().into()));
    }

    #[test]
    fn get_list() {
        let client = Client::new(rocket(dummy_config())).unwrap();
        let mut response = client.get("/list").dispatch();
        assert_eq!(response.status(), Status::Ok);
        let country_codes = ["AD", "AE", "AF", "ZA", "ZM", "ZW"];
        assert_eq!(
            response.body_string(),
            Some(format!("{:?}", country_codes).into())
        );
    }

    fn dummy_config() -> Config {
        let mut countries = HashMap::new();
        countries.insert("AD".to_string(), "Andorra".to_string());
        countries.insert("AE".to_string(), "United Arab Emirates".to_string());
        countries.insert("AF".to_string(), "Afghanistan".to_string());
        countries.insert("ZA".to_string(), "South Africa".to_string());
        countries.insert("ZM".to_string(), "Zambia".to_string());
        countries.insert("ZW".to_string(), "Zimbabwe".to_string());
        Config {
            flag_dir_path: COUNTRY_FLAGS_DIR.to_string(),
            countries,
        }
    }
}
