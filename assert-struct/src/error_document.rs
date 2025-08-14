//! Structured error document representation for assert-struct error rendering.
//!
//! This module provides a structured approach to building error messages that eliminates
//! manual character counting and off-by-one positioning errors. Instead of building strings
//! directly with position calculations, we build a structured representation that tracks
//! positions automatically.
//!
//! ## Design Philosophy
//!
//! The core insight is to separate **building** (what to display) from **rendering**
//! (how to display it). This separation provides:
//!
//! - **Automatic position tracking**: Positions are captured when content is added
//! - **Impossible off-by-one errors**: No manual position calculations
//! - **Separation of concerns**: Building vs rendering vs styling
//! - **Testability**: Each component can be tested independently
//! - **Extensibility**: Easy to add colors, alternative formats, etc.
//!
//! ## Core Workflow
//!
//! ```rust
//! use assert_struct::error_document::*;
//!
//! // 1. Build structured representation
//! let mut builder = LineBuilder::new();
//! builder.add("    ", SegmentStyle::Normal);              // indent
//! builder.add("field: ", SegmentStyle::FieldName);        // field name
//! let (start, end) = builder.add("value", SegmentStyle::Pattern);  // track position
//!
//! // 2. Create error document
//! let content_line = Line::from_builder("   | ", builder);
//! let underline = UnderlineLine::new("   | ", start, end, "actual_value".to_string());
//! let error_line = ErrorLine::new(content_line, underline, "field".to_string(), 42);
//! let section = ErrorSection::new(error_line);
//! let mut document = ErrorDocument::new();
//! document.add_section(section);
//!
//! // 3. Render to string
//! let output = document.render();
//! ```

/// Represents a complete error document with multiple error sections.
///
/// This is the top-level container for all errors in a single assert_struct! failure.
/// It handles the overall document structure and coordinates rendering of all sections.
#[derive(Debug)]
pub struct ErrorDocument {
    /// List of error sections (one per mismatch)
    pub sections: Vec<ErrorSection>,
}

/// A single error section representing one mismatch.
///
/// Each section includes:
/// - Context lines (breadcrumbs leading to the error)
/// - The error line itself (content + underline)
/// - Closing lines (breadcrumb closures)
#[derive(Debug)]
pub struct ErrorSection {
    /// Lines before the error (breadcrumbs like "GameObject { ... Transform {")
    pub context_lines: Vec<Line>,
    /// The main error line with underline
    pub error_line: ErrorLine,
    /// Lines after the error (closing breadcrumbs like "} ... }")
    pub closing_lines: Vec<Line>,
}

/// A single line in the output with positioned segments.
///
/// Lines are composed of segments that each have a specific position and style.
/// This allows for precise control over spacing and formatting.
#[derive(Debug)]
pub struct Line {
    /// The gutter (typically "   | ")
    pub gutter: &'static str,
    /// Content segments with their positions
    pub segments: Vec<Segment>,
}

/// A segment of content within a line.
///
/// Each segment represents a piece of text with its starting position and semantic style.
/// Positions are automatically tracked by LineBuilder to ensure accuracy.
#[derive(Debug)]
pub struct Segment {
    /// The text content
    pub text: String,
    /// Starting column position (0-based, relative to start of line content)
    pub start_col: usize,
    /// Semantic styling information
    pub style: SegmentStyle,
}

/// Semantic styling information for segments.
///
/// This allows the rendering pipeline to apply different formatting based on
/// the meaning of the content, not just its position.
#[derive(Debug, Clone)]
pub enum SegmentStyle {
    /// Normal text (indentation, punctuation)
    Normal,
    /// Field names in struct contexts
    FieldName,
    /// Pattern content (values, operators, etc.)
    Pattern,
    /// Actual values in error messages
    Actual,
    /// Underline characters
    Underline,
}

/// Specialized line for errors that includes both content and underline.
///
/// This represents the two-line error format:
/// ```text
/// "   |     field: pattern,"
/// "   |           ^^^^^^^ actual: value"
/// ```
#[derive(Debug)]
pub struct ErrorLine {
    /// The main content line
    pub content: Line,
    /// The underline line below the content
    pub underline: UnderlineLine,
    /// Field path for error context
    pub field_path: String,
    /// Line number for error context
    pub line_number: u32,
}

/// Underline line with precise positioning.
///
/// Represents the underline portion of an error with automatic position tracking.
/// The positions are captured when the content is built, not calculated afterward.
#[derive(Debug)]
pub struct UnderlineLine {
    /// The gutter (typically "   | ")
    pub gutter: &'static str,
    /// Starting column of the underline (0-based, relative to line content)
    pub underline_start: usize,
    /// Ending column of the underline (0-based, exclusive)
    pub underline_end: usize,
    /// The actual value text to display
    pub actual_text: String,
}

/// Builder for constructing lines with automatic position tracking.
///
/// This is the core component that eliminates manual position calculations.
/// As text is added, positions are tracked automatically and can be retrieved
/// for use in underlines or other positioned elements.
#[derive(Debug)]
pub struct LineBuilder {
    /// Accumulated segments
    pub segments: Vec<Segment>,
    /// Current position in the line (next character to be written)
    current_position: usize,
}

impl LineBuilder {
    /// Create a new line builder starting at position 0.
    pub fn new() -> Self {
        LineBuilder {
            segments: Vec::new(),
            current_position: 0,
        }
    }

    /// Add text to the line and return the range it occupies.
    ///
    /// This is the fundamental operation that provides automatic position tracking.
    /// The returned range can be used for underlines or other positioning needs.
    ///
    /// # Returns
    /// A tuple `(start, end)` where:
    /// - `start` is the starting column (inclusive)
    /// - `end` is the ending column (exclusive)
    ///
    /// # Example
    /// ```rust
    /// use assert_struct::error_document::{LineBuilder, SegmentStyle};
    ///
    /// let mut builder = LineBuilder::new();
    /// let (start, end) = builder.add("hello", SegmentStyle::Pattern);
    /// assert_eq!(start, 0);
    /// assert_eq!(end, 5);
    ///
    /// let (start2, end2) = builder.add(" world", SegmentStyle::Normal);
    /// assert_eq!(start2, 5);
    /// assert_eq!(end2, 11);
    /// ```
    pub fn add(&mut self, text: impl Into<String>, style: SegmentStyle) -> (usize, usize) {
        let text = text.into();
        let start = self.current_position;
        let end = start + text.len();

        self.segments.push(Segment {
            text,
            start_col: start,
            style,
        });

        self.current_position = end;
        (start, end)
    }

    /// Add text and track the position of a specific substring within it.
    ///
    /// This is useful when you need to add a larger piece of text but only
    /// want to track the position of a specific part of it.
    ///
    /// # Returns
    /// `Some((start, end))` if the substring is found, `None` otherwise.
    ///
    /// # Example
    /// ```rust
    /// use assert_struct::error_document::{LineBuilder, SegmentStyle};
    ///
    /// let mut builder = LineBuilder::new();
    /// let pos = builder.add_with_tracking("prefix: value", "value", SegmentStyle::Pattern);
    /// assert_eq!(pos, Some((8, 13)));  // "value" starts at column 8
    /// ```
    pub fn add_with_tracking(
        &mut self,
        text: impl Into<String>,
        track_substring: &str,
        style: SegmentStyle,
    ) -> Option<(usize, usize)> {
        let text = text.into();
        let start = self.current_position;

        // Find the substring within the text
        let tracked_pos = text
            .find(track_substring)
            .map(|pos| (start + pos, start + pos + track_substring.len()));

        self.segments.push(Segment {
            text: text.clone(),
            start_col: start,
            style,
        });

        self.current_position = start + text.len();
        tracked_pos
    }

    /// Get the current position (where the next character would be written).
    pub fn current_position(&self) -> usize {
        self.current_position
    }
}

impl Default for LineBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl ErrorDocument {
    /// Create a new empty error document.
    pub fn new() -> Self {
        ErrorDocument {
            sections: Vec::new(),
        }
    }

    /// Add an error section to the document.
    pub fn add_section(&mut self, section: ErrorSection) {
        self.sections.push(section);
    }

    /// Render the complete document to a string.
    ///
    /// This produces the final output string that matches the current error format.
    /// The rendering is deterministic and consistent across different error types.
    pub fn render(&self) -> String {
        let mut output = String::new();

        // Add header based on number of errors
        if self.sections.len() == 1 {
            output.push_str("assert_struct! failed:\n\n");
        } else {
            output.push_str(&format!(
                "assert_struct! failed: {} mismatches\n",
                self.sections.len()
            ));
        }

        // Render all sections
        for section in &self.sections {
            section.render_to(&mut output);
        }

        output
    }
}

impl Default for ErrorDocument {
    fn default() -> Self {
        Self::new()
    }
}

impl ErrorSection {
    /// Create a new error section.
    pub fn new(error_line: ErrorLine) -> Self {
        ErrorSection {
            context_lines: Vec::new(),
            error_line,
            closing_lines: Vec::new(),
        }
    }

    /// Add a context line (breadcrumb) before the error.
    pub fn add_context_line(&mut self, line: Line) {
        self.context_lines.push(line);
    }

    /// Add a closing line after the error.
    pub fn add_closing_line(&mut self, line: Line) {
        self.closing_lines.push(line);
    }

    /// Render this section to the output string.
    pub fn render_to(&self, output: &mut String) {
        // Render context lines (breadcrumbs)
        for line in &self.context_lines {
            line.render_to(output);
        }

        // Render the main error line
        self.error_line.render_to(output);

        // Render closing lines
        for line in &self.closing_lines {
            line.render_to(output);
        }
    }
}

impl Line {
    /// Create a new line with the given gutter.
    pub fn new(gutter: &'static str) -> Self {
        Line {
            gutter,
            segments: Vec::new(),
        }
    }

    /// Create a line from a LineBuilder.
    pub fn from_builder(gutter: &'static str, builder: LineBuilder) -> Self {
        Line {
            gutter,
            segments: builder.segments,
        }
    }

    /// Render this line to the output string.
    ///
    /// This handles proper spacing between segments based on their positions.
    pub fn render_to(&self, output: &mut String) {
        output.push_str(self.gutter);

        let mut last_end = 0;
        for segment in &self.segments {
            // Add any padding needed to reach the segment's start position
            for _ in last_end..segment.start_col {
                output.push(' ');
            }
            output.push_str(&segment.text);
            last_end = segment.start_col + segment.text.len();
        }
        output.push('\n');
    }
}

impl ErrorLine {
    /// Create a new error line.
    pub fn new(
        content: Line,
        underline: UnderlineLine,
        field_path: String,
        line_number: u32,
    ) -> Self {
        ErrorLine {
            content,
            underline,
            field_path,
            line_number,
        }
    }

    /// Render this error line (content + underline) to the output string.
    pub fn render_to(&self, output: &mut String) {
        // Add the error type header
        output.push_str("mismatch:\n");

        // Add location information
        output.push_str(&format!(
            "  --> `{}` (line {})\n",
            self.field_path, self.line_number
        ));

        // Render the content line
        self.content.render_to(output);

        // Render the underline line
        self.underline.render_to(output);
    }
}

impl UnderlineLine {
    /// Create a new underline line.
    pub fn new(
        gutter: &'static str,
        underline_start: usize,
        underline_end: usize,
        actual_text: String,
    ) -> Self {
        UnderlineLine {
            gutter,
            underline_start,
            underline_end,
            actual_text,
        }
    }

    /// Render this underline to the output string.
    pub fn render_to(&self, output: &mut String) {
        output.push_str(self.gutter);

        // Add spaces up to the underline start
        for _ in 0..self.underline_start {
            output.push(' ');
        }

        // Add underline characters
        for _ in self.underline_start..self.underline_end {
            output.push('^');
        }

        // Add the actual value
        output.push_str(&format!(" actual: {}\n", self.actual_text));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_line_builder_position_tracking() {
        let mut builder = LineBuilder::new();

        let (start1, end1) = builder.add("hello", SegmentStyle::Normal);
        assert_eq!(start1, 0);
        assert_eq!(end1, 5);

        let (start2, end2) = builder.add(" ", SegmentStyle::Normal);
        assert_eq!(start2, 5);
        assert_eq!(end2, 6);

        let (start3, end3) = builder.add("world", SegmentStyle::Pattern);
        assert_eq!(start3, 6);
        assert_eq!(end3, 11);

        assert_eq!(builder.current_position(), 11);
    }

    #[test]
    fn test_line_builder_with_tracking() {
        let mut builder = LineBuilder::new();

        let pos = builder.add_with_tracking("prefix: value", "value", SegmentStyle::Pattern);
        assert_eq!(pos, Some((8, 13)));

        let pos_missing = builder.add_with_tracking("no match here", "xyz", SegmentStyle::Normal);
        assert_eq!(pos_missing, None);
    }

    #[test]
    fn test_line_rendering() {
        let mut builder = LineBuilder::new();
        builder.add("    ", SegmentStyle::Normal); // indent
        builder.add("field: ", SegmentStyle::FieldName);
        builder.add("value", SegmentStyle::Pattern);

        let line = Line::from_builder("   | ", builder);
        let mut output = String::new();
        line.render_to(&mut output);

        assert_eq!(output, "   |     field: value\n");
    }

    #[test]
    fn test_underline_rendering() {
        let underline = UnderlineLine::new("   | ", 4, 9, "actual".to_string());
        let mut output = String::new();
        underline.render_to(&mut output);

        assert_eq!(output, "   |     ^^^^^ actual: actual\n");
    }

    #[test]
    fn test_complete_error_section() {
        // Build content line
        let mut content_builder = LineBuilder::new();
        content_builder.add("    ", SegmentStyle::Normal);
        content_builder.add("age: ", SegmentStyle::FieldName);
        let (pattern_start, pattern_end) = content_builder.add("25", SegmentStyle::Pattern);
        content_builder.add(",", SegmentStyle::Normal);

        let content_line = Line::from_builder("   | ", content_builder);

        // Build underline
        let underline = UnderlineLine::new("   | ", pattern_start, pattern_end, "17".to_string());

        // Create error line
        let error_line = ErrorLine::new(content_line, underline, "test.age".to_string(), 42);

        // Create section
        let section = ErrorSection::new(error_line);

        // Render
        let mut output = String::new();
        section.render_to(&mut output);

        // Verify structure (specific format will be refined)
        assert!(output.contains("mismatch:"));
        assert!(output.contains("age: 25,"));
        assert!(output.contains("^^ actual: 17"));
    }
}
