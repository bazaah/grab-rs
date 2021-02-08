use crate::{
    error::input::InputError,
    input::Input,
    parsers::{File, InputType, Parser, Stdin, Text, Weight, WeightedParser as WP},
};

use std::{ffi::OsStr, fmt};

#[derive(Debug, Clone, Default)]
pub struct Builder {
    stdin: Option<Stdin>,
    file: Option<File>,
    text: Option<Text>,
}

impl Builder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with<F>(self, f: F) -> Self
    where
        F: FnMut(&mut Self) -> &mut Self,
    {
        let mut this = self;
        let mut actions = f;

        actions(&mut this);

        this
    }

    pub fn build(self) -> Result<Config, Self> {
        if self.is_valid() {
            return Ok(Config { inner: self });
        }

        Err(self)
    }

    pub fn text(&mut self) -> &mut Self {
        self.with_text(Text::new())
    }

    pub fn with_text(&mut self, t: Text) -> &mut Self {
        self.text = Some(t);

        self
    }

    pub fn stdin(&mut self) -> &mut Self {
        self.with_stdin(Stdin::new())
    }

    pub fn with_stdin(&mut self, s: Stdin) -> &mut Self {
        self.stdin = Some(s);

        self
    }

    pub fn file(&mut self) -> &mut Self {
        self.with_file(File::new())
    }

    pub fn with_file(&mut self, f: File) -> &mut Self {
        self.file = Some(f);

        self
    }

    pub fn is_valid(&self) -> bool {
        let b = self;

        b.text.is_some() || b.stdin.is_some() || b.file.is_some()
    }
}

#[derive(Clone)]
pub struct Config {
    inner: Builder,
}

impl Config {
    pub fn parse(&self, input: &str) -> Result<Input, InputError> {
        self.parse_str(input).map(|t| Input::from_input_type(t))
    }

    pub fn parse_os(&self, input: &OsStr) -> Result<Input, InputError> {
        self.parse_os_str(input).map(|t| Input::from_input_type(t))
    }

    fn internal_parse<F>(&self, mut f: F) -> Result<InputType, InputError>
    where
        F: FnMut(&dyn WP) -> Result<InputType, InputError>,
    {
        let b = &self.inner;
        let mut list = [
            b.file.as_ref().map(|p| p as &dyn WP),
            b.stdin.as_ref().map(|p| p as &dyn WP),
            b.text.as_ref().map(|p| p as &dyn WP),
        ];

        // Sort parsers by weight, with lower numbers taking
        // priority.
        list.sort_by_key(|opt| opt.map(|p| p.weight()));

        let mut error: Option<InputError> = None;

        for parser in list.iter().filter_map(|o| *o) {
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
        self.internal_parse(|p| p.parse_str(input))
    }

    fn parse_os_str(&self, input: &OsStr) -> Result<InputType, InputError> {
        self.internal_parse(|p| p.parse_os_str(input))
    }

    fn parse_bytes(&self, input: &[u8]) -> Result<InputType, InputError> {
        self.internal_parse(|p| p.parse_bytes(input))
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
