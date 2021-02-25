//! Contains the error returned during input parsing.

use std::fmt;

pub use kind::EKind;

/// An error originating from an attempt to parse some input into a well understood
/// [Input][crate::input::Input]. This type may accumulate multiple errors, particularly in cases
/// where multiple attempts at parsing are made.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InputError {
    flags: kind::EKind,
}

impl InputError {
    const ALL_KINDS: [EKind; 4] = [EKind::TEXT, EKind::STDIN, EKind::FILE, EKind::REQUIRES_UTF8];

    /// Create a new error from the given kind
    pub fn new(kind: EKind) -> Self {
        Self { flags: kind }
    }

    /// Convenience function for adding additional errors
    pub fn with<F>(self, f: F) -> Self
    where
        F: FnMut(&mut Self) -> &mut Self,
    {
        let mut this = self;
        let mut actions = f;

        actions(&mut this);

        this
    }

    /// Add the given kind to this error
    pub fn insert(&mut self, kind: EKind) -> &mut Self {
        self.flags.insert(kind);

        self
    }

    /// Extend this error from another
    pub fn extend(&mut self, other: Self) -> &mut Self {
        self.insert(other.flags);

        self
    }

    /// Check if this error contains the given kind
    pub fn contains(&self, kind: EKind) -> bool {
        self.flags.contains(kind)
    }

    /// Count the number of kinds this error contains
    pub fn count(&self) -> usize {
        Self::ALL_KINDS
            .iter()
            .filter(|&&k| self.contains(k))
            .count()
    }
}

impl fmt::Display for InputError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.count() > 1 {
            write!(f, "Multiple parsers failed [{:?}]", self.flags)?;
        } else {
            write!(f, "Parser failed [{:?}]", self.flags)?;
        }

        Ok(())
    }
}

impl std::error::Error for InputError {}

impl From<EKind> for InputError {
    fn from(kind: EKind) -> Self {
        Self::new(kind)
    }
}

mod kind {
    use bitflags::bitflags;

    bitflags! {
        /// A error variant that can occur during input parsing
        pub struct EKind: u32 {
            // Parser Variant Errors

            /// Error originates from the [Text][crate::parsers::Text] parser
            const TEXT = 0b000_0000_0000_0000_0000_0000_0000_0001;
            /// Error originates from the [Stdin][crate::parsers::Stdin] parser
            const STDIN = 0b000_0000_0000_0000_0000_0000_0000_0010;
            /// Error originates from the [File][crate::parsers::File] parser
            const FILE = 0b000_0000_0000_0000_0000_0000_0000_0100;

            // General Errors

            /// A parser reported that it requires UTF8 input
            const REQUIRES_UTF8 = 0b000_0000_0000_0001_0000_0000_0000_0000;
        }
    }
}
