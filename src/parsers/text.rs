use super::{
    nom::{self, Finish},
    EKind, InputError, InputType, NomError, Parser, Weight, WeightedParser,
};

use std::fmt;

pub type TextParser = for<'a, 'b> fn(&'a str, &'b str) -> nom::IResult<&'a str, String>;

/// Construct for treating the given input to parse as a readable input source. By default, this
/// parser will consume any valid utf8 strings and return it as an input source. Consequently, this
/// parser by default has the lowest possible priority so it will always be the last parser run.
#[derive(Clone, Default)]
pub struct Text {
    marker: Option<String>,
    parser: Option<TextParser>,
    weight: Option<u8>,
}

impl Text {
    pub const DEFAULT_WEIGHT: u8 = 255;
    pub const DEFAULT_MARKER: &'static str = "";
    pub const DEFAULT_PARSER: TextParser = default_text_parser;

    /// Instantiate a new Text parser with sensible defaults
    pub fn new() -> Self {
        Self::default()
    }

    /// Convenience function for modifying the semantics of
    /// this parser
    ///
    /// Example:
    ///
    /// ```
    /// use grab::parsers::Text;
    ///
    /// // Require the text to be prefaced with '###'
    /// let Text = Text::new().with(|this| this.marker("###"));
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

    /// Modify the marker string for triggering this Text parser.
    /// This marker is passed to the parser function as the second &str
    /// argument.
    pub fn marker(&mut self, marker: impl AsRef<str>) -> &mut Self {
        self.marker = Some(marker.as_ref().to_string());

        self
    }

    /// Replace the parser for this File with a different one. Expects a
    /// _function_ (not closure) with the following arguments + return:
    ///
    /// fn my_parser<'a, 'b>(input: &'a str, marker: &'b str) -> crate::nom::IResult<&'a str, String>
    /// {
    ///     /* ... */
    /// }
    pub fn parser(&mut self, parser: TextParser) -> &mut Self {
        self.parser = Some(parser);

        self
    }

    /// Set this parser's weight. Lower numbers will be ran before greater.
    pub fn weight(&mut self, weight: u8) -> &mut Self {
        self.weight = Some(weight);

        self
    }

    fn get_marker(&self) -> &str {
        self.marker.as_deref().unwrap_or(Self::DEFAULT_MARKER)
    }

    fn get_weight(&self) -> u8 {
        self.weight.unwrap_or(Self::DEFAULT_WEIGHT)
    }

    fn parse<'a>(&self, input: &'a str) -> Result<String, NomError<&'a str>> {
        let marker = self.get_marker();

        let (_, text) = self
            .parser
            .map(|p| p(input, marker))
            .unwrap_or_else(|| Self::DEFAULT_PARSER(input, marker))
            .finish()?;

        Ok(text)
    }

    fn new_error(&self, _p_error: NomError<&str>) -> InputError {
        InputError::new(EKind::TEXT)
    }
}

impl Parser for Text {
    fn parse_str(&self, s: &str) -> Result<InputType, InputError> {
        self.parse(s)
            .map(InputType::UTF8)
            .map_err(|e| self.new_error(e))
    }
}

impl Weight for Text {
    fn weight(&self) -> u8 {
        self.get_weight()
    }
}

impl fmt::Debug for Text {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Text")
            .field("marker", &self.get_marker())
            .field(
                "parser",
                &self
                    .parser
                    .map_or("Default TextParser", |_| "Custom TextParser"),
            )
            .finish()
    }
}

/// Default text parser, if the given marker is empty (i.e "") it returns
/// the entire input unmodified, otherwise it will return everything after
/// the given marker
pub fn default_text_parser<'a, 'b>(
    input: &'a str,
    marker: &'b str,
) -> nom::IResult<&'a str, String> {
    // If the marker is empty (the default) we just return everything
    if marker.is_empty() {
        Ok(("", input.to_string()))
    } else {
        nom::context("TEXT", nom::tag(marker))(input).map(|(path, _)| ("", String::from(path)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const INPUT: &'static str = "some arbitrary text";

    #[test]
    fn defaults_success() {
        let input = INPUT;
        let output = String::from(input);

        let parser = Text::new();

        let result = parser.parse_str(input);

        assert_eq!(result, Ok(InputType::UTF8(output)))
    }

    #[test]
    fn c_marker_success() {
        let mkr = "!!";

        let input = "!!valid text";
        let output = String::from("valid text");

        let parser = Text::new().with(|this| this.marker(mkr));

        let result = parser.parse_str(input);

        assert_eq!(result, Ok(InputType::UTF8(output)))
    }

    #[test]
    fn c_marker_failure() {
        let mkr = "!!";

        let input = "no marker";

        let parser = Text::new().with(|this| this.marker(mkr));

        let result = parser.parse_str(input);

        assert_eq!(result, Err(EKind::TEXT.into()))
    }

    #[test]
    fn c_parser_success() {
        let input = INPUT;
        let output = String::from(input);

        let parser = Text::new().with(|this| this.parser(test_custom_parser));

        let result = parser.parse_str(input);

        assert_eq!(result, Ok(InputType::UTF8(output)))
    }

    #[test]
    fn c_parser_failure() {
        let input = "";

        let parser = Text::new().with(|this| this.parser(test_custom_parser));

        let result = parser.parse_str(input);

        assert_eq!(result, Err(EKind::TEXT.into()))
    }

    fn test_custom_parser<'a, 'b>(
        input: &'a str,
        marker: &'b str,
    ) -> nom::IResult<&'a str, String> {
        use ::nom::error::{make_error, ErrorKind};
        if input.is_empty() {
            Err(::nom::Err::Error(make_error(input, ErrorKind::NonEmpty)))
        } else {
            nom::context("TEXT", nom::tag(marker))(input).map(|(path, _)| ("", String::from(path)))
        }
    }
}
