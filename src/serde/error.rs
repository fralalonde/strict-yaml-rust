use crate::{
    emitter::EmitError,
    parser::Event,
    scanner::{Marker, ScanError},
};
use std::{fmt, str};

/// An error type that contains all possible error variants that may occur
/// when serializing or deserializing StrictYaml data.
#[derive(Debug)]
pub enum Error {
    /// Wraps errors that originate from serde.
    Message(String),
    /// Enhances errors from serde with a marker to the location where
    /// the error occured.
    MarkedMessage { msg: String, mark: Marker },
    /// Raised when the deserializer calls an unsupported `deserialize_*`
    /// method based on the input data.
    UnsupportedType(&'static str),
    /// Raised when the deserializer expected the start of a YAML stream,
    /// but encounters something different.
    UnexpectedStreamStart {
        mark: Marker,
        expected: &'static str,
    },
    /// Raised when the deserializer expected the end of a YAML stream,
    /// but encounters something different.
    UnexpectedStreamEnd {
        mark: Marker,
        expected: &'static str,
    },
    /// Raised when the deserializer expected the start of a YAML document,
    /// but encounters something different.
    UnexpectedDocumentStart {
        mark: Marker,
        expected: &'static str,
    },
    /// Raised when the deserializer expected the end of a YAML document,
    /// but encounters something different.
    UnexpectedDocumentEnd {
        mark: Marker,
        expected: &'static str,
    },
    /// Raised when the deserializer expected a scalar value, but encounters
    /// something different.
    UnexpectedScalar {
        mark: Marker,
        expected: &'static str,
        value: String,
    },
    /// Raised when the deserializer expected the start of a YAML sequence,
    /// but encounters something different.
    UnexpectedSequenceStart {
        mark: Marker,
        expected: &'static str,
    },
    /// Raised when the deserializer expected the end of a YAML sequence,
    /// but encounters something different.
    UnexpectedSequenceEnd {
        mark: Marker,
        expected: &'static str,
    },
    /// Raised when the deserializer expected the start of a YAML map,
    /// but encounters something different.
    UnexpectedMappingStart {
        mark: Marker,
        expected: &'static str,
    },
    /// Raised when the deserializer expected the end of a YAML map,
    /// but encounters something different.
    UnexpectedMappingEnd {
        mark: Marker,
        expected: &'static str,
    },
}

impl Error {
    /// Construct the appropiate error variant based on the event
    /// emitted by the parser, wich did not match the event expected
    /// by the deserializer.
    pub(crate) fn from_event(ev: Event, mark: Marker, expected: &'static str) -> Self {
        match ev {
            Event::StreamStart => Self::UnexpectedStreamStart { mark, expected },
            Event::StreamEnd => Self::UnexpectedStreamEnd { mark, expected },
            Event::DocumentStart => Self::UnexpectedDocumentStart { mark, expected },
            Event::DocumentEnd => Self::UnexpectedDocumentEnd { mark, expected },
            Event::Scalar(value, _, _) => Self::UnexpectedScalar {
                mark,
                expected,
                value,
            },
            Event::SequenceStart(_) => Self::UnexpectedSequenceStart { mark, expected },
            Event::SequenceEnd => Self::UnexpectedSequenceEnd { mark, expected },
            Event::MappingStart(_) => Self::UnexpectedMappingStart { mark, expected },
            Event::MappingEnd => Self::UnexpectedMappingEnd { mark, expected },
            _ => unreachable!(),
        }
    }
}

impl serde::de::Error for Error {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        Error::Message(msg.to_string())
    }
}

impl serde::ser::Error for Error {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        Error::Message(msg.to_string())
    }
}

impl From<ScanError> for Error {
    fn from(error: ScanError) -> Self {
        Error::Message(error.to_string())
    }
}

impl From<EmitError> for Error {
    fn from(error: EmitError) -> Self {
        Error::Message(error.to_string())
    }
}

impl From<fmt::Error> for Error {
    fn from(error: fmt::Error) -> Self {
        Error::Message(error.to_string())
    }
}

impl From<str::Utf8Error> for Error {
    fn from(error: str::Utf8Error) -> Self {
        Error::Message(error.to_string())
    }
}

impl std::ops::Add<Marker> for Error {
    type Output = Self;

    /// The `add` operator is overloaded to enhance a serde error
    /// with a location provided by the parser.
    fn add(self, mark: Marker) -> Self {
        match self {
            Self::Message(msg) => Self::MarkedMessage { msg, mark },
            _ => self,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Message(msg) => formatter.write_str(msg),
            Error::MarkedMessage { msg, mark } => {
                write!(formatter, "{} at line {}", msg, mark.line())
            }
            Error::UnsupportedType(t) => {
                write!(formatter, "(de)serialization of {} is not supported", t)
            }
            Error::UnexpectedStreamStart { mark, expected } => {
                write!(
                    formatter,
                    "expected {}, but found the start of the stream at line {}",
                    expected,
                    mark.line()
                )
            }
            Error::UnexpectedStreamEnd { mark, expected } => {
                write!(
                    formatter,
                    "expected {}, but found the end of the stream at line {}",
                    expected,
                    mark.line()
                )
            }
            Error::UnexpectedDocumentStart { mark, expected } => {
                write!(
                    formatter,
                    "expected {}, but found the start of a document at line {}",
                    expected,
                    mark.line()
                )
            }
            Error::UnexpectedDocumentEnd { mark, expected } => {
                write!(
                    formatter,
                    "expected {}, but found the end of a document at line {}",
                    expected,
                    mark.line()
                )
            }
            Error::UnexpectedScalar {
                mark,
                expected,
                value,
            } => {
                write!(
                    formatter,
                    "expected {}, but found scalar \"{}\" at line {}",
                    expected,
                    value,
                    mark.line()
                )
            }
            Error::UnexpectedSequenceStart { mark, expected } => {
                write!(
                    formatter,
                    "expected {}, but found the start of a sequence at line {}",
                    expected,
                    mark.line()
                )
            }
            Error::UnexpectedSequenceEnd { mark, expected } => {
                write!(
                    formatter,
                    "expected {}, but found the end of a sequence at line {}",
                    expected,
                    mark.line()
                )
            }
            Error::UnexpectedMappingStart { mark, expected } => {
                write!(
                    formatter,
                    "expected {}, but found the start of a mapping at line {}",
                    expected,
                    mark.line()
                )
            }
            Error::UnexpectedMappingEnd { mark, expected } => {
                write!(
                    formatter,
                    "expected {}, but found the end of a mapping at line {}",
                    expected,
                    mark.line()
                )
            }
        }
    }
}

impl std::error::Error for Error {}
