//! This module contains the individual parsers that the high level API uses for actually
//! processing raw input into a well understood [Input][crate::Input]. Under the hood, each of the
//! parsers use [nom](https://github.com/Geal/nom) combinators for driving the parsing. See the
//! exposed parsers themselves for details on how each works.
//!
//! The module does also provide access to the underlying nom library, though it is hidden to avoid
//! cluttering up this crates documentation. You can use it via:
//!
//! ```
//! use grab::parsers::reexport::nom;
//! ```

mod file;
mod stdin;
mod text;

use std::ffi::OsStr;

use crate::error::input::{EKind, InputError};

use self::nom::NomError;

pub use {file::File, stdin::Stdin, text::Text};

/// Private trait that describes the conversion of some input into a reference to some kind of
/// input type.
pub(crate) trait Parser {
    fn parse_str(&self, input: &str) -> Result<InputType, InputError>;

    fn parse_os_str(&self, input: &OsStr) -> Result<InputType, InputError> {
        let input = input.to_str().ok_or(EKind::REQUIRES_UTF8)?;

        self.parse_str(input)
    }

    fn parse_bytes(&self, input: &[u8]) -> Result<InputType, InputError> {
        let input = std::str::from_utf8(input).map_err(|_| EKind::REQUIRES_UTF8)?;

        self.parse_str(input)
    }
}

/// Describes the expected priority (or weight) of a parser. Used for deterministically sorting a
/// series of parsers, allowing higher priority parsers an attempt before lower priority ones.
pub(crate) trait Weight {
    fn weight(&self) -> u8;
}

/// Glue trait for creating trait objects with both Parser and Weight methods
pub(crate) trait WeightedParser: Parser + Weight {}

impl<T> WeightedParser for T where T: Parser + Weight {}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum InputType {
    Stdin,
    File(file::FilePath),
    UTF8(String),
}

// Reexport nom parsers in a manner that doesn't
// make me want to shoot myself.
mod nom {
    pub type IResult<I, O, E = NomError<I>> = Result<(I, O), nom::Err<E>>;
    pub type NomError<I> = nom::error::Error<I>;

    pub use nom::Finish;

    pub use nom::bytes::complete::tag;

    pub use nom::combinator::{all_consuming, into, map, value};

    pub use nom::branch::alt;

    pub use nom::error::{context, ParseError};
}

/// This is hidden by default to avoid cluttering this crate's docs. If you want to create custom
/// parser functions however, you can use this to ensure your version of nom's is the same as this
/// crates.
#[doc(hidden)]
pub mod reexport {
    pub use nom;
}
