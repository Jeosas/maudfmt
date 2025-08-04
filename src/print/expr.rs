use syn::{Expr, spanned::Spanned as _};

use crate::{
    format::line_column_to_byte,
    print::Printer,
    unparse::{unparse_expr, unparse_stmts},
};

impl<'a, 'b> Printer<'a, 'b> {
    pub fn print_expr(&mut self, expr: Expr, indent_level: usize) {
        let span = expr.span();
        let lines: Vec<String> = match std::panic::catch_unwind(|| match expr {
            Expr::Block(expr_block) => {
                unparse_stmts(&expr_block.block.stmts, self.base_indent + indent_level)
            }
            _ => unparse_expr(&expr, self.base_indent + indent_level),
        }) {
            Ok(lines) => lines,
            Err(_) => {
                let start_byte = line_column_to_byte(self.source, span.start());
                let end_byte = line_column_to_byte(self.source, span.end());
                let original_text = self.source.byte_slice(start_byte..end_byte).to_string();
                eprintln!(
                    "Warning: prettyplease panicked formatting expression, leaving unchanged: {original_text}"
                );
                vec![original_text]
            }
        };

        match lines.len() {
            0 => (),
            1 => self.write(lines[0].trim()),
            _ => {
                self.write("{\n");
                self.write(&lines.join("\n"));
                self.new_line(indent_level);
                self.write("}");
            }
        }
    }

    pub fn print_toggle_expr(&mut self, expr: Expr, indent_level: usize) {
        match expr {
            Expr::Block(expr_block) => {
                let lines =
                    unparse_stmts(&expr_block.block.stmts, self.base_indent + indent_level + 1);

                if lines.is_empty() || (lines.len() == 1 && lines[0].trim().is_empty()) {
                    self.write("{}");
                } else {
                    self.write("{\n");
                    self.write(&lines.join("\n"));
                    self.new_line(indent_level + 1);
                    self.write("}");
                }
            }
            _ => {
                let lines = unparse_expr(&expr, self.base_indent + indent_level + 1);

                match lines.len() {
                    0 => (),
                    1 => self.write(lines[0].trim()),
                    _ => {
                        self.write("\n");
                        self.write(&lines.join("\n"));
                        self.new_line(indent_level + 1);
                    }
                }
            }
        }
    }
}
