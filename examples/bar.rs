use clap_nested::{file_stem, Command};

pub fn bar<'a>() -> Command<'a, str> {
    Command::new(file_stem!())
        .description("Shows bar")
        .runner(|args, _matches| {
            println!("Running bar, env = {}", args);
        })
}
