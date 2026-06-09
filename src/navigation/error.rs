use std::{error::Error, fmt::Display};

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum NavigationError {
    UnknownError,
    InvalidSura,
    InvalidVerse,
    OutOfBounds,
    NegativeLines,
}

impl Display for NavigationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownError => write!(f, "Unknown navigation error"),
            Self::InvalidSura => write!(f, "Invalid sura number"),
            Self::InvalidVerse => write!(f, "Invalid verse number"),
            Self::OutOfBounds => write!(f, "Navigation result is out of bounds"),
            Self::NegativeLines => write!(f, "Negative lines are not allowed"),
        }
    }
}

impl Error for NavigationError {}

#[derive(Debug)]
pub enum CalculatingLinesError {
    WrongBoundary,
}

impl Display for CalculatingLinesError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            &Self::WrongBoundary => write!(f, "Wrong Boundary"),
        }
    }
}
impl Error for CalculatingLinesError {}

#[derive(Debug)]
pub enum LookupError {
    InvalidSura(u8),
    SuraNotFound(u8),
    InvalidVerse(u16),
    VerseNotFound(u16),
    OutOfBounds,
}

impl Display for LookupError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidSura(i) => write!(f, "InvalidSura {i}"),
            Self::SuraNotFound(i) => write!(f, "SuraNotFound {i}"),
            Self::InvalidVerse(i) => write!(f, "InvalidVerse {i}"),
            Self::VerseNotFound(i) => write!(f, "VerseNotFound {i}"),
            Self::OutOfBounds => write!(f, "OutOfBounds"),
        }
    }
}
impl Error for LookupError {}
