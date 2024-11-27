use alloc::string::String;
#[derive(Debug, Clone, PartialEq, Eq)]

/// Enum representing error types.
pub enum Error {
    Network(String),
    UnexpectedInput(String),
    InvalidUI(String),
    Other(String),
}
