use clap::Parser;
use rocket::fs::NamedFile;
use rocket::response::Redirect;
use rocket::{Build, Config, Rocket, State};
use rocket_dyn_templates::Template;
use std::collections::HashMap;
use std::env;
use std::net::{IpAddr, Ipv4Addr};
use std::path::Path;

const ENV_FLAG_DIR: &str = "FLOCK_FLAG_DIR";

const ENV_TEMPLATE_DIR: &str = "FLOCK_TEMPLATE_DIR";
const DEFAULT_TEMPLATE_DIR: &str = "templates";

const ENV_RELEASE_FILE: &str = "FLOCK_RELEASE";
const DEFAULT_RELEASE_FILE: &str = "/usr/local/etc/flock-release";

/// Read cli args
#[derive(Parser, Debug)]
#[clap(version, about)]
struct Args {
    /// Port number to use
    #[clap(short, long, value_parser, default_value_t = 8000, validator = flock::is_valid_port)]
    port: u16,

    /// Bind on local interface
    #[clap(short, long)]
    local: bool,

    /// Flag dir
    #[clap(short, long, value_parser, env = ENV_FLAG_DIR, validator = flock::is_valid_flag_dir)]
    flag_dir: String,

    /// Template dir
    #[clap(short, long, value_parser, env = ENV_TEMPLATE_DIR, default_value = DEFAULT_TEMPLATE_DIR, validator = flock::is_valid_template_dir)]
    template_dir: String,
}

struct AppConfig {
    local: bool,
    port: u16,
    flag_dir: String,
    template_dir: String,
    countries: HashMap<String, String>,
}

#[rocket::get("/")]
fn index() -> Redirect {
    Redirect::to("/quiz")
}

#[rocket::get("/healthcheck")]
fn healthcheck() -> &'static str {
    "Ok"
}

#[rocket::get("/version")]
fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[rocket::get("/release")]
async fn release() -> Option<NamedFile> {
    let release_file = match env::var(ENV_RELEASE_FILE) {
        Ok(val) => val,
        Err(_e) => DEFAULT_RELEASE_FILE.to_string(),
    };
    NamedFile::open(Path::new(&release_file)).await.ok()
}

#[rocket::get("/list")]
fn list(config: &State<AppConfig>) -> String {
    format!("{:?}", flock::get_country_codes(&config.countries))
}

#[rocket::get("/quiz")]
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
            rocket::routes![index, healthcheck, version, release, list, quiz],
        )
        .attach(Template::fairing())
        .manage(app_config)
}

#[rocket::launch]
fn launch() -> Rocket<Build> {
    let args = Args::parse();
    let countries = flock::get_countries(&args.flag_dir);

    println!(
        "Using flag dir: {} , countries: {}",
        args.flag_dir,
        countries.len()
    );
    let config = AppConfig {
        local: args.local,
        port: args.port,
        flag_dir: args.flag_dir.to_string(),
        template_dir: args.template_dir.to_string(),
        countries,
    };

    rocket(config)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rocket::http::Status;
    use rocket::local::blocking::Client;

    const DEFAULT_PORT: u16 = 8000;
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
        assert_eq!(response.into_string().unwrap(), env!("CARGO_PKG_VERSION"));
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
