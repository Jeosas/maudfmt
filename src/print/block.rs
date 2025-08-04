use syn::spanned::Spanned as _;

use crate::{
    line_length::block_len,
    print::Printer,
    vendor::ast::{Block, Element},
};

impl<'a, 'b> Printer<'a, 'b> {
    pub fn print_block<E: Into<Element>>(&mut self, block: Block<E>, indent_level: usize) {
        self.print_inline_comment_and_whitespace(
            block.brace_token.span.span().start(),
            indent_level,
            true,
        );

        let expand = self.block_contains_comments(block.brace_token.span) || {
            if let Some(blk_len) = block_len(&block) {
                (self.line_len() + blk_len) > self.options.line_length
            } else {
                true
            }
        };
        if block.markups.markups.is_empty() && !self.block_contains_comments(block.brace_token.span)
        {
            self.write("{}");
            self.print_attr_comment(block.brace_token.span.close().span().end());
        } else if !expand {
            self.write("{");
            if self.print_attr_comment(block.brace_token.span.open().span().end()) {
                // expand if comment
                self.new_line(indent_level + 1);
                for markup in block.markups.markups {
                    // there should be only one value
                    self.print_markup(markup, indent_level + 1, true);
                }
                self.new_line(indent_level);
                self.write("}");
            } else {
                for markup in block.markups.markups {
                    self.write(" ");
                    self.print_markup(markup, indent_level, false);
                }
                self.write(" }");
            }
            self.print_attr_comment(block.brace_token.span.close().span().end());
        } else {
            self.write("{");
            self.print_attr_comment(block.brace_token.span.open().span().end());

            if block.markups.markups.is_empty() {
                // Handle empty block with comments
                self.print_block_comments(block.brace_token.span, indent_level + 1);
            } else {
                for markup in block.markups.markups {
                    self.new_line(indent_level + 1);
                    self.print_markup(markup, indent_level + 1, true);
                }
                self.print_trailing_comments(block.brace_token.span, indent_level + 1);
            }

            self.new_line(indent_level);
            self.write("}");
            self.print_attr_comment(block.brace_token.span.close().span().end());
        }
    }
}
