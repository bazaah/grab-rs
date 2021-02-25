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
pub mod nom {
    pub type IResult<I, O, E = NomError<I>> = Result<(I, O), nom::Err<E>>;
    pub type NomError<I> = nom::error::VerboseError<I>;

    pub use nom::Finish;

    pub use nom::bytes::complete::tag;

    pub use nom::combinator::{all_consuming, into, map, value};

    pub use nom::branch::alt;

    pub use nom::error::{context, ParseError};
}
