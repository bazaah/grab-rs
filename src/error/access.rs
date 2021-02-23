use std::{
    fmt, io,
    path::{Path, PathBuf},
};

#[derive(Debug)]
pub struct AccessError {
    inner: Inner,
}

impl AccessError {
    pub fn kind(&self) -> Kind {
        self.inner.kind()
    }

    pub fn file_with_context(err: io::Error, context: impl AsRef<Path>) -> Self {
        Self {
            inner: Inner::file_cxt(err, context.as_ref().to_owned()),
        }
    }
}

impl fmt::Display for AccessError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} access failed: {}", self.kind(), self.inner)
    }
}

impl PartialEq for AccessError {
    fn eq(&self, other: &AccessError) -> bool {
        self.kind() == other.kind()
    }
}

impl std::error::Error for AccessError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Kind {
    File,
}

impl fmt::Display for Kind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let kind = match self {
            Self::File => "file",
        };

        write!(f, "{}", kind)
    }
}

#[derive(Debug)]
enum Inner {
    File {
        context: Option<PathBuf>,
        err: io::Error,
    },
}

impl Inner {
    fn kind(&self) -> Kind {
        match self {
            Self::File { .. } => Kind::File,
        }
    }
}

impl Inner {
    #[allow(dead_code)]
    fn file(err: io::Error) -> Self {
        Self::File { context: None, err }
    }

    fn file_cxt(err: io::Error, context: PathBuf) -> Self {
        Self::File {
            context: Some(context),
            err,
        }
    }
}

impl fmt::Display for Inner {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Inner::*;
        match self {
            File { context, err } => match context {
                Some(path) => write!(f, "unable to open {}: {}", path.display(), err),
                None => write!(f, "unable to open file: {}", err),
            },
        }
    }
}
