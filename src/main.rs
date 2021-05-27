#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;

use clap::{App, Arg};
use rocket::config::{Config, Environment};
use rocket::response::NamedFile;
use rocket::response::Redirect;
use rocket::{Rocket, State};
use rocket_contrib::templates::Template;
use std::collections::HashMap;
use std::env;
use std::path::Path;

const ARG_PORT: &str = "port";
const ARG_LOCAL: &str = "local";
const ARG_FLAG_DIR: &str = "flag-dir";
const ENV_FLAG_DIR: &str = "FLOCK_FLAG_DIR";

const DEFAULT_PORT: u16 = 8000;
const BIND_ALL: &str = "0.0.0.0";
const BIND_LOCALHOST: &str = "127.0.0.1";

const ARG_TEMPLATE_DIR: &str = "template-dir";
const ENV_TEMPLATE_DIR: &str = "FLOCK_TEMPLATE_DIR";
const DEFAULT_TEMPLATE_DIR: &str = "templates";

const ENV_RELEASE_FILE: &str = "FLOCK_RELEASE";
const DEFAULT_RELEASE_FILE: &str = "/usr/local/etc/flock-release";

struct AppConfig {
    local: bool,
    port: u16,
    flag_dir: String,
    template_dir: String,
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

#[get("/release")]
fn release() -> Option<NamedFile> {
    let release_file = match env::var(ENV_RELEASE_FILE) {
        Ok(val) => val,
        Err(_e) => DEFAULT_RELEASE_FILE.to_string(),
    };
    NamedFile::open(Path::new(&release_file)).ok()
}

#[get("/list")]
fn list(config: State<AppConfig>) -> String {
    format!("{:?}", flock::get_country_codes(&config.countries))
}

#[get("/quiz")]
fn quiz(config: State<AppConfig>) -> Template {
    let question = flock::get_question(&config.countries, &config.flag_dir);
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
        .extra("template_dir", app_config.template_dir.to_string())
        .unwrap();
    rocket::custom(rocket_config)
        .mount(
            "/",
            routes![index, healthcheck, version, release, list, quiz],
        )
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
            Arg::with_name(ARG_FLAG_DIR)
                .short("d")
                .long(ARG_FLAG_DIR)
                .env(ENV_FLAG_DIR)
                .help("Flag dir")
                .takes_value(true)
                .validator(flock::is_valid_flag_dir)
                .required(true),
        )
        .arg(
            Arg::with_name(ARG_TEMPLATE_DIR)
                .long(ARG_TEMPLATE_DIR)
                .env(ENV_TEMPLATE_DIR)
                .help("Template dir")
                .default_value(DEFAULT_TEMPLATE_DIR)
                .validator(flock::is_valid_template_dir),
        )
        .get_matches();

    // bind details
    let port = args.value_of(ARG_PORT).unwrap();
    let port: u16 = port.parse().unwrap();
    let local = args.is_present(ARG_LOCAL);

    let flag_dir = args.value_of(ARG_FLAG_DIR).unwrap();
    let template_dir = args.value_of(ARG_TEMPLATE_DIR).unwrap();
    let countries = flock::get_countries(flag_dir);

    println!(
        "Using flag dir: {} , countries: {}",
        flag_dir,
        countries.len()
    );
    let config = AppConfig {
        local,
        port,
        flag_dir: flag_dir.to_string(),
        template_dir: template_dir.to_string(),
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
            flag_dir: COUNTRY_FLAGS_DIR.to_string(),
            template_dir: DEFAULT_TEMPLATE_DIR.to_string(),
            countries,
        }
    }
}
