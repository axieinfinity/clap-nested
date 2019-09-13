use clap_nested::Command;

pub fn cmd<'a>() -> Command<'a, str> {
    Command::new("bar")
        .description("Shows bar")
        .runner(|args, _matches| {
            println!("Running bar, env = {}", args);
        })
}
