/// Edit operation tracking
use std::time::SystemTime;

#[derive(Debug, Clone)]
pub enum OperationType {
    Update,
    Insert,
    Delete,
}

#[derive(Debug, Clone)]
pub struct EditOperation {
    pub line_number: usize,
    pub original_content: String,
    pub new_content: String,
    pub timestamp: SystemTime,
    pub operation_type: OperationType,
}
