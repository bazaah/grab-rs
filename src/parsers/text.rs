use super::{
    nom::{self, Finish},
    EKind, InputError, InputType, NomError, Parser,
};

use std::fmt;

pub type TextParser = for<'a, 'b> fn(&'a str, &'b str) -> nom::IResult<&'a str, String>;

#[derive(Clone, Default)]
pub struct Text {
    marker: Option<String>,
    parser: Option<TextParser>,
}

impl Text {
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

    fn get_marker(&self) -> &str {
        self.marker.as_deref().unwrap_or(Self::DEFAULT_MARKER)
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
    fn parse_str<'a>(&self, s: &'a str) -> Result<InputType, InputError> {
        self.parse(s)
            .map(|s| InputType::UTF8(s))
            .map_err(|e| self.new_error(e))
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
    if marker == "" {
        Ok(("", input.to_string()))
    } else {
        nom::context("TEXT", nom::map(nom::tag(marker), |s| String::from(s)))(input)
    }
}
