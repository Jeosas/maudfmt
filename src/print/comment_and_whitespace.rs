use crop::RopeSlice;
use proc_macro2::{LineColumn, extra::DelimSpan};
use syn::spanned::Spanned as _;

use crate::print::Printer;

impl<'a, 'b> Printer<'a, 'b> {
    // Returns true if a comment was inserted
    pub fn print_attr_comment(&mut self, loc: LineColumn) -> bool {
        if !self.is_trailing(loc) {
            return false;
        }

        // LineColumn.line is 1-indexed
        let token_end_byte = self.source.byte_of_line(loc.line - 1) + loc.column;
        let next_line_start_byte = self.source.byte_of_line(loc.line);

        if let Some(comment) = self
            .source
            .byte_slice(token_end_byte..next_line_start_byte)
            .to_string()
            .split_once("//")
            .map(|(_, txt)| txt)
            .map(str::trim_end)
            .map(str::to_string)
        {
            self.write("  ");
            self.write_comment_text(&comment);
            return true;
        }

        false
    }

    pub fn print_inline_comment_and_whitespace(
        &mut self,
        loc: LineColumn,
        indent_level: usize,
        preserve_blank_lines: bool,
    ) {
        let mut cursor_line = loc.line - 1; // LineColumn.line is 1-indexed
        if cursor_line == 0 {
            // line is already the top of the document
            return;
        }

        if !self.is_leading(loc) {
            return;
        }

        // Keep whitespace
        if preserve_blank_lines
            && self
                .source
                .line(cursor_line - 1)
                .to_string()
                .trim()
                .is_empty()
        {
            self.buf = String::new(); // remove indent for less bytes in final file
            self.new_line(indent_level);
            return;
        }

        let mut comments = Vec::new();

        while let Some(comment) = extract_inline_comment(self.source.line(cursor_line - 1)) {
            comments.push(comment);
            cursor_line -= 1;
        }

        while let Some(comment) = comments.pop() {
            self.write_comment_text(&comment);
            self.new_line(indent_level);
        }
    }

    pub fn print_block_comments(&mut self, delim_span: DelimSpan, indent_level: usize) {
        // LineColumn.line is 1-indexed
        let start_line = delim_span.span().start().line - 1;
        let end_line = delim_span.span().end().line - 1;

        for line_idx in (start_line + 1)..end_line {
            let line = self.source.line(line_idx);
            if let Some((_, comment_part)) = line.to_string().split_once("//") {
                self.write_comment_line(comment_part, indent_level);
            }
        }
    }

    pub fn block_contains_comments(&self, delim_span: DelimSpan) -> bool {
        // LineColumn.line is 1-indexed
        let start_line = delim_span.span().start().line - 1;
        let end_line = delim_span.span().end().line - 1;

        if start_line == end_line {
            // closed brackets, let attr_comment handle it
            return false;
        }

        (start_line..=end_line).any(|line| {
            self.source
                .line(line)
                .to_string()
                .split_once("//")
                .is_some()
        })
    }

    pub fn print_trailing_comments(&mut self, delim_span: DelimSpan, indent_level: usize) {
        let start_line = delim_span.span().start().line - 1;
        let end_line = delim_span.span().end().line - 1;

        for line_idx in (start_line + 1)..end_line {
            let line = self.source.line(line_idx);
            let line_string = line.to_string();

            if let Some((before_comment, comment_part)) = line_string.split_once("//") {
                if before_comment.trim().is_empty() {
                    let has_content_after = ((line_idx + 1)..end_line).any(|later_line_idx| {
                        let later_line = self.source.line(later_line_idx);
                        let later_line_string = later_line.to_string();

                        if let Some((before_comment, _)) = later_line_string.split_once("//") {
                            !before_comment.trim().is_empty()
                        } else {
                            !later_line_string.trim().is_empty()
                        }
                    });

                    if !has_content_after {
                        self.write_comment_line(comment_part, indent_level);
                    }
                }
            }
        }
    }

    fn write_comment_text(&mut self, comment: &str) {
        self.write("//");
        if !comment.is_empty() {
            if !comment.starts_with(" ") {
                self.write(" ");
            }
            self.write(comment);
        }
    }

    fn write_comment_line(&mut self, comment_part: &str, indent_level: usize) {
        self.new_line(indent_level);
        let comment = comment_part.trim_end();
        self.write_comment_text(comment);
    }

    // Check if a Markup location is leading a line or not
    // Prevents inline comments and whitespace
    // from being printed more than once
    fn is_leading(&self, loc: LineColumn) -> bool {
        // LineColumn.line is 1-indexed
        let line = self.source.line(loc.line - 1);
        // is start of the line ?
        line.byte_slice(..loc.column).to_string().trim().is_empty()
    }

    // Check if a Markup location is trainling a line or not
    // Prevents attrs comments from being printed more than once
    fn is_trailing(&self, loc: LineColumn) -> bool {
        // LineColumn.line is 1-indexed
        let line = self.source.line(loc.line - 1);

        // is start of the line ?
        let line_string = line.byte_slice(loc.column..).to_string();
        line_string
            .split_once("//") // remove comment if exist
            .map(|(txt, _)| txt)
            .unwrap_or(&line_string)
            .trim()
            .is_empty()
    }
}

fn extract_inline_comment(line: RopeSlice) -> Option<String> {
    let line_string = line.to_string();
    if line_string.trim().starts_with("//") {
        line_string
            .split_once("//")
            .map(|(_, txt)| txt)
            .map(str::trim_end)
            .map(str::to_string)
    } else {
        None
    }
}
