use super::{
    nom::{self, Finish},
    EKind, InputError, InputType, NomError, Parser, Weight, WeightedParser,
};

use std::fmt;

/// Function signature of the parser Stdin calls for processing input
pub type StdinParser = for<'a, 'b> fn(&'a str, &'b str) -> nom::IResult<&'a str, ()>;

/// A construct for handling the parsing of a given input string and determining
/// if the program's stdin should be called in leu of. By default, it will only
/// indicate stdin should be used if the given input is a single dash ('-'),
/// with no other input.
#[derive(Clone, Default)]
pub struct Stdin {
    marker: Option<String>,
    parser: Option<StdinParser>,
    weight: Option<u8>,
}

impl Stdin {
    pub const DEFAULT_WEIGHT: u8 = 140;
    pub const DEFAULT_MARKER: &'static str = "-";
    pub const DEFAULT_PARSER: StdinParser = default_stdin_parser;

    /// Instantiate a new Stdin parser with sensible defaults
    pub fn new() -> Self {
        Self::default()
    }

    /// Convenience function for modifying the semantics of
    /// this parser
    ///
    /// Example:
    ///
    /// ```
    /// use grab::parsers::Stdin;
    ///
    /// // Use a 'curl --data' styled stdin
    /// let stdin = Stdin::new().with(|this| this.marker("@-"));
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

    /// Modify the marker string for triggering this Stdin parser.
    /// This marker is passed to the parser function as the second &str
    /// argument.
    pub fn marker(&mut self, marker: impl AsRef<str>) -> &mut Self {
        self.marker = Some(marker.as_ref().to_string());

        self
    }

    /// Replace the parser for this Stdin with a different one. Expects a
    /// _function_ (not closure) with the following arguments + return:
    ///
    /// fn my_parser<'a, 'b>(input: &'a str, marker: &'b str) -> crate::nom::IResult<&'a str, ()>
    /// {
    ///     /* ... */
    /// }
    pub fn parser(&mut self, parser: StdinParser) -> &mut Self {
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

    fn parse<'a>(&self, input: &'a str) -> Result<(), NomError<&'a str>> {
        let marker = self.get_marker();

        self.parser
            .map(|p| p(input, marker))
            .unwrap_or_else(|| Self::DEFAULT_PARSER(input, marker))
            .finish()?;

        Ok(())
    }

    fn new_error(&self, _p_error: NomError<&str>) -> InputError {
        InputError::new(EKind::STDIN)
    }
}

impl Parser for Stdin {
    fn parse_str(&self, s: &str) -> Result<InputType, InputError> {
        self.parse(s)
            .map(|_| InputType::Stdin)
            .map_err(|e| self.new_error(e))
    }
}

impl Weight for Stdin {
    fn weight(&self) -> u8 {
        self.get_weight()
    }
}

impl WeightedParser for Stdin {}

impl fmt::Debug for Stdin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Stdin")
            .field("marker", &self.get_marker())
            .field(
                "parser",
                &self
                    .parser
                    .map_or("Default StdinParser", |_| "Custom StdinParser"),
            )
            .finish()
    }
}

/// The default parser implementation for reading from stdin. It will only trigger on
/// a singular '-', in the style of kubectl, e.g kubectl apply -f -
pub fn default_stdin_parser<'a, 'b>(input: &'a str, marker: &'b str) -> nom::IResult<&'a str, ()> {
    let child = nom::context("STDIN", nom::all_consuming(nom::tag(marker)));

    nom::value((), child)(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    const BAD_INPUT: &'static str = "invalid stdin input";

    #[test]
    fn defaults_success() {
        let input = Stdin::DEFAULT_MARKER;

        let parser = Stdin::new();

        let result = parser.parse_str(input);

        assert_eq!(result, Ok(InputType::Stdin))
    }

    #[test]
    fn defaults_failure() {
        let input = BAD_INPUT;

        let parser = Stdin::new();

        let result = parser.parse_str(input);

        assert_eq!(result, Err(EKind::STDIN.into()))
    }

    #[test]
    fn c_marker_success() {
        let mkr = "@-";

        let input = mkr;

        let parser = Stdin::new().with(|this| this.marker(mkr));

        let result = parser.parse_str(input);

        assert_eq!(result, Ok(InputType::Stdin))
    }

    #[test]
    fn c_marker_failure() {
        let mkr = "@-";

        let input = BAD_INPUT;

        let parser = Stdin::new().with(|this| this.marker(mkr));

        let result = parser.parse_str(input);

        assert_eq!(result, Err(EKind::STDIN.into()))
    }

    #[test]
    fn c_parser_success() {
        let input = "- extra stuff";

        let parser = Stdin::new().with(|this| this.parser(test_custom_parser));

        let result = parser.parse_str(input);

        assert_eq!(result, Ok(InputType::Stdin))
    }

    #[test]
    fn c_parser_failure() {
        let input = "extra stuff -";

        let parser = Stdin::new().with(|this| this.parser(test_custom_parser));

        let result = parser.parse_str(input);

        assert_eq!(result, Err(EKind::STDIN.into()))
    }

    fn test_custom_parser<'a, 'b>(input: &'a str, marker: &'b str) -> nom::IResult<&'a str, ()> {
        let child = nom::context("STDIN", nom::tag(marker));

        nom::value((), child)(input)
    }
}
