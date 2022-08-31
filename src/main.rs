mod dotbootstrap;
mod dotignore;
mod crawlers;

use dotbootstrap::DotBootstrap;

use std::path::{Path, PathBuf};

use clap::{App, Arg};
use env_logger::Env;
use log;

use dirs::home_dir;

fn main() {
    let arg_matches = App::new("dotbootstrap")
        .about("dotfile symlinker")
        .version("v0.1.0")
        .author("Daniel Montgomery")
        .arg(
            Arg::with_name("srcdir")
                .short('s')
                .long("srcdir")
                .default_value(".")
                .help("Sets the dotfiles source directory")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("destdir")
                .short('d')
                .long("destdir")
                .help("Sets the dotfiles target directory")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("dotignore")
                .short('f')
                .long("dotignore")
                .value_name("dotignore_fn")
                .default_value(".dotignore")
                .help("Sets the dotignore source dirctory")
                .takes_value(true),
        )
        .arg(Arg::with_name("dryrun").short('n').long("dryrun").help(""))
        .arg(
            Arg::with_name("verbosity")
                .short('v')
                .multiple_occurrences(true)
                .help("Verbose output"),
        )
        .get_matches();

    let dotignore_path = Path::new(arg_matches.get_one::<String>("dotignore").unwrap());
    let src_path = Path::new(arg_matches.get_one::<String>("srcdir").unwrap());
    let dest_path = match arg_matches.get_one::<String>("destdir") {
        Some(dest_dir) => PathBuf::from(dest_dir),
        None => home_dir().unwrap(),
    };

    let loglevel = match arg_matches.occurrences_of("verbosity") {
        0 => "error",
        1 => "warning",
        2 => "info",
        _ => "debug",
    };

    env_logger::Builder::from_env(Env::default().default_filter_or(loglevel)).init();
    log::debug!("Configured logger with log level: {}", loglevel);

    let mut bootstrap = DotBootstrap::new(dotignore_path, src_path, &dest_path);

    if arg_matches.is_present("dryrun") {
        log::debug!("Dryrun argument present, performing dryrun.");
        bootstrap.dryrun();
    } else {
        bootstrap.bootstrap();
    }
}
