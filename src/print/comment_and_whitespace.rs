use crop::RopeSlice;
use proc_macro2::{LineColumn, extra::DelimSpan};
use syn::spanned::Spanned as _;

use crate::{format::line_column_to_byte, print::Printer};

impl<'a, 'b> Printer<'a, 'b> {
    // Returns true if a comment was inserted
    pub fn print_attr_comment(&mut self, loc: LineColumn) -> bool {
        if !self.is_trailing(loc) {
            return false;
        }

        let token_end_byte = line_column_to_byte(self.source, loc);
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
        let mut cursor_line = loc.line - 1;
        if cursor_line == 0 || !self.is_leading(loc) {
            return;
        }

        if preserve_blank_lines
            && self
                .source
                .line(cursor_line - 1)
                .to_string()
                .trim()
                .is_empty()
        {
            self.buf = String::new();
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
        let start_line = delim_span.span().start().line - 1;
        let end_line = delim_span.span().end().line - 1;

        if start_line == end_line {
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

    fn is_leading(&self, loc: LineColumn) -> bool {
        let line_start_byte = self.source.byte_of_line(loc.line - 1);
        let token_start_byte = line_column_to_byte(self.source, loc);

        let before_token = self
            .source
            .byte_slice(line_start_byte..token_start_byte)
            .to_string();

        before_token.trim().is_empty()
    }

    fn is_trailing(&self, loc: LineColumn) -> bool {
        let token_end_byte = line_column_to_byte(self.source, loc);
        let next_line_start_byte = self.source.byte_of_line(loc.line);

        let line_string = self
            .source
            .byte_slice(token_end_byte..next_line_start_byte)
            .to_string();

        line_string
            .split_once("//")
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

#[cfg(test)]
mod test {
    use crate::testing::*;

    test_default!(
        comment_markup,
        r##"
        use maud::DOCTYPE;
        html!{
        (DOCTYPE)     // <!DOCTYPE html>
        }
        "##,
        r##"
        use maud::DOCTYPE;
        html! {
            (DOCTYPE)  // <!DOCTYPE html>
        }
        "##
    );

    test_default!(
        comment_empty_block,
        r#"
        html!{
            p {
                // lonely comment
            }
        }
        "#,
        r#"
        html! {
            p {
                // lonely comment
            }
        }
        "#
    );

    test_default!(
        comment_end_block,
        r#"
        html! {
            p {
                "test"
                // trailing comment
            }
        }
        "#,
        r#"
        html! {
            p {
                "test"
                // trailing comment
            }
        }
        "#
    );

    test_default!(
        comment_end_block_before_control,
        r#"
        html! {
            p {
                "test"
                // trailing comment
                @for x in y { "hi" }
            }
        }
        "#,
        r#"
        html! {
            p {
                "test"
                // trailing comment
                @for x in y { "hi" }
            }
        }
        "#
    );

    test_default!(
        keep_whitespace,
        r##"
        html!{
        "Hello"

        "World"
        }
        "##,
        r##"
        html! {
            "Hello"

            "World"
        }
        "##
    );

    test_default!(
        keep_single_whitespace,
        r##"
        html!{
        "Hello"



        "World"
        }
        "##,
        r##"
        html! {
            "Hello"

            "World"
        }
        "##
    );

    test_default!(
        force_expand_inline,
        r#"
        html! {
        h1 {
        // keep expanded
        "Poem"
        }
        }
        "#,
        r#"
        html! {
            h1 {
                // keep expanded
                "Poem"
            }
        }
        "#
    );

    test_default!(
        force_expand_attrs,
        r#"
        html! { 
        h1 { //
        "Poem"
        }
        }
        "#,
        r#"
        html! {
            h1 {  //
                "Poem"
            }
        }
        "#
    );

    test_default!(
        keep_comment_1,
        r#"
        html! {
            (DOCTYPE)
            html lang="en" {
                head {
                    // meta
                    .first {}
                    .second {}
                }
            }
        }
        "#,
        r#"
        html! {
            (DOCTYPE)
            html lang="en" {
                head {
                    // meta
                    .first {}
                    .second {}
                }
            }
        }
        "#
    );

    test_default!(
        comments_slashes_in_string,
        r#"
        html! {
            a href="http://example.org" { "This is not a comment" }
        }
        "#,
        r#"
        html! {
            a href="http://example.org" { "This is not a comment" }
        }
        "#
    );

    test_default!(
        keep_indents_in_comments_attrs,
        r#"
        html! {
        // p {
        //     "pls keep indent"
        // }
        p { }
        }
        "#,
        r#"
        html! {
            // p {
            //     "pls keep indent"
            // }
            p {}
        }
        "#
    );

    test_default!(
        keep_indents_in_comments_blocks,
        r#"
        html! {
        p { 
        // p {
        //     "pls keep indent"
        // }
        }
        }
        "#,
        r#"
        html! {
            p {
                // p {
                //     "pls keep indent"
                // }
            }
        }
        "#
    );

    test_default!(
        ensure_leading_space_in_comments,
        r#"
        html! {
        //please add leading space
        p { }
        }
        "#,
        r#"
        html! {
            // please add leading space
            p {}
        }
        "#
    );

    test_default!(
        comments_before_after_elements,
        r#"
        html! {
            // comment before element
            p { "content" }
            // comment after element
            div { "more content" }
            // final comment
        }
        "#,
        r#"
        html! {
            // comment before element
            p { "content" }
            // comment after element
            div { "more content" }
            // final comment
        }
        "#
    );

    test_default!(
        comments_before_after_control_structures,
        r#"
        html! {
            // before if
            @if condition {
                "true"
            }
            // between if and for
            @for item in items {
                // inside for
                span { (item) }
            }
            // after for
            @let x = 5;
            // after let
        }
        "#,
        r#"
        html! {
            // before if
            @if condition { "true" }
            // between if and for
            @for item in items {
                // inside for
                span { (item) }
            }
            // after for
            @let x = 5;
            // after let
        }
        "#
    );

    test_default!(
        comments_with_nested_blocks,
        r#"
        html! {
            div {
                // comment in outer block
                p {
                    // comment in inner block
                    "text"
                    // trailing comment in inner
                }
                // comment between elements
                span { "more text" }
                // final comment in outer
            }
        }
        "#,
        r#"
        html! {
            div {
                // comment in outer block
                p {
                    // comment in inner block
                    "text"
                    // trailing comment in inner
                }
                // comment between elements
                span { "more text" }
                // final comment in outer
            }
        }
        "#
    );

    test_default!(
        comments_with_attributes,
        r#"
        html! {
            // before element with attrs
            div class="test" id="main" {
                "content"
            }
            // after element with attrs
        }
        "#,
        r#"
        html! {
            // before element with attrs
            div class="test" id="main" { "content" }
            // after element with attrs
        }
        "#
    );

    test_default!(
        inline_comments_on_constructs,
        r#"
        html! {
            p { "text" }  // inline on element
            @if true { "yes" }  // inline on control
            (variable)  // inline on splice
            div;  // inline on void element
        }
        "#,
        r#"
        html! {
            p { "text" }  // inline on element
            @if true { "yes" }  // inline on control
            (variable)  // inline on splice
            div;  // inline on void element
        }
        "#
    );

    test_default!(
        comments_with_match_expressions,
        r#"
        html! {
            // before match
            @match value {
                // comment in match
                Some(x) => {
                    // comment in arm
                    span { (x) }
                },
                // comment between arms
                None => {
                    "empty"
                    // trailing in arm
                }
                // final comment in match
            }
            // after match
        }
        "#,
        r#"
        html! {
            // before match
            @match value {
                Some(x) => {
                    // comment in arm
                    span { (x) }
                }
                None => {
                    "empty"
                    // trailing in arm
                }
                // final comment in match
            }
            // after match
        }
        "#
    );

    test_default!(
        comments_with_while_loops,
        r#"
        html! {
            // before while
            @while condition {
                // inside while
                p { "looping" }
                // more in while
            }
            // after while
            @while let Some(x) = iter.next() {
                // inside while let
                span { (x) }
            }
            // final comment
        }
        "#,
        r#"
        html! {
            // before while
            @while condition {
                // inside while
                p { "looping" }
                // more in while
            }
            // after while
            @while let Some(x) = iter.next() {
                // inside while let
                span { (x) }
            }
            // final comment
        }
        "#
    );

    test_default!(
        comments_with_complex_splices,
        r#"
        html! {
            // before splice
            (complex_expression())  // inline on splice
            // after splice
            ({
                // comment in block splice
                let x = 5;
                x + 1
            })
            // after block splice
        }
        "#,
        r#"
        html! {
            // before splice
            (complex_expression())  // inline on splice
            // after splice
            ({
                let x = 5;
                x + 1
            })
            // after block splice
        }
        "#
    );

    test_default!(
        comments_with_classes_and_ids,
        r#"
        html! {
            // before element with class
            div.class1.class2 {
                "content"
            }
            // between elements
            p #id.class {
                "more"
            }  // inline after element
            // final comment
        }
        "#,
        r#"
        html! {
            // before element with class
            div.class1.class2 { "content" }
            // between elements
            p #id.class {
                "more"
            }  // inline after element
            // final comment
        }
        "#
    );

    test_default!(
        comments_at_block_boundaries,
        r#"
        html! {
            // start of main block
            div {
                // start of div block
                p { "content" }
                // end of div block
            }
            // end of main block
        }
        "#,
        r#"
        html! {
            // start of main block
            div {
                // start of div block
                p { "content" }
                // end of div block
            }
            // end of main block
        }
        "#
    );

    test_default!(
        comments_mixed_with_control_and_elements,
        r#"
        html! {
            // header comment
            h1 { "Title" }
            // before conditional
            @if show_content {
                // inside if
                p { "Content" }
                // before loop
                @for item in list {
                    // inside loop
                    li { (item) }  // inline in loop
                }
                // after loop
            }
            // before else
            @else {
                // inside else
                p { "No content" }
            }
            // footer comment
        }
        "#,
        r#"
        html! {
            // header comment
            h1 { "Title" }
            // before conditional
            @if show_content {
                // inside if
                p { "Content" }
                // before loop
                @for item in list {
                    // inside loop
                    li { (item) }  // inline in loop
                }
                // after loop
            } @else {
                // inside else
                p { "No content" }
            }
            // footer comment
        }
        "#
    );

    test_default!(
        utf8_characters_in_content_and_comments,
        r#"
        html! {
            p { "✕ ❌ 🚫 ⛔" }  // Various UTF-8 symbols ✓ ✗ ⚠️
            div { "こんにちは世界" }  // Japanese text 日本語
            span { "🎉🎊🎈" }  // Emojis 🌟
        }
        "#,
        r#"
        html! {
            p { "✕ ❌ 🚫 ⛔" }  // Various UTF-8 symbols ✓ ✗ ⚠️
            div { "こんにちは世界" }  // Japanese text 日本語
            span { "🎉🎊🎈" }  // Emojis 🌟
        }
        "#
    );
}
