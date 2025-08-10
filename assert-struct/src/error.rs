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

fn truncate_path(path: &str, max_len: usize) -> String {
    if path.len() <= max_len {
        return path.to_string();
    }

    // Try to truncate at a sensible boundary (field separator)
    let ellipsis = "...";
    let available = max_len.saturating_sub(ellipsis.len());

    // Find the last complete field name that fits
    let parts: Vec<&str> = path.split('.').collect();
    if parts.len() <= 2 {
        // If very few parts, just truncate with ellipsis
        let start = path.len().saturating_sub(available);
        return format!("{}{}", ellipsis, &path[start..]);
    }

    // Keep first part and as many last parts as possible
    let first = parts[0];
    let mut kept_parts = vec![first];
    let mut length = first.len();

    for part in parts.iter().rev() {
        let part_len = part.len() + 1; // +1 for the dot
        if length + ellipsis.len() + part_len <= max_len {
            length += part_len;
            kept_parts.insert(1, part); // Insert after first element
        } else {
            break;
        }
    }

    if kept_parts.len() == parts.len() {
        // All parts fit, no truncation needed
        path.to_string()
    } else {
        // Join first part, ellipsis, and remaining parts
        let first = kept_parts[0];
        let rest = &kept_parts[1..];
        if rest.is_empty() {
            format!("{}...{}", first, parts.last().unwrap())
        } else {
            format!("{}...{}", first, rest.join("."))
        }
    }
}

pub fn format_error(error: ErrorContext) -> String {
    let truncated_path = truncate_path(&error.field_path, 60);
    format!(
        "assert_struct! failed:\n\n{} mismatch:\n  --> `{}` ({}:{})\n  actual: {}\n  expected: {}",
        error.error_type,
        truncated_path,
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
        let truncated_path = truncate_path(&error.field_path, 60);
        result.push_str(&format!(
            "{} mismatch:\n  --> `{}` ({}:{})\n  actual: {}\n  expected: {}\n",
            error.error_type,
            truncated_path,
            error.file_name,
            error.line_number,
            error.actual_value,
            error.pattern_str
        ));
    }

    result
}
