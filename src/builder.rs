//! This module contains [Config] and its sister [Builder] which are used for converting input
//! into a concrete [Input] kind which can be then be accessed and read from. The two main methods
//! for this conversion are [Config::parse] and [Config::parse_os], which take as some hopefully
//! parsable input, like say `@/some/file/path` for reading from a file or `-` for reading from
//! [Stdin](std::io::Stdin).

use crate::{
    error::input::InputError,
    input::Input,
    parsers::{File, InputType, Parser, Stdin, Text, WeightedParser as WP},
};

use std::{ffi::OsStr, fmt};

/// Represents a set of parsers that will be called in ascending order according to their weight
/// until the list is exhausted or a parser returns successfully.
///
/// Typically, you can construct one via a [Builder], however if you aren't interested in
/// customizing your parser config, you can also use [Config::default].
#[derive(Clone)]
pub struct Config {
    inner: Builder,
}

impl Config {
    /// Attempt to parse the input into a concrete handle which can be [accessed](Input::access)
    pub fn parse(&self, input: &str) -> Result<Input, InputError> {
        self.parse_str(input).map(Input::from_input_type)
    }

    /// Attempt to parse the given [OsStr] into a concrete handle which can be
    /// [accessed](Input::access).
    pub fn parse_os(&self, input: &OsStr) -> Result<Input, InputError> {
        self.parse_os_str(input).map(Input::from_input_type)
    }

    /// Generates a list of parsers from the available, sorts them by weight,
    /// then applies the given closure to the sorted list
    fn with_parsers<F, R>(&self, f: F) -> R
    where
        F: FnMut(&[Option<&dyn WP>]) -> R,
    {
        let b = &self.inner;
        let mut callback = f;

        let mut list = [
            b.file.as_ref().map(|p| p as &dyn WP),
            b.stdin.as_ref().map(|p| p as &dyn WP),
            b.text.as_ref().map(|p| p as &dyn WP),
        ];

        // Sort parsers by weight, with lower numbers taking
        // priority.
        list.sort_by_key(|opt| opt.map(|p| p.weight()));

        callback(&list)
    }

    /// Iterates over the given list of parsers, trying the given closure on each
    /// and returning the first success.
    ///
    /// Notably, this function _does not_ provide the input on which a parser
    /// operates, this should be pulled in by the closure.
    fn apply<'a, F, I>(&self, parsers: I, mut f: F) -> Result<InputType, InputError>
    where
        F: FnMut(&dyn WP) -> Result<InputType, InputError>,
        I: IntoIterator<Item = &'a dyn WP>,
    {
        let mut error: Option<InputError> = None;

        for parser in parsers {
            match f(parser) {
                Ok(success) => return Ok(success),
                Err(e) => match error {
                    Some(ref mut prev) => {
                        prev.extend(e);
                    }
                    None => error = Some(e),
                },
            }
        }

        Err(error.expect("Config should never have less than one parser, this is a bug"))
    }
}

impl Parser for Config {
    fn parse_str(&self, input: &str) -> Result<InputType, InputError> {
        self.with_parsers(|parsers| {
            let iter = parsers.iter().filter_map(|o| *o);
            self.apply(iter, |p| p.parse_str(input))
        })
    }

    fn parse_os_str(&self, input: &OsStr) -> Result<InputType, InputError> {
        self.with_parsers(|parsers| {
            let iter = parsers.iter().filter_map(|o| *o);
            self.apply(iter, |p| p.parse_os_str(input))
        })
    }

    fn parse_bytes(&self, input: &[u8]) -> Result<InputType, InputError> {
        self.with_parsers(|parsers| {
            let iter = parsers.iter().filter_map(|o| *o);
            self.apply(iter, |p| p.parse_bytes(input))
        })
    }
}

impl fmt::Debug for Config {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut dbg = f.debug_struct("Config");

        if let Some(text) = &self.inner.text {
            dbg.field("text", &text);
        }

        if let Some(stdin) = &self.inner.stdin {
            dbg.field("stdin", &stdin);
        }

        if let Some(file) = &self.inner.file {
            dbg.field("file", &file);
        }

        dbg.finish()
    }
}

impl Default for Config {
    fn default() -> Self {
        let cfg = Builder::new().with(|b| b.text().stdin().file());

        debug_assert!(cfg.is_valid());

        Self { inner: cfg }
    }
}

/// A [Config] builder, you can use this struct to customize which parsers are available to be called
/// when attempting to parse input.
///
/// If you just want the default configuration, use [Config::default] and skip this struct
/// completely.
#[derive(Debug, Clone, Default)]
pub struct Builder {
    stdin: Option<Stdin>,
    file: Option<File>,
    text: Option<Text>,
}

impl Builder {
    /// Create a new, empty config builder
    pub fn new() -> Self {
        Self::default()
    }

    /// Convenience function for applying configuration options
    pub fn with<F>(self, f: F) -> Self
    where
        F: FnMut(&mut Self) -> &mut Self,
    {
        let mut this = self;
        let mut actions = f;

        actions(&mut this);

        this
    }

    /// Consume this builder returning a [Config] that can be
    /// used for parsing.
    ///
    /// # Panics
    ///
    /// Panics if there are no parsers set
    pub fn build(self) -> Config {
        assert!(
            self.is_valid(),
            "A grab::Builder must contain at least one parser"
        );

        Config { inner: self }
    }

    /// Attempt to create a [Config] from the given parser,
    /// if the current configuration is valid, returning the
    /// builder otherwise.
    ///
    /// This is the safe variant of [build][Builder::build]
    pub fn try_build(self) -> Result<Config, Self> {
        if self.is_valid() {
            return Ok(Config { inner: self });
        }

        Err(self)
    }

    /// Enable [text](Text) parsing, with the default parser
    pub fn text(&mut self) -> &mut Self {
        self.with_text(Text::new())
    }

    /// Enable [text](Text) parsing, with the given parser
    pub fn with_text(&mut self, t: Text) -> &mut Self {
        self.text = Some(t);

        self
    }

    /// Enable [stdin](Stdin) parsing with the default parser
    pub fn stdin(&mut self) -> &mut Self {
        self.with_stdin(Stdin::new())
    }

    /// Enable [stdin](Stdin) parsing, using the given parser
    pub fn with_stdin(&mut self, s: Stdin) -> &mut Self {
        self.stdin = Some(s);

        self
    }

    /// Enable [file path](File) parsing with the default parser
    pub fn file(&mut self) -> &mut Self {
        self.with_file(File::new())
    }

    /// Enable [file path](File) parsing, using the given parser
    pub fn with_file(&mut self, f: File) -> &mut Self {
        self.file = Some(f);

        self
    }

    /// Checks if you can successfully convert into a [Config]
    pub fn is_valid(&self) -> bool {
        let b = self;

        b.text.is_some() || b.stdin.is_some() || b.file.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_default_is_valid() {
        let _cfg = Config::default();
    }

    #[test]
    fn builder_set_text() {
        let b = Builder::new().with(|this| this.text());

        assert!(b.text.is_some())
    }

    #[test]
    fn builder_set_file() {
        let b = Builder::new().with(|this| this.file());

        assert!(b.file.is_some())
    }

    #[test]
    fn builder_set_stdin() {
        let b = Builder::new().with(|this| this.stdin());

        assert!(b.stdin.is_some())
    }

    #[test]
    fn sorted_by_weight_ascending() {
        let cfg = Config::default();

        let mut last = 0;
        cfg.with_parsers(|list| {
            // TODO: replace with list.is_sorted_by(|wp| wp.weight()) when method is stabilized
            for wp in list.iter().filter_map(|o| *o) {
                let weight = wp.weight();

                assert!(weight >= last);

                last = weight;
            }
        })
    }

    #[test]
    fn config_default_parse_stdin() {
        let input = "-";
        let cfg = Config::default();

        let t = cfg.parse_str(input).expect("a successful parse");

        match t {
            InputType::Stdin => {}
            bad => panic!("expected Stdin, got: {:?}", bad),
        }
    }

    #[test]
    fn config_default_parse_file() {
        let input = "@some/relative/path";
        let cfg = Config::default();

        let t = cfg.parse_str(input).expect("a successful parse");

        match t {
            InputType::File(_) => {}
            bad => panic!("expected File, got: {:?}", bad),
        }
    }

    #[test]
    fn config_default_parse_text() {
        let input = "basic textual input";
        let cfg = Config::default();

        let t = cfg.parse_str(input).expect("a successful parse");

        match t {
            InputType::UTF8(_) => {}
            bad => panic!("expected Text, got: {:?}", bad),
        }
    }
}
