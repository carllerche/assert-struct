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
    // First try tree-based formatting if available
    if let (Some(pattern_tree), Some(error_node)) = (error.pattern_tree, error.error_node) {
        // Use tree-based formatting with default context lines
        format_error_with_tree(pattern_tree, error_node, error, 3)
    } else if let (Some(full_pattern), Some(location)) =
        (error.full_pattern, error.pattern_location.take())
    {
        // Fall back to string-based fancy formatting
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

    // Format multiple errors by showing each error using the same logic as single errors
    let mut result = format!("assert_struct! failed: {} mismatches\n", errors.len());

    for (i, error) in errors.into_iter().enumerate() {
        if i > 0 {
            result.push_str("\n---\n");
        }

        // Format each error using the same logic as single errors
        let single_error = format_error(error);

        // Strip the "assert_struct! failed:" prefix since we already have one
        let error_body = if single_error.starts_with("assert_struct! failed:") {
            single_error
                .strip_prefix("assert_struct! failed:")
                .unwrap()
                .trim_start()
        } else {
            &single_error
        };

        result.push_str(error_body);
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
    error_node: &'static PatternNode,
}

impl TreeFormatter {
    fn new(error_node: &'static PatternNode) -> Self {
        Self {
            output: String::new(),
            current_line: 0,
            error_line: None,
            error_col_start: None,
            error_col_end: None,
            error_node,
        }
    }

    fn format_with_context(&mut self, root: &'static PatternNode, context_lines: usize) {
        // Find path to error node
        let path = find_path_to_node(root, self.error_node, Vec::new());

        // Format the tree with the error highlighted
        self.format_node(root, 0, &path, 0, context_lines);
    }

    fn format_node(
        &mut self,
        node: &'static PatternNode,
        depth: usize,
        error_path: &[String],
        path_index: usize,
        context_lines: usize,
    ) {
        let is_error = std::ptr::eq(node, self.error_node);
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
                            self.output.push_str(&format!("   | {}...\n", indent));
                            self.current_line += 1;
                        }

                        for i in start..end {
                            let (field_name, field_node) = fields[i];
                            // Skip rest pattern in field iteration
                            if field_name == ".." {
                                continue;
                            }
                            self.format_field(
                                field_name,
                                field_node,
                                depth + 1,
                                error_path,
                                if i == idx { path_index + 1 } else { usize::MAX },
                                context_lines,
                            );
                        }

                        // Check if we need to show the rest pattern (if it exists and is in range)
                        let has_rest = fields.iter().any(|(name, _)| name == &"..");
                        if has_rest && end >= fields.len() - 1 {
                            // Show rest pattern if it's at the end and in range
                            self.output
                                .push_str(&format!("   | {}..\n", "    ".repeat(depth + 1)));
                            self.current_line += 1;
                        } else if end < fields.len() && !has_rest {
                            // Show ellipsis for pruned fields only if there's no rest pattern
                            self.output.push_str(&format!("   | {}...\n", indent));
                            self.current_line += 1;
                        }
                    }
                } else {
                    // Show all fields
                    for (field_name, field_node) in fields.iter() {
                        // Handle rest pattern specially
                        if field_name == &".." {
                            // Don't format rest patterns as fields
                            continue;
                        }

                        let is_on_path =
                            path_index < error_path.len() && field_name == &error_path[path_index];
                        self.format_field(
                            field_name,
                            field_node,
                            depth + 1,
                            error_path,
                            if is_on_path {
                                path_index + 1
                            } else {
                                usize::MAX
                            },
                            context_lines,
                        );
                    }

                    // If there's a rest pattern, show it after the fields
                    if fields.iter().any(|(name, _)| name == &"..") {
                        self.output
                            .push_str(&format!("   | {}..\n", "    ".repeat(depth + 1)));
                        self.current_line += 1;
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
                self.output.push('(');
                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }

                    // Check if this item is the error node
                    let item_is_error = std::ptr::eq(*item, self.error_node);

                    if item_is_error {
                        // Mark the position before formatting this item
                        // Get the current line's content to calculate column position
                        let current_line_content = self.output.lines().last().unwrap_or("");
                        self.error_col_start = Some(current_line_content.len());
                        self.error_line = Some(self.current_line);
                    }

                    // Format the item inline
                    let item_str = self.format_inline(item);
                    self.output.push_str(&item_str);

                    if item_is_error && self.error_col_start.is_some() {
                        // Mark the end position after formatting this item
                        let current_line_content = self.output.lines().last().unwrap_or("");
                        self.error_col_end = Some(current_line_content.len());
                    }
                }
                self.output.push(')');
            }
            PatternNode::Simple { value } => {
                if is_error && self.error_col_start.is_none() {
                    // Only set position if not already set by format_field
                    let current_line = self.output.lines().last().unwrap_or("");
                    let col_start = current_line.len() - 5; // Subtract "   | " prefix
                    self.error_col_start = Some(col_start);
                    self.error_col_end = Some(col_start + value.len());
                    self.error_line = Some(self.current_line);
                }
                self.output.push_str(value);
            }
            PatternNode::Comparison { op, value } => {
                if is_error && self.error_col_start.is_none() {
                    // Only set position if not already set by format_field
                    // Calculate position BEFORE adding the pattern to output
                    let current_line = self.output.lines().last().unwrap_or("");
                    let base_col = current_line.len() - 5; // Subtract "   | " prefix

                    // For equality patterns (== and !=), underline just the value part
                    // For comparison patterns (<, >, <=, >=), underline the whole pattern
                    if op == &"==" || op == &"!=" {
                        // Skip the operator and space (e.g., "== " is 3 chars)
                        let op_with_space_len = op.len() + 1;
                        self.error_col_start = Some(base_col + op_with_space_len);
                        self.error_col_end = Some(base_col + op_with_space_len + value.len());
                    } else {
                        self.error_col_start = Some(base_col);
                        self.error_col_end = Some(base_col + op.len() + 1 + value.len());
                    }

                    self.error_line = Some(self.current_line);
                }
                // Now add the pattern to output
                let pattern = format!("{} {}", op, value);
                self.output.push_str(&pattern);
            }
            PatternNode::Range { pattern } => {
                if is_error && self.error_col_start.is_none() {
                    let current_line = self.output.lines().last().unwrap_or("");
                    let col_start = current_line.len() - 5; // Subtract "   | " prefix
                    self.error_col_start = Some(col_start);
                    self.error_col_end = Some(col_start + pattern.len());
                    self.error_line = Some(self.current_line);
                }
                self.output.push_str(pattern);
            }
            PatternNode::Regex { pattern } => {
                let full_pattern = format!("=~ {}", pattern);
                if is_error && self.error_col_start.is_none() {
                    let current_line = self.output.lines().last().unwrap_or("");
                    let col_start = current_line.len() - 5; // Subtract "   | " prefix
                    self.error_col_start = Some(col_start);
                    self.error_col_end = Some(col_start + full_pattern.len());
                    self.error_line = Some(self.current_line);
                }
                self.output.push_str(&full_pattern);
            }
            PatternNode::Like { expr } => {
                let full_pattern = format!("=~ {}", expr);
                if is_error && self.error_col_start.is_none() {
                    let current_line = self.output.lines().last().unwrap_or("");
                    let col_start = current_line.len() - 5; // Subtract "   | " prefix
                    self.error_col_start = Some(col_start);
                    self.error_col_end = Some(col_start + full_pattern.len());
                    self.error_line = Some(self.current_line);
                }
                self.output.push_str(&full_pattern);
            }
            PatternNode::EnumVariant { path, args } => {
                // For root-level enum variants, add the "   | " prefix
                if depth == 0 {
                    self.output.push_str("   | ");
                }

                // Mark position for error if this is the error node
                if is_error {
                    self.error_line = Some(self.current_line);
                    // Position is relative to content after "   | " prefix
                    self.error_col_start = Some(0);
                    self.error_col_end = Some(path.len());
                }

                self.output.push_str(path);

                if let Some(args) = args {
                    if !args.is_empty() {
                        self.output.push('(');
                        for (i, arg) in args.iter().enumerate() {
                            if i > 0 {
                                self.output.push_str(", ");
                            }
                            // Format the argument node
                            self.format_node(arg, depth, error_path, path_index, context_lines);
                        }
                        self.output.push(')');

                        // Update end position if this is the error
                        if is_error {
                            let mut full_len = path.len() + 2; // Add 2 for "()"
                            // Count arg lengths
                            for arg in args.iter() {
                                let arg_str = self.format_inline(arg);
                                full_len += arg_str.len();
                            }
                            if args.len() > 1 {
                                full_len += (args.len() - 1) * 2; // ", " between args
                            }
                            self.error_col_end = Some(full_len);
                        }
                    }
                }

                // Add newline if at root level
                if depth == 0 {
                    self.output.push('\n');
                    self.current_line += 1;
                }
            }
            PatternNode::Rest => {
                self.output.push_str("..");
            }
        }
    }

    fn format_field(
        &mut self,
        name: &str,
        node: &'static PatternNode,
        depth: usize,
        error_path: &[String],
        path_index: usize,
        context_lines: usize,
    ) {
        let indent = "    ".repeat(depth);
        self.output.push_str(&format!("   | {}{}: ", indent, name));

        // Check if this field's node contains the error node somewhere within it
        let contains_error = node_contains(node, self.error_node);

        // If this field contains the error, we need to handle it specially
        if contains_error {
            self.error_line = Some(self.current_line);

            // The prefix before the value (not including "   | " which is position 0)
            let prefix_len = indent.len() + name.len() + 2; // indent + name + ": "

            // If the error is in a tuple field, find the exact position
            if let PatternNode::Tuple { items } = node {
                self.output.push('(');
                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }

                    let item_is_error = std::ptr::eq(*item, self.error_node);

                    if item_is_error {
                        // Calculate position from the start of the line
                        let mut current_pos = prefix_len + 1; // +1 for '('
                        // Add the length of previous items
                        for j in 0..i {
                            let prev_item_str = self.format_inline(items[j]);
                            if j > 0 {
                                current_pos += 2; // ", "
                            }
                            current_pos += prev_item_str.len();
                        }
                        self.error_col_start = Some(current_pos);
                    }

                    let item_str = self.format_inline(item);
                    self.output.push_str(&item_str);

                    if item_is_error {
                        self.error_col_end = Some(self.error_col_start.unwrap() + item_str.len());
                    }
                }
                self.output.push(')');
            } else if std::ptr::eq(node, self.error_node) {
                // The entire field value is the error
                // For Comparison nodes, let format_node handle the position calculation
                if !matches!(node, PatternNode::Comparison { .. }) {
                    self.error_col_start = Some(prefix_len);
                }
                let value_start = self.output.len();
                self.format_node(node, depth, error_path, path_index, context_lines);
                let value_end = self.output.len();
                // Only set end position if we set the start
                if self.error_col_start == Some(prefix_len) {
                    self.error_col_end = Some(prefix_len + (value_end - value_start));
                }
            } else {
                // Some other container that has the error inside it
                self.format_node(node, depth, error_path, path_index, context_lines);
            }
        } else {
            // No error in this field, format normally
            self.format_node(node, depth, error_path, path_index, context_lines);
        }

        self.output.push_str(",\n");
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

fn node_contains(root: &'static PatternNode, target: &'static PatternNode) -> bool {
    if std::ptr::eq(root, target) {
        return true;
    }

    match root {
        PatternNode::Struct { fields, .. } => {
            fields.iter().any(|(_, node)| node_contains(node, target))
        }
        PatternNode::EnumVariant {
            args: Some(args), ..
        } => args.iter().any(|arg| node_contains(arg, target)),
        PatternNode::Slice { items, .. } | PatternNode::Tuple { items } => {
            items.iter().any(|item| node_contains(item, target))
        }
        _ => false,
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
    let mut formatter = TreeFormatter::new(error_node);
    formatter.format_with_context(root, context_lines);

    let location = formatter.get_error_location();

    // Build the full error message
    let mut result = String::from("assert_struct! failed:\n\n");

    let lines: Vec<&str> = formatter.output.lines().collect();

    // Check if this is a simple enum variant at root (single line)
    let is_simple_enum = matches!(root, PatternNode::EnumVariant { .. }) && lines.len() == 1;

    if !is_simple_enum {
        // Build context for nested structures
        let error_path = find_path_to_node(root, error_node, Vec::new());

        // Check if we have a nested struct (not just a tuple/slice element)
        let mut has_nested_struct = false;
        if error_path.len() > 1 {
            // Walk the path to see if we encounter nested structs
            let mut current = root;
            for path_component in error_path.iter().take(error_path.len() - 1) {
                if let PatternNode::Struct { fields, .. } = current {
                    for (field_name, field_node) in fields.iter() {
                        if field_name == path_component {
                            if matches!(field_node, PatternNode::Struct { .. }) {
                                has_nested_struct = true;
                            }
                            current = field_node;
                            break;
                        }
                    }
                }
            }
        }

        if has_nested_struct {
            // We're in a nested structure, show the context
            result.push_str("   | ");

            // Show the root struct name
            if let PatternNode::Struct { name, .. } = root {
                result.push_str(name);
                result.push_str(" { ... ");

                // Find and show the immediate parent of the error
                let mut current = root;
                for path_component in error_path.iter().take(error_path.len() - 1) {
                    if let PatternNode::Struct { fields, .. } = current {
                        for (field_name, field_node) in fields.iter() {
                            if field_name == path_component {
                                if let PatternNode::Struct {
                                    name: nested_name, ..
                                } = field_node
                                {
                                    result.push_str(nested_name);
                                    result.push_str(" {");
                                    break;
                                }
                                current = field_node;
                                break;
                            }
                        }
                    }
                }
            }
            result.push('\n');
        } else {
            // Not nested structs, just show the opening line
            if !lines.is_empty() {
                result.push_str(lines[0]);
                result.push('\n');
            }
        }
    }

    // Add error details
    result.push_str(&format!("{} mismatch:\n", error.error_type));
    result.push_str(&format!(
        "  --> `{}` (line {})\n",
        error.field_path, error.line_number
    ));

    // Add the specific error line with underline
    if location.line_in_pattern < lines.len() {
        // Show the line with the pattern
        let error_line = lines[location.line_in_pattern];

        // The error line already has "   | " prefix from the formatter
        result.push_str(error_line);
        result.push('\n');

        // Add underline
        result.push_str("   | ");
        // The start_col value already accounts for content after "   | " prefix
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

        // Check if the next line is a rest pattern or ellipsis
        if location.line_in_pattern + 1 < lines.len() {
            let next_line = lines[location.line_in_pattern + 1];
            let trimmed = next_line.trim_start().trim_start_matches("| ").trim();
            if trimmed == ".." || trimmed == "..." {
                result.push_str(next_line);
                result.push('\n');
            }
        }
    } else {
        // If location wasn't properly set, just show the actual value
        result.push_str("   |  actual: ");
        result.push_str(&error.actual_value);
        result.push('\n');
    }

    // Show the closing brace only for structs
    if !is_simple_enum && lines.len() > 1 {
        if let Some(last) = lines.last() {
            // Only show if it's a closing brace
            if last.contains('}') {
                result.push_str(last);
                result.push('\n');
            }
        }
    }

    result
}
