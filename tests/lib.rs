extern crate clap;
extern crate clap_nested;

use clap::Arg;
use clap_nested::{Command, Commander};

// TODO(trung): Test for stdout and stderr after commands are run.

type Result = std::result::Result<(), Box<dyn std::error::Error + Send + Sync>>;

#[test]
fn two_level_commander() -> Result {
    let foo = Command::new("foo")
        .description("Shows foo")
        .runner(|args, matches| {
            println!("foo: {:?} {:?}", args, matches);
            Ok(())
        });

    let bar = Command::new("bar")
        .description("Shows bar")
        .runner(|args, matches| {
            println!("bar: {:?} {:?}", args, matches);
            Ok(())
        });

    let show = Commander::new()
        .options(|app| {
            app.arg(
                Arg::with_name("debug")
                    .short("d")
                    .help("Prints debug information verbosely"),
            )
        })
        .add_cmd(foo)
        .add_cmd(bar)
        .no_cmd(|args, matches| {
            println!("show: {:?} {:?}", args, matches);
            Ok(())
        })
        .into_cmd("show")
        .description("Shows things");

    let what = Command::new("what")
        .description("So what")
        .runner(|args, matches| {
            println!("what: {:?} {:?}", args, matches);
            Ok(())
        });

    let commander = Commander::new().add_cmd(show).add_cmd(what);

    commander.run_with_args(&["program", "show"])?;
    commander.run_with_args(&["program", "show", "foo"])?;
    commander.run_with_args(&["program", "show", "bar"])?;
    commander.run_with_args(&["program", "show", "bar"])?;
    commander.run_with_args(&["program", "what"])?;
    commander.run_with_args(&["program"])?;
    commander.run()?;

    Ok(())
}

/*
#[test]
#[ignore]
fn show_help() {
    Commander::new()
        .add_cmd(
            Command::new("foo")
                .description("Shows foo")
                .runner(|_args, _matches| Ok(())),
        )
        .run_with_args(&["test", "foo", "--help"])
        .unwrap();
}
*/

#[test]
fn show_substituted_help() -> Result {
    let commander = Commander::new().add_cmd(Command::new("foo").description("Shows foo"));
    commander.run_with_args(&["test", "foo", "-e"])?;
    commander.run_with_args(&["test", "bar"])?;
    Ok(())
}

#[test]
#[should_panic(expected = "Kind(Other)")]
fn failed_command() {
    Commander::new()
        .add_cmd(
            Command::new("fail")
                .description("Fails")
                .options(|app| {
                    app.arg(
                        Arg::with_name("debug")
                            .short("d")
                            .help("Prints debug information verbosely"),
                    )
                })
                .runner(|_args, _matches| {
                    Err(std::io::Error::from(std::io::ErrorKind::Other).into())
                }),
        )
        .run_with_args(&["test", "fail"])
        .unwrap();
}
