extern crate clap_nested;

use clap_nested::{Command, Commander};

#[test]
fn two_level_commander() {
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

    Commander::new().add_cmd(show).add_cmd(what).run().unwrap();
}

#[test]
fn show_help() {
    Commander::new()
        .add_cmd(
            Command::new("foo")
                .description("Shows foo")
                .runner(|_args, _matches| {
                    Ok(())
                }),
        )
        .run_with_args(&["test", "foo", "--help"])
        .unwrap();
}

#[test]
fn show_substituted_help() {
    Commander::new()
        .add_cmd(
            Command::new("foo")
                .description("Shows foo")
                .runner(|_args, _matches| {
                    Ok(())
                }),
        )
        .run_with_args(&["test", "bar", "-e"])
        .unwrap();
}

#[test]
#[should_panic(expected = "Kind(Other)")]
fn failed_command() {
    Commander::new()
        .add_cmd(
            Command::new("fail")
                .description("Fails")
                .runner(|_args, _matches| {
                    Err(std::io::Error::from(std::io::ErrorKind::Other))?;
                    Ok(())
                }),
        )
        .run_with_args(&["test", "fail"])
        .unwrap();
}
