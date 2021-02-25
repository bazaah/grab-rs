use super::{
    nom::{self, Finish},
    EKind, InputError, InputType, NomError, Parser, Weight,
};
use std::{fmt, path::PathBuf};

pub type FileParser = for<'a, 'b> fn(&'a str, &'b str) -> nom::IResult<&'a str, PathBuf>;

/// Parser
#[derive(Clone, Default)]
pub struct File {
    marker: Option<String>,
    parser: Option<FileParser>,
    weight: Option<u8>,
}

impl File {
    pub const DEFAULT_WEIGHT: u8 = 130;
    pub const DEFAULT_MARKER: &'static str = "@";
    pub const DEFAULT_PARSER: FileParser = default_file_parser;

    /// Instantiate a new File parser with sensible defaults
    pub fn new() -> Self {
        Self::default()
    }

    /// Convenience function for modifying the semantics of
    /// this parser
    ///
    /// Example:
    ///
    /// ```
    /// use grab::parsers::File;
    ///
    /// // Use a URI compliant file parser
    /// let file = File::new().with(|this| this.marker("file://"));
    /// ```
    pub fn with<F>(self, f: F) -> Self
    where
        F: FnMut(&mut Self) -> &mut Self,
    {
        let mut this = self;
        let mut actions = f;

        actions(&mut this);

        this
    }

    /// Modify the marker string for triggering this File parser.
    /// This marker is passed to the parser function as the second &str
    /// argument.
    pub fn marker(&mut self, marker: impl AsRef<str>) -> &mut Self {
        self.marker = Some(marker.as_ref().to_string());

        self
    }

    /// Replace the parser for this File with a different one. Expects a
    /// _function_ (not closure) with the following arguments + return:
    ///
    /// fn my_parser<'a, 'b>(input: &'a str, marker: &'b str) -> crate::nom::IResult<&'a str, PathBuf>
    /// {
    ///     /* ... */
    /// }
    pub fn parser(&mut self, parser: FileParser) -> &mut Self {
        self.parser = Some(parser);

        self
    }

    /// Set this parser's weight. Lower numbers will be ran before greater.
    pub fn weight(&mut self, weight: u8) -> &mut Self {
        self.weight = Some(weight);

        self
    }

    fn get_weight(&self) -> u8 {
        self.weight.unwrap_or(Self::DEFAULT_WEIGHT)
    }

    fn get_marker(&self) -> &str {
        self.marker.as_deref().unwrap_or(Self::DEFAULT_MARKER)
    }

    fn parse<'a>(&self, input: &'a str) -> Result<FilePath, NomError<&'a str>> {
        let marker = self.get_marker();

        let (_, path) = self
            .parser
            .map(|p| p(input, marker))
            .unwrap_or_else(|| Self::DEFAULT_PARSER(input, marker))
            .finish()?;

        Ok(FilePath::new(path))
    }

    // TODO: Allow potentially passing contextual data to InputErrors
    fn new_error(&self, _p_error: NomError<&str>) -> InputError {
        InputError::new(EKind::FILE)
    }
}

impl Parser for File {
    fn parse_str(&self, s: &str) -> Result<InputType, InputError> {
        self.parse(s)
            .map(InputType::File)
            .map_err(|e| self.new_error(e))
    }
}

impl Weight for File {
    fn weight(&self) -> u8 {
        self.get_weight()
    }
}

impl fmt::Debug for File {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("File")
            .field("marker", &self.get_marker())
            .field(
                "parser",
                &self
                    .parser
                    .map_or("Default FileParser", |_| "Custom FileParser"),
            )
            .finish()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct FilePath {
    pub path: PathBuf,
}

impl FilePath {
    fn new(path: PathBuf) -> Self {
        Self { path }
    }
}

/// Default parser for files. It expects input starting with the 'marker' and
/// takes the rest of the input as a file path.
pub fn default_file_parser<'a, 'b>(
    input: &'a str,
    marker: &'b str,
) -> nom::IResult<&'a str, PathBuf> {
    nom::context("FILE", nom::tag(marker))(input).map(|(path, _)| ("", PathBuf::from(path)))
}

#[cfg(test)]
mod tests {
    use super::*;

    const BAD_INPUT: &'static str = "invalid file input";

    #[test]
    fn defaults_success() {
        let input = "@some/file/here";
        let output = FilePath::new(PathBuf::from("some/file/here"));

        let parser = File::new();

        let result = parser.parse_str(input);

        assert_eq!(result, Ok(InputType::File(output)))
    }

    #[test]
    fn defaults_failure() {
        let input = BAD_INPUT;

        let parser = File::new();

        let result = parser.parse_str(input);

        assert_eq!(result, Err(EKind::FILE.into()))
    }

    #[test]
    fn c_marker_success() {
        let mkr = "+";

        let input = "+/some/file/here";
        let output = FilePath::new(PathBuf::from("/some/file/here"));

        let parser = File::new().with(|this| this.marker(mkr));

        let result = parser.parse_str(input);

        assert_eq!(result, Ok(InputType::File(output)))
    }

    #[test]
    fn c_marker_failure() {
        let mkr = "+";

        let input = BAD_INPUT;

        let parser = File::new().with(|this| this.marker(mkr));

        let result = parser.parse_str(input);

        assert_eq!(result, Err(EKind::FILE.into()))
    }

    #[test]
    fn c_parser_success() {
        let input = "file://foo/bar/baz";
        let output = FilePath::new(PathBuf::from("foo/bar/baz"));

        let parser = File::new().with(|this| this.parser(test_custom_parser));

        let result = parser.parse_str(input);

        assert_eq!(result, Ok(InputType::File(output)))
    }

    #[test]
    fn c_parser_failure() {
        let input = "@foo/bar/baz";

        let parser = File::new().with(|this| this.parser(test_custom_parser));

        let result = parser.parse_str(input);

        assert_eq!(result, Err(EKind::FILE.into()))
    }

    fn test_custom_parser<'a, 'b>(input: &'a str, _: &'b str) -> nom::IResult<&'a str, PathBuf> {
        nom::context("FILE", nom::tag("file://"))(input).map(|(path, _)| ("", PathBuf::from(path)))
    }
}
