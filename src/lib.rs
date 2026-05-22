use ::core::fmt;

pub mod core;
pub mod env;
pub mod scope;

#[derive(Debug)]
pub enum RiddleError {
    NotAnEnvironment(String),
    NotAClass(String),
    NotAPredicate(String),
    TypeError(String),
    NotFound(String),
    InconsistencyError(String),
    RuntimeError(String),
}

impl fmt::Display for RiddleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RiddleError::NotAnEnvironment(name) => write!(f, "Variable '{}' is not an environment", name),
            RiddleError::NotAClass(name) => write!(f, "Type '{}' is not a class", name),
            RiddleError::NotAPredicate(name) => write!(f, "Predicate '{}' not found", name),
            RiddleError::TypeError(msg) => write!(f, "Type error: {}", msg),
            RiddleError::NotFound(name) => write!(f, "'{}' not found", name),
            RiddleError::InconsistencyError(msg) => write!(f, "Inconsistency error: {}", msg),
            RiddleError::RuntimeError(msg) => write!(f, "Runtime error: {}", msg),
        }
    }
}
