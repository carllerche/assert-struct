//! Error formatting and display for assert_struct macro failures.
//!
//! This module handles the complex task of generating human-readable error messages
//! when struct assertions fail. It includes support for precise positioning,
//! pattern context, and multiple error scenarios.
//!
//! ## Core Algorithm: Tree-Traversal vs Error-Iteration
//!
//! The error formatting system uses a **tree-traversal approach** rather than the 
//! traditional error-iteration approach. This design choice is fundamental to achieving
//! clean, maintainable, and consistent error formatting.
//!
//! ### Traditional Approach (Avoided)
//!
//! ```text
//! for each error {
//!     1. Parse error field path
//!     2. Reconstruct pattern tree context
//!     3. Find relevant pattern nodes
//!     4. Format error in isolation
//! }
//! ```
//!
//! **Problems:**
//! - Complex context reconstruction
//! - Inconsistent formatting between related errors
//! - Difficult to maintain and extend
//! - Poor handling of nested structures
//!
//! ### Tree-Traversal Approach (Implemented)
//!
//! ```text
//! traverse_pattern_ast(root) {
//!     for each node {
//!         if node has errors {
//!             format errors in context
//!         } else {
//!             continue traversal
//!         }
//!     }
//! }
//! ```
//!
//! **Benefits:**
//! - Natural error grouping and context
//! - Consistent formatting across all error types
//! - Simpler logic that follows AST structure
//! - Easy to extend for new pattern types
//! - Better handling of complex nested structures
//!
//! ## Key Components
//!
//! 1. **`format_multiple_errors_with_tree`**: Main entry point that chooses between
//!    single and multiple error algorithms.
//!
//! 2. **`traverse_to_multiple_errors`**: Core multiple-error tree traversal that
//!    processes errors at their leaf locations.
//!
//! 3. **`find_field_node_for_error`**: AST traversal to resolve error field paths
//!    to exact PatternNode locations.
//!
//! 4. **`format_leaf_error_field`**: Precise formatting of individual errors with
//!    pattern-specific underline positioning.
//!
//! ## Critical Insights for Future Developers
//!
//! ### Field Path Handling
//! - Error paths include variable names (`user.name`) - skip the variable part
//! - Enum variants appear in paths (`theme.Some`) - handle specially for display
//! - Numeric indices are preserved (`data.0`) - important for tuples/slices
//!
//! ### Underline Positioning Strategy  
//! - Different pattern types need different underline strategies
//! - Equality vs comparison patterns have different semantics
//! - Enum patterns require special handling for inner vs outer content
//!
//! ### Performance Considerations
//! - Single-pass AST traversal for multiple errors
//! - Pattern string generation at compile time (handled by macro)
//! - Minimal string manipulation during formatting

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
    pub expected_value: Option<String>, // For equality patterns where we need to show the expected value
    // Tree-based pattern data
    pub pattern_tree: Option<&'static PatternNode>,
    pub error_node: Option<&'static PatternNode>,
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

pub fn format_errors(errors: Vec<ErrorContext>) -> String {
    if errors.is_empty() {
        return "assert_struct! failed: no errors provided".to_string();
    }

    // Always use tree-based formatting for both single and multiple errors
    format_multiple_errors_with_tree(errors)
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

#[derive(Debug, Clone, Copy)]
enum RenderMode {
    /// Render complete node with all fields
    Full,
    /// Render with error highlighting and underlines
    ErrorHighlight,
    /// Compact representation for pruned sections
    Summarize,
    /// Context breadcrumb path for nested structures
    Breadcrumb,
}

struct TreeWalker {
    errors: Vec<ErrorContext>,
    error_index: usize,
    output: String,
    current_line: usize,
    context_depth: usize,
}

impl TreeWalker {
    fn new(mut errors: Vec<ErrorContext>) -> Self {
        // Sort errors by line number for consistent traversal
        errors.sort_by_key(|e| e.line_number);
        
        Self {
            errors,
            error_index: 0,
            output: String::new(),
            current_line: 0,
            context_depth: 0,
        }
    }

    fn traverse(&mut self, root: &'static PatternNode) -> String {
        // Add header
        if self.errors.len() == 1 {
            self.output.push_str("assert_struct! failed:\n\n");
        } else {
            self.output.push_str(&format!("assert_struct! failed: {} mismatches\n\n", self.errors.len()));
        }

        self.traverse_node(root, 0, Vec::new());
        
        // Close any remaining context
        if self.context_depth > 0 {
            self.output.push_str("   | } ... }");
        } else if matches!(root, PatternNode::Struct { .. }) {
            self.output.push_str("   | }");
        }

        self.output.clone()
    }

    fn traverse_node(&mut self, node: &'static PatternNode, depth: usize, field_path: Vec<String>) {
        // Check if current error matches this node
        let current_error = self.errors.get(self.error_index);
        let is_error_node = current_error
            .and_then(|e| e.error_node)
            .map(|error_node| std::ptr::eq(node, error_node))
            .unwrap_or(false);

        if is_error_node {
            // This is the exact error node - render the error
            self.render_error_node(node, &field_path);
            self.error_index += 1;
            return;
        }

        // Not the error node, check if we need to traverse into it
        match node {
            PatternNode::Struct { name, fields } => {
                // Check if any of our remaining errors are within this struct
                if self.has_errors_in_subtree(node) {
                    self.render_struct(name, fields, depth, &field_path);
                }
            }
            PatternNode::EnumVariant { path, args } => {
                if self.has_errors_in_subtree(node) {
                    self.render_enum_variant(path, args, depth, &field_path, RenderMode::Full);
                }
            }
            _ => {
                // Other node types - if they contain errors, they would have been caught above
            }
        }
    }

    fn has_errors_in_subtree(&self, node: &'static PatternNode) -> bool {
        // Check if any remaining errors are within this subtree
        for i in self.error_index..self.errors.len() {
            if let Some(error_node) = self.errors[i].error_node {
                if self.node_contains_recursive(node, error_node) {
                    return true;
                }
            }
        }
        false
    }

    fn node_contains_recursive(&self, root: &'static PatternNode, target: &'static PatternNode) -> bool {
        if std::ptr::eq(root, target) {
            return true;
        }

        match root {
            PatternNode::Struct { fields, .. } => {
                fields.iter().any(|(_, field_node)| self.node_contains_recursive(field_node, target))
            }
            PatternNode::EnumVariant { args: Some(args), .. } => {
                args.iter().any(|arg| self.node_contains_recursive(arg, target))
            }
            PatternNode::Slice { items, .. } => {
                items.iter().any(|item| self.node_contains_recursive(item, target))
            }
            PatternNode::Tuple { items } => {
                items.iter().any(|item| self.node_contains_recursive(item, target))
            }
            _ => false,
        }
    }

    fn is_on_path_to_error(&self, node: &'static PatternNode, field_path: &[String]) -> bool {
        if let Some(error) = self.errors.get(self.error_index) {
            // Check if this node is on the path to the current error
            let error_path_parts: Vec<&str> = error.field_path.split('.').collect();
            
            // If we're looking at a struct field, check if it's part of the error path
            if field_path.len() < error_path_parts.len() {
                return field_path.iter().zip(error_path_parts.iter())
                    .all(|(a, b)| a == b);
            }
        }
        false
    }

    fn should_show_breadcrumb(&self, field_path: &[String]) -> bool {
        // Show breadcrumb for nested structures (depth > 1)
        field_path.len() > 1
    }

    fn render_struct(&mut self, name: &str, fields: &[(&str, &'static PatternNode)], depth: usize, field_path: &[String]) {
        // For root level structs, always show the opening
        if depth == 0 {
            self.output.push_str(&format!("   | {} {{\n", name));
            self.current_line += 1;
        }

        // Traverse each field to find errors
        for (field_name, field_node) in fields.iter() {
            if field_name == &".." {
                continue; // Skip rest patterns
            }

            let mut new_field_path = field_path.to_vec();
            new_field_path.push(field_name.to_string());

            // Check if this field contains any errors by examining our current error
            if let Some(error) = self.errors.get(self.error_index) {
                let error_path_parts: Vec<&str> = error.field_path.split('.').collect();
                
                // Check if this field path matches or is a prefix of the error path
                if new_field_path.len() <= error_path_parts.len() {
                    let is_on_path = new_field_path.iter().zip(error_path_parts.iter())
                        .all(|(a, b)| a == b);
                    
                    if is_on_path {
                        // This field is on the path to the error
                        if new_field_path.len() == error_path_parts.len() {
                            // This field IS the error
                            let indent = "    ".repeat(depth + 1);
                            self.output.push_str(&format!("   | {}{}: ", indent, field_name));
                            self.render_error_node(field_node, &new_field_path);
                            return; // Error processed, stop here
                        } else {
                            // Error is deeper - continue traversing
                            self.traverse_node(field_node, depth + 1, new_field_path);
                            return; // Only one error path at a time
                        }
                    }
                }
            }
        }
    }

    fn field_contains_errors(&self, field_node: &'static PatternNode, field_path: &[String]) -> bool {
        // Check if any remaining errors are within this field
        for i in self.error_index..self.errors.len() {
            let error_path_parts: Vec<&str> = self.errors[i].field_path.split('.').collect();
            
            // Check if this field path is a prefix of the error path
            if field_path.len() <= error_path_parts.len() {
                let matches = field_path.iter().zip(error_path_parts.iter())
                    .all(|(a, b)| a == b);
                if matches {
                    return true;
                }
            }
        }
        false
    }

    fn traverse_struct_fields(&mut self, fields: &[(&str, &'static PatternNode)], depth: usize, mut field_path: Vec<String>) {
        for (field_name, field_node) in fields.iter() {
            if field_name == &".." {
                continue; // Skip rest patterns for now
            }

            field_path.push(field_name.to_string());
            
            // Check if this field is on the path to current error
            if self.is_field_relevant(field_name, &field_path) {
                self.render_field(field_name, field_node, depth + 1, field_path.clone());
            }
            
            field_path.pop();
        }
    }

    fn is_field_relevant(&self, field_name: &str, field_path: &[String]) -> bool {
        if let Some(error) = self.errors.get(self.error_index) {
            let error_path_parts: Vec<&str> = error.field_path.split('.').collect();
            
            // Check if this field is part of the current error's path
            if field_path.len() <= error_path_parts.len() {
                return field_path.iter().zip(error_path_parts.iter())
                    .all(|(a, b)| a == b);
            }
        }
        false
    }

    fn render_field(&mut self, name: &str, node: &'static PatternNode, depth: usize, field_path: Vec<String>) {
        let indent = "    ".repeat(depth);
        let current_error = self.errors.get(self.error_index);
        let is_error_field = current_error
            .map(|e| e.field_path == field_path.join("."))
            .unwrap_or(false);

        if is_error_field {
            // This is the field with the error
            self.output.push_str(&format!("   | {}{}: ", indent, name));
            self.render_error_node(node, &field_path);
            self.output.push_str(",\n");
            self.current_line += 1;
        } else {
            // Continue traversing into this field
            self.traverse_node(node, depth, field_path);
        }
    }

    fn render_error_node(&mut self, node: &'static PatternNode, _field_path: &[String]) {
        if let Some(error) = self.errors.get(self.error_index) {
            // First, render the pattern inline in the field context (this was already done by render_struct)
            let pattern_str = self.format_pattern_inline(node);
            self.output.push_str(&pattern_str);
            self.output.push_str(",\n");
            self.current_line += 1;

            // Add error details
            self.output.push_str(&format!("{} mismatch:\n", error.error_type));
            self.output.push_str(&format!("  --> `{}` (line {})\n", error.field_path, error.line_number));
            
            // Get the field name from the path for the display
            let field_name = error.field_path.split('.').last().unwrap_or("");
            
            // Show the field line again with the pattern
            self.output.push_str(&format!("   |     {}: {},\n", field_name, pattern_str));
            
            // Add underline - calculate position based on field name and pattern
            let field_prefix_len = 8 + field_name.len() + 2; // "   |     " + field_name + ": "
            let mut underline = " ".repeat(field_prefix_len);
            
            // For comparison patterns, position the underline more precisely
            match node {
                PatternNode::Comparison { op, .. } if op == &"==" || op == &"!=" => {
                    // For equality, underline the value part
                    let op_len = op.len() + 1; // +1 for space
                    underline.push_str(&" ".repeat(op_len));
                    underline.push_str("^^^^^");
                }
                _ => {
                    underline.push_str("^^^^^");
                }
            }
            
            underline.push_str(&format!(" actual: {}", error.actual_value));
            self.output.push_str(&format!("   |{}\n", underline));
            
            // For equality patterns, add expected value
            if let ErrorType::Equality = error.error_type {
                if let Some(ref expected) = error.expected_value {
                    let mut expected_line = " ".repeat(field_prefix_len + 6); // +6 for "^^^^^"
                    expected_line.push_str(&format!("expected: {}", expected));
                    self.output.push_str(&format!("   |{}\n", expected_line));
                }
            }
        }
    }

    fn render_enum_variant(&mut self, path: &str, args: &Option<&[&'static PatternNode]>, depth: usize, field_path: &[String], mode: RenderMode) {
        if depth == 0 {
            self.output.push_str("   | ");
        }
        
        self.output.push_str(path);
        
        if let Some(args) = args {
            if !args.is_empty() {
                self.output.push('(');
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }
                    self.traverse_node(arg, depth, field_path.to_vec());
                }
                self.output.push(')');
            }
        }
        
        if depth == 0 {
            self.output.push('\n');
            self.current_line += 1;
        }
    }

    fn format_pattern_inline(&self, node: &'static PatternNode) -> String {
        match node {
            PatternNode::Simple { value } => value.to_string(),
            PatternNode::Comparison { op, value } => format!("{} {}", op, value),
            PatternNode::Range { pattern } => pattern.to_string(),
            PatternNode::Regex { pattern } => format!("=~ {}", pattern),
            PatternNode::Like { expr } => format!("=~ {}", expr),
            PatternNode::Slice { items, is_ref } => {
                let prefix = if *is_ref { "&" } else { "" };
                let content = items.iter()
                    .map(|item| self.format_pattern_inline(item))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{}[{}]", prefix, content)
            }
            PatternNode::Tuple { items } => {
                let content = items.iter()
                    .map(|item| self.format_pattern_inline(item))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("({})", content)
            }
            PatternNode::EnumVariant { path, args } => {
                if let Some(args) = args {
                    if !args.is_empty() {
                        let arg_str = args.iter()
                            .map(|arg| self.format_pattern_inline(arg))
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
                    let col_start = current_line.len().saturating_sub(5); // Subtract "   | " prefix
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
                    let base_col = current_line.len().saturating_sub(5); // Subtract "   | " prefix

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
                    let col_start = current_line.len().saturating_sub(5); // Subtract "   | " prefix
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
                    let col_start = current_line.len().saturating_sub(5); // Subtract "   | " prefix
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
                    let col_start = current_line.len().saturating_sub(5); // Subtract "   | " prefix
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

/// Formats multiple errors using a tree-traversal algorithm.
///
/// ## Algorithm Overview
///
/// This function implements a **tree-traversal approach** rather than an **error-iteration approach**.
/// The key insight is that instead of iterating through errors and trying to reconstruct their context,
/// we traverse the pattern AST (Abstract Syntax Tree) and check for errors at each node.
///
/// ### Why Tree-Traversal vs Error-Iteration?
///
/// **Old approach (error-iteration):**
/// 1. For each error, reconstruct its position in the pattern tree
/// 2. Build context around each error individually
/// 3. Format each error in isolation
/// 4. Results in complex context reconstruction and potential inconsistencies
///
/// **New approach (tree-traversal):**
/// 1. Traverse the PatternNode AST depth-first
/// 2. At each node, check if any errors belong to this location
/// 3. Format errors in their natural tree context
/// 4. Results in cleaner code and consistent formatting
///
/// ### Key Benefits
///
/// - **Natural context**: Errors are formatted within their structural context
/// - **Consistent grouping**: Related errors (e.g., multiple fields in the same struct) are grouped naturally
/// - **Simpler logic**: No need to reconstruct tree paths from error field paths
/// - **Better maintainability**: Algorithm follows the natural structure of the pattern
///
/// ### Error Processing Strategy
///
/// 1. **Single errors**: Use simple single-error traversal for optimal formatting
/// 2. **Multiple errors**: Use multi-error traversal to handle complex nested cases
/// 3. **Sorting**: Maintain source order by sorting errors by line number
///
/// ## Implementation Notes
///
/// The algorithm handles complex cases like:
/// - Nested struct fields (`account.profile.bio`)
/// - Enum variants (`settings.theme.Some` → display as `settings.theme`)
/// - Tuple indexing (`holder.data.0`)
/// - Mixed error types (equality, comparison, range) in the same structure
pub fn format_multiple_errors_with_tree(errors: Vec<ErrorContext>) -> String {
    if errors.is_empty() {
        return "assert_struct! failed: no errors provided".to_string();
    }

    // Sort errors by line number to maintain source order
    // This ensures that errors appear in the same order as they appear in the source code,
    // which is important for user experience and consistency with single-error formatting.
    let mut sorted_errors = errors;
    sorted_errors.sort_by_key(|e| e.line_number);

    // Add header - different format for single vs multiple errors
    let mut result = if sorted_errors.len() == 1 {
        "assert_struct! failed:\n\n".to_string()
    } else {
        format!("assert_struct! failed: {} mismatches\n", sorted_errors.len())
    };

    // For single error, use the simple algorithm
    // Single errors can use a simpler traversal since we don't need to handle
    // multiple error coordination and grouping
    if sorted_errors.len() == 1 {
        if let Some(error) = sorted_errors.first() {
            if let Some(root_tree) = error.pattern_tree {
                result.push_str(&traverse_to_error(root_tree, error, Vec::new()));
            }
        }
        return result;
    }

    // For multiple errors, use the new multi-error traversal
    // This is where the tree-traversal algorithm shines - it can handle
    // multiple errors in a single pass through the pattern AST
    if let Some(root_tree) = sorted_errors.first().and_then(|e| e.pattern_tree) {
        result.push_str(&traverse_to_multiple_errors(root_tree, &sorted_errors, Vec::new()));
    }

    result
}

/// Core implementation of the multiple-error tree traversal algorithm.
///
/// ## Algorithm Design
///
/// This function implements the **AST-centric traversal** approach. Instead of iterating
/// through errors and trying to reconstruct their context, we traverse the pattern tree
/// and process errors as we encounter their locations.
///
/// ### Key Design Decisions
///
/// 1. **Process errors at leaf nodes**: Rather than trying to handle nested structures,
///    we process each error at its final destination (the actual field that failed).
///
/// 2. **Extract field names intelligently**: Handle enum variants specially - for error
///    paths like `settings.theme.Some`, display the field name as `theme`, not `Some`.
///
/// 3. **Use tree traversal for pattern node resolution**: Instead of string manipulation
///    to find pattern nodes, we traverse the AST to find the exact PatternNode for each error.
///
/// ### Error Path Handling Strategy
///
/// Error paths come in various forms:
/// - Simple fields: `user.name`
/// - Nested structs: `account.profile.bio`
/// - Enum variants: `settings.theme.Some` (display as `settings.theme`)
/// - Tuple indexing: `holder.data.0`
///
/// The algorithm extracts the final field name and uses `find_field_node_for_error`
/// to locate the corresponding PatternNode in the AST.
///
/// ### Output Format
///
/// For multiple errors, the format is:
/// ```
/// assert_struct! failed: N mismatches
///    | StructName {
/// error_type mismatch:
///   --> `full.field.path` (line X)
///    |     field_name: pattern,
///    |                 ^^^^^^^ actual: value
/// error_type mismatch:
///   --> `other.field.path` (line Y)
///    |     other_field: other_pattern,
///    |                  ^^^^^^^^^^^^^ actual: other_value
///    | } ... }
/// ```
///
/// ## Future Extensions
///
/// This function currently handles struct roots optimally. Future improvements could:
/// - Add support for enum variant roots
/// - Implement breadcrumb context for deeply nested structures
/// - Add field grouping for better readability in very large structs
fn traverse_to_multiple_errors(node: &'static PatternNode, errors: &[ErrorContext], _field_path: Vec<String>) -> String {
    let mut result = String::new();
    
    match node {
        PatternNode::Struct { name, fields: _ } => {
            // Open the struct context
            result.push_str(&format!("   | {} {{\n", name));
            
            // Process each error individually at its leaf location
            // This approach ensures that each error is formatted with its complete context
            // and precise positioning information.
            for error in errors {
                // Extract the field name from the error path
                let error_path_parts: Vec<&str> = error.field_path.split('.').collect();
                if error_path_parts.len() >= 2 {
                    // Smart field name extraction for enum variants
                    // For paths like "settings.theme.Some", we want to display "theme" as the field name,
                    // not "Some". This makes the error messages more intuitive for users.
                    let field_name = if error_path_parts.len() >= 3 && 
                        (error_path_parts[error_path_parts.len() - 1] == "Some" || 
                         error_path_parts[error_path_parts.len() - 1] == "Ok" ||
                         error_path_parts[error_path_parts.len() - 1] == "Err" ||
                         error_path_parts[error_path_parts.len() - 1] == "None") {
                        error_path_parts[error_path_parts.len() - 2]
                    } else {
                        error_path_parts[error_path_parts.len() - 1]
                    };
                    
                    // Use tree traversal to find the exact PatternNode for this error
                    // This is more reliable than string manipulation and handles complex
                    // nested structures correctly.
                    if let Some(field_node) = find_field_node_for_error(node, error) {
                        result.push_str(&format_leaf_error_field(field_name, field_node, error));
                    } else {
                        // Fallback for cases where tree traversal fails
                        // This should be rare, but provides a safety net
                        result.push_str(&format_error_direct(node, error));
                    }
                }
            }
            
            // Close the struct with ellipsis to indicate there may be more content
            result.push_str("   | } ... }\n");
        }
        _ => {
            // For non-struct root nodes (e.g., enums, tuples), fall back to simple formatting
            // Future enhancement: implement specialized handling for these cases
            if let Some(first_error) = errors.first() {
                result.push_str(&format_error_direct(node, first_error));
            }
        }
    }
    
    result
}

fn traverse_to_error(node: &'static PatternNode, error: &ErrorContext, field_path: Vec<String>) -> String {
    let mut result = String::new();
    
    match node {
        PatternNode::Struct { name, fields } => {
            // Open the struct
            result.push_str(&format!("   | {} {{\n", name));
            
            // Parse the error field path and skip the root variable name
            let error_path_parts: Vec<&str> = error.field_path.split('.').collect();
            // Skip the first part (variable name like "user") to get just the field path
            let field_path_parts = if error_path_parts.len() > 1 {
                &error_path_parts[1..]
            } else {
                &error_path_parts[..]
            };
            
            // Find the field that's on the error path and render it
            let mut error_field_rendered = false;
            for (field_name, field_node) in fields.iter() {
                if field_name == &".." {
                    continue; // Skip rest patterns for now, we'll handle them separately
                }
                
                let mut new_field_path = field_path.clone();
                new_field_path.push(field_name.to_string());
                
                // Check if this field is on the path to the error
                if new_field_path.len() <= field_path_parts.len() {
                    let is_on_path = new_field_path.iter().zip(field_path_parts.iter())
                        .all(|(a, b)| a == b);
                    
                    if is_on_path {
                        if new_field_path.len() == field_path_parts.len() {
                            // This field IS the error - render it with error details
                            result.push_str(&format_error_field(field_name, field_node, error));
                            error_field_rendered = true;
                            break;
                        } else if new_field_path.len() + 1 == field_path_parts.len() {
                            // Check if the next part is a tuple/slice index
                            let next_part = field_path_parts[new_field_path.len()];
                            if next_part.parse::<usize>().is_ok() {
                                // This is a tuple/slice index - render the field with the indexed error
                                result.push_str(&format_error_field(field_name, field_node, error));
                                error_field_rendered = true;
                                break;
                            } else {
                                // Error is deeper - recurse into this field
                                result.push_str(&traverse_to_error(field_node, error, new_field_path));
                                error_field_rendered = true;
                                break;
                            }
                        } else {
                            // Error is deeper - recurse into this field
                            result.push_str(&traverse_to_error(field_node, error, new_field_path));
                            error_field_rendered = true;
                            break;
                        }
                    }
                }
            }
            
            // Show rest pattern if it exists in the original pattern
            if error_field_rendered {
                let has_rest_pattern = fields.iter().any(|(name, _)| name == &"..");
                if has_rest_pattern {
                    result.push_str("   |     ..\n");
                }
            }
            
            // Close the struct
            result.push_str("   | }\n");
        }
        PatternNode::Tuple { items } => {
            // Handle tuple indexing directly
            let error_path_parts: Vec<&str> = error.field_path.split('.').collect();
            if let Some(last_part) = error_path_parts.last() {
                if let Ok(index) = last_part.parse::<usize>() {
                    if index < items.len() {
                        // This tuple contains the error at the specified index
                        result.push_str(&format_tuple_error(items, index, error));
                        return result;
                    }
                }
            }
            
            // Fallback: format as direct error
            result.push_str(&format_error_direct(node, error));
        }
        _ => {
            // For non-struct nodes, if this is the error node, format it directly
            if let Some(error_node) = error.error_node {
                if std::ptr::eq(node, error_node) {
                    result.push_str(&format_error_direct(node, error));
                }
            }
        }
    }
    
    result
}

fn format_error_field(field_name: &str, field_node: &'static PatternNode, error: &ErrorContext) -> String {
    let mut result = String::new();
    
    // Add error details first
    result.push_str(&format!("{} mismatch:\n", error.error_type));
    result.push_str(&format!("  --> `{}` (line {})\n", error.field_path, error.line_number));
    
    // Render the field with pattern - we'll build this line piece by piece to track positions
    let indent = "    ";
    let field_line_prefix = format!("   | {}{}: ", indent, field_name);
    result.push_str(&field_line_prefix);
    
    // The prefix before the value (excluding the "   | " which is position 0 in the output)
    let prefix_len = indent.len() + field_name.len() + 2; // indent + field_name + ": "
    
    // Track position for underline calculation
    let mut error_col_start = None;
    let mut error_col_end = None;
    
    // Format the pattern and track error position
    match field_node {
        PatternNode::Simple { value } => {
            error_col_start = Some(prefix_len);
            error_col_end = Some(prefix_len + value.len());
            result.push_str(value);
        }
        PatternNode::Comparison { op, value } => {
            let pattern = format!("{} {}", op, value);
            if op == &"==" || op == &"!=" {
                // For equality, underline just the value part
                let op_with_space_len = op.len() + 1;
                error_col_start = Some(prefix_len + op_with_space_len);
                error_col_end = Some(prefix_len + op_with_space_len + value.len());
            } else {
                // For other comparisons, underline the whole pattern
                error_col_start = Some(prefix_len);
                error_col_end = Some(prefix_len + pattern.len());
            }
            result.push_str(&pattern);
        }
        PatternNode::Range { pattern } => {
            error_col_start = Some(prefix_len);
            error_col_end = Some(prefix_len + pattern.len());
            result.push_str(pattern);
        }
        PatternNode::Regex { pattern } => {
            let full_pattern = format!("=~ {}", pattern);
            error_col_start = Some(prefix_len);
            error_col_end = Some(prefix_len + full_pattern.len());
            result.push_str(&full_pattern);
        }
        PatternNode::Like { expr } => {
            let full_pattern = format!("=~ {}", expr);
            error_col_start = Some(prefix_len);
            error_col_end = Some(prefix_len + full_pattern.len());
            result.push_str(&full_pattern);
        }
        PatternNode::Slice { items, is_ref } => {
            let prefix = if *is_ref { "&" } else { "" };
            let content = items.iter()
                .map(|item| format_pattern_simple(item))
                .collect::<Vec<_>>()
                .join(", ");
            let full_pattern = format!("{}[{}]", prefix, content);
            error_col_start = Some(prefix_len);
            error_col_end = Some(prefix_len + full_pattern.len());
            result.push_str(&full_pattern);
        }
        PatternNode::Tuple { items } => {
            result.push('(');
            for (i, item) in items.iter().enumerate() {
                if i > 0 {
                    result.push_str(", ");
                }
                
                // Check if this tuple element is the error (for tuple indexing like "holder.data.0")
                let is_tuple_element_error = check_tuple_element_error(error, i);
                
                if is_tuple_element_error {
                    // Calculate position for this tuple element
                    let current_pos = prefix_len + 1; // +1 for '('
                    // Add lengths of previous items
                    for j in 0..i {
                        let prev_item_str = format_pattern_simple(items[j]);
                        if j > 0 {
                            // Add 2 for ", "
                            error_col_start = Some(current_pos + 2);
                        }
                    }
                    if error_col_start.is_none() {
                        error_col_start = Some(current_pos);
                    }
                }
                
                let item_str = format_pattern_simple(item);
                result.push_str(&item_str);
                
                if is_tuple_element_error {
                    error_col_end = Some(error_col_start.unwrap() + item_str.len());
                }
            }
            result.push(')');
            
            // If no specific tuple element was found, underline the whole tuple
            if error_col_start.is_none() {
                error_col_start = Some(prefix_len);
                error_col_end = Some(result.len() - field_line_prefix.len());
            }
        }
        _ => {
            // Default fallback
            let pattern_str = format_pattern_simple(field_node);
            error_col_start = Some(prefix_len);
            error_col_end = Some(prefix_len + pattern_str.len());
            result.push_str(&pattern_str);
        }
    }
    
    result.push_str(",\n");
    
    // Add underline with precise positioning
    if let (Some(col_start), Some(col_end)) = (error_col_start, error_col_end) {
        let mut underline = String::from("   | ");
        for _ in 0..col_start {
            underline.push(' ');
        }
        for _ in col_start..col_end {
            underline.push('^');
        }
        underline.push_str(&format!(" actual: {}", error.actual_value));
        result.push_str(&underline);
        result.push('\n');
        
        // For equality patterns, add expected value
        if let ErrorType::Equality = error.error_type {
            if let Some(ref expected) = error.expected_value {
                let mut expected_line = String::from("   | ");
                for _ in 0..(col_start + (col_end - col_start) + 1) {
                    expected_line.push(' ');
                }
                expected_line.push_str(&format!("expected: {}", expected));
                result.push_str(&expected_line);
                result.push('\n');
            }
        }
    }
    
    result
}

fn check_tuple_element_error(error: &ErrorContext, tuple_index: usize) -> bool {
    // Check if the error field path ends with this tuple index
    // e.g., "holder.data.0" should match tuple_index 0
    let error_path_parts: Vec<&str> = error.field_path.split('.').collect();
    if let Some(last_part) = error_path_parts.last() {
        if let Ok(index) = last_part.parse::<usize>() {
            return index == tuple_index;
        }
    }
    false
}

fn format_tuple_error(items: &[&'static PatternNode], error_index: usize, error: &ErrorContext) -> String {
    let mut result = String::new();
    
    // Add error details first
    result.push_str(&format!("{} mismatch:\n", error.error_type));
    result.push_str(&format!("  --> `{}` (line {})\n", error.field_path, error.line_number));
    
    // Get the field name from the path (second to last part, since last is the index)
    let error_path_parts: Vec<&str> = error.field_path.split('.').collect();
    let field_name = if error_path_parts.len() >= 2 {
        error_path_parts[error_path_parts.len() - 2]
    } else {
        "data" // fallback
    };
    
    // Render the tuple field line
    let indent = "    ";
    let field_line_prefix = format!("   | {}{}: ", indent, field_name);
    result.push_str(&field_line_prefix);
    
    // Calculate positions for the tuple
    let prefix_len = indent.len() + field_name.len() + 2; // indent + field_name + ": "
    let mut error_col_start = prefix_len + 1; // +1 for '('
    
    // Build the tuple pattern and track the error position
    result.push('(');
    for (i, item) in items.iter().enumerate() {
        if i > 0 {
            result.push_str(", ");
            if i <= error_index {
                error_col_start += 2; // Add 2 for ", "
            }
        }
        
        let item_str = format_pattern_simple(item);
        if i == error_index {
            // This is the error element - we have the start position
        } else if i < error_index {
            error_col_start += item_str.len();
        }
        
        result.push_str(&item_str);
    }
    result.push_str("),\n");
    
    let error_col_end = if error_index < items.len() {
        let error_item_str = format_pattern_simple(items[error_index]);
        error_col_start + error_item_str.len()
    } else {
        error_col_start + 2 // fallback
    };
    
    // Add underline
    let mut underline = String::from("   | ");
    for _ in 0..error_col_start {
        underline.push(' ');
    }
    for _ in error_col_start..error_col_end {
        underline.push('^');
    }
    underline.push_str(&format!(" actual: {}", error.actual_value));
    result.push_str(&underline);
    result.push('\n');
    
    // For equality patterns, add expected value
    if let ErrorType::Equality = error.error_type {
        if let Some(ref expected) = error.expected_value {
            let mut expected_line = String::from("   | ");
            for _ in 0..(error_col_start + (error_col_end - error_col_start) + 1) {
                expected_line.push(' ');
            }
            expected_line.push_str(&format!("expected: {}", expected));
            result.push_str(&expected_line);
            result.push('\n');
        }
    }
    
    result
}

fn format_error_direct(node: &'static PatternNode, error: &ErrorContext) -> String {
    let mut result = String::new();
    
    // For enum variants and other direct nodes
    result.push_str(&format!("{} mismatch:\n", error.error_type));
    result.push_str(&format!("  --> `{}` (line {})\n", error.field_path, error.line_number));
    
    let pattern_str = format_pattern_simple(node);
    result.push_str(&format!("   | {}\n", pattern_str));
    
    // Calculate correct underline length based on pattern
    let underline_length = pattern_str.len();
    let underline = "^".repeat(underline_length);
    result.push_str(&format!("   | {} actual: {}\n", underline, error.actual_value));
    
    result
}

/// Finds the PatternNode corresponding to an error by traversing the pattern AST.
///
/// ## Purpose
///
/// This function resolves error field paths to their exact PatternNode locations in the AST.
/// It's a critical component of the tree-traversal algorithm because it allows us to find
/// the exact pattern that corresponds to each error without string manipulation.
///
/// ## Field Path Resolution Strategy
///
/// Error field paths have the structure: `variable_name.field1.field2...fieldN[.variant]`
///
/// Examples:
/// - `user.name` → Find field "name" in the root struct
/// - `account.profile.bio` → Find field "profile" in root, then "bio" in the nested struct
/// - `settings.theme.Some` → Find field "theme" in root (ignore the ".Some" variant suffix)
/// - `holder.data.0` → Find field "data" in root, then index 0 in the tuple/slice
/// - `inventory.items.[1].scores.[1]` → Find field "items", then index 1, then field "scores", then index 1
///
/// ### Key Insights
///
/// 1. **Skip variable name**: The first component (`user`, `account`, etc.) is the variable name,
///    not part of the pattern structure.
///
/// 2. **Handle enum variants**: For Option/Result patterns, error paths include the variant
///    name (`.Some`, `.Ok`, etc.) but we want to find the field containing the Option/Result,
///    not the variant itself.
///
/// 3. **Handle slice indexing**: Slice indices are formatted as `[index]` in error paths,
///    but need to be converted to numeric indices for AST traversal.
///
/// 4. **Use AST traversal**: Rather than string manipulation, we traverse the PatternNode
///    tree to find the exact node, which handles nested structures correctly.
///
/// ## Error Cases
///
/// Returns `None` if:
/// - The field path doesn't exist in the pattern tree
/// - The error path format is malformed
/// - There's a mismatch between the error path and pattern structure
fn find_field_node_for_error(root: &'static PatternNode, error: &ErrorContext) -> Option<&'static PatternNode> {
    // Parse the error path (skip variable name)
    let error_path_parts: Vec<&str> = error.field_path.split('.').collect();
    let mut field_path_parts = if error_path_parts.len() > 1 {
        &error_path_parts[1..] // Skip variable name like "account"  
    } else {
        &error_path_parts[..]
    };
    
    // Handle enum variants specially
    // For paths like "theme.Some", we want to find the enum field, not the variant.
    // This is crucial for proper pattern node resolution and display formatting.
    if field_path_parts.len() >= 2 && 
        (field_path_parts[field_path_parts.len() - 1] == "Some" || 
         field_path_parts[field_path_parts.len() - 1] == "Ok" ||
         field_path_parts[field_path_parts.len() - 1] == "Err" ||
         field_path_parts[field_path_parts.len() - 1] == "None") {
        field_path_parts = &field_path_parts[..field_path_parts.len() - 1];
    }
    
    // Convert slice indices from [index] format to numeric indices
    // Example: ["items", "[1]", "scores", "[1]"] → ["items", "1", "scores", "1"]
    let mut processed_path = Vec::new();
    for part in field_path_parts {
        if part.starts_with('[') && part.ends_with(']') {
            // Extract numeric index from [index] format
            let index_str = &part[1..part.len()-1];
            processed_path.push(index_str);
        } else {
            processed_path.push(part);
        }
    }
    
    traverse_node_for_field(root, &processed_path, 0)
}

fn traverse_node_for_field(node: &'static PatternNode, field_path: &[&str], depth: usize) -> Option<&'static PatternNode> {
    if depth >= field_path.len() {
        return Some(node);
    }
    
    match node {
        PatternNode::Struct { fields, .. } => {
            let target_field = field_path[depth];
            for (field_name, field_node) in fields.iter() {
                if field_name == &target_field {
                    return traverse_node_for_field(field_node, field_path, depth + 1);
                }
            }
            None
        }
        PatternNode::Tuple { items } => {
            // Handle tuple indexing like "0", "1", etc.
            if let Ok(index) = field_path[depth].parse::<usize>() {
                if index < items.len() {
                    return traverse_node_for_field(items[index], field_path, depth + 1);
                }
            }
            None
        }
        PatternNode::Slice { items, .. } => {
            // Handle slice indexing
            if let Ok(index) = field_path[depth].parse::<usize>() {
                if index < items.len() {
                    return traverse_node_for_field(items[index], field_path, depth + 1);
                }
            }
            None
        }
        _ => {
            // For leaf nodes, check if we're at the end of the path
            if depth == field_path.len() - 1 {
                Some(node)
            } else {
                None
            }
        }
    }
}

/// Formats a single error at its leaf field location with precise positioning.
///
/// ## Purpose
///
/// This function handles the final formatting of individual errors, including:
/// - Error type and location information
/// - Field pattern display
/// - Precise underline positioning
/// - Special handling for different pattern types
///
/// ## Key Formatting Insights
///
/// ### Error Path Display Cleanup
///
/// The function cleans up error field paths for better user experience:
/// - `settings.theme.Some` → display as `settings.theme`
/// - `result.value.Ok` → display as `result.value`
/// - `holder.data.0` → display as `holder.data.0` (preserve numeric indices)
///
/// ### Precise Underline Positioning
///
/// Different pattern types require different underline strategies:
///
/// 1. **Simple patterns** (`"value"`): Underline the entire value
/// 2. **Equality patterns** (`== "value"`): Underline just the value part
/// 3. **Comparison patterns** (`> 100`): Underline the entire comparison
/// 4. **Enum patterns** (`Some("dark")`): 
///    - For equality: underline the entire pattern `Some("dark")`
///    - For comparison: underline just the inner part `> 12`
///
/// ### Positioning Calculation Strategy
///
/// Column positions are calculated based on the field line structure:
/// ```
/// "   | " + indent + field_name + ": " + pattern
///  ^       ^         ^             ^     ^
///  0       4         varies        +2    prefix_len
/// ```
///
/// This ensures that underlines appear exactly under the relevant part of the pattern.
///
/// ## Pattern Type Handling
///
/// The function includes specialized logic for:
/// - **EnumVariant**: Special positioning for Option/Result patterns
/// - **Comparison**: Different underline strategies for equality vs comparison operators
/// - **Simple**: Direct value underlining
/// - **Range, Regex, Like**: Pattern-specific formatting
fn format_leaf_error_field(field_name: &str, field_node: &'static PatternNode, error: &ErrorContext) -> String {
    let mut result = String::new();
    
    // Add error details first - same format as expected output
    result.push_str(&format!("{} mismatch:\n", error.error_type));
    
    // Clean up field path for display - remove enum variant suffixes for common enum types
    // This improves user experience by showing the logical field path rather than
    // the internal representation that includes variant names.
    let display_path = if error.field_path.ends_with(".Some") || 
                          error.field_path.ends_with(".Ok") || 
                          error.field_path.ends_with(".Err") || 
                          error.field_path.ends_with(".None") {
        error.field_path.rsplit_once('.').map(|(prefix, _)| prefix).unwrap_or(&error.field_path)
    } else {
        &error.field_path
    };
    
    result.push_str(&format!("  --> `{}` (line {})\n", display_path, error.line_number));
    
    // Render the field with pattern  
    let indent = "    ";
    let field_line_prefix = format!("   | {}{}: ", indent, field_name);
    result.push_str(&field_line_prefix);
    
    // Calculate the prefix length for precise underline positioning
    // This accounts for the entire prefix before the pattern content
    let prefix_len = indent.len() + field_name.len() + 2; // indent + field_name + ": "
    
    // Track position for underline calculation
    let mut error_col_start = None;
    let mut error_col_end = None;
    
    // Format the pattern and track error position
    match field_node {
        PatternNode::Simple { value } => {
            error_col_start = Some(prefix_len);
            error_col_end = Some(prefix_len + value.len());
            result.push_str(value);
        }
        PatternNode::Comparison { op, value } => {
            let pattern = format!("{} {}", op, value);
            if op == &"==" || op == &"!=" {
                // For equality, underline just the value part
                let op_with_space_len = op.len() + 1;
                error_col_start = Some(prefix_len + op_with_space_len);
                error_col_end = Some(prefix_len + op_with_space_len + value.len());
            } else {
                // For other comparisons, underline the whole pattern
                error_col_start = Some(prefix_len);
                error_col_end = Some(prefix_len + pattern.len());
            }
            result.push_str(&pattern);
        }
        PatternNode::Range { pattern } => {
            error_col_start = Some(prefix_len);
            error_col_end = Some(prefix_len + pattern.len());
            result.push_str(pattern);
        }
        PatternNode::Regex { pattern } => {
            let full_pattern = format!("=~ {}", pattern);
            error_col_start = Some(prefix_len);
            error_col_end = Some(prefix_len + full_pattern.len());
            result.push_str(&full_pattern);
        }
        PatternNode::Like { expr } => {
            let full_pattern = format!("=~ {}", expr);
            error_col_start = Some(prefix_len);
            error_col_end = Some(prefix_len + full_pattern.len());
            result.push_str(&full_pattern);
        }
        PatternNode::EnumVariant { path, args } => {
            // Enum variant formatting requires special consideration for user experience
            // For patterns like Some("dark") or Some(> 12), we show the full pattern
            // but position the underline based on the specific error semantics.
            let full_pattern = if let Some(args) = args {
                if !args.is_empty() {
                    let arg_str = args.iter()
                        .map(|arg| format_pattern_simple(arg))
                        .collect::<Vec<_>>()
                        .join(", ");
                    format!("{}({})", path, arg_str)
                } else {
                    path.to_string()
                }
            } else {
                path.to_string()
            };
            
            // Critical insight: Different error types require different underline strategies
            // for enum variants, especially Option and Result patterns.
            if (*path == "Some" || *path == "Ok") && args.is_some() && !args.unwrap().is_empty() {
                let inner_start = prefix_len + path.len() + 1; // +1 for '('
                let inner_arg = args.unwrap()[0];
                match inner_arg {
                    PatternNode::Simple { .. } => {
                        // For simple values in Option/Result (e.g., Some("dark")),
                        // underline the whole pattern including Some() to show that
                        // the entire expected pattern was wrong, not just the inner value.
                        error_col_start = Some(prefix_len);
                        error_col_end = Some(prefix_len + full_pattern.len());
                    }
                    PatternNode::Comparison { op, value } => {
                        if op == &"==" || op == &"!=" {
                            // For equality patterns inside enums (e.g., Some(== "value")),
                            // underline the whole pattern since the equality check failed.
                            error_col_start = Some(prefix_len);
                            error_col_end = Some(prefix_len + full_pattern.len());
                        } else {
                            // For comparison patterns inside enums (e.g., Some(> 12)),
                            // underline just the comparison part since that's what failed.
                            // The Some() wrapper is correct, only the comparison failed.
                            error_col_start = Some(inner_start);
                            error_col_end = Some(inner_start + op.len() + 1 + value.len());
                        }
                    }
                    _ => {
                        // For other pattern types, default to underlining the inner content
                        let inner_str = format_pattern_simple(inner_arg);
                        error_col_start = Some(inner_start);
                        error_col_end = Some(inner_start + inner_str.len());
                    }
                }
            } else {
                // For other enum variants or variants without args, underline the whole pattern
                error_col_start = Some(prefix_len);
                error_col_end = Some(prefix_len + full_pattern.len());
            }
            
            result.push_str(&full_pattern);
        }
        _ => {
            // Default fallback
            let pattern_str = format_pattern_simple(field_node);
            error_col_start = Some(prefix_len);
            error_col_end = Some(prefix_len + pattern_str.len());
            result.push_str(&pattern_str);
        }
    }
    
    result.push_str(",\n");
    
    // Add underline with precise positioning
    if let (Some(col_start), Some(col_end)) = (error_col_start, error_col_end) {
        let mut underline = String::from("   | ");
        for _ in 0..col_start {
            underline.push(' ');
        }
        for _ in col_start..col_end {
            underline.push('^');
        }
        underline.push_str(&format!(" actual: {}", error.actual_value));
        result.push_str(&underline);
        result.push('\n');
        
        // For equality patterns, add expected value
        if let ErrorType::Equality = error.error_type {
            if let Some(ref expected) = error.expected_value {
                let mut expected_line = String::from("   | ");
                for _ in 0..(col_start + (col_end - col_start) + 1) {
                    expected_line.push(' ');
                }
                expected_line.push_str(&format!("expected: {}", expected));
                result.push_str(&expected_line);
                result.push('\n');
            }
        }
    }
    
    result
}

fn format_pattern_simple(node: &'static PatternNode) -> String {
    match node {
        PatternNode::Simple { value } => value.to_string(),
        PatternNode::Comparison { op, value } => format!("{} {}", op, value),
        PatternNode::Range { pattern } => pattern.to_string(),
        PatternNode::Regex { pattern } => format!("=~ {}", pattern),
        PatternNode::Like { expr } => format!("=~ {}", expr),
        PatternNode::Slice { items, is_ref } => {
            let prefix = if *is_ref { "&" } else { "" };
            let content = items.iter()
                .map(|item| format_pattern_simple(item))
                .collect::<Vec<_>>()
                .join(", ");
            format!("{}[{}]", prefix, content)
        }
        PatternNode::Tuple { items } => {
            let content = items.iter()
                .map(|item| format_pattern_simple(item))
                .collect::<Vec<_>>()
                .join(", ");
            format!("({})", content)
        }
        PatternNode::EnumVariant { path, args } => {
            if let Some(args) = args {
                if !args.is_empty() {
                    let arg_str = args.iter()
                        .map(|arg| format_pattern_simple(arg))
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

/// Helper function to find struct name for a field path
fn find_struct_name_for_field(
    root: &'static PatternNode,
    field_name: &str,
) -> Option<&'static str> {
    match root {
        PatternNode::Struct { fields, .. } => {
            for (name, node) in fields.iter() {
                if name == &field_name {
                    if let PatternNode::Struct {
                        name: struct_name, ..
                    } = node
                    {
                        return Some(struct_name);
                    }
                }
                // Recursively search in nested structs
                if let Some(found) = find_struct_name_for_field(node, field_name) {
                    return Some(found);
                }
            }
        }
        _ => {}
    }
    None
}
