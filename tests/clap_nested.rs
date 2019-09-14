extern crate clap_nested;

use clap_nested::{Command, Commander};

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

    Commander::new().add_cmd(show).add_cmd(what).run()
}
