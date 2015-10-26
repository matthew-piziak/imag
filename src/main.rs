#[macro_use] extern crate clap;
#[macro_use] extern crate log;

use cli::Config;
use config::Configuration;
use runtime::{ImagLogger, Runtime};
use clap::App;

mod cli;
mod config;
mod runtime;
mod module;
mod storage;

fn main() {
    let early_logger = ImagLogger::early().unwrap();
    let yaml = load_yaml!("../etc/cli.yml");
    let app = App::from_yaml(yaml);
    let mut config = Config::new(app);
    let configuration = Configuration::new(&config);

    let logger = ImagLogger::init(&config);
    let rt = Runtime::new(config);

    info!("Hello, world!");
}
