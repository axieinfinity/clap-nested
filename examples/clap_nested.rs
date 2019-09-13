#[macro_use]
extern crate clap;
extern crate clap_nested;

use clap::Arg;
use clap_nested::Commander;

mod foo {
    use clap::Arg;
    use clap_nested::Command;

    pub fn cmd<'a>() -> Command<'a, str> {
        Command::new("foo")
            .description("Shows foo")
            .options(|app| {
                app.arg(
                    Arg::with_name("debug")
                        .short("d")
                        .help("Prints debug information verbosely"),
                )
            })
            .runner(|args, matches| {
                let debug = clap::value_t!(matches, "debug", bool).unwrap_or_default();
                println!("Running foo, env = {}, debug = {}", args, debug);
            })
    }
}

mod bar {
    use clap_nested::Command;

    pub fn cmd<'a>() -> Command<'a, str> {
        Command::new("bar")
            .description("Shows bar")
            .runner(|args, _matches| {
                println!("Running bar, env = {}", args);
            })
    }
}

fn main() {
    Commander::new()
        .options(|app| {
            app.arg(
                Arg::with_name("environment")
                    .short("e")
                    .long("env")
                    .global(true)
                    .takes_value(true)
                    .value_name("STRING")
                    .help("Sets an environment value, defaults to \"dev\""),
            )
        })
        .args(|_args, matches| matches.value_of("environment").unwrap_or("dev"))
        .add_cmd(foo::cmd())
        .add_cmd(bar::cmd())
        .no_cmd(|_args, _matches| {
            println!("No subcommand matched.");
        })
        .run(&());
}
