//! Error formatting and display for assert_struct macro failures.
//!
//! This module handles the complex task of generating human-readable error messages
//! when struct assertions fail. It uses a two-pass system that separates building
//! the structural representation from rendering it to a string.
//!
//! ## Two-Pass Architecture
//!
//! Pass 1: Build a structural representation of errors
//! - Traverse the pattern tree to find error locations
//! - Build Fragment types representing the pattern structure
//! - Track breadcrumbs for nested contexts
//!
//! Pass 2: Render the structure to a formatted string
//! - Convert Fragments to text with proper indentation
//! - Track positions for underline annotations
//! - Maintain consistent formatting across error types

use std::fmt;

// ========== CORE DATA STRUCTURES ==========

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

    // Enum patterns
    EnumVariant {
        path: &'static str,
        args: Option<&'static [&'static PatternNode]>,
    },

    // Leaf patterns
    Simple {
        value: &'static str,
    },
    Comparison {
        op: &'static str,
        value: &'static str,
    },
    Range {
        pattern: &'static str,
    },
    Regex {
        pattern: &'static str,
    },
    Like {
        expr: &'static str,
    },

    // Special
    Rest,
}

impl fmt::Display for PatternNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PatternNode::Struct { name, .. } => write!(f, "{} {{ ... }}", name),
            PatternNode::Slice { is_ref, .. } => {
                if *is_ref {
                    write!(f, "&[...]")
                } else {
                    write!(f, "[...]")
                }
            }
            PatternNode::Tuple { items } => write!(f, "({})", ".., ".repeat(items.len())),
            PatternNode::EnumVariant { path, args } => {
                if args.is_some() {
                    write!(f, "{}(...)", path)
                } else {
                    write!(f, "{}", path)
                }
            }
            PatternNode::Simple { value } => write!(f, "{}", value),
            PatternNode::Comparison { op, value } => write!(f, "{} {}", op, value),
            PatternNode::Range { pattern } => write!(f, "{}", pattern),
            PatternNode::Regex { pattern } => write!(f, "=~ {}", pattern),
            PatternNode::Like { expr } => write!(f, "=~ {}", expr),
            PatternNode::Rest => write!(f, ".."),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ErrorContext {
    pub field_path: String,
    pub pattern_str: String,
    pub actual_value: String,
    pub line_number: u32,
    pub file_name: &'static str,
    pub error_type: ErrorType,
    pub expected_value: Option<String>, // For equality patterns where we need to show the expected value
    // Tree-based pattern data - only the specific node that failed
    pub error_node: Option<&'static PatternNode>,
}

#[derive(Debug, Clone)]
pub enum ErrorType {
    Comparison,
    Range,
    Regex,
    Value,
    EnumVariant,
    Slice,
    Equality, // For == patterns where we show both actual and expected
}

// ========== TWO-PASS ERROR RENDERING SYSTEM ==========

/// Main structure representing all error output
#[derive(Debug)]
pub struct ErrorDisplay {
    /// Header like "assert_struct! failed" or with error count
    pub header: String,
    /// One section per error
    pub sections: Vec<ErrorSection>,
    /// Closing breadcrumbs for opened structs (e.g., "}", "} ... }")
    pub closing_breadcrumbs: Vec<String>,
}

/// A single error section in the output
#[derive(Debug)]
pub struct ErrorSection {
    /// Opening breadcrumbs showing path to error (e.g., ["Response {", "... Profile {"])
    pub opening_breadcrumbs: Vec<String>,
    /// Location information for the error
    pub location: ErrorLocation,
    /// The pattern fragment containing the error
    pub fragment: Fragment,
}

/// Location information for an error
#[derive(Debug)]
pub struct ErrorLocation {
    /// Full field path (e.g., "user.profile.age")
    pub field_path: String,
    /// Line number in source file
    pub line_number: u32,
}

/// Represents a fragment of the pattern AST with optional error annotation
#[derive(Debug)]
pub enum Fragment {
    /// Simple leaf patterns (may or may not have error)
    Annotated {
        /// The pattern as string (e.g., ">= 18", "\"hello\"")
        pattern: String,
        /// Some if this is the error location
        annotation: Option<ErrorAnnotation>,
    },

    /// Struct patterns (including enum structs like Status::Active { ... })
    Struct {
        /// Struct name (e.g., "Profile" or "Status::Active")
        name: String,
        /// Fields in the struct
        fields: Vec<Fragment>,
        /// Whether the struct has a rest pattern (..)
        has_rest: bool,
    },

    /// Tuple patterns (including enum tuples like Some(...))
    Tuple {
        /// None for plain tuples, Some("Some") for enum variants
        name: Option<String>,
        /// The tuple elements
        elements: Vec<Fragment>,
    },

    /// Slice patterns
    Slice {
        /// Whether it's &[...] or just [...]
        is_ref: bool,
        /// The slice elements
        elements: Vec<Fragment>,
    },

    /// Struct field (for use within Fragment::Struct)
    Field {
        /// Field name
        name: String,
        /// The field's pattern (recursive)
        value: Box<Fragment>,
    },

    /// Rest pattern ".."
    Rest,
}

/// Error annotation information
#[derive(Debug)]
pub struct ErrorAnnotation {
    /// The actual value that didn't match
    pub actual_value: String,
    /// Type of error
    pub error_type: ErrorType,
}

// ========== PASS 1: STRUCTURE BUILDING ==========

/// Build the error display structure from the pattern tree and errors.
///
/// This is Pass 1 of the two-pass system. It traverses the pattern tree,
/// identifies error locations, and builds a structural representation
/// suitable for rendering.
pub fn build_error_display(root: &'static PatternNode, errors: Vec<ErrorContext>) -> ErrorDisplay {
    if errors.is_empty() {
        return ErrorDisplay {
            header: "assert_struct! failed: no errors provided".to_string(),
            sections: vec![],
            closing_breadcrumbs: vec![],
        };
    }

    // Sort errors by line number to maintain source order
    let mut sorted_errors = errors;
    sorted_errors.sort_by_key(|e| e.line_number);

    // Create header based on error count
    let header = if sorted_errors.len() == 1 {
        "assert_struct! failed:".to_string()
    } else {
        format!("assert_struct! failed: {} mismatches", sorted_errors.len())
    };

    // Create traversal state
    let mut state = TraversalState {
        errors: sorted_errors,
        error_index: 0,
        sections: Vec::new(),
        breadcrumb_stack: Vec::new(),
        current_depth: 0,
    };

    // Traverse the pattern tree from root
    traverse_pattern_tree(root, &mut state, vec![]);

    // Build closing breadcrumbs
    let closing_breadcrumbs = build_closing_breadcrumbs(&state.breadcrumb_stack);

    ErrorDisplay {
        header,
        sections: state.sections,
        closing_breadcrumbs,
    }
}

/// State maintained during tree traversal
struct TraversalState {
    /// Sorted list of errors to process
    errors: Vec<ErrorContext>,
    /// Current index into errors array
    error_index: usize,
    /// Accumulated sections
    sections: Vec<ErrorSection>,
    /// Stack of breadcrumb entries for nested contexts
    breadcrumb_stack: Vec<BreadcrumbEntry>,
    /// Current traversal depth
    current_depth: usize,
}

/// Represents a breadcrumb entry in the nested structure
#[derive(Debug)]
struct BreadcrumbEntry {
    /// The struct name for this breadcrumb
    name: String,
    /// The depth at which this breadcrumb occurs
    depth: usize,
    /// Whether this breadcrumb has been rendered
    rendered: bool,
}

impl TraversalState {
    /// Check if there are more errors to process
    fn has_next_error(&self) -> bool {
        self.error_index < self.errors.len()
    }

    /// Check if the current node matches the next error
    fn next_error_matches(&self, node: &'static PatternNode) -> bool {
        self.has_next_error()
            && self.errors[self.error_index]
                .error_node
                .map(|error_node| std::ptr::eq(error_node, node))
                .unwrap_or(false)
    }

    /// Get the current error being processed
    fn current_error(&self) -> Option<&ErrorContext> {
        if self.has_next_error() {
            Some(&self.errors[self.error_index])
        } else {
            None
        }
    }

    /// Mark the current error as processed
    fn advance_error(&mut self) {
        self.error_index += 1;
    }
}

/// Traverse the pattern tree and build error sections
fn traverse_pattern_tree(
    node: &'static PatternNode,
    state: &mut TraversalState,
    field_path: Vec<String>,
) {
    // Check if current node is an error node
    if state.next_error_matches(node) {
        // Extract error data to avoid borrow issues
        let error_data = state.current_error().map(|e| {
            (
                e.field_path.clone(),
                e.line_number,
                e.actual_value.clone(),
                e.error_type.clone(),
            )
        });

        if let Some((error_field_path, error_line, _actual_value, _error_type)) = error_data {
            // Check for tuple element errors (should be handled at parent level)
            if is_tuple_element_error(&error_field_path) {
                // Skip - will be handled when we reach the tuple itself
                state.advance_error();
                return;
            }

            // Build the fragment for this error (need to recreate error context)
            let error_ctx = state.current_error().unwrap();
            let fragment = build_error_fragment(node, error_ctx, state);

            // Collect opening breadcrumbs
            let opening_breadcrumbs = collect_opening_breadcrumbs(state);

            // Create the error section
            let section = ErrorSection {
                opening_breadcrumbs,
                location: ErrorLocation {
                    field_path: error_field_path,
                    line_number: error_line,
                },
                fragment,
            };

            state.sections.push(section);
            state.advance_error();
        }
    }

    // Continue traversal based on node type
    match node {
        PatternNode::Struct { name, fields, .. } => {
            // Check if this struct contains future errors
            let contains_errors = contains_future_errors(node, state);

            if contains_errors {
                state.breadcrumb_stack.push(BreadcrumbEntry {
                    name: name.to_string(),
                    depth: state.current_depth,
                    rendered: false,
                });
            }

            state.current_depth += 1;
            for (field_name, field_node) in fields.iter() {
                if *field_name != ".." {
                    let mut new_path = field_path.clone();
                    new_path.push(field_name.to_string());
                    traverse_pattern_tree(field_node, state, new_path);
                }
            }
            state.current_depth -= 1;
        }
        PatternNode::Tuple { items } => {
            // Check for tuple element errors at this level
            let tuple_errors = collect_tuple_child_errors(node, state, &field_path);

            if !tuple_errors.is_empty() {
                // Extract error data to avoid borrow issues
                let error_count = tuple_errors.len();
                let first_error_line = tuple_errors[0].line_number;
                let first_error_path = tuple_errors[0].field_path.clone();

                // Build a tuple fragment with all elements
                let fragment = build_tuple_fragment_with_errors(node, &tuple_errors, &field_path);

                let opening_breadcrumbs = collect_opening_breadcrumbs(state);

                // Note: Currently the macro only generates one error at a time,
                // but the rendering code now handles multiple errors gracefully

                // Use the error's field path, but remove the numeric suffix for display
                let display_path = {
                    let path = &first_error_path;
                    // Remove the numeric index at the end for tuple display
                    if let Some(dot_pos) = path.rfind('.') {
                        let (base, suffix) = path.split_at(dot_pos + 1);
                        if suffix.parse::<usize>().is_ok() {
                            base[..base.len() - 1].to_string() // Remove the dot too
                        } else {
                            path.to_string()
                        }
                    } else {
                        path.to_string()
                    }
                };

                let section = ErrorSection {
                    opening_breadcrumbs,
                    location: ErrorLocation {
                        field_path: display_path,
                        line_number: first_error_line,
                    },
                    fragment,
                };

                state.sections.push(section);

                // Advance past all tuple errors
                for _ in 0..error_count {
                    state.advance_error();
                }
            } else {
                // No errors in this tuple, continue traversal
                state.current_depth += 1;
                for (i, item) in items.iter().enumerate() {
                    let mut new_path = field_path.clone();
                    new_path.push(i.to_string());
                    traverse_pattern_tree(item, state, new_path);
                }
                state.current_depth -= 1;
            }
        }
        PatternNode::Slice { items, .. } => {
            state.current_depth += 1;
            for (i, item) in items.iter().enumerate() {
                let mut new_path = field_path.clone();
                new_path.push(format!("[{}]", i));
                traverse_pattern_tree(item, state, new_path);
            }
            state.current_depth -= 1;
        }
        PatternNode::EnumVariant { args, .. } => {
            if let Some(args) = args {
                // Enum variants with args are handled like tuples
                let tuple_errors = collect_tuple_child_errors(node, state, &field_path);

                if !tuple_errors.is_empty() {
                    // Extract error data to avoid borrow issues
                    let error_count = tuple_errors.len();
                    let first_error_line = tuple_errors[0].line_number;
                    let first_error_path = tuple_errors[0].field_path.clone();

                    let fragment =
                        build_enum_tuple_fragment_with_errors(node, &tuple_errors, &field_path);
                    let opening_breadcrumbs = collect_opening_breadcrumbs(state);

                    // Note: Currently the macro only generates one error at a time,
                    // but the rendering code now handles multiple errors gracefully

                    // Use the error's field path, but remove the numeric suffix for display
                    let display_path = {
                        let path = &first_error_path;
                        // Remove the numeric index at the end for tuple display
                        if let Some(dot_pos) = path.rfind('.') {
                            let (base, suffix) = path.split_at(dot_pos + 1);
                            if suffix.parse::<usize>().is_ok() {
                                base[..base.len() - 1].to_string() // Remove the dot too
                            } else {
                                path.to_string()
                            }
                        } else {
                            path.to_string()
                        }
                    };

                    let section = ErrorSection {
                        opening_breadcrumbs,
                        location: ErrorLocation {
                            field_path: display_path,
                            line_number: first_error_line,
                        },
                        fragment,
                    };

                    state.sections.push(section);

                    for _ in 0..error_count {
                        state.advance_error();
                    }
                } else {
                    state.current_depth += 1;
                    for (i, arg) in args.iter().enumerate() {
                        let mut new_path = field_path.clone();
                        new_path.push(i.to_string());
                        traverse_pattern_tree(arg, state, new_path);
                    }
                    state.current_depth -= 1;
                }
            }
        }
        // Leaf nodes don't need further traversal
        _ => {}
    }
}

/// Build a fragment for an error node
fn build_error_fragment(
    node: &'static PatternNode,
    error: &ErrorContext,
    _state: &TraversalState,
) -> Fragment {
    // Extract field name if this is a struct field
    let field_name = error.field_path.split('.').last().unwrap_or("");

    // Check if field_name is all digits to identify tuple element paths
    // This is checking whether the last part of the field path (e.g., "0" in "user.0")
    // consists entirely of digits, which indicates it's a tuple element index rather than
    // a named struct field
    let is_tuple_element = field_name.chars().all(|c| c.is_ascii_digit());

    // Determine if this is a complex pattern that shouldn't show field names
    let is_complex_pattern = matches!(
        node,
        PatternNode::EnumVariant { .. }
            | PatternNode::Range { .. }
            | PatternNode::Regex { .. }
            | PatternNode::Like { .. }
    );

    // Build the appropriate fragment based on node type and context
    if !is_tuple_element && !field_name.is_empty() && !is_complex_pattern {
        // This is a simple struct field
        Fragment::Field {
            name: field_name.to_string(),
            value: Box::new(build_pattern_fragment(node, Some(error))),
        }
    } else {
        // Direct pattern without field wrapper (complex patterns, tuple elements, etc.)
        build_pattern_fragment(node, Some(error))
    }
}

/// Format a pattern node to a simple string representation
fn format_pattern_simple(node: &'static PatternNode) -> String {
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
                .map(|item| format_pattern_simple(item))
                .collect::<Vec<_>>()
                .join(", ");
            format!("{}[{}]", prefix, content)
        }
        PatternNode::Tuple { items } => {
            let content = items
                .iter()
                .map(|item| format_pattern_simple(item))
                .collect::<Vec<_>>()
                .join(", ");
            format!("({})", content)
        }
        PatternNode::EnumVariant { path, args } => {
            if let Some(args) = args {
                if !args.is_empty() {
                    let arg_str = args
                        .iter()
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

/// Build a pattern fragment from a PatternNode
fn build_pattern_fragment(node: &'static PatternNode, error: Option<&ErrorContext>) -> Fragment {
    let annotation = error.map(|e| ErrorAnnotation {
        actual_value: e.actual_value.clone(),
        error_type: e.error_type.clone(),
    });

    match node {
        PatternNode::Simple { value } => Fragment::Annotated {
            pattern: value.to_string(),
            annotation,
        },
        PatternNode::Comparison { op, value } => Fragment::Annotated {
            pattern: format!("{} {}", op, value),
            annotation,
        },
        PatternNode::Range { pattern } => Fragment::Annotated {
            pattern: pattern.to_string(),
            annotation,
        },
        PatternNode::Regex { pattern } => Fragment::Annotated {
            pattern: format!("=~ {}", pattern),
            annotation,
        },
        PatternNode::Like { expr } => Fragment::Annotated {
            pattern: format!("=~ {}", expr),
            annotation,
        },
        PatternNode::EnumVariant { path, args } => {
            if let Some(args) = args {
                if !args.is_empty() {
                    // Render full enum variant with arguments
                    let arg_str = args
                        .iter()
                        .map(|arg| format_pattern_simple(arg))
                        .collect::<Vec<_>>()
                        .join(", ");
                    Fragment::Annotated {
                        pattern: format!("{}({})", path, arg_str),
                        annotation,
                    }
                } else {
                    // Unit variant
                    Fragment::Annotated {
                        pattern: path.to_string(),
                        annotation,
                    }
                }
            } else {
                // Unit variant
                Fragment::Annotated {
                    pattern: path.to_string(),
                    annotation,
                }
            }
        }
        PatternNode::Rest => Fragment::Rest,
        _ => Fragment::Annotated {
            pattern: "<complex>".to_string(),
            annotation,
        },
    }
}

/// Build a tuple fragment with error annotations
fn build_tuple_fragment_with_errors(
    node: &'static PatternNode,
    errors: &[&ErrorContext],
    field_path: &[String],
) -> Fragment {
    if let PatternNode::Tuple { items } = node {
        // Build field fragment if this tuple is a struct field
        let field_name = field_path
            .last()
            .filter(|name| !name.chars().all(|c| c.is_ascii_digit()))
            .cloned();

        let elements = items
            .iter()
            .enumerate()
            .map(|(i, item)| {
                // Check if this element has an error
                let element_error = errors
                    .iter()
                    .find(|e| e.field_path.ends_with(&format!(".{}", i)));

                build_pattern_fragment(item, element_error.copied())
            })
            .collect();

        let tuple_fragment = Fragment::Tuple {
            name: None,
            elements,
        };

        if let Some(name) = field_name {
            Fragment::Field {
                name,
                value: Box::new(tuple_fragment),
            }
        } else {
            tuple_fragment
        }
    } else {
        Fragment::Annotated {
            pattern: "<error>".to_string(),
            annotation: None,
        }
    }
}

/// Build an enum tuple fragment with error annotations
fn build_enum_tuple_fragment_with_errors(
    node: &'static PatternNode,
    errors: &[&ErrorContext],
    field_path: &[String],
) -> Fragment {
    if let PatternNode::EnumVariant {
        path,
        args: Some(args),
        ..
    } = node
    {
        let field_name = field_path
            .last()
            .filter(|name| !name.chars().all(|c| c.is_ascii_digit()))
            .cloned();

        let elements = args
            .iter()
            .enumerate()
            .map(|(i, item)| {
                let element_error = errors
                    .iter()
                    .find(|e| e.field_path.ends_with(&format!(".{}", i)));

                build_pattern_fragment(item, element_error.copied())
            })
            .collect();

        let tuple_fragment = Fragment::Tuple {
            name: Some(path.to_string()),
            elements,
        };

        if let Some(name) = field_name {
            Fragment::Field {
                name,
                value: Box::new(tuple_fragment),
            }
        } else {
            tuple_fragment
        }
    } else {
        Fragment::Annotated {
            pattern: "<error>".to_string(),
            annotation: None,
        }
    }
}

/// Check if a field path represents a tuple element error
fn is_tuple_element_error(field_path: &str) -> bool {
    field_path
        .split('.')
        .last()
        .map(|s| s.parse::<usize>().is_ok())
        .unwrap_or(false)
}

/// Collect tuple child errors that should be rendered at this level
fn collect_tuple_child_errors<'a>(
    tuple_node: &'static PatternNode,
    state: &'a TraversalState,
    _field_path: &[String],
) -> Vec<&'a ErrorContext> {
    let mut tuple_errors = Vec::new();

    for i in state.error_index..state.errors.len() {
        let error = &state.errors[i];

        if is_tuple_element_error(&error.field_path) {
            if let Some(error_node) = error.error_node {
                if node_contains_recursive(tuple_node, error_node) {
                    tuple_errors.push(error);
                }
            }
        }
    }

    tuple_errors
}

/// Check if a node contains any future errors
fn contains_future_errors(node: &'static PatternNode, state: &TraversalState) -> bool {
    for i in state.error_index..state.errors.len() {
        if let Some(error_node) = state.errors[i].error_node {
            if node_contains_recursive(node, error_node) {
                return true;
            }
        }
    }
    false
}

/// Check if a root node contains a target node recursively
fn node_contains_recursive(root: &'static PatternNode, target: &'static PatternNode) -> bool {
    if std::ptr::eq(root, target) {
        return true;
    }

    match root {
        PatternNode::Struct { fields, .. } => fields
            .iter()
            .any(|(_, field_node)| node_contains_recursive(field_node, target)),
        PatternNode::EnumVariant {
            args: Some(args), ..
        } => args.iter().any(|arg| node_contains_recursive(arg, target)),
        PatternNode::Slice { items, .. } => items
            .iter()
            .any(|item| node_contains_recursive(item, target)),
        PatternNode::Tuple { items } => items
            .iter()
            .any(|item| node_contains_recursive(item, target)),
        _ => false,
    }
}

/// Collect opening breadcrumbs that haven't been rendered yet
fn collect_opening_breadcrumbs(state: &mut TraversalState) -> Vec<String> {
    let mut breadcrumbs = Vec::new();

    // Check if any breadcrumbs have been rendered globally before this call
    let any_previously_rendered = state.breadcrumb_stack.iter().any(|e| e.rendered);

    for entry in &mut state.breadcrumb_stack {
        if !entry.rendered && entry.depth <= state.current_depth {
            // Add "... " prefix if:
            // 1. This is not the very first breadcrumb ever (any_previously_rendered), OR
            // 2. There are already breadcrumbs in this current batch
            let prefix = if breadcrumbs.is_empty() && !any_previously_rendered {
                ""
            } else {
                "... "
            };
            breadcrumbs.push(format!("{}{} {{", prefix, entry.name));
            entry.rendered = true;
        }
    }

    breadcrumbs
}

/// Build closing breadcrumbs for all rendered entries
fn build_closing_breadcrumbs(breadcrumb_stack: &[BreadcrumbEntry]) -> Vec<String> {
    let rendered: Vec<_> = breadcrumb_stack.iter().filter(|e| e.rendered).collect();

    if rendered.is_empty() {
        return vec![];
    }

    // Build closing line like "} ... }"
    let closings: Vec<_> = rendered.iter().map(|_| "}").collect();
    vec![closings.join(" ... ")]
}

// ========== PASS 2: RENDERING ==========

/// Render the error display structure to a string.
///
/// This is Pass 2 of the two-pass system. It takes the structural
/// representation and renders it to a formatted string.
pub fn render_error_display(display: &ErrorDisplay) -> String {
    let mut output = String::new();

    // Render header
    output.push_str(&display.header);
    output.push('\n');

    if !display.sections.is_empty() {
        output.push('\n');
    }

    // Track indentation level across all sections
    let mut indentation_level = 0;

    // Render each section
    for section in &display.sections {
        render_section(section, &mut output, &mut indentation_level);
    }

    // Render closing breadcrumbs
    for closing in &display.closing_breadcrumbs {
        output.push_str("   | ");
        output.push_str(closing);
        output.push('\n');
    }

    output
}

/// Render a single error section
fn render_section(section: &ErrorSection, output: &mut String, indentation_level: &mut usize) {
    // Render opening breadcrumbs and update indentation level
    for breadcrumb in &section.opening_breadcrumbs {
        if output.ends_with('\n') {
            output.push_str("   | ");
        } else {
            output.push(' ');
        }
        output.push_str(breadcrumb);

        // If this breadcrumb opens a container (ends with "{"), increment indentation level
        if breadcrumb.ends_with('{') {
            *indentation_level += 1;
        }
    }

    if !section.opening_breadcrumbs.is_empty() {
        output.push('\n');
    }

    // Render error location
    output.push_str("mismatch:\n");
    output.push_str(&format!(
        "  --> `{}` (line {})\n",
        section.location.field_path, section.location.line_number
    ));

    // Determine indentation based on current indentation level and fragment type
    // Check if this is a field fragment (which needs indentation when inside a container)
    let is_field_fragment = matches!(section.fragment, Fragment::Field { .. });

    let base_indent = if *indentation_level > 0 && is_field_fragment {
        "    "
    } else {
        ""
    };

    // Render the fragment and collect annotation positions
    let mut pattern_line = String::from("   | ");
    pattern_line.push_str(base_indent);

    let mut annotations = Vec::new();
    render_fragment(
        &section.fragment,
        &mut pattern_line,
        &mut annotations,
        base_indent.len(),
    );

    pattern_line.push('\n');
    output.push_str(&pattern_line);

    // Render underlines for annotations
    if annotations.len() == 1 {
        // Single annotation: use simple format
        let (start_pos, end_pos, annotation) = annotations[0];
        let spaces = " ".repeat(start_pos);
        let underline = "^".repeat(end_pos - start_pos);
        output.push_str(&format!(
            "   | {}{} actual: {}\n",
            spaces, underline, annotation.actual_value
        ));
    } else if annotations.len() > 1 {
        // Multiple annotations: use box-drawing format

        // First line: all underlines with rightmost error
        output.push_str("   | ");
        output.push_str(&" ".repeat(base_indent.len()));

        let mut last_pos = base_indent.len();
        for (start_pos, end_pos, _) in &annotations {
            if *start_pos > last_pos {
                output.push_str(&" ".repeat(start_pos - last_pos));
            }
            output.push_str(&"^".repeat(end_pos - start_pos));
            last_pos = *end_pos;
        }

        // Rightmost error on same line
        if let Some((_, _, ann)) = annotations.last() {
            output.push_str(" actual: ");
            output.push_str(&ann.actual_value);
        }
        output.push('\n');

        // Remaining annotations from right to left
        for i in (0..annotations.len() - 1).rev() {
            output.push_str("   | ");
            output.push_str(&" ".repeat(base_indent.len()));

            let mut current_pos = base_indent.len();

            // Walk through annotations 0..=i to place box characters
            for j in 0..=i {
                let (start_pos, _, _) = annotations[j];

                // Add spacing to reach annotation j's position
                if start_pos > current_pos {
                    output.push_str(&" ".repeat(start_pos - current_pos));
                }

                if j == i {
                    // This is our annotation - draw corner
                    output.push_str("└─ actual: ");
                    output.push_str(&annotations[i].2.actual_value);
                    break;
                } else {
                    // Draw vertical line for annotations that come before
                    output.push('│');
                    current_pos = start_pos + 1;
                }
            }
            output.push('\n');
        }
    }
}

/// Render a fragment and track annotation positions
fn render_fragment<'a>(
    fragment: &'a Fragment,
    output: &mut String,
    annotations: &mut Vec<(usize, usize, &'a ErrorAnnotation)>,
    current_indent: usize,
) {
    match fragment {
        Fragment::Annotated {
            pattern,
            annotation,
        } => {
            // Position is relative to start of line content (after "   | ")
            let start_pos = output.len() - 5; // Current position after "   | "
            output.push_str(pattern);
            let end_pos = output.len() - 5;

            if let Some(ann) = annotation {
                annotations.push((start_pos, end_pos, ann));
            }
        }
        Fragment::Field { name, value } => {
            output.push_str(name);
            output.push_str(": ");
            render_fragment(value, output, annotations, current_indent);
            output.push(',');
        }
        Fragment::Struct {
            name,
            fields,
            has_rest,
        } => {
            output.push_str(name);
            output.push_str(" { ");

            for (i, field) in fields.iter().enumerate() {
                if i > 0 {
                    output.push_str(", ");
                }
                render_fragment(field, output, annotations, current_indent);
            }

            if *has_rest {
                if !fields.is_empty() {
                    output.push_str(", ");
                }
                output.push_str("..");
            }

            output.push_str(" }");
        }
        Fragment::Tuple { name, elements } => {
            if let Some(n) = name {
                output.push_str(n);
            }
            output.push('(');

            for (i, element) in elements.iter().enumerate() {
                if i > 0 {
                    output.push_str(", ");
                }
                render_fragment(element, output, annotations, current_indent);
            }

            output.push(')');
        }
        Fragment::Slice { is_ref, elements } => {
            if *is_ref {
                output.push('&');
            }
            output.push('[');

            for (i, element) in elements.iter().enumerate() {
                if i > 0 {
                    output.push_str(", ");
                }
                render_fragment(element, output, annotations, current_indent);
            }

            output.push(']');
        }
        Fragment::Rest => {
            output.push_str("..");
        }
    }
}

// ========== PUBLIC API ==========

/// Main entry point for formatting errors with the pattern tree
pub fn format_errors_with_root(root: &'static PatternNode, errors: Vec<ErrorContext>) -> String {
    let display = build_error_display(root, errors);
    render_error_display(&display)
}
