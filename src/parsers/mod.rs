mod file;
mod stdin;
mod text;

use std::ffi::OsStr;

use crate::error::input::{EKind, InputError};

use self::nom::NomError;

pub use {file::File, stdin::Stdin, text::Text};

pub(crate) trait Parser {
    fn parse_str(&self, input: &str) -> Result<InputType, InputError>;

    fn parse_os_str(&self, input: &OsStr) -> Result<InputType, InputError> {
        let input = input.to_str().ok_or_else(|| EKind::REQUIRES_UTF8)?;

        self.parse_str(input)
    }

    fn parse_bytes(&self, input: &[u8]) -> Result<InputType, InputError> {
        let input = std::str::from_utf8(input).map_err(|_| EKind::REQUIRES_UTF8)?;

        self.parse_str(input)
    }
}

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
