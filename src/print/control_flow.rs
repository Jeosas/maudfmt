use syn::{Expr, spanned::Spanned as _};

use crate::{
    print::Printer,
    unparse::{unparse_local, unparse_pat},
    vendor::ast::{ControlFlow, ControlFlowKind, Element, IfExpr, IfOrBlock},
};

impl<'a, 'b> Printer<'a, 'b> {
    pub fn print_control_flow<E: Into<Element>>(
        &mut self,
        control_flow: ControlFlow<E>,
        indent_level: usize,
    ) {
        self.print_inline_comment_and_whitespace(
            control_flow.at_token.span.span().start(),
            indent_level,
            true,
        );
        match control_flow.kind {
            ControlFlowKind::If(if_expr) => {
                self.write("@");
                self.print_if_expr(if_expr, indent_level);
            }
            ControlFlowKind::For(for_expr) => {
                self.write("@for ");
                self.write(&unparse_pat(&for_expr.pat, self.base_indent + indent_level).join("\n"));
                self.write(" in ");
                self.print_expr(for_expr.expr, indent_level);
                self.write(" ");
                self.print_block(for_expr.body, indent_level);
            }
            ControlFlowKind::Let(local) => {
                self.write("@");
                self.write(&unparse_local(&local, self.base_indent + indent_level).join("\n"));
                self.write(";");
                self.print_attr_comment(local.semi_token.span().end());
            }
            ControlFlowKind::Match(match_expr) => {
                self.write("@match ");
                self.print_expr(match_expr.expr, indent_level);
                self.write(" {");
                self.print_attr_comment(match_expr.brace_token.span.open().span().end());
                for arm in match_expr.arms {
                    self.new_line(indent_level + 1);
                    self.write(&unparse_pat(&arm.pat, self.base_indent + indent_level).join("\n"));
                    if let Some((_, guard_cond)) = arm.guard {
                        self.write(" if ");
                        self.print_expr(guard_cond, indent_level);
                    }
                    self.write(" => ");
                    self.print_markup(arm.body, indent_level + 1, true);
                }
                self.print_trailing_comments(match_expr.brace_token.span, indent_level + 1);
                self.new_line(indent_level);
                self.write("}");
                self.print_attr_comment(match_expr.brace_token.span.close().span().end());
            }
            ControlFlowKind::While(while_expr) => {
                self.write("@while ");
                match while_expr.cond {
                    Expr::Let(expr_let) => {
                        // crashes prettyplease > syn can't parse it
                        self.write("let ");
                        self.write(
                            &unparse_pat(&expr_let.pat, self.base_indent + indent_level).join("\n"),
                        );
                        self.write(" = ");
                        self.print_expr(*expr_let.expr, indent_level);
                        self.write(" ");
                    }
                    _ => {
                        // usual case
                        self.print_expr(while_expr.cond, indent_level);
                        self.write(" ");
                    }
                }
                self.print_block(while_expr.body, indent_level);
            }
        }
    }

    fn print_if_expr<E: Into<Element>>(&mut self, if_expr: IfExpr<E>, indent_level: usize) {
        self.write("if ");
        match if_expr.cond {
            Expr::Let(expr_let) => {
                // crashes prettyplease > syn can't parse it
                self.write("let ");
                self.write(&unparse_pat(&expr_let.pat, self.base_indent + indent_level).join("\n"));
                self.write(" = ");
                self.print_expr(*expr_let.expr, indent_level);
                self.write(" ");
            }
            _ => {
                // usual case
                self.print_expr(if_expr.cond, indent_level);
                self.write(" ");
            }
        }

        self.print_block(if_expr.then_branch, indent_level);

        if let Some((_, _, if_or_block)) = if_expr.else_branch {
            self.write(" @else ");

            match *if_or_block {
                IfOrBlock::If(else_if_expr) => {
                    self.print_if_expr(else_if_expr, indent_level);
                }
                IfOrBlock::Block(block) => {
                    self.print_block(block, indent_level);
                }
            }
        }
    }
}
