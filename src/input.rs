use std::{convert::TryFrom, fmt, io, str::FromStr};

use crate::{
    builder::{Builder, Config},
    error::{access::AccessError, input::InputError},
    parsers::InputType,
};

/// Represents some kind of input source which can be read from.
#[derive(Debug)]
pub struct Input {
    kind: InputType,
}

impl Input {
    /// This structure cannot be directly created, instead you may create and configure a builder
    /// which can then be used to generate Input.
    ///
    /// Alternatively, you can use [Config::with_defaults] to parse some input using the default [Config].
    pub fn builder() -> Builder {
        Builder::default()
    }

    /// Attempt to locate an input source with the default configuration from the given input
    pub fn with_defaults(input: impl AsRef<str>) -> Result<Self, InputError> {
        Config::default().parse(input.as_ref())
    }

    /// Attempt to access the input source. Note that this function may block, depending on the
    /// what underlying input source is.
    pub fn access(&self) -> Result<InputReader, AccessError> {
        Read::try_from(&self.kind).map(InputReader::new)
    }

    pub(crate) fn from_input_type(i: InputType) -> Self {
        Self { kind: i }
    }
}

impl FromStr for Input {
    type Err = InputError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::with_defaults(s)
    }
}

/// An opaque handle that implements std::io::Read
#[derive(Debug)]
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
    File(std::fs::File),
    Stdin(std::io::Stdin),
    Text(io::Cursor<String>),
}

impl Read {
    fn stdin() -> Self {
        Self::Stdin(io::stdin())
    }

    fn file(f: std::fs::File) -> Self {
        Self::File(f)
    }

    fn text(s: impl AsRef<str>) -> Self {
        let s = s.as_ref().to_string();

        Self::Text(io::Cursor::new(s))
    }
}

impl TryFrom<&InputType> for Read {
    type Error = AccessError;

    fn try_from(kind: &InputType) -> Result<Self, Self::Error> {
        match kind {
            InputType::Stdin => Ok(Read::stdin()),
            InputType::File(ref f) => std::fs::File::open(f.path.as_path())
                .map(Read::file)
                .map_err(|e| AccessError::file_with_context(e, f.path.as_path())),
            InputType::UTF8(ref s) => Ok(Self::text(s)),
        }
    }
}

impl io::Read for Read {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        use Read::*;
        match self {
            File(ref mut file) => io::Read::read(file, buf),
            Stdin(ref mut stdin) => io::Read::read(stdin, buf),
            Text(ref mut cursor) => io::Read::read(cursor, buf),
        }
    }
}

impl fmt::Debug for Read {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Read::*;
        let mut dbg = f.debug_struct("Read");

        match self {
            File(f) => dbg.field("file", &f),
            Stdin(s) => dbg.field("stdin", &s),
            Text(t) => dbg.field("cursor", &t),
        };

        dbg.finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn input_from_default() {
        let input = "@/some/file/path";
        let res = Input::with_defaults(input);

        assert!(res.is_ok())
    }

    #[test]
    fn input_from_str() {
        let input = "some text";
        let res = Input::from_str(input);

        assert!(res.is_ok())
    }

    #[test]
    fn input_reader() {
        use std::io::Read;

        let input = "some random text";
        let mut output = String::new();
        let i = Input::with_defaults(input).unwrap();

        i.access().unwrap().read_to_string(&mut output).unwrap();

        assert_eq!(input, output.as_str())
    }
}
