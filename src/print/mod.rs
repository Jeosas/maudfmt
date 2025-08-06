use crop::Rope;

use crate::{ast::*, collect::MaudMacro, format::FormatOptions};

mod block;
mod comment_and_whitespace;
mod control_flow;
mod element;
mod expr;
mod lit;
mod markup;
mod splice;

pub fn print<'b>(
    ast: Markups<Element>,
    mac: &'b MaudMacro<'b>,
    source: &Rope,
    options: &FormatOptions,
) -> String {
    #[cfg(debug_assertions)]
    dbg!(&ast); // print ast when debugging (not release mode)

    let mut printer = Printer {
        lines: Vec::new(),
        buf: String::new(),
        base_indent: mac.indent.tabs + mac.indent.spaces / 4,
        indent_str: &String::from(" ").repeat(4),
        mac,
        source,
        options,
    };

    printer.print_ast(ast);

    printer.finish()
}

struct Printer<'a, 'b> {
    lines: Vec<String>,
    buf: String,
    base_indent: usize,
    indent_str: &'a str,
    mac: &'b MaudMacro<'b>,
    source: &'a Rope,
    options: &'a FormatOptions,
}

impl<'a, 'b> Printer<'a, 'b> {
    fn print_ast(&mut self, ast: Markups<Element>) {
        let indent_level = 0;

        self.write(&self.mac.macro_name);
        self.write("! ");

        if ast.markups.is_empty() {
            self.write("{}")
        } else {
            self.write("{");
            self.print_attr_comment(self.mac.macro_.delimiter.span().open().end());
            for markup in ast.markups {
                self.new_line(indent_level + 1);
                self.print_markup(markup, indent_level + 1, true);
            }
            self.print_trailing_comments(*self.mac.macro_.delimiter.span(), indent_level + 1);
            self.new_line(indent_level);

            let close_location = self.mac.macro_.delimiter.span().close().end();
            self.print_inline_comment_and_whitespace(close_location, indent_level, true);
            self.write("}");
            self.print_attr_comment(close_location);
        }
    }

    fn new_line(&mut self, indent_level: usize) {
        self.lines.push(self.buf.clone());
        self.buf = String::from(self.indent_str).repeat(self.base_indent + indent_level);
    }

    fn write(&mut self, content: &str) {
        self.buf += content;
    }

    fn line_len(&self) -> usize {
        self.buf.len()
    }

    fn finish(mut self) -> String {
        self.new_line(0);
        self.lines.join("\n")
    }
}

#[cfg(test)]
mod test {
    use crate::testing::*;

    test_default!(empty, "html!{ }", "html! {}");

    test_default!(
        empty_full_macro_declaration,
        "maud::html!{ }",
        "maud::html! {}"
    );

    test_default!(
        hypertext_maud_empty,
        "hypertext::maud!{ }",
        "hypertext::maud! {}"
    );

    test_default!(
        hypertext_maud_borrow_empty,
        "hypertext::maud_borrow!{ }",
        "hypertext::maud_borrow! {}"
    );

    test_default!(
        hypertext_maud_static_empty,
        "hypertext::maud_static!{ }",
        "hypertext::maud_static! {}"
    );

    test_default!(maud_empty, "maud!{ }", "maud! {}");

    test_default!(maud_borrow_empty, "maud_borrow!{ }", "maud_borrow! {}");

    test_default!(maud_static_empty, "maud_static!{ }", "maud_static! {}");
}
