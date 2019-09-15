#[macro_use]
extern crate clap;
extern crate clap_nested;

use clap::Arg;
use clap_nested::Commander;

mod bar;
mod foo;

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
        .add_cmd(foo::get_cmd())
        .add_cmd(bar::get_cmd())
        .no_cmd(|_args, _matches| {
            println!("No subcommand matched");
            Ok(())
        })
        .run()
        .unwrap();
}
