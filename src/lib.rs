use std::collections::HashMap;
use std::ffi::OsString;

extern crate clap;

use clap::{App, ArgMatches, SubCommand};

mod macros;

type Result = std::result::Result<(), Box<dyn std::error::Error + Send + Sync>>;

#[doc(hidden)]
pub trait CommandLike<T: ?Sized> {
    fn name(&self) -> &str;
    fn app(&self) -> App;
    fn run(&self, args: &T, matches: &ArgMatches<'_>, help: &Help) -> Result;
}

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
            self.eprintln_help(&help, &[]);
            Ok(())
        }
    }

    fn eprintln_help(&self, mut help: &Help, path: &[&str]) {
        use std::io::Write;

        for &segment in path {
            match help.cmds.get(segment) {
                Some(inner) => help = inner,
                None => unreachable!("Bad help structure (doesn't match with path)"),
            }
        }

        std::io::stderr().write_all(&help.data).unwrap();
        eprintln!();
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
        app.p.gen_completions_to(clap::Shell::Bash, &mut tmp);

        // Also propagate author to subcommands since `clap` doesn't do it
        if let Some(author) = app.p.meta.author {
            propagate_author(&mut app, author);
        }

        let help = Help::from(&app);

        match app.get_matches_from_safe(args) {
            Ok(matches) => self.run_with_data(&(), &matches, &help),
            Err(err) => match err.kind {
                clap::ErrorKind::HelpDisplayed | clap::ErrorKind::VersionDisplayed => err.exit(),
                _ => {
                    let mut msg = err.message;
                    let mut help_printed = false;

                    if let Some(index) = msg.find("\nUSAGE") {
                        let usage = msg.split_off(index);
                        let mut lines = usage.lines();

                        eprintln!("{}", msg);

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
                                self.eprintln_help(&help, &path);
                                help_printed = true;
                            }
                        }
                    }

                    if !help_printed {
                        unreachable!("The help message from clap is missing a usage section.");
                    }

                    Ok(())
                }
            },
        }
    }
}

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
