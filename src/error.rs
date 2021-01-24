use nom::error as nom;
use std::{error::Error as StdError, fmt};

#[derive(Debug, Clone)]
pub struct InputParseError {
    message: String,
    errors: Vec<nom::VerboseErrorKind>,
}
impl InputParseError {
    fn from_nom_verbose(error: nom::VerboseError<&str>) -> Self {
        use std::fmt::Write;
        let mut message = String::new();
        write!(&mut message, "{}", &error).expect("Writing to a string should never fail");

        let errors = error.errors.into_iter().map(|(_, k)| k).collect();

        Self { message, errors }
    }

    fn from_nom(error: nom::Error<&str>) -> Self {
        use std::fmt::Write;
        let mut message = String::new();
        write!(&mut message, "{}", &error).expect("Writing to a string should never fail");

        Self {
            message,
            errors: Vec::new(),
        }
    }
}

impl From<nom::VerboseError<&str>> for InputParseError {
    fn from(error: nom::VerboseError<&str>) -> Self {
        Self::from_nom_verbose(error)
    }
}

impl From<nom::Error<&str>> for InputParseError {
    fn from(error: nom::Error<&str>) -> Self {
        Self::from_nom(error)
    }
}

impl fmt::Display for InputParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}", self.message)
    }
}

impl StdError for InputParseError {}
