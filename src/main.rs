#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;

use clap::{App, Arg};
use rocket::State;
use rocket_contrib::templates::Template;
use std::collections::HashMap;

const ARG_DIR_PATH: &str = "dirpath";

struct Config {
    flag_dir_path: String,
    countries: HashMap<String, String>,
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
    format!("{:?}", config.countries.keys())
}

#[get("/quiz")]
fn quiz(config: State<Config>) -> Template {
    let question = flock::get_question(&config.countries, &config.flag_dir_path);
    Template::render("quiz", &question)
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

    rocket::ignite()
        .mount("/", routes![healthcheck, version, list, quiz])
        .attach(Template::fairing())
        .manage(config)
        .launch();
}
