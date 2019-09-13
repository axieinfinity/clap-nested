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
