extern crate clap;
extern crate clap_nested;
extern crate regex;

use clap::Arg;
use clap_nested::{Command, Commander};

mod common;

use common::{assert_output, assert_result};

#[test]
fn two_level_commander() {
    let foo = Command::new("foo")
        .options(|app| {
            app.arg(
                Arg::with_name("debug")
                    .short("d")
                    .help("Prints debug information verbosely"),
            )
        })
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

    let commander = Commander::new()
        .options(|app| app.name("program"))
        .add_cmd(show)
        .add_cmd(what);

    assert!(commander.run_with_args_result(&["program", "show"]).is_ok());
    assert!(commander.run_with_args_result(&["program", "show", "foo"]).is_ok());
    assert!(commander.run_with_args_result(&["program", "show", "bar"]).is_ok());
    assert!(commander.run_with_args_result(&["program", "what"]).is_ok());

    assert_result(
        commander.run_result(),
        "error: program __VERSION__
__AUTHOR__
__DESC__

USAGE:
    __BIN_NAME__ [SUBCOMMAND]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

SUBCOMMANDS:
    help    Prints this message or the help of the given subcommand(s)
    show    Shows things
    what    So what",
        false,
    );
}

#[test]
fn help() {
    assert_output(
        &Commander::new().add_cmd(Command::new("foo").description("Shows foo")),
        &["program", "foo", "--help"],
        "program-foo __VERSION__
__AUTHOR__
Shows foo

USAGE:
    program foo

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information",
        false,
    );
}

#[test]
fn substituted_help() {
    let commander = Commander::new()
        .options(|app| app.name("program"))
        .add_cmd(Command::new("foo").description("Shows foo"));

    assert_output(
        &commander,
        &["program", "foo", "-e"],
        "error: error: Found argument '-e' which wasn't expected, or isn't valid in this context

program-foo __VERSION__
__AUTHOR__
Shows foo

USAGE:
    program foo

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information",
        false,
    );

    assert_output(
        &commander,
        &["program", "bar"],
        "error: error: Found argument 'bar' which wasn't expected, or isn't valid in this context

program __VERSION__
__AUTHOR__
__DESC__

USAGE:
    program [SUBCOMMAND]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

SUBCOMMANDS:
    foo     Shows foo
    help    Prints this message or the help of the given subcommand(s)",
        false,
    );
}

#[test]
fn failed_command() {
    assert_output(
        &Commander::new().add_cmd(
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
        ),
        &["test", "fail"],
        "error: other os error",
        true,
    );
}
