use std::ops::Range;

use crate::ast::{DiagnosticParse, Element};
use anyhow::{Context, Result};
use crop::Rope;
use syn::{
    parse::{ParseStream, Parser},
    spanned::Spanned,
};

use crate::{ast::Markups, collect::MaudMacro, print::print};

const IGNORE_PLACEHOLDER: &str = "\"__MAUDFMT_IGNORED_PLACEHOLDER__\"";

pub struct FormatOptions {
    pub line_length: usize,
    pub macro_names: Vec<String>,
}

impl Default for FormatOptions {
    fn default() -> Self {
        FormatOptions {
            line_length: 100,
            macro_names: vec![String::from("maud::html"), String::from("html")],
        }
    }
}

#[derive(Debug)]
struct TextEdit {
    range: Range<usize>,
    new_text: String,
}

pub fn format_source(
    source: &mut Rope,
    macros: Vec<MaudMacro<'_>>,
    options: &FormatOptions,
) -> String {
    let mut edits = Vec::new();

    for maud_mac in macros {
        let mac = maud_mac.macro_;
        let start = mac.path.span().start();
        let end = mac.delimiter.span().close().end();
        let start_byte = line_column_to_byte(source, start);
        let end_byte = line_column_to_byte(source, end);

        match format_macro(&maud_mac, source, options) {
            Ok(new_text) => edits.push(TextEdit {
                range: start_byte..end_byte,
                new_text,
            }),
            Err(e) => eprintln!("{e}"),
        }
    }

    let mut last_offset: isize = 0;
    for edit in edits {
        let start = edit.range.start;
        let end = edit.range.end;
        let new_text = edit.new_text;

        source.replace(
            (start as isize + last_offset) as usize..(end as isize + last_offset) as usize,
            &new_text,
        );
        last_offset += new_text.len() as isize - (end as isize - start as isize);
    }

    source.to_string()
}

fn format_macro(mac: &MaudMacro, source: &Rope, options: &FormatOptions) -> Result<String> {
    let mut diagnostics = Vec::new();
    let markups: Markups<Element> = Parser::parse2(
        |input: ParseStream| Markups::diagnostic_parse(input, &mut diagnostics),
        mac.macro_.tokens.clone(),
    )
    .context("Failed to parse maud macro")?;

    Ok(print(markups, mac, source, options))
}

pub fn line_column_to_byte(source: &Rope, point: proc_macro2::LineColumn) -> usize {
    let line_byte = source.byte_of_line(point.line - 1);
    let line = source.line(point.line - 1);
    let char_byte: usize = line.chars().take(point.column).map(|c| c.len_utf8()).sum();
    line_byte + char_byte
}

pub fn preprocess_source_for_ignore(source: &str) -> (String, Vec<&str>) {
    let lines: Vec<&str> = source.lines().collect();
    let mut processed_lines = Vec::with_capacity(lines.len());
    let mut ignore_info = Vec::new();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i];

        if let Some((_, comment_part)) = line.split_once("//") {
            let comment_trimmed = comment_part.trim();
            if comment_trimmed.starts_with("maudfmt-ignore") && i + 1 < lines.len() {
                ignore_info.push(lines[i + 1]);

                processed_lines.push(line);
                processed_lines.push(IGNORE_PLACEHOLDER);

                i += 2;
                continue;
            }
        }

        processed_lines.push(line);
        i += 1;
    }

    if source.ends_with('\n') {
        processed_lines.push("");
    }

    (processed_lines.join("\n"), ignore_info)
}

pub fn reinsert_ignored_lines_in_source(formatted_source: &str, ignore_info: &[&str]) -> String {
    let lines: Vec<&str> = formatted_source.lines().collect();
    let mut result_lines = Vec::with_capacity(lines.len());
    let mut ignore_index = 0;

    for line in lines {
        if line.trim() == IGNORE_PLACEHOLDER && ignore_index < ignore_info.len() {
            result_lines.push(ignore_info[ignore_index]);
            ignore_index += 1;
        } else {
            result_lines.push(line);
        }
    }

    if formatted_source.ends_with('\n') {
        result_lines.push("");
    }

    result_lines.join("\n")
}

#[cfg(test)]
mod test {
    use crate::testing::*;

    test_default!(
        maudfmt_ignore_multiple_lines,
        r#"
        html! {
            p {"formatted" }
            // maudfmt-ignore
            div class="unformatted"   id="test" { "content" }
            // maudfmt-ignore
            span  style="color:red;" { "text" }
            h1 {"formatted" }
        }
        "#,
        r#"
        html! {
            p { "formatted" }
            // maudfmt-ignore
            div class="unformatted"   id="test" { "content" }
            // maudfmt-ignore
            span  style="color:red;" { "text" }
            h1 { "formatted" }
        }
        "#
    );

    test_default!(
        maudfmt_ignore_with_extra_brace,
        r#"
        html! {
            p {"formatted" }
            //maudfmt-ignore
            div class="unformatted"   id="test" { "content" } { {
            span {"formatted again" }
        }
        "#,
        r#"
        html! {
            p { "formatted" }
            // maudfmt-ignore
            div class="unformatted"   id="test" { "content" } { {
            span { "formatted again" }
        }
        "#
    );

    test_default!(
        maudfmt_ignore_with_comment_text,
        r#"
        html! {
            p {"formatted" }
            // maudfmt-ignore this line is special
            div class="unformatted"   id="test" { "content" }
            span {"formatted again" }
        }
        "#,
        r#"
        html! {
            p { "formatted" }
            // maudfmt-ignore this line is special
            div class="unformatted"   id="test" { "content" }
            span { "formatted again" }
        }
        "#
    );

    test_default!(
        maudfmt_ignore_long_line_no_split,
        r#"
        html! {
            div {
                p { "formatted"}
                // maudfmt-ignore
                span class="unformatted"   id="test" { "content" "more content to split the lines" "even more content to split the lines further" }
                h1 {"formatted" }
            }
        }
        "#,
        r#"
        html! {
            div {
                p { "formatted" }
                // maudfmt-ignore
                span class="unformatted"   id="test" { "content" "more content to split the lines" "even more content to split the lines further" }
                h1 { "formatted" }
            }
        }
        "#
    );
}
