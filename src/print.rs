use quote::quote;
use syn::{Expr, File, Item, Local, Pat};

use crate::{ast::*, collect::MaudMacro, format::FormatOptions};

pub fn print(ast: Markups<Element>, mac: &MaudMacro, _options: &FormatOptions) -> String {
    #[cfg(debug_assertions)]
    dbg!(&ast); // print ast when debugging (not release mode)

    let mut printer = Printer {
        lines: Vec::new(),
        buf: String::new(),
        base_indent: String::from("\t").repeat(mac.indent.tabs)
            + &String::from(" ").repeat(mac.indent.spaces),
        indent_str: &String::from(" ").repeat(4),
        macro_name: &mac.macro_name,
    };

    printer.print_ast(ast);

    printer.finish()
}

struct Printer<'a> {
    lines: Vec<String>,
    buf: String,
    base_indent: String,
    indent_str: &'a str,
    macro_name: &'a str,
}

impl<'a> Printer<'a> {
    fn finish(mut self) -> String {
        self.new_line(0);
        self.lines.join("\n")
    }

    fn print_ast(&mut self, ast: Markups<Element>) {
        let indent_level = 0;

        self.write(self.macro_name);
        self.write("! ");

        self.print_block(ast, indent_level, true);
    }

    fn new_line(&mut self, indent_level: usize) {
        self.lines.push(self.buf.clone());
        self.buf =
            String::from(&self.base_indent) + &String::from(self.indent_str).repeat(indent_level);
    }

    fn write(&mut self, content: &str) {
        self.buf += content;
    }

    fn print_html_name(&mut self, name: HtmlName) {
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
        markups: Markups<E>,
        indent_level: usize,
        force_expand: bool,
    ) {
        match markups.markups.len() {
            0 => self.write("{}"),
            1 if !force_expand => {
                self.write("{ ");
                for markup in markups.markups {
                    // there should be only one value
                    self.print_markup(markup, indent_level);
                }
                self.write(" }");
            }
            _ => {
                self.write("{");
                for markup in markups.markups {
                    self.new_line(indent_level + 1);
                    self.print_markup(markup, indent_level + 1);
                }
                self.new_line(indent_level);
                self.write("}");
            }
        }
    }

    fn print_markup<E: Into<Element>>(&mut self, markup: Markup<E>, indent_level: usize) {
        match markup {
            Markup::Lit(html_lit) => self.print_lit(html_lit),
            Markup::Splice { expr, .. } => self.print_splice(expr, indent_level),
            Markup::Element(element) => {
                self.print_element_with_contents(element.into(), indent_level)
            }
            Markup::Block(Block { markups, .. }) => self.print_block(markups, indent_level, false),
            Markup::ControlFlow(control_flow) => {
                self.print_control_flow(control_flow, indent_level)
            }
            Markup::Semi(_semi) => todo!("didn't manage to find its usage yet"),
        }
    }

    fn print_lit(&mut self, html_lit: HtmlLit) {
        let lit = &html_lit.lit;
        self.write(&quote!(#lit).to_string());
    }

    fn print_splice(&mut self, expr: Expr, indent_level: usize) {
        self.write("(");
        self.print_expr(expr, indent_level);
        self.write(")");
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
            is_first_attr = false;
            self.write(&html_name.to_string());
        }

        // sorting out attributes
        let mut id_name: Option<HtmlNameOrMarkup> = None;
        let mut classes: Vec<(HtmlNameOrMarkup, Option<Toggler>)> = Vec::new();
        let mut named_attrs: Vec<(HtmlName, AttributeType)> = Vec::new();
        for attr in attrs {
            match attr {
                Attribute::Id { name, .. } => id_name = Some(name),
                Attribute::Class { name, toggler, .. } => classes.push((name, toggler)),
                Attribute::Named { name, attr_type } => named_attrs.push((name, attr_type)),
            }
        }

        // printing id
        if let Some(name) = id_name {
            if !is_first_attr {
                self.write(" ");
            }
            self.write("#");
            match name {
                HtmlNameOrMarkup::HtmlName(html_name) => self.print_html_name(html_name),
                HtmlNameOrMarkup::Markup(markup) => self.print_markup(markup, indent_level),
            }
        }

        // printing classes
        for (name, maybe_toggler) in classes {
            self.write(".");
            match name {
                HtmlNameOrMarkup::HtmlName(html_name) => self.print_html_name(html_name),
                HtmlNameOrMarkup::Markup(markup) => self.print_markup(markup, indent_level),
            }
            if let Some(toggler) = maybe_toggler {
                self.write("[");
                self.print_expr(toggler.cond, indent_level);
                self.write("]");
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
                    self.print_expr(toggler.cond, indent_level);
                    self.write("]");
                }
                AttributeType::Empty(maybe_toggler) => {
                    if let Some(toggler) = maybe_toggler {
                        self.write("[");
                        self.print_expr(toggler.cond, indent_level);
                        self.write("]");
                    }
                }
            }
        }

        match body {
            ElementBody::Void(_) => self.write(";"),
            ElementBody::Block(Block { markups, .. }) => {
                self.write(" ");
                self.print_block(markups, indent_level, false);
            }
        }
    }

    fn print_control_flow<E: Into<Element>>(
        &mut self,
        control_flow: ControlFlow<E>,
        indent_level: usize,
    ) {
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
                self.print_block(for_expr.body.markups, indent_level, true);
            }
            ControlFlowKind::Let(local) => {
                self.write("@");
                self.write(&unparse_local(&local).join("\n")); //TODO(jeosas): manage line length
                self.write(";");
            }
            ControlFlowKind::Match(match_expr) => {
                self.write("@match ");
                self.write(&unparse_expr(&match_expr.expr).join("\n")); // TODO(jeosas): manage line_length
                self.write(" {");
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

        self.print_block(if_expr.then_branch.markups, indent_level, true);

        if let Some((_, _, if_or_block)) = if_expr.else_branch {
            self.write(" @else ");

            match *if_or_block {
                IfOrBlock::If(else_if_expr) => {
                    self.print_if_expr(else_if_expr, indent_level);
                }
                IfOrBlock::Block(block) => {
                    self.print_block(block.markups, indent_level, true);
                }
            }
        }
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
    dbg!(&wrapped);
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
    dbg!(&wrapped);
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
