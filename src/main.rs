#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;

use clap::{App, Arg};
use rocket::config::{Config, Environment};
use rocket::response::Redirect;
use rocket::{Rocket, State};
use rocket_contrib::templates::Template;
use std::collections::HashMap;

const ARG_PORT: &str = "port";
const ARG_LOCAL: &str = "local";
const ARG_DIR_PATH: &str = "dir-path";

const DEFAULT_PORT: u16 = 8000;
const BIND_ALL: &str = "0.0.0.0";
const BIND_LOCALHOST: &str = "127.0.0.1";

struct AppConfig {
    local: bool,
    port: u16,
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
fn list(config: State<AppConfig>) -> String {
    format!("{:?}", flock::get_country_codes(&config.countries))
}

#[get("/quiz")]
fn quiz(config: State<AppConfig>) -> Template {
    let question = flock::get_question(&config.countries, &config.flag_dir_path);
    Template::render("quiz", &question)
}

fn rocket(app_config: AppConfig) -> Rocket {
    // decide bind interface
    let address = if app_config.local {
        BIND_LOCALHOST
    } else {
        BIND_ALL // default
    };
    let rocket_config = Config::build(Environment::Staging)
        .address(address)
        .port(app_config.port)
        .unwrap();
    rocket::custom(rocket_config)
        .mount("/", routes![index, healthcheck, version, list, quiz])
        .attach(Template::fairing())
        .manage(app_config)
}

fn main() {
    let args = App::new(clap::crate_name!())
        .about(clap::crate_description!())
        .version(clap::crate_version!())
        .arg(
            Arg::with_name(ARG_PORT)
                .long(ARG_PORT)
                .help("Port number to use")
                .default_value("8000")
                .validator(flock::is_valid_port),
        )
        .arg(
            Arg::with_name(ARG_LOCAL)
                .long(ARG_LOCAL)
                .help("Bind on local interface")
                .takes_value(false),
        )
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

    // bind details
    let port = args.value_of(ARG_PORT).unwrap();
    let port: u16 = port.parse().unwrap();
    let local = args.is_present(ARG_LOCAL);

    let flag_dir_path = args.value_of(ARG_DIR_PATH).unwrap();
    let countries = flock::get_countries(flag_dir_path);

    println!(
        "Using flag dir: {} , countries: {}",
        flag_dir_path,
        countries.len()
    );
    let config = AppConfig {
        local,
        port,
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
    fn rocket_for_dummy_config() {
        let rocket = rocket(dummy_config());
        assert_eq!(rocket.config().environment, Environment::Staging);
        assert_eq!(rocket.config().address, BIND_ALL);
        assert_eq!(rocket.config().port, DEFAULT_PORT);
    }

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

    fn dummy_config() -> AppConfig {
        let mut countries = HashMap::new();
        countries.insert("AD".to_string(), "Andorra".to_string());
        countries.insert("AE".to_string(), "United Arab Emirates".to_string());
        countries.insert("AF".to_string(), "Afghanistan".to_string());
        countries.insert("ZA".to_string(), "South Africa".to_string());
        countries.insert("ZM".to_string(), "Zambia".to_string());
        countries.insert("ZW".to_string(), "Zimbabwe".to_string());
        AppConfig {
            local: false,
            port: DEFAULT_PORT,
            flag_dir_path: COUNTRY_FLAGS_DIR.to_string(),
            countries,
        }
    }
}
