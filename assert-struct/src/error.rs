use std::fmt;

/// Tree-based pattern representation for error formatting
#[derive(Debug)]
pub enum PatternNode {
    // Structural patterns
    Struct {
        name: &'static str,
        fields: &'static [(&'static str, &'static PatternNode)],
    },

    // Collection patterns
    Slice {
        items: &'static [&'static PatternNode],
        is_ref: bool, // Whether it's &[...] or just [...]
    },
    Tuple {
        items: &'static [&'static PatternNode],
    },

    // Simple patterns
    Simple {
        value: &'static str, // "123", "\"hello\"", etc.
    },
    Comparison {
        op: &'static str, // ">", ">=", "<", "<=", "==", "!="
        value: &'static str,
    },
    Range {
        pattern: &'static str, // "18..=65", "0.0..100.0"
    },
    Regex {
        pattern: &'static str,
    },
    Like {
        expr: &'static str,
    },

    // Enum patterns
    EnumVariant {
        path: &'static str, // "Some", "Ok", "Status::Active"
        args: Option<&'static [&'static PatternNode]>,
    },

    // Special
    Rest, // The ".." pattern
}

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
    // Tree-based pattern data
    pub pattern_tree: Option<&'static PatternNode>,
    pub error_node: Option<&'static PatternNode>,
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

// Tree-based formatting implementation

struct TreeFormatter {
    output: String,
    current_line: usize,
    error_line: Option<usize>,
    error_col_start: Option<usize>,
    error_col_end: Option<usize>,
}

impl TreeFormatter {
    fn new() -> Self {
        Self {
            output: String::new(),
            current_line: 0,
            error_line: None,
            error_col_start: None,
            error_col_end: None,
        }
    }

    fn format_with_context(
        &mut self,
        root: &'static PatternNode,
        error_node: &'static PatternNode,
        context_lines: usize,
    ) {
        // Find path to error node
        let path = find_path_to_node(root, error_node, Vec::new());

        // Format the tree with the error highlighted
        self.format_node(
            root,
            0,
            std::ptr::eq(root, error_node),
            &path,
            0,
            context_lines,
        );
    }

    #[allow(clippy::too_many_arguments)]
    fn format_node(
        &mut self,
        node: &'static PatternNode,
        depth: usize,
        is_error: bool,
        error_path: &[String],
        path_index: usize,
        context_lines: usize,
    ) {
        let indent = "    ".repeat(depth);

        match node {
            PatternNode::Struct { name, fields } => {
                self.output
                    .push_str(&format!("   | {}{} {{\n", indent, name));
                self.current_line += 1;

                // Check if we should show all fields or prune
                let should_prune = fields.len() > context_lines * 2 && !error_path.is_empty();

                if should_prune {
                    // Find which field is on the error path
                    let error_field_idx = if path_index < error_path.len() {
                        fields
                            .iter()
                            .position(|(name, _)| name == &error_path[path_index])
                    } else {
                        None
                    };

                    if let Some(idx) = error_field_idx {
                        // Show fields around the error
                        let start = idx.saturating_sub(context_lines);
                        let end = (idx + context_lines + 1).min(fields.len());

                        if start > 0 {
                            self.output.push_str(&format!("   | {}    ...\n", indent));
                            self.current_line += 1;
                        }

                        for i in start..end {
                            let (field_name, field_node) = fields[i];
                            let is_error_field = i == idx;
                            self.format_field(
                                field_name,
                                field_node,
                                depth + 1,
                                is_error_field && path_index == error_path.len() - 1,
                                error_path,
                                if is_error_field {
                                    path_index + 1
                                } else {
                                    usize::MAX
                                },
                                context_lines,
                            );
                        }

                        if end < fields.len() {
                            self.output.push_str(&format!("   | {}    ...\n", indent));
                            self.current_line += 1;
                        }
                    }
                } else {
                    // Show all fields
                    for (field_name, field_node) in fields.iter() {
                        let is_on_path =
                            path_index < error_path.len() && field_name == &error_path[path_index];
                        self.format_field(
                            field_name,
                            field_node,
                            depth + 1,
                            is_on_path && is_error, // is_error already indicates if this field node is the error
                            error_path,
                            if is_on_path {
                                path_index + 1
                            } else {
                                usize::MAX
                            },
                            context_lines,
                        );
                    }
                }

                self.output.push_str(&format!("   | {}}}\n", indent));
                self.current_line += 1;
            }
            PatternNode::Slice { items, is_ref } => {
                let prefix = if *is_ref { "&" } else { "" };
                let content = items
                    .iter()
                    .map(|item| self.format_inline(item))
                    .collect::<Vec<_>>()
                    .join(", ");

                if is_error {
                    let line_start = self.output.lines().last().map(|l| l.len()).unwrap_or(0);
                    self.error_col_start = Some(line_start);
                    self.error_col_end = Some(line_start + prefix.len() + content.len() + 2); // +2 for []
                    self.error_line = Some(self.current_line);
                }

                self.output.push_str(&format!("{}[{}]", prefix, content));
            }
            PatternNode::Tuple { items } => {
                let content = items
                    .iter()
                    .map(|item| self.format_inline(item))
                    .collect::<Vec<_>>()
                    .join(", ");

                if is_error {
                    let line_start = self.output.lines().last().map(|l| l.len()).unwrap_or(0);
                    self.error_col_start = Some(line_start);
                    self.error_col_end = Some(line_start + content.len() + 2); // +2 for ()
                    self.error_line = Some(self.current_line);
                }

                self.output.push_str(&format!("({})", content));
            }
            PatternNode::Simple { value } => {
                if is_error {
                    let line_start = self.output.lines().last().map(|l| l.len()).unwrap_or(0);
                    self.error_col_start = Some(line_start);
                    self.error_col_end = Some(line_start + value.len());
                    self.error_line = Some(self.current_line);
                }
                self.output.push_str(value);
            }
            PatternNode::Comparison { op, value } => {
                let pattern = format!("{} {}", op, value);
                if is_error {
                    let line_start = self.output.lines().last().map(|l| l.len()).unwrap_or(0);
                    self.error_col_start = Some(line_start);
                    self.error_col_end = Some(line_start + pattern.len());
                    self.error_line = Some(self.current_line);
                }
                self.output.push_str(&pattern);
            }
            PatternNode::Range { pattern } => {
                if is_error {
                    let line_start = self.output.lines().last().map(|l| l.len()).unwrap_or(0);
                    self.error_col_start = Some(line_start);
                    self.error_col_end = Some(line_start + pattern.len());
                    self.error_line = Some(self.current_line);
                }
                self.output.push_str(pattern);
            }
            PatternNode::Regex { pattern } => {
                let full_pattern = format!("=~ {}", pattern);
                if is_error {
                    let line_start = self.output.lines().last().map(|l| l.len()).unwrap_or(0);
                    self.error_col_start = Some(line_start);
                    self.error_col_end = Some(line_start + full_pattern.len());
                    self.error_line = Some(self.current_line);
                }
                self.output.push_str(&full_pattern);
            }
            PatternNode::Like { expr } => {
                let full_pattern = format!("=~ {}", expr);
                if is_error {
                    let line_start = self.output.lines().last().map(|l| l.len()).unwrap_or(0);
                    self.error_col_start = Some(line_start);
                    self.error_col_end = Some(line_start + full_pattern.len());
                    self.error_line = Some(self.current_line);
                }
                self.output.push_str(&full_pattern);
            }
            PatternNode::EnumVariant { path, args } => {
                self.output.push_str(path);
                if let Some(args) = args {
                    if !args.is_empty() {
                        self.output.push('(');
                        for (i, arg) in args.iter().enumerate() {
                            if i > 0 {
                                self.output.push_str(", ");
                            }
                            // Pass is_error for the argument being the error node
                            // (would need proper path tracking for enum variant args)
                            self.format_node(
                                arg,
                                depth,
                                false,
                                error_path,
                                path_index,
                                context_lines,
                            );
                        }
                        self.output.push(')');
                    }
                }

                if is_error {
                    // Mark the whole enum variant as the error
                    let _line_start = self.output.lines().last().map(|l| l.len()).unwrap_or(0);
                    self.error_line = Some(self.current_line);
                    // We'd need to track where we started to get proper col positions
                }
            }
            PatternNode::Rest => {
                self.output.push_str("..");
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn format_field(
        &mut self,
        name: &str,
        node: &'static PatternNode,
        depth: usize,
        is_error: bool,
        error_path: &[String],
        path_index: usize,
        context_lines: usize,
    ) {
        let indent = "    ".repeat(depth);
        self.output
            .push_str(&format!("   | {}    {}: ", indent, name));

        let field_start_col = self.output.lines().last().map(|l| l.len() - 7).unwrap_or(0); // -7 for "   | "

        self.format_node(node, depth, is_error, error_path, path_index, context_lines);
        self.output.push_str(",\n");

        if is_error && self.error_col_start.is_none() {
            // If the error position wasn't set by the node itself, set it to the field value
            self.error_col_start = Some(field_start_col + name.len() + 2); // +2 for ": "
            self.error_line = Some(self.current_line);
        }

        self.current_line += 1;
    }

    #[allow(clippy::only_used_in_recursion)]
    fn format_inline(&self, node: &'static PatternNode) -> String {
        match node {
            PatternNode::Simple { value } => value.to_string(),
            PatternNode::Comparison { op, value } => format!("{} {}", op, value),
            PatternNode::Range { pattern } => pattern.to_string(),
            PatternNode::Regex { pattern } => format!("=~ {}", pattern),
            PatternNode::Like { expr } => format!("=~ {}", expr),
            PatternNode::Slice { items, is_ref } => {
                let prefix = if *is_ref { "&" } else { "" };
                let content = items
                    .iter()
                    .map(|item| self.format_inline(item))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{}[{}]", prefix, content)
            }
            PatternNode::Tuple { items } => {
                let content = items
                    .iter()
                    .map(|item| self.format_inline(item))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("({})", content)
            }
            PatternNode::EnumVariant { path, args } => {
                if let Some(args) = args {
                    if !args.is_empty() {
                        let arg_str = args
                            .iter()
                            .map(|arg| self.format_inline(arg))
                            .collect::<Vec<_>>()
                            .join(", ");
                        return format!("{}({})", path, arg_str);
                    }
                }
                path.to_string()
            }
            PatternNode::Struct { name, .. } => format!("{} {{ ... }}", name),
            PatternNode::Rest => "..".to_string(),
        }
    }

    fn get_error_location(&self) -> PatternLocation {
        PatternLocation {
            line_in_pattern: self.error_line.unwrap_or(0),
            start_col: self.error_col_start.unwrap_or(0),
            end_col: self.error_col_end.unwrap_or(0),
            field_name: String::new(), // Would need to track this
        }
    }
}

fn find_path_to_node(
    root: &'static PatternNode,
    target: &'static PatternNode,
    path: Vec<String>,
) -> Vec<String> {
    if std::ptr::eq(root, target) {
        return path;
    }

    match root {
        PatternNode::Struct { fields, .. } => {
            for (name, node) in fields.iter() {
                let mut new_path = path.clone();
                new_path.push(name.to_string());
                let result = find_path_to_node(node, target, new_path);
                if !result.is_empty() {
                    return result;
                }
            }
        }
        PatternNode::EnumVariant {
            args: Some(args), ..
        } => {
            for (i, arg) in args.iter().enumerate() {
                let mut new_path = path.clone();
                new_path.push(format!("[{}]", i));
                let result = find_path_to_node(arg, target, new_path);
                if !result.is_empty() {
                    return result;
                }
            }
        }
        PatternNode::EnumVariant { args: None, .. } => {}
        PatternNode::Slice { items, .. } | PatternNode::Tuple { items } => {
            for (i, item) in items.iter().enumerate() {
                let mut new_path = path.clone();
                new_path.push(format!("[{}]", i));
                let result = find_path_to_node(item, target, new_path);
                if !result.is_empty() {
                    return result;
                }
            }
        }
        _ => {}
    }

    Vec::new()
}

pub fn format_error_with_tree(
    root: &'static PatternNode,
    error_node: &'static PatternNode,
    error: ErrorContext,
    context_lines: usize,
) -> String {
    let mut formatter = TreeFormatter::new();
    formatter.format_with_context(root, error_node, context_lines);

    let location = formatter.get_error_location();

    // Build the full error message
    let mut result = String::from("assert_struct! failed:\n\n");
    result.push_str(&formatter.output);
    result.push('\n');

    // Add error details
    result.push_str(&format!("{} mismatch:\n", error.error_type));
    result.push_str(&format!(
        "  --> `{}` (line {})\n",
        error.field_path, error.line_number
    ));

    // Add underline and actual value
    if location.line_in_pattern < formatter.output.lines().count() {
        result.push_str("   | ");
        for _ in 0..location.start_col {
            result.push(' ');
        }
        let underline_len = location.end_col.saturating_sub(location.start_col);
        for _ in 0..underline_len {
            result.push('^');
        }
        result.push_str(" actual: ");
        result.push_str(&error.actual_value);
        result.push('\n');

        // For equality patterns, also show expected
        if let ErrorType::Equality = error.error_type {
            if let Some(ref expected) = error.expected_value {
                result.push_str("   | ");
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

    result
}
