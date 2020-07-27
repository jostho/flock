#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;

use clap::{App, Arg};
use rocket::response::content;
use rocket::State;
use std::collections::HashMap;

const ARG_DIR_PATH: &str = "dirpath";

struct Config {
    flag_dir_path: String,
    countries: HashMap<String, String>,
}

#[get("/list")]
fn list(config: State<Config>) -> String {
    format!("{:?}", config.countries.keys())
}

#[get("/quiz")]
fn quiz(config: State<Config>) -> content::Html<String> {
    let country = flock::get_random_country(&config.countries, &config.flag_dir_path);
    let resp = format!(
        "<html><body><h2>{} ({})</h2><img style='border:5px solid black' src='data:image/png;base64,{}'/></body></html>",
        country["name"],
        country["cca2"],
        country["flag"]
    );
    content::Html(resp)
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

    println!("flag dir path: {}", flag_dir_path);
    let config = Config {
        flag_dir_path: flag_dir_path.to_string(),
        countries,
    };

    rocket::ignite()
        .mount("/", routes![list, quiz])
        .manage(config)
        .launch();
}
