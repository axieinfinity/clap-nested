use clap::Arg;
use clap_nested::{file_stem, Command};

pub fn foo<'a>() -> Command<'a, str> {
    Command::new(file_stem!())
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
