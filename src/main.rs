#[macro_use]
extern crate rocket;

use clap::{App, Arg};
use rocket::fs::NamedFile;
use rocket::response::Redirect;
use rocket::{Build, Config, Rocket, State};
use rocket_dyn_templates::Template;
use std::collections::HashMap;
use std::env;
use std::net::{IpAddr, Ipv4Addr};
use std::path::Path;

const ARG_PORT: &str = "port";
const ARG_LOCAL: &str = "local";
const ARG_FLAG_DIR: &str = "flag-dir";
const ENV_FLAG_DIR: &str = "FLOCK_FLAG_DIR";

const DEFAULT_PORT: u16 = 8000;

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
async fn release() -> Option<NamedFile> {
    let release_file = match env::var(ENV_RELEASE_FILE) {
        Ok(val) => val,
        Err(_e) => DEFAULT_RELEASE_FILE.to_string(),
    };
    NamedFile::open(Path::new(&release_file)).await.ok()
}

#[get("/list")]
fn list(config: &State<AppConfig>) -> String {
    format!("{:?}", flock::get_country_codes(&config.countries))
}

#[get("/quiz")]
fn quiz(config: &State<AppConfig>) -> Template {
    let question = flock::get_question(&config.countries, &config.flag_dir);
    Template::render("quiz", &question)
}

fn rocket(app_config: AppConfig) -> Rocket<Build> {
    // decide bind interface
    let address = if app_config.local {
        IpAddr::V4(Ipv4Addr::LOCALHOST)
    } else {
        IpAddr::V4(Ipv4Addr::UNSPECIFIED)
    };

    let figment = Config::figment()
        .merge(("port", app_config.port))
        .merge(("address", address))
        .merge(("template_dir", app_config.template_dir.to_string()));
    rocket::custom(figment)
        .mount(
            "/",
            routes![index, healthcheck, version, release, list, quiz],
        )
        .attach(Template::fairing())
        .manage(app_config)
}

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
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

    rocket(config).launch().await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rocket::http::Status;
    use rocket::local::blocking::Client;

    const COUNTRY_FLAGS_DIR: &str = "target/country-flags";

    #[test]
    fn rocket_for_dummy_config() {
        let rocket = rocket(dummy_config());
        let figment = rocket.figment();
        let ipaddr: IpAddr = figment.extract_inner("address").unwrap();
        assert_eq!(ipaddr, IpAddr::V4(Ipv4Addr::UNSPECIFIED));
        let port: u16 = figment.extract_inner("port").unwrap();
        assert_eq!(port, DEFAULT_PORT);
    }

    #[test]
    fn get_index() {
        let client = Client::tracked(rocket(dummy_config())).unwrap();
        let response = client.get("/").dispatch();
        assert_eq!(response.status(), Status::SeeOther);
        assert_eq!(response.headers().get_one("Location"), Some("/quiz"));
    }

    #[test]
    fn get_healthcheck() {
        let client = Client::tracked(rocket(dummy_config())).unwrap();
        let response = client.get("/healthcheck").dispatch();
        assert_eq!(response.status(), Status::Ok);
        assert_eq!(response.into_string().unwrap(), "Ok");
    }

    #[test]
    fn get_version() {
        let client = Client::tracked(rocket(dummy_config())).unwrap();
        let response = client.get("/version").dispatch();
        assert_eq!(response.status(), Status::Ok);
        let version: String = clap::crate_version!().into();
        assert_eq!(response.into_string().unwrap(), version);
    }

    #[test]
    fn get_list() {
        let client = Client::tracked(rocket(dummy_config())).unwrap();
        let response = client.get("/list").dispatch();
        assert_eq!(response.status(), Status::Ok);
        let country_codes = ["AD", "AE", "AF", "ZA", "ZM", "ZW"];
        let country_codes_output = format!("{:?}", country_codes);
        assert_eq!(response.into_string().unwrap(), country_codes_output);
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
