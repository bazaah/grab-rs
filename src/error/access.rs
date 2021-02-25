//! Contains the error returned attempting to [access][crate::Input::access] an
//! [Input][crate::Input].

use std::{
    fmt, io,
    path::{Path, PathBuf},
};

/// A error representing some error condition that occurred when attempting to access the source of
/// some input. This type is fairly opaque, but does expose a [kind][AccessError::kind] method that
/// returns an enum which best describes the underlying source.
#[derive(Debug)]
pub struct AccessError {
    inner: Inner,
}

impl AccessError {
    /// Returns a enum describes the type of error encountered
    pub fn kind(&self) -> Kind {
        self.inner.kind()
    }

    /// Create a new error that originates from an attempt to access a file
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

/// A cheap descriptor of the kind of access error encountered
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Kind {
    /// The underlying error originates from attempting to access a file
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

/// The actual representation of an AccessError
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
