use std::fmt;

#[derive(Debug)]
pub struct ErrorContext {
    pub field_path: String,
    pub pattern_str: String,
    pub actual_value: String,
    pub line_number: u32,
    pub file_name: &'static str,
    pub error_type: ErrorType,
}

#[derive(Debug)]
pub enum ErrorType {
    Comparison,
    Range,
    Regex,
    Value,
    EnumVariant,
    Slice,
    Tuple,
}

impl fmt::Display for ErrorType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorType::Comparison => write!(f, "comparison"),
            ErrorType::Range => write!(f, "range"),
            ErrorType::Regex => write!(f, "regex pattern"),
            ErrorType::Value => write!(f, "value"),
            ErrorType::EnumVariant => write!(f, "enum variant"),
            ErrorType::Slice => write!(f, "slice"),
            ErrorType::Tuple => write!(f, "tuple"),
        }
    }
}

pub fn format_error(error: ErrorContext) -> String {
    // For now, simple formatting - will enhance in later phases
    format!(
        "assert_struct! failed:\n\n{} mismatch:\n  --> `{}` ({}:{})\n  actual: {}\n  expected: {}",
        error.error_type,
        error.field_path,
        error.file_name,
        error.line_number,
        error.actual_value,
        error.pattern_str
    )
}

pub fn format_multiple_errors(errors: Vec<ErrorContext>) -> String {
    if errors.is_empty() {
        return "assert_struct! failed: no errors provided".to_string();
    }

    if errors.len() == 1 {
        return format_error(errors.into_iter().next().unwrap());
    }

    let mut result = format!("assert_struct! failed: {} mismatches\n\n", errors.len());

    for (i, error) in errors.iter().enumerate() {
        if i > 0 {
            result.push('\n');
        }
        result.push_str(&format!(
            "{} mismatch:\n  --> `{}` ({}:{})\n  actual: {}\n  expected: {}\n",
            error.error_type,
            error.field_path,
            error.file_name,
            error.line_number,
            error.actual_value,
            error.pattern_str
        ));
    }

    result
}
