use std::fmt;

#[derive(Debug)]
pub struct ErrorContext {
    pub field_path: String,
    pub pattern_str: String,
    pub actual_value: String,
    pub line_number: u32,
    pub file_name: &'static str,
    pub error_type: ErrorType,
    // New fields for fancy formatting
    pub full_pattern: Option<&'static str>, // The complete pattern as a string
    pub pattern_location: Option<PatternLocation>, // Where in the pattern this assertion is
    pub expected_value: Option<String>, // For equality patterns where we need to show the expected value
}

#[derive(Debug, Clone)]
pub struct PatternLocation {
    pub line_in_pattern: usize, // Which line of the pattern (0-indexed)
    pub start_col: usize,       // Start column for underlining
    pub end_col: usize,         // End column for underlining
    pub field_name: String,     // The field being checked
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
    Equality, // For == patterns where we show both actual and expected
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
            ErrorType::Equality => write!(f, "equality"),
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

pub fn format_error(mut error: ErrorContext) -> String {
    // If we have full pattern and location, use fancy formatting
    if let (Some(full_pattern), Some(location)) =
        (error.full_pattern, error.pattern_location.take())
    {
        format_fancy_error(error, full_pattern, location)
    } else {
        // Fall back to simple formatting
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
}

fn format_fancy_error(
    error: ErrorContext,
    full_pattern: &str,
    location: PatternLocation,
) -> String {
    let mut result = String::from("assert_struct! failed:\n\n");

    // Split the pattern into lines
    let pattern_lines: Vec<&str> = full_pattern.lines().collect();

    // Determine context - are we in a nested structure?
    // A path like "user.name" is not nested, but "user.profile.name" is
    let path_parts: Vec<&str> = error.field_path.split('.').collect();
    let is_nested = path_parts.len() > 2;

    // Show the opening context for all multi-line patterns
    if pattern_lines.len() > 1 {
        if is_nested {
            // Build context prefix like "User { ... Profile {"
            let parts: Vec<&str> = error.field_path.split('.').collect();
            if parts.len() > 2 {
                result.push_str("   | ");
                // Show first type
                if let Some(first_line) = pattern_lines.first() {
                    if let Some(brace_pos) = first_line.find('{') {
                        result.push_str(&first_line[..=brace_pos]);
                        result.push_str(" ... ");

                        // Show the immediate parent context
                        if location.line_in_pattern > 0 {
                            // Find the struct/enum that contains our field
                            for i in (0..location.line_in_pattern).rev() {
                                let line = pattern_lines[i];
                                if line.contains('{') && !line.contains('}') {
                                    let trimmed = line.trim();
                                    if let Some(name_end) = trimmed.find('{') {
                                        // Extract the struct/enum name before the {
                                        let before_brace = &trimmed[..name_end].trim_end();
                                        // Get last word before the brace
                                        if let Some(last_word) =
                                            before_brace.split_whitespace().last()
                                        {
                                            result.push_str(last_word);
                                            result.push_str(" {");
                                            break;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                result.push('\n');
            }
        } else {
            // For non-nested multi-line patterns, show the opening line
            result.push_str("   | ");
            result.push_str(pattern_lines[0]);
            result.push('\n');
        }
    }

    // Add error type line
    result.push_str(&format!("{} mismatch:\n", error.error_type));

    // Add location line
    result.push_str(&format!(
        "  --> `{}` (line {})\n",
        error.field_path, error.line_number
    ));

    // Add the specific pattern line with underline
    if location.line_in_pattern < pattern_lines.len() {
        let pattern_line = pattern_lines[location.line_in_pattern];
        result.push_str("   | ");
        result.push_str(pattern_line);
        result.push('\n');

        // Add underline and annotation
        result.push_str("   | ");
        // Add spaces before the underline (accounting for any leading whitespace)
        for _ in 0..location.start_col {
            result.push(' ');
        }
        // Add the underline
        let underline_len = location.end_col.saturating_sub(location.start_col);
        for _ in 0..underline_len {
            result.push('^');
        }
        result.push_str(" actual: ");
        result.push_str(&error.actual_value);
        result.push('\n');

        // For equality patterns, also show the expected value aligned with actual
        if let ErrorType::Equality = error.error_type {
            if let Some(ref expected) = error.expected_value {
                result.push_str("   | ");
                // Add spaces to align with "actual: "
                for _ in 0..location.start_col {
                    result.push(' ');
                }
                for _ in 0..underline_len {
                    result.push(' ');
                }
                result.push_str(" expected: ");
                result.push_str(expected);
                result.push('\n');
            }
        }
    }

    // Add closing context if multi-line
    if pattern_lines.len() > 1 {
        // Show any rest patterns on the next line
        if location.line_in_pattern + 1 < pattern_lines.len() {
            let next_line = pattern_lines[location.line_in_pattern + 1];
            if next_line.trim() == ".." || next_line.trim().starts_with("..") {
                result.push_str("   | ");
                result.push_str(next_line);
                result.push('\n');
            }
        }

        // Show closing braces
        if let Some(last_line) = pattern_lines.last() {
            if !last_line.trim().is_empty() {
                result.push_str("   | ");
                result.push_str(last_line);
            }
        }
    }

    result
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
