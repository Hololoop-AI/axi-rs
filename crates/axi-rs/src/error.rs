//! Structured errors that carry actionable suggestions and map to stable exit
//! codes — the AXI contract that tells an agent how to fix its own call.

use serde_json::{json, Value};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCode {
    Validation,
    NotFound,
    Server,
}

#[derive(Debug, Clone)]
pub struct AxiError {
    pub message: String,
    pub code: ErrorCode,
    pub suggestions: Vec<String>,
}

impl AxiError {
    pub fn new(message: impl Into<String>, code: ErrorCode) -> Self {
        AxiError {
            message: message.into(),
            code,
            suggestions: Vec::new(),
        }
    }

    pub fn validation(message: impl Into<String>) -> Self {
        Self::new(message, ErrorCode::Validation)
    }

    pub fn not_found(message: impl Into<String>) -> Self {
        Self::new(message, ErrorCode::NotFound)
    }

    pub fn server(message: impl Into<String>) -> Self {
        Self::new(message, ErrorCode::Server)
    }

    /// Attach one actionable suggestion; chainable.
    pub fn suggest(mut self, s: impl Into<String>) -> Self {
        self.suggestions.push(s.into());
        self
    }

    /// The TOON-serializable shape emitted to stderr on failure.
    pub fn to_value(&self) -> Value {
        json!({
            "error": self.message,
            "code": code_name(self.code),
            "suggestions": self.suggestions,
        })
    }
}

impl std::fmt::Display for AxiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for AxiError {}

fn code_name(code: ErrorCode) -> &'static str {
    match code {
        ErrorCode::Validation => "VALIDATION_ERROR",
        ErrorCode::NotFound => "NOT_FOUND",
        ErrorCode::Server => "SERVER_ERROR",
    }
}

/// The process exit code for an error. Distinct codes per class so a caller can
/// branch without parsing the message.
pub fn exit_code_for(err: &AxiError) -> i32 {
    match err.code {
        ErrorCode::Server => 1,
        ErrorCode::Validation => 2,
        ErrorCode::NotFound => 3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exit_codes_are_stable_per_class() {
        assert_eq!(exit_code_for(&AxiError::server("x")), 1);
        assert_eq!(exit_code_for(&AxiError::validation("x")), 2);
        assert_eq!(exit_code_for(&AxiError::not_found("x")), 3);
    }

    #[test]
    fn value_carries_code_and_suggestions() {
        let e = AxiError::validation("bad path").suggest("Run `axi open <file>`");
        let v = e.to_value();
        assert_eq!(v["code"], "VALIDATION_ERROR");
        assert_eq!(v["error"], "bad path");
        assert_eq!(v["suggestions"][0], "Run `axi open <file>`");
    }
}
