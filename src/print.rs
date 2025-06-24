use crop::{Rope, RopeSlice};
use proc_macro2::LineColumn;
use quote::quote;
use syn::{
    Expr, File, Item, Local, Pat,
    spanned::Spanned as _,
    token::{Dot, Paren, Pound},
};

use crate::{ast::*, collect::MaudMacro, format::FormatOptions};

pub fn print<'b>(
    ast: Markups<Element>,
    mac: &'b MaudMacro<'b>,
    source: &Rope,
    _options: &FormatOptions,
) -> String {
    #[cfg(debug_assertions)]
    dbg!(&ast); // print ast when debugging (not release mode)

    let mut printer = Printer {
        lines: Vec::new(),
        buf: String::new(),
        base_indent: String::from("\t").repeat(mac.indent.tabs)
            + &String::from(" ").repeat(mac.indent.spaces),
        indent_str: &String::from(" ").repeat(4),
        mac,
        source,
    };

    printer.print_ast(ast);

    printer.finish()
}

struct Printer<'a, 'b> {
    lines: Vec<String>,
    buf: String,
    base_indent: String,
    indent_str: &'a str,
    mac: &'b MaudMacro<'b>,
    source: &'a Rope,
}

impl<'a, 'b> Printer<'a, 'b> {
    fn finish(mut self) -> String {
        self.new_line(0);
        self.lines.join("\n")
    }

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
                self.print_markup(markup, indent_level + 1);
            }
            self.new_line(indent_level);

            let close_location = self.mac.macro_.delimiter.span().close().end();
            self.print_inline_comment_and_whitespace(close_location, indent_level);
            self.write("}");
            self.print_attr_comment(close_location);
        }
    }

    fn new_line(&mut self, indent_level: usize) {
        self.lines.push(self.buf.clone());
        self.buf =
            String::from(&self.base_indent) + &String::from(self.indent_str).repeat(indent_level);
    }

    fn write(&mut self, content: &str) {
        self.buf += content;
    }

    fn print_html_name(&mut self, name: &HtmlName) {
        for child in name.name.pairs() {
            match child.value() {
                HtmlNameFragment::LitStr(lit) => self.write(&quote!(#lit).to_string()),
                value => self.write(&value.to_string()),
            }
            if let Some(punct) = child.punct() {
                match punct {
                    HtmlNamePunct::Hyphen(_) => self.write("-"),
                    HtmlNamePunct::Colon(_) => self.write(","),
                }
            }
        }
    }

    fn print_block<E: Into<Element>>(
        &mut self,
        block: Block<E>,
        indent_level: usize,
        force_expand: bool,
    ) {
        self.print_inline_comment_and_whitespace(
            block.brace_token.span.span().start(),
            indent_level,
        );
        match block.markups.markups.len() {
            0 => {
                self.write("{}");
                self.print_attr_comment(block.brace_token.span.close().span().end());
            }
            1 if !force_expand => {
                self.write("{");
                if self.print_attr_comment(block.brace_token.span.open().span().end()) {
                    // expand if comment
                    self.new_line(indent_level + 1);
                    for markup in block.markups.markups {
                        // there should be only one value
                        self.print_markup(markup, indent_level + 1);
                    }
                    self.new_line(indent_level);
                    self.write("}");
                } else {
                    self.write(" ");
                    for markup in block.markups.markups {
                        // there should be only one value
                        self.print_markup(markup, indent_level);
                    }
                    self.write(" }");
                }
                self.print_attr_comment(block.brace_token.span.close().span().end());
            }
            _ => {
                self.write("{");
                self.print_attr_comment(block.brace_token.span.open().span().end());
                for markup in block.markups.markups {
                    self.new_line(indent_level + 1);
                    self.print_markup(markup, indent_level + 1);
                }
                self.new_line(indent_level);
                self.write("}");
                self.print_attr_comment(block.brace_token.span.close().span().end());
            }
        }
    }

    fn print_markup<E: Into<Element>>(&mut self, markup: Markup<E>, indent_level: usize) {
        match markup {
            Markup::Lit(html_lit) => self.print_lit(html_lit, indent_level),
            Markup::Splice { paren_token, expr } => {
                self.print_splice(expr, paren_token, indent_level)
            }
            Markup::Element(element) => {
                self.print_element_with_contents(element.into(), indent_level)
            }
            Markup::Block(block) => self.print_block(block, indent_level, false),
            Markup::ControlFlow(control_flow) => {
                self.print_control_flow(control_flow, indent_level)
            }
            Markup::Semi(_semi) => todo!("didn't manage to find its usage yet"),
        }
    }

    fn print_lit(&mut self, html_lit: HtmlLit, indent_level: usize) {
        self.print_inline_comment_and_whitespace(html_lit.span().start(), indent_level);
        let lit = &html_lit.lit;
        self.write(&quote!(#lit).to_string());
        self.print_attr_comment(html_lit.span().end());
    }

    fn print_splice(&mut self, expr: Expr, paren: Paren, indent_level: usize) {
        self.print_inline_comment_and_whitespace(paren.span.span().start(), indent_level);
        self.write("(");
        if self.print_attr_comment(paren.span.open().span().end()) {
            // expand if comment
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

    fn print_expr(&mut self, expr: Expr, indent_level: usize) {
        let lines = unparse_expr(&expr);
        match lines.len() {
            0 => (),
            1 => self.write(&lines[0]),
            _ => {
                let mut iter = lines.iter();
                if let Some(first) = iter.next() {
                    self.write(first);
                }
                for next in iter {
                    self.new_line(indent_level);
                    self.write(next);
                }
            }
        };
    }

    fn print_element_with_contents(
        &mut self,
        Element { name, attrs, body }: Element,
        indent_level: usize,
    ) {
        let mut is_first_attr = true;

        // element tag name
        if let Some(html_name) = name {
            self.print_inline_comment_and_whitespace(html_name.span().start(), indent_level);
            is_first_attr = false;
            self.print_html_name(&html_name);
            self.print_attr_comment(html_name.span().end());
        }

        // sorting out attributes
        let mut id_name: Option<(Pound, HtmlNameOrMarkup)> = None;
        let mut classes: Vec<(Dot, HtmlNameOrMarkup, Option<Toggler>)> = Vec::new();
        let mut named_attrs: Vec<(HtmlName, AttributeType)> = Vec::new();
        for attr in attrs {
            match attr {
                Attribute::Id { pound_token, name } => id_name = Some((pound_token, name)),
                Attribute::Class {
                    dot_token,
                    name,
                    toggler,
                } => classes.push((dot_token, name, toggler)),
                Attribute::Named { name, attr_type } => named_attrs.push((name, attr_type)),
            }
        }

        // printing id
        if let Some((pound_token, name)) = id_name {
            if !is_first_attr {
                self.write(" ");
            } else {
                self.print_inline_comment_and_whitespace(pound_token.span().start(), indent_level);
                is_first_attr = false;
            }
            self.write("#");
            match name {
                HtmlNameOrMarkup::HtmlName(html_name) => {
                    self.print_html_name(&html_name);
                    self.print_attr_comment(html_name.span().end());
                }
                HtmlNameOrMarkup::Markup(markup) => self.print_markup(markup, indent_level),
            }
        }

        // printing classes
        for (dot_token, name, maybe_toggler) in classes {
            if is_first_attr {
                self.print_inline_comment_and_whitespace(dot_token.span().start(), indent_level);
            }
            self.write(".");
            match name {
                HtmlNameOrMarkup::HtmlName(html_name) => {
                    self.print_html_name(&html_name);
                    self.print_attr_comment(html_name.span().end());
                }
                HtmlNameOrMarkup::Markup(markup) => self.print_markup(markup, indent_level),
            }
            if let Some(toggler) = maybe_toggler {
                self.write("[");
                if self.print_attr_comment(toggler.bracket_token.span.open().span().end()) {
                    self.print_expr(toggler.cond, indent_level + 1);
                    self.new_line(indent_level);
                    self.write("]");
                } else {
                    self.print_expr(toggler.cond, indent_level);
                    self.write("]");
                }
                self.print_attr_comment(toggler.bracket_token.span.close().span().end());
            }
        }

        // printing other attributes
        for (name, attr_type) in named_attrs {
            self.write(" ");
            self.write(&name.to_string());
            match attr_type {
                AttributeType::Normal { value, .. } => {
                    self.write("=");
                    self.print_markup(value, indent_level)
                }
                AttributeType::Optional { toggler, .. } => {
                    self.write("=[");
                    if self.print_attr_comment(toggler.bracket_token.span.open().span().end()) {
                        self.print_expr(toggler.cond, indent_level + 1);
                        self.new_line(indent_level);
                        self.write("]");
                    } else {
                        self.print_expr(toggler.cond, indent_level);
                        self.write("]");
                    }
                    self.print_attr_comment(toggler.bracket_token.span.close().span().end());
                }
                AttributeType::Empty(maybe_toggler) => {
                    if let Some(toggler) = maybe_toggler {
                        self.write("[");
                        if self.print_attr_comment(toggler.bracket_token.span.open().span().end()) {
                            self.print_expr(toggler.cond, indent_level + 1);
                            self.new_line(indent_level);
                            self.write("]");
                        } else {
                            self.print_expr(toggler.cond, indent_level);
                            self.write("]");
                        }
                        self.print_attr_comment(toggler.bracket_token.span.close().span().end());
                    }
                }
            }
        }

        match body {
            ElementBody::Void(semi) => {
                self.write(";");
                self.print_attr_comment(semi.span().end());
            }
            ElementBody::Block(block) => {
                self.write(" ");
                self.print_block(block, indent_level, false);
            }
        }
    }

    fn print_control_flow<E: Into<Element>>(
        &mut self,
        control_flow: ControlFlow<E>,
        indent_level: usize,
    ) {
        self.print_inline_comment_and_whitespace(
            control_flow.at_token.span.span().end(),
            indent_level,
        );
        match control_flow.kind {
            ControlFlowKind::If(if_expr) => {
                self.write("@");
                self.print_if_expr(if_expr, indent_level);
            }
            ControlFlowKind::For(for_expr) => {
                self.write("@for ");
                self.write(&unparse_pat(&for_expr.pat).join("\n")); //TODO(jeosas): manage line length
                self.write(" in ");
                self.print_expr(for_expr.expr, indent_level);
                self.write(" ");
                self.print_block(for_expr.body, indent_level, true);
            }
            ControlFlowKind::Let(local) => {
                self.write("@");
                self.write(&unparse_local(&local).join("\n")); //TODO(jeosas): manage line length
                self.write(";");
                self.print_attr_comment(local.semi_token.span().end());
            }
            ControlFlowKind::Match(match_expr) => {
                self.write("@match ");
                self.write(&unparse_expr(&match_expr.expr).join("\n")); // TODO(jeosas): manage line_length
                self.write(" {");
                self.print_attr_comment(match_expr.brace_token.span.open().span().end());
                for arm in match_expr.arms {
                    self.new_line(indent_level + 1);
                    self.write(&unparse_pat(&arm.pat).join("\n")); //TODO(jeosas): manage line length
                    if let Some((_, guard_cond)) = arm.guard {
                        self.write(" if ");
                        self.write(&unparse_expr(&guard_cond).join("\n")); //TODO(jeosas): manage line length
                    }
                    self.write(" => ");
                    self.print_markup(arm.body, indent_level + 1);
                }
                self.new_line(indent_level);
                self.write("}");
                self.print_attr_comment(match_expr.brace_token.span.close().span().end());
            }
            ControlFlowKind::While(_while_expr) => todo!(),
        }
    }

    fn print_if_expr<E: Into<Element>>(&mut self, if_expr: IfExpr<E>, indent_level: usize) {
        self.write("if ");
        match if_expr.cond {
            Expr::Let(expr_let) => {
                // crashes prettyplease > syn can't parse it
                self.write("let ");
                self.write(&unparse_pat(&expr_let.pat).join("\n")); //TODO(jeosas): manage line length
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

        self.print_block(if_expr.then_branch, indent_level, true);

        if let Some((_, _, if_or_block)) = if_expr.else_branch {
            self.write(" @else ");

            match *if_or_block {
                IfOrBlock::If(else_if_expr) => {
                    self.print_if_expr(else_if_expr, indent_level);
                }
                IfOrBlock::Block(block) => {
                    self.print_block(block, indent_level, true);
                }
            }
        }
    }

    // Returns true if a comment was inserted
    fn print_attr_comment(&mut self, loc: LineColumn) -> bool {
        if !self.is_trailing(loc) {
            return false;
        }

        let cursor_line = loc.line - 1; // LineColumn.line is 1-indexed

        if let Some(comment) = self
            .source
            .line(cursor_line)
            .to_string()
            .split_once("//")
            .map(|(_, txt)| txt)
            .map(str::trim)
            .map(str::to_string)
        {
            self.write("  //");
            if !comment.is_empty() {
                self.write(" ");
                self.write(&comment);
            }
            return true;
        }

        false
    }

    fn print_inline_comment_and_whitespace(&mut self, loc: LineColumn, indent_level: usize) {
        let mut cursor_line = loc.line - 1; // LineColumn.line is 1-indexed
        if cursor_line == 0 {
            // line is already the top of the document
            return;
        }

        if !self.is_leading(loc) {
            return;
        }

        // Keep whitespace
        if self
            .source
            .line(cursor_line - 1)
            .to_string()
            .trim()
            .is_empty()
        {
            self.buf = String::new(); // remove indent for less bytes in final file
            self.new_line(indent_level);
            return;
        }

        let mut comments = Vec::new();

        while let Some(comment) = extract_inline_comment(self.source.line(cursor_line - 1)) {
            comments.push(comment);
            cursor_line -= 1;
        }

        while let Some(comment) = comments.pop() {
            self.write("//");
            if !comment.is_empty() {
                self.write(" ");
                self.write(&comment);
            }
            self.new_line(indent_level);
        }
    }

    // Check if a Markup location is leading a line or not
    // Prevents inline comments and whitespace
    // from being printed more than once
    fn is_leading(&self, loc: LineColumn) -> bool {
        let line = self.source.line(loc.line - 1);
        // is start of the line ?
        line.byte_slice(..loc.column).to_string().trim().is_empty()
    }

    // Check if a Markup location is trainling a line or not
    // Prevents attrs comments from being printed more than once
    fn is_trailing(&self, loc: LineColumn) -> bool {
        let line = self.source.line(loc.line - 1);

        // is start of the line ?
        let line_string = line.byte_slice(loc.column..).to_string();
        line_string
            .split_once("//") // remove comment if exist
            .map(|(txt, _)| txt)
            .unwrap_or(&line_string)
            .trim()
            .is_empty()
    }
}

fn extract_inline_comment(line: RopeSlice) -> Option<String> {
    let line_string = line.to_string();
    if line_string.trim().starts_with("//") {
        line_string
            .split_once("//")
            .map(|(_, txt)| txt)
            .map(str::trim)
            .map(str::to_string)
    } else {
        None
    }
}

fn unparse_local(local: &Local) -> Vec<String> {
    let file = File {
        shebang: None,
        attrs: vec![],
        items: vec![
            //
            Item::Verbatim(quote::quote! {
                fn main() {
                    #local;
                }
            }),
        ],
    };

    let wrapped = prettyplease::unparse(&file);
    wrapped
        .strip_prefix("fn main() {\n    ")
        .unwrap()
        .strip_suffix(";\n}\n")
        .unwrap()
        .lines()
        .map(|line| line.to_string())
        .collect::<Vec<_>>()
}

fn unparse_pat(pat: &Pat) -> Vec<String> {
    let file = File {
        shebang: None,
        attrs: vec![],
        items: vec![
            //
            Item::Verbatim(quote::quote! {
                fn main() {
                    let #pat;
                }
            }),
        ],
    };

    let wrapped = prettyplease::unparse(&file);
    wrapped
        .strip_prefix("fn main() {\n    let ")
        .unwrap()
        .strip_suffix(";\n}\n")
        .unwrap()
        .lines()
        .map(|line| line.to_string())
        .collect::<Vec<_>>()
}

fn unparse_expr(expr: &Expr) -> Vec<String> {
    let file = File {
        shebang: None,
        attrs: vec![],
        items: vec![
            //
            Item::Verbatim(quote::quote! {
                fn main() {
                    #expr
                }
            }),
        ],
    };

    let wrapped = prettyplease::unparse(&file);
    wrapped
        .strip_prefix("fn main() {\n")
        .unwrap()
        .strip_suffix("}\n")
        .unwrap()
        .lines()
        .map(|line| line.strip_prefix("    ").unwrap())
        .map(|line| line.to_string())
        .collect::<Vec<_>>()
}
