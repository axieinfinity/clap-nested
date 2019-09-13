#[macro_use]
extern crate clap;

use clap::{AppSettings, Arg, SubCommand};

fn main() {
    let matches = clap::app_from_crate!()
        .global_setting(AppSettings::GlobalVersion)
        .arg(
            Arg::with_name("environment")
                .short("e")
                .long("env")
                .global(true)
                .takes_value(true)
                .value_name("STRING")
                .help("Sets an environment value, defaults to \"dev\""),
        )
        .subcommand(
            SubCommand::with_name("foo")
                .about("Shows foo")
                .author(clap::crate_authors!())
                .arg(
                    Arg::with_name("debug")
                        .short("d")
                        .help("Prints debug information verbosely"),
                ),
        )
        .subcommand(
            SubCommand::with_name("bar")
                .about("Shows bar")
                .author(clap::crate_authors!()),
        )
        .get_matches();

    let env = matches.value_of("environment").unwrap_or("dev");

    match matches.subcommand() {
        ("foo", Some(matches)) => {
            let debug = clap::value_t!(matches, "debug", bool).unwrap_or_default();
            println!("Running foo, env = {}, debug = {}", env, debug);
        }

        ("bar", Some(_matches)) => println!("Running bar, env = {}", env),

        _ => println!("No subcommand matched."),
    }
}
