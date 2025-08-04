use syn::{Expr, spanned::Spanned as _, token::Paren};

use crate::print::Printer;

impl<'a, 'b> Printer<'a, 'b> {
    pub fn print_splice(
        &mut self,
        expr: Expr,
        paren: Paren,
        indent_level: usize,
        preserve_blank_lines: bool,
    ) {
        self.print_inline_comment_and_whitespace(
            paren.span.span().start(),
            indent_level,
            preserve_blank_lines,
        );
        self.write("(");

        if self.print_attr_comment(paren.span.open().span().end()) {
            // expand if comment or line_length exceeded
            // NOTE: comments on splice lines aren't supported
            //       since syn/prettyprinter do not support them
            self.new_line(indent_level + 1);
            self.print_expr(expr, indent_level + 1);
            self.new_line(indent_level);
            self.write(")");
        } else {
            self.print_expr(expr, indent_level);
            self.write(")");
        }
        self.print_attr_comment(paren.span.close().span().end());
    }
}
