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

#[cfg(test)]
mod test {
    use crate::testing::*;

    test_default!(
        escaping,
        r#"
        use maud::PreEscaped;
        html!{"<script>alert(\"XSS\")</script>" (PreEscaped("<script>alert(\"XSS\")</script>"))}
        "#,
        r#"
        use maud::PreEscaped;
        html! {
            "<script>alert(\"XSS\")</script>"
            (PreEscaped("<script>alert(\"XSS\")</script>"))
        }
        "#
    );

    test_default!(
        doctype,
        r#"
        use maud::DOCTYPE;
        html!{(DOCTYPE)}
        "#,
        r#"
        use maud::DOCTYPE;
        html! {
            (DOCTYPE)
        }
        "#
    );

    test_default!(
        splices,
        r#"
        html! { p { "Hi, " (best_pony) "!" }
            p{"I have "(numbers.len())" numbers, ""and the first one is "(numbers[0])}}
        "#,
        r#"
        html! {
            p { "Hi, " (best_pony) "!" }
            p { "I have " (numbers.len()) " numbers, " "and the first one is " (numbers[0]) }
        }
        "#
    );

    test_default!(
        splices_block,
        r#"
        html!{p{({
        let f: Foo = something_convertible_to_foo()?; f.time().format("%H%Mh") })}}
        "#,
        r#"
        html! {
            p {
                ({
                    let f: Foo = something_convertible_to_foo()?;
                    f.time().format("%H%Mh")
                })
            }
        }
        "#
    );

    test_default!(
        line_length_long_splice,
        r##"
        html! {
            (super_long_splice.with_a_super_long_method().and_an_other_super_super_long_method_to_call_afer().unwarp())
        }
        "##,
        r##"
        html! {
            ({
                super_long_splice
                    .with_a_super_long_method()
                    .and_an_other_super_super_long_method_to_call_afer()
                    .unwarp()
            })
        }
        "##
    );

    test_default!(
        blank_line_above_splice,
        r#"
        html!{
            .test {

            .test3 {

            (a)
            }
            }
        }
        "#,
        r#"
        html! {
            .test {
                .test3 { (a) }
            }
        }
        "#
    );
}
