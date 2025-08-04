use crate::{
    print::Printer,
    vendor::ast::{Element, Markup},
};

impl<'a, 'b> Printer<'a, 'b> {
    pub fn print_markup<E: Into<Element>>(
        &mut self,
        markup: Markup<E>,
        indent_level: usize,
        preserve_blank_lines: bool,
    ) {
        match markup {
            Markup::Lit(html_lit) => self.print_lit(html_lit, indent_level, preserve_blank_lines),
            Markup::Splice { paren_token, expr } => {
                self.print_splice(expr, paren_token, indent_level, preserve_blank_lines)
            }
            Markup::Element(element) => {
                self.print_element_with_contents(element.into(), indent_level, preserve_blank_lines)
            }
            Markup::Block(block) => self.print_block(block, indent_level),
            Markup::ControlFlow(control_flow) => {
                self.print_control_flow(control_flow, indent_level)
            }
            Markup::Semi(_semi) => self.write(";"),
        }
    }
}
