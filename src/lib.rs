//! # Convenient `clap` for CLI apps with multi-level subcommands
//!
//! `clap-nested` provides a convenient way for setting up CLI apps
//! with multi-level subcommands.
//!
//! We all know that [`clap`][clap] really shines when it comes to
//! parsing CLI arguments. It even supports nicely formatted help messages,
//! subcommands, and shell completion out of the box.
//!
//! However, [`clap`][clap] is very much unopinionated in how we should
//! structure and execute logic. Even when we have tens of subcommands
//! (and arguments!), we still have to manually match against
//! all possible options and handle them accordingly. That process quickly
//! becomes tedious and unorganized.
//!
//! So, `clap-nested` add a little sauce of opinion into [`clap`][clap]
//! to help with that.
//!
//! # Use case: Easy subcommands and command execution
//!
//! In `clap-nested`, commands are defined together with how to execute them.
//!
//! Making it that way instead of going through a separate
//! matching-and-executing block of code like in [`clap`][clap],
//! it's very natural to separate commands into different files
//! in an organized and structured way.
//!
//! ```
//! #[macro_use]
//! extern crate clap;
//!
//! use clap::{Arg, ArgMatches};
//! use clap_nested::{Command, Commander};
//!
//! fn main() {
//!     let foo = Command::new("foo")
//!         .description("Shows foo")
//!         .options(|app| {
//!             app.arg(
//!                 Arg::with_name("debug")
//!                     .short("d")
//!                     .help("Prints debug information verbosely"),
//!             )
//!         })
//!         // Putting argument types here for clarity
//!         .runner(|args: &str, matches: &ArgMatches<'_>| {
//!             let debug = clap::value_t!(matches, "debug", bool).unwrap_or_default();
//!             println!("Running foo, env = {}, debug = {}", args, debug);
//!             Ok(())
//!         });
//!
//!     let bar = Command::new("bar")
//!         .description("Shows bar")
//!         // Putting argument types here for clarity
//!         .runner(|args: &str, _matches: &ArgMatches<'_>| {
//!             println!("Running bar, env = {}", args);
//!             Ok(())
//!         });
//!
//!     Commander::new()
//!         .options(|app| {
//!             app.arg(
//!                 Arg::with_name("environment")
//!                     .short("e")
//!                     .long("env")
//!                     .global(true)
//!                     .takes_value(true)
//!                     .value_name("STRING")
//!                     .help("Sets an environment value, defaults to \"dev\""),
//!             )
//!         })
//!         // `Commander::args()` derives arguments to pass to subcommands.
//!         // Notice all subcommands (i.e. `foo` and `bar`) will accept `&str` as arguments.
//!         .args(|_args, matches| matches.value_of("environment").unwrap_or("dev"))
//!         // Add all subcommands
//!         .add_cmd(foo)
//!         .add_cmd(bar)
//!         // To handle when no subcommands match
//!         .no_cmd(|_args, _matches| {
//!             println!("No subcommand matched");
//!             Ok(())
//!         })
//!         .run()
//!         .unwrap();
//! }
//! ```
//!
//! # Use case: Straightforward multi-level subcommands
//!
//! [`Commander`](struct.Commander.html) acts like a runnable group
//! of subcommands, calling [`run`](struct.Commander.html#method.run)
//! on a [`Commander`](struct.Commander.html)
//! gets the whole execution process started.
//!
//! On the other hand, [`Commander`](struct.Commander.html)
//! could also be converted into a [`MultiCommand`](struct.MultiCommand.html)
//! to be further included (and executed)
//! under another [`Commander`](struct.Commander.html).
//! This makes writing multi-level subcommands way easy.
//!
//! ```
//! use clap_nested::{Commander, MultiCommand};
//!
//! let multi_cmd: MultiCommand<(), ()> = Commander::new()
//!     // Add some theoretical subcommands
//!     // .add_cmd(model)
//!     // .add_cmd(controller)
//!     // Specify a name for the newly converted command
//!     .into_cmd("generate")
//!     // Optionally specify a description
//!     .description("Generates resources");
//! ```
//!
//! # Use case: Printing help messages directly on errors
//!
//! [`clap`][clap] is also the CLI parsing library which powers [Cargo][cargo].
//!
//! Sometimes when you run a [Cargo][cargo] command wrongly,
//! you may see this:
//!
//! ```shell
//! $ cargo run -x
//! error: Found argument '-x' which wasn't expected, or isn't valid in this context
//!
//! USAGE:
//!     cargo run [OPTIONS] [--] [args]...
//!
//! For more information try --help
//! ```
//!
//! While it works and is better for separation of concern
//! (one command, one job, no suprise effect),
//! we often wish for more. We want the help message to be printed directly
//! on errors, so it doesn't take us one more command to show the help message
//! (and then maybe one more to run the supposedly correct command).
//!
//! That's why we take a bit of trade-off to change the default behavior
//! of [`clap`][clap]. It now works this way:
//!
//! ```shell
//! $ cargo run -x
//! error: Found argument '-x' which wasn't expected, or isn't valid in this context
//!
//! cargo-run
//! Run a binary or example of the local package
//!
//! USAGE:
//!     cargo run [OPTIONS] [--] [args]...
//!
//! OPTIONS:
//!     -q, --quiet                      No output printed to stdout
//!         --bin <NAME>...              Name of the bin target to run
//!         --example <NAME>...          Name of the example target to run
//!     -p, --package <SPEC>             Package with the target to run
//!     -j, --jobs <N>                   Number of parallel jobs, defaults to # of CPUs
//!     <...omitted for brevity...>
//!
//! ARGS:
//!     <args>...
//!
//! <...omitted for brevity...>
//! ```
//!
//! [cargo]: https://github.com/rust-lang/cargo
//! [clap]: https://github.com/clap-rs/clap

use std::collections::HashMap;
use std::ffi::OsString;
use std::io::Write;
use std::result::Result as StdResult;

extern crate clap;

use clap::{
    App, AppSettings, ArgMatches, Error as ClapError, ErrorKind as ClapErrorKind, SubCommand,
};

mod macros;

type Result = StdResult<(), ClapError>;

#[doc(hidden)]
pub trait CommandLike<T: ?Sized> {
    fn name(&self) -> &str;
    fn app(&self) -> App;
    fn run(&self, args: &T, matches: &ArgMatches<'_>, help: &Help) -> Result;
}

/// Define a single-purpose command to be included
/// in a [`Commander`](struct.Commander.html)
pub struct Command<'a, T: ?Sized> {
    name: &'a str,
    desc: Option<&'a str>,
    opts: Option<Box<dyn for<'x, 'y> Fn(App<'x, 'y>) -> App<'x, 'y> + 'a>>,
    runner: Option<Box<dyn Fn(&T, &ArgMatches<'_>) -> Result + 'a>>,
}

impl<'a, T: ?Sized> Command<'a, T> {
    pub fn new(name: impl Into<&'a str>) -> Self {
        Self {
            name: name.into(),
            desc: None,
            opts: None,
            runner: None,
        }
    }

    pub fn description(mut self, desc: impl Into<&'a str>) -> Self {
        self.desc = Some(desc.into());
        self
    }

    pub fn options(mut self, opts: impl for<'x, 'y> Fn(App<'x, 'y>) -> App<'x, 'y> + 'a) -> Self {
        self.opts = Some(Box::new(opts));
        self
    }

    pub fn runner(mut self, run: impl Fn(&T, &ArgMatches<'_>) -> Result + 'a) -> Self {
        self.runner = Some(Box::new(run));
        self
    }
}

impl<'a, T: ?Sized> CommandLike<T> for Command<'a, T> {
    fn name(&self) -> &str {
        self.name
    }

    fn app(&self) -> App {
        let mut app = SubCommand::with_name(self.name);

        if let Some(desc) = self.desc {
            app = app.about(desc);
        }

        if let Some(cmd) = &self.opts {
            app = cmd(app);
        }

        app
    }

    fn run(&self, args: &T, matches: &ArgMatches<'_>, _help: &Help) -> Result {
        if let Some(runner) = &self.runner {
            runner(args, matches)?;
        }

        Ok(())
    }
}

/// Define a group of subcommands to be run directly,
/// or converted as a whole into a higher-order command
pub struct Commander<'a, S: ?Sized, T: ?Sized> {
    opts: Option<Box<dyn for<'x, 'y> Fn(App<'x, 'y>) -> App<'x, 'y> + 'a>>,
    args: Box<dyn for<'x> Fn(&'x S, &'x ArgMatches<'_>) -> &'x T + 'a>,
    cmds: Vec<Box<dyn CommandLike<T> + 'a>>,
    no_cmd: Option<Box<dyn Fn(&T, &ArgMatches<'_>) -> Result + 'a>>,
}

impl<'a, S: ?Sized> Commander<'a, S, S> {
    pub fn new() -> Self {
        Self {
            opts: None,
            args: Box::new(|args, _matches| args),
            cmds: Vec::new(),
            no_cmd: None,
        }
    }
}

impl<'a, S: ?Sized, T: ?Sized> Commander<'a, S, T> {
    pub fn options(mut self, opts: impl for<'x, 'y> Fn(App<'x, 'y>) -> App<'x, 'y> + 'a) -> Self {
        self.opts = Some(Box::new(opts));
        self
    }

    pub fn args<U: ?Sized>(
        self,
        args: impl for<'x> Fn(&'x S, &'x ArgMatches<'_>) -> &'x U + 'a,
    ) -> Commander<'a, S, U> {
        Commander {
            opts: self.opts,
            args: Box::new(args),
            // All other settings are reset.
            cmds: Vec::new(),
            no_cmd: None,
        }
    }

    pub fn add_cmd(mut self, cmd: impl CommandLike<T> + 'a) -> Self {
        self.cmds.push(Box::new(cmd));
        self
    }

    pub fn no_cmd(mut self, no_cmd: impl Fn(&T, &ArgMatches<'_>) -> Result + 'a) -> Self {
        self.no_cmd = Some(Box::new(no_cmd));
        self
    }

    fn app(&self) -> App {
        let mut app = App::new(clap::crate_name!())
            .version(clap::crate_version!())
            .about(clap::crate_description!())
            .author(clap::crate_authors!());

        if let Some(opts) = &self.opts {
            app = opts(app);
        }

        self.cmds
            .iter()
            .fold(app, |app, cmd| app.subcommand(cmd.app()))
    }

    fn run_with_data(&self, args: &S, matches: &ArgMatches<'_>, help: &Help) -> Result {
        let args = (self.args)(args, matches);

        for cmd in &self.cmds {
            if let Some(matches) = matches.subcommand_matches(cmd.name()) {
                let help = help.cmds.get(cmd.name()).unwrap();
                return cmd.run(args, matches, help);
            }
        }

        if let Some(no_cmd) = &self.no_cmd {
            no_cmd(args, matches)
        } else {
            let mut buf = Vec::new();

            self.write_help(&help, &[], &mut buf);

            Err(ClapError::with_description(
                &String::from_utf8(buf).unwrap(),
                ClapErrorKind::HelpDisplayed,
            ))
        }
    }

    fn write_help(&self, mut help: &Help, path: &[&str], out: &mut impl Write) {
        for &segment in path {
            match help.cmds.get(segment) {
                Some(inner) => help = inner,
                None => unreachable!("Bad help structure (doesn't match with path)"),
            }
        }

        out.write(&help.data).unwrap();
    }

    pub fn into_cmd(self, name: &'a str) -> MultiCommand<'a, S, T> {
        MultiCommand {
            name,
            desc: None,
            cmd: self,
        }
    }
}

impl<'a, T: ?Sized> Commander<'a, (), T> {
    pub fn run(&self) -> Result {
        self.run_with_args(std::env::args_os())
    }

    pub fn run_with_args(
        &self,
        args: impl IntoIterator<Item = impl Into<OsString> + Clone>,
    ) -> Result {
        let mut args = args.into_iter().peekable();
        let mut app = self.app();

        // Infer binary name
        if let Some(name) = args.peek() {
            let name = name.clone().into();
            let path = std::path::Path::new(&name);

            if let Some(filename) = path.file_name() {
                if let Some(binary_name) = filename.to_os_string().to_str() {
                    if app.p.meta.bin_name.is_none() {
                        app.p.meta.bin_name = Some(binary_name.to_owned());
                    }
                }
            }
        }

        fn propagate_author<'a>(app: &mut App<'_, 'a>, author: &'a str) {
            app.p.meta.author = Some(author);

            for subcmd in &mut app.p.subcommands {
                propagate_author(subcmd, author);
            }
        }

        let mut tmp = Vec::new();
        // This hack is used to propagate all needed information to subcommands.
        app.p.set(AppSettings::GlobalVersion);
        app.p.gen_completions_to(clap::Shell::Bash, &mut tmp);

        // Also propagate author to subcommands since `clap` doesn't do it
        if let Some(author) = app.p.meta.author {
            propagate_author(&mut app, author);
        }

        let help = Help::from(&app);

        match app.get_matches_from_safe(args) {
            Ok(matches) => self.run_with_data(&(), &matches, &help),
            Err(err) => match err.kind {
                clap::ErrorKind::HelpDisplayed | clap::ErrorKind::VersionDisplayed => Err(err),
                _ => {
                    let mut msg = err.message;
                    let mut buf = Vec::new();
                    let mut help_captured = false;

                    if let Some(index) = msg.find("\nUSAGE") {
                        let usage = msg.split_off(index);
                        let mut lines = usage.lines();

                        buf.extend_from_slice(msg.as_bytes());
                        buf.push('\n' as u8);

                        lines.next();
                        lines.next();

                        if let Some(usage) = lines.next() {
                            let mut usage = usage.to_owned();

                            if let Some(index) = usage.find("[") {
                                usage.truncate(index);
                            }

                            let mut path: Vec<_> = usage.split_whitespace().collect();

                            if path.len() > 0 {
                                path.remove(0);
                                self.write_help(&help, &path, &mut buf);
                                help_captured = true;
                            }
                        }
                    }

                    if help_captured {
                        Err(ClapError::with_description(
                            &String::from_utf8(buf).unwrap(),
                            ClapErrorKind::HelpDisplayed,
                        ))
                    } else {
                        unreachable!("The help message from clap is missing a usage section.");
                    }
                }
            },
        }
    }
}

/// The result of converting a [`Commander`](struct.Commander.html)
/// into a higher-order command
pub struct MultiCommand<'a, S: ?Sized, T: ?Sized> {
    name: &'a str,
    desc: Option<&'a str>,
    cmd: Commander<'a, S, T>,
}

impl<'a, S: ?Sized, T: ?Sized> MultiCommand<'a, S, T> {
    pub fn description(mut self, desc: impl Into<&'a str>) -> Self {
        self.desc = Some(desc.into());
        self
    }
}

impl<'a, S: ?Sized, T: ?Sized> CommandLike<S> for MultiCommand<'a, S, T> {
    fn name(&self) -> &str {
        self.name
    }

    fn app(&self) -> App {
        let mut app = self.cmd.app().name(self.name);

        if let Some(desc) = self.desc {
            app = app.about(desc);
        }

        app
    }

    fn run(&self, args: &S, matches: &ArgMatches<'_>, help: &Help) -> Result {
        self.cmd.run_with_data(args, matches, help)
    }
}

#[doc(hidden)]
pub struct Help {
    data: Vec<u8>,
    cmds: HashMap<String, Help>,
}

impl Help {
    fn from(app: &App) -> Self {
        let mut data = Vec::new();
        let mut cmds = HashMap::new();

        app.write_help(&mut data).unwrap();

        for app in &app.p.subcommands {
            cmds.insert(app.p.meta.name.clone(), Self::from(app));
        }

        Self { data, cmds }
    }
}
