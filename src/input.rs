use std::{convert::TryFrom, fmt, io, str::FromStr};

use crate::{error::InputParseError, parsers::InputType};

#[derive(Debug)]
pub struct Input {
    kind: InputType,
}

impl Input {
    pub fn try_parse(s: impl AsRef<str>) -> Result<Self, InputParseError> {
        let kind = InputType::try_from(s.as_ref())?;

        Ok(Self { kind })
    }

    pub fn access(&self) -> Result<InputReader, AccessError> {
        Read::try_from(&self.kind).map(|r| InputReader::new(r))
    }
}

impl FromStr for Input {
    type Err = InputParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_parse(s)
    }
}

pub struct InputReader {
    input: Read,
}

impl InputReader {
    fn new(input: Read) -> Self {
        Self { input }
    }
}

impl io::Read for InputReader {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        io::Read::read(&mut self.input, buf)
    }
}

enum Read {
    F(std::fs::File),
    S(std::io::Stdin),
}

impl Read {
    fn stdin() -> Self {
        Self::S(io::stdin())
    }

    fn file(f: std::fs::File) -> Self {
        Self::F(f)
    }
}

impl TryFrom<&InputType> for Read {
    type Error = AccessError;

    fn try_from(kind: &InputType) -> Result<Self, Self::Error> {
        match kind {
            InputType::Stdin(_) => Ok(Read::stdin()),
            InputType::File(ref f) => std::fs::File::open(f.path.as_path())
                .map(|f| Read::file(f))
                .map_err(|_e| AccessError::default()),
        }
    }
}

impl io::Read for Read {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        use Read::*;
        match self {
            F(ref mut file) => io::Read::read(file, buf),
            S(ref mut stdin) => io::Read::read(stdin, buf),
        }
    }
}

#[derive(Debug, Default)]
pub struct AccessError;

impl fmt::Display for AccessError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "hey idiot maintainer, you should probably fix this error"
        )
    }
}

impl std::error::Error for AccessError {}
