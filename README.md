# clap-nested

[![Cargo Crate](https://img.shields.io/crates/v/clap-nested.svg)](https://crates.io/crates/clap-nested)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Build Status](https://travis-ci.com/skymavis/clap-nested.svg?branch=master)](https://travis-ci.com/skymavis/clap-nested)

A convenient [`clap`][clap] setup for multi-level CLI subcommands.

* [How to install?](#installation)
* [Why `clap-nested` exists?](#why)
* [How to use?](#usage)

## Installation

Add `clap-nested` to your `Cargo.toml`:

```toml
[dependencies]
clap-nested = "0.1.1"
```

## Why?

First of all, [`clap`][clap] is awesome!

It provides a fast, simple-to-use, and full-featured library for parsing CLI
arguments as well as subcommands.

However, while supporting parsing nicely, [`clap`][clap] is very unopinionated
when it comes to how we should structure and execute logic given provided
arguments and subcommands.

That's why we often find ourselves matching [`clap`][clap]'s parsing result with
tens of subcommands, let alone a lot of arguments, in our CLI application which
includes multi-level subcommands. The bad experience also escalates quickly,
imagine suddenly we have a lot of subcommand logic grouped under a very long
file.

So, we add a little sauce of opinion into [`clap`][clap] to help with that
awkward process.

## Usage

With `clap-nested`, we can write it in a more organized way:

```rust
// foo.rs
pub fn cmd<'a>() -> Command<'a, str> {
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
            Ok(())
        })
}

// bar.rs
pub fn cmd<'a>() -> Command<'a, str> {
    Command::new(file_stem!())
        .description("Shows bar")
        .runner(|args, _matches| {
            println!("Running bar, env = {}", args);
            Ok(())
        })
}

// main.rs
fn main() {
    Commander::new()
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
        .add_cmd(foo::cmd())
        .add_cmd(bar::cmd())
        .no_cmd(|_args, _matches| {
            println!("No subcommand matched.");
            Ok(())
        })
        .run();
}
```

Kindly see [`examples/clap_nested/`](examples/clap_nested/)
and [`examples/clap.rs`](examples/clap.rs) for comparison.

[clap]: https://github.com/clap-rs/clap
