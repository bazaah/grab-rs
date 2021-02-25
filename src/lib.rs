#![warn(missing_docs)]
//! This library contains types for supercharging and streamlining grabbing input from the
//! command line. Gone are the days needing to messily handle whether the user wants you to read
//! their input from stdin, or a file, or directly from the argument. Welcome to the future.
//!
//! But as they say, _"An example is worth a thousand words"_. Consider then, the following super
//! simple rust CLI program:
//!
//! ```
//! use structopt::StructOpt;
//! use grab::Input;
//!
//! # type Result = std::result::Result<(), Box<dyn std::error::Error>>;
//!
//! #[derive(StructOpt)]
//! struct HelloCLI {
//!     /// The name we're going to greet
//!     name: Input
//! }
//!
//! # impl HelloCLI {
//! #   fn from_args() -> Self {
//! #       Self::from_iter(vec!["hello", "John"])
//! #   }
//! # }
//!
//! # fn main() -> Result {
//!     // Parse our argument(s)
//!     let args = HelloCLI::from_args();
//!
//!     // Access the user's input, reading it to a string
//!     let name = args.name.access()?.read_to_string()?;
//!
//!     // Say hello!
//!     println!("Hello, {}!", &name);
//! #   Ok(())
//! # }
//! ```
//!
//! Just by using [grab::Input] we can now respond to user input three ways:
//!
//! ```bash
//! # The basics, read the argument directly from the command line
//! $ hello John
//! Hello, John!
//!
//! # Read from stdin... in unixy fashion!
//! $ echo Bob | hello -
//! Hello, Bob!
//!
//! # Or even from a file!
//! $ echo Fred >name.txt; hello @name.txt
//! Hello, Fred!
//! ```
//!
//! Couldn't be simpler right?
//!
//! "Okay, okay" you say, "this is great and all... but I want my CLI's users to refer to stdin as
//! '<--' and files as '...'! Anything less just won't do. So thanks but no th...". Whoa, whoa! No
//! problem (you psychopath) we can accommodate your (insane) needs. Simply modify the parser
//! configuration to suit your needs!
//!
//! ```
//! # use structopt::StructOpt;
//! # use std::str::FromStr;
//! use grab::{Input, Builder, parsers::{Stdin, File}, error::input::InputError};
//!
//! // Build our custom stdin parser
//! fn my_stdin() -> Stdin {
//!     Stdin::new().with(|this| this.marker("<--"))
//! }
//!
//! // And our custom file parser
//! fn my_file() -> File {
//!     File::new().with(|this| this.marker("..."))
//! }
//!
//! // Then we define our newtype wrapper and implement FromStr
//! struct MyCustomParser(Input);
//!
//! impl FromStr for MyCustomParser {
//!     type Err = InputError;
//!
//!     fn from_str(s: &str) -> Result<Self, Self::Err> {
//!         let cfg = Builder::new().with(|this| {
//!             this
//!                 .text()
//!                 .with_stdin(my_stdin())
//!                 .with_file(my_file())
//!         });
//!
//!         let input = cfg.build().parse(s)?;
//!
//!         Ok(Self(input))
//!     }
//! }
//!
//! // And use it in our CLI!
//! #[derive(StructOpt)]
//! struct MyCLI {
//!     user_input: MyCustomParser
//! }
//! ```
//!
//! There we have it. A custom parser which you can use however you like (you monster)!

mod builder;
mod input;

pub mod error;
pub mod parsers;

pub use input::{Input, InputReader};

pub use builder::{Builder, Config};
