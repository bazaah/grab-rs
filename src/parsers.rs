use std::{convert::TryFrom, path::PathBuf};

use {self::nom::*, crate::error::InputParseError};

#[derive(Debug, Clone)]
pub(crate) enum InputType {
    Stdin(Stdin),
    File(File),
}

impl InputType {
    fn try_parse(s: impl AsRef<str>) -> Result<Self, InputParseError> {
        coalesce(s.as_ref())
    }
}

impl TryFrom<&str> for InputType {
    type Error = InputParseError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        InputType::try_parse(value)
    }
}

fn coalesce(i: &str) -> Result<InputType, InputParseError> {
    let stdin = map(stdin, |s| InputType::Stdin(s));
    let file = map(file, |f| InputType::File(f));

    let (_, input) = alt((stdin, file))(i).finish()?;

    Ok(input)
}

fn file(i: &str) -> IResult<&str, File> {
    let (path, _) = context("STDIN", tag("@"))(i)?;

    Ok(("", File::new(path)))
}

fn stdin(i: &str) -> IResult<&str, Stdin> {
    let child = context("FILE", all_consuming(tag("-")));

    value(Stdin, child)(i)
}

#[derive(Debug, Clone)]
pub(crate) struct Stdin;

#[derive(Debug, Clone)]
pub(crate) struct File {
    pub path: PathBuf,
}

impl File {
    fn new(s: impl AsRef<str>) -> Self {
        let path = PathBuf::from(s.as_ref());

        Self { path }
    }
}

// Reexport nom parsers in a manner that doesn't
// make me want to shoot myself.
mod nom {
    pub type IResult<I, O, E = nom::error::VerboseError<I>> = Result<(I, O), nom::Err<E>>;

    pub use nom::Finish as _;

    pub use nom::bytes::complete::tag;

    pub use nom::combinator::{all_consuming, into, map, value};

    pub use nom::branch::alt;

    pub use nom::error::context;
}
