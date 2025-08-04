use quote::quote;
use syn::spanned::Spanned as _;

use crate::{print::Printer, vendor::ast::HtmlLit};

impl<'a, 'b> Printer<'a, 'b> {
    // NOTE: lit do not care about line length
    //       let user take care of it
    pub fn print_lit(
        &mut self,
        html_lit: HtmlLit,
        indent_level: usize,
        preserve_blank_lines: bool,
    ) {
        self.print_inline_comment_and_whitespace(
            html_lit.span().start(),
            indent_level,
            preserve_blank_lines,
        );
        let lit = &html_lit.lit;
        self.write(&quote!(#lit).to_string());
        self.print_attr_comment(html_lit.span().end());
    }
}

#[cfg(test)]
mod test {
    use crate::testing::*;

    test_default!(
        lit,
        r#"
        html!{ "Hello world!" }
        "#,
        r#"
        html! {
            "Hello world!"
        }
        "#
    );

    // NOTE: multiline string formatting is left to the users
    test_default!(
        whitespace_in_multi_line_strings_edge_case,
        r##"
        html! {
        p {
            (PreEscaped(r#"
            Multiline

            String
            "#))
        }
        }
        "##,
        r##"
        html! {
            p {
                ({
                    PreEscaped(
                        r#"
            Multiline

            String
            "#,
                    )
                })
            }
        }
        "##
    );

    // NOTE: multiline string formatting is left to the users
    test_default!(
        correct_multiline_string_indent_in_splices,
        r##"
        html! {
            (r#"
            Multiline
            String
            "#)
        }
        "##,
        r##"
        html! {
            ({
                r#"
            Multiline
            String
            "#
            })
        }
        "##
    );
}
