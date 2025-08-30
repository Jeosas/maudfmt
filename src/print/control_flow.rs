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
                // handle range separately, to avoid prettyplease adding unnecessary parentheses
                match for_expr.expr {
                    Expr::Range(range_expr) => {
                        self.print_range(range_expr, indent_level);
                    }
                    _ => {
                        self.print_expr(for_expr.expr, indent_level);
                    }
                }
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

    fn print_range(&mut self, range_expr: syn::ExprRange, indent_level: usize) {
        if let Some(ref start) = range_expr.start {
            self.print_expr(*start.clone(), indent_level);
        }
        match range_expr.limits {
            syn::RangeLimits::HalfOpen(_) => self.write(".."),
            syn::RangeLimits::Closed(_) => self.write("..="),
        }
        if let Some(ref end) = range_expr.end {
            self.print_expr(*end.clone(), indent_level);
        }
    }
}

#[cfg(test)]
mod test {
    use crate::testing::*;

    test_default!(
        control_if,
        r#"
        html! { @if user == Princess::Luna {h1{"Super secret woona to-do list"}
        ul{li{"Nuke the Crystal Empire"}li{"Kick a puppy"}li{"Evil laugh"}}}}
        "#,
        r#"
        html! {
            @if user == Princess::Luna {
                h1 { "Super secret woona to-do list" }
                ul {
                    li { "Nuke the Crystal Empire" }
                    li { "Kick a puppy" }
                    li { "Evil laugh" }
                }
            }
        }
        "#
    );

    test_default!(
        control_if_else,
        r#"
        html! { @if user == Princess::Luna {h1{"Super secret woona to-do list"}
        ul{li{"Nuke the Crystal Empire"}li{"Kick a puppy"}li{"Evil laugh"}}}
        @else { p { "Nothing to see here; move along." } }}
        "#,
        r#"
        html! {
            @if user == Princess::Luna {
                h1 { "Super secret woona to-do list" }
                ul {
                    li { "Nuke the Crystal Empire" }
                    li { "Kick a puppy" }
                    li { "Evil laugh" }
                }
            } @else {
                p { "Nothing to see here; move along." }
            }
        }
        "#
    );

    test_default!(
        control_if_elseif_else,
        r#"
        html! { @if user == Princess::Luna {h1{"Super secret woona to-do list"}
        ul{li{"Nuke the Crystal Empire"}li{"Kick a puppy"}li{"Evil laugh"}}}
        @else if user==Princess::Celestia{p{"Sister, please stop reading my diary. It's rude."}}
        @else { p { "Nothing to see here; move along." } }}
        "#,
        r#"
        html! {
            @if user == Princess::Luna {
                h1 { "Super secret woona to-do list" }
                ul {
                    li { "Nuke the Crystal Empire" }
                    li { "Kick a puppy" }
                    li { "Evil laugh" }
                }
            } @else if user == Princess::Celestia {
                p { "Sister, please stop reading my diary. It's rude." }
            } @else {
                p { "Nothing to see here; move along." }
            }
        }
        "#
    );

    test_default!(
        if_let,
        r#"
        html! { p { "Hello, " @if let Some(name) = user { (name) } @else { "stranger" } "!"}}
        "#,
        r#"
        html! {
            p {
                "Hello, "
                @if let Some(name) = user { (name) } @else { "stranger" }
                "!"
            }
        }
        "#
    );

    test_default!(
        control_for,
        r#"
        html!{p{"My favorite ponies are:"}ol{@for name in &names{li{(name)}}}}
        "#,
        r#"
        html! {
            p { "My favorite ponies are:" }
            ol {
                @for name in &names {
                    li { (name) }
                }
            }
        }
        "#
    );

    test_default!(
        control_let,
        r#"
        html!{@for name in &names{@let first_letter=name.chars().next().unwrap();
        p{"The first letter of " b{(name)}" is " b{(first_letter)}"."}}}
        "#,
        r#"
        html! {
            @for name in &names {
                @let first_letter = name.chars().next().unwrap();
                p {
                    "The first letter of "
                    b { (name) }
                    " is "
                    b { (first_letter) }
                    "."
                }
            }
        }
        "#
    );

    test_default!(
        control_match,
        r#"
        html! { @match user { Princess::Luna => { h1 { "Super secret woona to-do list" } ul { li {
        "Nuke the Crystal Empire" } li { "Kick a puppy" } li { "Evil laugh" } } }, 
        Princess::Celestia => { p { "Sister, please stop reading my diary. It's rude." } }, _ => p
        { "Nothing to see here; move along." } } }
        "#,
        r#"
        html! {
            @match user {
                Princess::Luna => {
                    h1 { "Super secret woona to-do list" }
                    ul {
                        li { "Nuke the Crystal Empire" }
                        li { "Kick a puppy" }
                        li { "Evil laugh" }
                    }
                }
                Princess::Celestia => {
                    p { "Sister, please stop reading my diary. It's rude." }
                }
                _ => p { "Nothing to see here; move along." }
            }
        }
        "#
    );

    test_default!(
        control_match_with_guard,
        r#"
        html!{@match user{Princess::Luna if !is_asleep=>{h1{"Title"}
        h2{"Subtitle"}} _=>p{"Nothing to see here; move along."}}}
        "#,
        r#"
        html! {
            @match user {
                Princess::Luna if !is_asleep => {
                    h1 { "Title" }
                    h2 { "Subtitle" }
                }
                _ => p { "Nothing to see here; move along." }
            }
        }
        "#
    );

    test_default!(
        control_while,
        r#"
        html! { @while flag {p{"flag is true"}}}
        "#,
        r#"
        html! {
            @while flag {
                p { "flag is true" }
            }
        }
        "#
    );

    test_default!(
        control_while_let,
        r#"
        html! { @while let Some(value) = iter {p{(value)}}}
        "#,
        r#"
        html! {
            @while let Some(value) = iter {
                p { (value) }
            }
        }
        "#
    );

    test_default!(
        comment_inline,
        r##"
        use maud::DOCTYPE;
        html!{
        // <!DOCTYPE html>
        (DOCTYPE)
        }
        "##,
        r##"
        use maud::DOCTYPE;
        html! {
            // <!DOCTYPE html>
            (DOCTYPE)
        }
        "##
    );

    test_default!(
        control_for_range,
        r##"
        html!{ @for i in 0..10 { p { (i) } } }
        "##,
        r##"
        html! {
            @for i in 0..10 {
                p { (i) }
            }
        }
        "##
    );
}
