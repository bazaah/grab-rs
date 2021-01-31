use std::fmt;

pub use kind::EKind;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InputError {
    flags: kind::EKind,
}

impl InputError {
    const ALL_KINDS: [EKind; 4] = [EKind::TEXT, EKind::STDIN, EKind::FILE, EKind::REQUIRES_UTF8];

    pub fn new(kind: EKind) -> Self {
        Self { flags: kind }
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

    pub fn insert(&mut self, kind: EKind) -> &mut Self {
        self.flags.insert(kind);

        self
    }

    pub fn extend(&mut self, other: Self) -> &mut Self {
        self.insert(other.flags);

        self
    }

    pub fn contains(&self, kind: EKind) -> bool {
        self.flags.contains(kind)
    }

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
        pub struct EKind: u32 {
            // Parser Variant Errors
            const TEXT = 0b00000000_00000000_00000000_00000001;
            const STDIN = 0b00000000_00000000_00000000_00000010;
            const FILE = 0b00000000_00000000_00000000_00000100;

            // General Errors
            const REQUIRES_UTF8 = 0b00000000_00000001_00000000_0000000;
        }
    }
}
