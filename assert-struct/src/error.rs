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
    // Tuple context for better error formatting
    pub tuple_context: Option<TupleErrorContext>,
}

#[derive(Debug, Clone)]
pub struct TupleErrorContext {
    pub element_index: usize,
    pub element_pattern: String, // Pattern for the failing element (e.g., "60")
    pub full_tuple_pattern: String, // Full tuple pattern (e.g., "(60, \"test\")")
}

#[derive(Debug, Clone)]
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
/// State maintained during tree traversal for error formatting.
///
/// This structure encapsulates all the state needed for the unified tree-traversal
/// algorithm that handles both single and multiple errors with the same code path.
struct TraversalState<'a> {
    /// The sorted list of errors to process
    errors: &'a [ErrorContext],

    /// Current index into the errors array
    error_index: usize,

    /// The accumulated output string
    output: String,

    /// Stack of breadcrumb entries for nested contexts
    breadcrumb_stack: Vec<BreadcrumbEntry>,

    /// Depth of last written breadcrumb (for managing intermediate breadcrumbs)
    last_written_depth: Option<usize>,

    /// Current traversal depth in the pattern tree
    current_depth: usize,

    /// Whether any breadcrumbs have been rendered globally (for multi-error context)
    any_breadcrumbs_rendered: bool,
}

/// Represents a breadcrumb entry in the nested structure context.
///
/// Breadcrumbs track the path through nested structures (Struct -> Slice -> Tuple, etc.)
/// and help determine when to render intermediate context like "... InnerStruct {".
#[derive(Debug)]
struct BreadcrumbEntry {
    /// The pattern node for this breadcrumb level
    node: &'static PatternNode,

    /// The depth at which this breadcrumb occurs
    depth: usize,

    /// Whether this breadcrumb has been rendered to output yet
    rendered: bool,
}

impl<'a> TraversalState<'a> {
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
        self.last_written_depth = Some(self.current_depth);
    }
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

/// NEW: Formats errors using unified tree-traversal algorithm with root node.
///
/// This function implements the core algorithm requested by the user:
/// - Takes root PatternNode AND list of errors as separate parameters
/// - Uses unified code path for single/multiple errors with conditionals
/// - Implements tree-traversal approach starting from root
pub fn format_errors_with_root(root: &'static PatternNode, errors: Vec<ErrorContext>) -> String {
    if errors.is_empty() {
        return "assert_struct! failed: no errors provided".to_string();
    }

    // Sort errors by line number to maintain source order
    let mut sorted_errors = errors;
    sorted_errors.sort_by_key(|e| e.line_number);

    // Create traversal state - unified for single/multiple errors
    let mut state = TraversalState {
        errors: &sorted_errors,
        error_index: 0,
        output: String::new(),
        breadcrumb_stack: Vec::new(),
        last_written_depth: None,
        current_depth: 0,
        any_breadcrumbs_rendered: false,
    };

    // Add header - conditional formatting based on error count
    if sorted_errors.len() == 1 {
        state.output.push_str("assert_struct! failed:\n\n");
    } else {
        state.output.push_str(&format!(
            "assert_struct! failed: {} mismatches\n",
            sorted_errors.len()
        ));
    }

    // Start unified tree traversal from root
    traverse_tree(root, &mut state);

    // Render closing breadcrumbs for all opened containers
    render_final_closing_breadcrumbs(&mut state);

    state.output
}

// Unified tree-traversal implementation

/// Core unified tree traversal function.
///
/// This function implements the unified tree-traversal algorithm that handles both
/// single and multiple errors with the same code path, using conditionals rather
/// than split logic.
fn traverse_tree(node: &'static PatternNode, state: &mut TraversalState) {
    // Check if current node matches next error
    let is_error_node = state.next_error_matches(node);

    if is_error_node {
        // Render any pending breadcrumbs before the error
        render_breadcrumbs_to_error(state);

        // Render the error based on node type
        match node {
            PatternNode::Struct { .. } => {
                render_struct_error(node, state);
            }
            PatternNode::Slice { .. } => {
                render_slice_error(node, state);
            }
            PatternNode::Tuple { .. } => {
                render_tuple_error(node, state);
            }
            PatternNode::EnumVariant { .. } => {
                render_enum_error(node, state);
            }
            PatternNode::Simple { .. } => {
                render_simple_error(node, state);
            }
            PatternNode::Comparison { .. } => {
                render_comparison_error(node, state);
            }
            PatternNode::Range { .. } => {
                render_range_error(node, state);
            }
            PatternNode::Regex { .. } => {
                render_regex_error(node, state);
            }
            PatternNode::Like { .. } => {
                render_like_error(node, state);
            }
            PatternNode::Rest => {
                // Rest patterns shouldn't be error nodes, but handle gracefully
                render_simple_error(node, state);
            }
        }

        state.advance_error();
    }

    // Continue traversal based on node type
    match node {
        PatternNode::Struct { fields, .. } => {
            // Add to breadcrumb stack if contains future errors
            let has_future_errors = contains_future_errors(node, state);
            if has_future_errors {
                state.breadcrumb_stack.push(BreadcrumbEntry {
                    node,
                    depth: state.current_depth,
                    rendered: false,
                });
            }

            state.current_depth += 1;
            for (field_name, field_node) in fields.iter() {
                if *field_name != ".." {
                    // Skip rest patterns
                    traverse_tree(field_node, state);
                }
            }
            state.current_depth -= 1;
        }
        PatternNode::Slice { items, .. } => {
            // Slices no longer use breadcrumbs - render inline with error
            state.current_depth += 1;
            for item in items.iter() {
                traverse_tree(item, state);
            }
            state.current_depth -= 1;
        }
        PatternNode::Tuple { items } => {
            // Tuples no longer use breadcrumbs - render inline with error
            state.current_depth += 1;
            for item in items.iter() {
                traverse_tree(item, state);
            }
            state.current_depth -= 1;
        }
        PatternNode::EnumVariant { args, .. } => {
            if let Some(args) = args {
                // Enum variants no longer use breadcrumbs - render inline with error
                state.current_depth += 1;
                for arg in args.iter() {
                    traverse_tree(arg, state);
                }
                state.current_depth -= 1;
            }
        }
        // Leaf nodes don't need further traversal
        PatternNode::Simple { .. }
        | PatternNode::Comparison { .. }
        | PatternNode::Range { .. }
        | PatternNode::Regex { .. }
        | PatternNode::Like { .. }
        | PatternNode::Rest => {}
    }
}

/// Check if a node contains any future errors that haven't been processed yet.
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

/// Check if a root node contains a target node recursively.
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

/// Render breadcrumbs leading up to an error.
fn render_breadcrumbs_to_error(state: &mut TraversalState) {
    for entry in &mut state.breadcrumb_stack {
        if !entry.rendered && entry.depth <= state.current_depth {
            match entry.node {
                PatternNode::Struct { name, .. } => {
                    if state.output.ends_with('\n') && !state.any_breadcrumbs_rendered {
                        // First breadcrumb ever (root level, first error)
                        state.output.push_str(&format!("   | {} {{", name));
                    } else if state.output.ends_with('\n') {
                        // Start of new section but breadcrumbs were rendered before
                        state.output.push_str(&format!("   | ... {} {{", name));
                    } else {
                        // Intermediate breadcrumb on same line
                        state.output.push_str(&format!(" ... {} {{", name));
                    }
                }
                _ => {
                    // Only struct patterns use breadcrumbs now
                }
            }
            entry.rendered = true;
            state.any_breadcrumbs_rendered = true;
        }
    }

    // Add newline before error if we rendered breadcrumbs
    if state.breadcrumb_stack.iter().any(|e| e.rendered) {
        state.output.push('\n');
    }
}

/// Render all closing breadcrumbs for rendered containers in a single line.
fn render_final_closing_breadcrumbs(state: &mut TraversalState) {
    // Collect all rendered breadcrumbs that need closing
    let rendered_breadcrumbs: Vec<_> = state
        .breadcrumb_stack
        .iter()
        .filter(|entry| entry.rendered)
        .collect();

    if rendered_breadcrumbs.is_empty() {
        return;
    }

    // Build the closing breadcrumb line: "   | } ... }" or "   | ) ... }"
    let mut closing_parts = Vec::new();

    // Process breadcrumbs in reverse order (innermost to outermost)
    for entry in rendered_breadcrumbs.iter().rev() {
        let closing_char = match entry.node {
            PatternNode::Struct { .. } => "}",
            _ => continue, // Only struct patterns use breadcrumbs now
        };
        closing_parts.push(closing_char);
    }

    if !closing_parts.is_empty() {
        // Join with " ... " to mirror opening breadcrumb style
        let closing_line = closing_parts.join(" ... ");
        state.output.push_str(&format!("   | {}\n", closing_line));
    }
}

// Error rendering functions for each node type

fn render_struct_error(node: &'static PatternNode, state: &mut TraversalState) {
    if let Some(_) = state.current_error() {
        // For struct errors, we typically render the field that failed
        // This is handled by the traversal finding the actual error node
        render_simple_error(node, state);
    }
}

fn render_slice_error(node: &'static PatternNode, state: &mut TraversalState) {
    render_simple_error(node, state);
}

fn render_tuple_error(node: &'static PatternNode, state: &mut TraversalState) {
    if let Some(error) = state.current_error() {
        // Extract values before borrowing state mutably
        let error_type = error.error_type.clone();
        let field_path = error.field_path.clone();
        let line_number = error.line_number;
        let actual_value = error.actual_value.clone();

        // Check if we have tuple context - if not, fall back to simple rendering
        let tuple_context = match &error.tuple_context {
            Some(ctx) => ctx.clone(), // Clone the tuple context to avoid borrowing issues
            None => {
                render_simple_error(node, state);
                return;
            }
        };

        state
            .output
            .push_str(&format!("{} mismatch:\n", error_type));
        state
            .output
            .push_str(&format!("  --> `{}` (line {})\n", field_path, line_number));

        // Determine if we're in a breadcrumb context and need field-level indentation
        let has_breadcrumbs = state.breadcrumb_stack.iter().any(|e| e.rendered);

        // Extract field name from path for context (e.g., "holder.data" -> "data")
        let field_name = field_path.split('.').last().unwrap_or("");

        // For tuples, always show field context if we have breadcrumbs and a field name
        let should_show_field_context = has_breadcrumbs && !field_name.is_empty();
        let indent = if should_show_field_context {
            "    "
        } else {
            ""
        }; // 4 spaces for struct fields

        if should_show_field_context {
            // Render with field context showing full tuple: "   |     data: (60, "test"),"
            state.output.push_str(&format!(
                "   | {}{}: {},\n",
                indent, field_name, tuple_context.full_tuple_pattern
            ));

            // Now we need to underline only the failing element within the tuple
            // Calculate position: indent + field_name + ": (" + position of failing element
            let field_prefix = format!("{}{}: (", indent, field_name);
            let field_prefix_len = field_prefix.len();

            // Find the position of the failing element pattern within the full tuple pattern
            let element_pattern = &tuple_context.element_pattern;
            let full_pattern = &tuple_context.full_tuple_pattern;

            // Look for the element pattern position within the tuple (after the opening parenthesis)
            let element_position = if let Some(pos) = full_pattern.find(element_pattern) {
                // Make sure we found the actual element, not a substring in another element
                pos - 1 // Subtract 1 to account for the opening parenthesis position
            } else {
                0 // Fallback if we can't find the exact position
            };

            let underline_spaces = " ".repeat(field_prefix_len + element_position);
            let underline = "^".repeat(element_pattern.len());
            state.output.push_str(&format!(
                "   | {}{} actual: {}\n",
                underline_spaces, underline, actual_value
            ));
        } else {
            // Render without field context (shouldn't happen for tuples in structs, but handle it)
            state.output.push_str(&format!(
                "   | {}{}\n",
                indent, tuple_context.full_tuple_pattern
            ));

            let underline = "^".repeat(tuple_context.element_pattern.len());
            state.output.push_str(&format!(
                "   | {}{} actual: {}\n",
                indent, underline, actual_value
            ));
        }
    }
}

fn render_enum_error(node: &'static PatternNode, state: &mut TraversalState) {
    render_simple_error(node, state);
}

fn render_simple_error(node: &'static PatternNode, state: &mut TraversalState) {
    if let Some(error) = state.current_error() {
        // Extract values before borrowing state mutably
        let error_type = error.error_type.clone();
        let field_path = error.field_path.clone();
        let line_number = error.line_number;
        let actual_value = error.actual_value.clone();

        state
            .output
            .push_str(&format!("{} mismatch:\n", error_type));
        state
            .output
            .push_str(&format!("  --> `{}` (line {})\n", field_path, line_number));

        // Determine if we're in a breadcrumb context and need field-level indentation
        let has_breadcrumbs = state.breadcrumb_stack.iter().any(|e| e.rendered);

        // Extract field name from path for context (e.g., "user.age" -> "age")
        let field_name = field_path.split('.').last().unwrap_or("");

        // Determine if we should show field context and indentation
        // Only for simple struct fields, not complex patterns (enum variants, comparisons, etc.)
        let is_complex_pattern = matches!(
            node,
            PatternNode::EnumVariant { .. }
                | PatternNode::Comparison { .. }
                | PatternNode::Range { .. }
                | PatternNode::Regex { .. }
                | PatternNode::Like { .. }
        );

        let should_show_field_context = has_breadcrumbs
            && !field_name.is_empty()
            && !field_name.chars().all(|c| c.is_ascii_digit())
            && !is_complex_pattern;

        let indent = if should_show_field_context {
            "    "
        } else {
            ""
        }; // 4 spaces for struct fields only

        let pattern_str = format_pattern_simple(node);

        if should_show_field_context {
            // Render with field context: "   |     age: 25,"
            state.output.push_str(&format!(
                "   | {}{}: {},\n",
                indent, field_name, pattern_str
            ));

            // Position underline under the pattern value, not the field name
            let field_prefix_len = field_name.len() + 2; // "age: " = field_name + ": "
            let underline = "^".repeat(pattern_str.len());
            let underline_spaces = " ".repeat(field_prefix_len);
            state.output.push_str(&format!(
                "   | {}{}{} actual: {}\n",
                indent, underline_spaces, underline, actual_value
            ));
        } else {
            // Render without field context: "   | 60" or "   |     60" if indented
            state
                .output
                .push_str(&format!("   | {}{}\n", indent, pattern_str));

            let underline = "^".repeat(pattern_str.len());
            state.output.push_str(&format!(
                "   | {}{} actual: {}\n",
                indent, underline, actual_value
            ));
        }
    }
}

fn render_comparison_error(node: &'static PatternNode, state: &mut TraversalState) {
    render_simple_error(node, state);
}

fn render_range_error(node: &'static PatternNode, state: &mut TraversalState) {
    render_simple_error(node, state);
}

fn render_regex_error(node: &'static PatternNode, state: &mut TraversalState) {
    render_simple_error(node, state);
}

fn render_like_error(node: &'static PatternNode, state: &mut TraversalState) {
    render_simple_error(node, state);
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
/// ```text
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
