use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub enum ErrorType {
    System,
    Process,
}

#[derive(Debug, Clone, Serialize)]
pub struct PaginatedError {
    pub error_type: ErrorType,
    pub page: Option<usize>,
    pub message: String,
}
