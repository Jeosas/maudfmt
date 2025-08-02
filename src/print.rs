use crop::{Rope, RopeSlice};
use proc_macro2::{LineColumn, extra::DelimSpan};
use quote::quote;
use syn::{
    Expr,
    spanned::Spanned as _,
    token::{Dot, Paren, Pound},
};

use crate::{ast::*, collect::MaudMacro, format::FormatOptions, line_length::*, unparse::*};

pub fn print<'b>(
    ast: Markups<Element>,
    mac: &'b MaudMacro<'b>,
    source: &Rope,
    options: &FormatOptions,
) -> String {
    #[cfg(debug_assertions)]
    dbg!(&ast); // print ast when debugging (not release mode)

    let mut printer = Printer {
        lines: Vec::new(),
        buf: String::new(),
        base_indent: mac.indent.tabs + mac.indent.spaces / 4,
        indent_str: &String::from(" ").repeat(4),
        mac,
        source,
        options,
    };

    printer.print_ast(ast);

    printer.finish()
}

struct Printer<'a, 'b> {
    lines: Vec<String>,
    buf: String,
    base_indent: usize,
    indent_str: &'a str,
    mac: &'b MaudMacro<'b>,
    source: &'a Rope,
    options: &'a FormatOptions,
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
                self.print_markup(markup, indent_level + 1, true);
            }
            self.new_line(indent_level);

            let close_location = self.mac.macro_.delimiter.span().close().end();
            self.print_inline_comment_and_whitespace(close_location, indent_level, true);
            self.write("}");
            self.print_attr_comment(close_location);
        }
    }

    fn new_line(&mut self, indent_level: usize) {
        self.lines.push(self.buf.clone());
        self.buf = String::from(self.indent_str).repeat(self.base_indent + indent_level);
    }

    fn write(&mut self, content: &str) {
        self.buf += content;
    }

    fn line_len(&self) -> usize {
        self.buf.len()
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

    fn print_html_attribute_name(&mut self, name: &HtmlName) {
        let value = name.to_string();
        if value.contains('@') || value.contains('.') || value.starts_with(":") {
            self.write(&quote!(#value).to_string());
        } else {
            self.write(&value);
        }
    }

    fn print_block<E: Into<Element>>(&mut self, block: Block<E>, indent_level: usize) {
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
            }

            self.new_line(indent_level);
            self.write("}");
            self.print_attr_comment(block.brace_token.span.close().span().end());
        }
    }

    fn print_markup<E: Into<Element>>(
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

    // NOTE: lit do not care about line length
    //       let user take care of it
    fn print_lit(&mut self, html_lit: HtmlLit, indent_level: usize, preserve_blank_lines: bool) {
        self.print_inline_comment_and_whitespace(
            html_lit.span().start(),
            indent_level,
            preserve_blank_lines,
        );
        let lit = &html_lit.lit;
        self.write(&quote!(#lit).to_string());
        self.print_attr_comment(html_lit.span().end());
    }

    fn print_splice(
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

    fn print_expr(&mut self, expr: Expr, indent_level: usize) {
        let lines: Vec<String> = match expr {
            Expr::Block(expr_block) => {
                unparse_stmts(&expr_block.block.stmts, self.base_indent + indent_level)
            }
            _ => unparse_expr(&expr, self.base_indent + indent_level),
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

    fn print_element_with_contents(
        &mut self,
        Element { name, attrs, body }: Element,
        indent_level: usize,
        preserve_blank_lines: bool,
    ) {
        // Check if this element's block will be collapsed
        let will_collapse_block = match &body {
            ElementBody::Block(block) => {
                !self.block_contains_comments(block.brace_token.span) && {
                    if let Some(blk_len) = block_len(block) {
                        (self.line_len() + blk_len) <= self.options.line_length
                    } else {
                        false
                    }
                }
            }
            _ => false,
        };

        // Don't preserve blank lines if this element's block will be collapsed
        let preserve_blank_lines = preserve_blank_lines && !will_collapse_block;
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

        let should_wrap = if let Some(element_len) =
            element_attrs_len(&name, &id_name, &classes, &named_attrs, &body)
        {
            (self.line_len() + element_len) > self.options.line_length
        } else {
            true
        };
        let mut is_first_attr = true;

        // element tag name
        if let Some(html_name) = name {
            self.print_inline_comment_and_whitespace(
                html_name.span().start(),
                indent_level,
                preserve_blank_lines,
            );
            is_first_attr = false;
            self.print_html_name(&html_name);
            self.print_attr_comment(html_name.span().end());
        }

        // printing id
        if let Some((pound_token, name)) = id_name {
            match (is_first_attr, should_wrap) {
                (false, false) => {
                    self.write(" ");
                }
                (false, true) => {
                    self.new_line(indent_level + 1);
                }
                (true, _) => {
                    self.print_inline_comment_and_whitespace(
                        pound_token.span().start(),
                        indent_level,
                        preserve_blank_lines,
                    );
                    is_first_attr = false;
                }
            }
            self.write("#");
            match name {
                HtmlNameOrMarkup::HtmlName(html_name) => {
                    self.print_html_name(&html_name);
                    self.print_attr_comment(html_name.span().end());
                }
                HtmlNameOrMarkup::Markup(markup) => self.print_markup(markup, indent_level, true),
            }
        }

        // printing classes
        for (dot_token, name, maybe_toggler) in classes {
            match (is_first_attr, should_wrap) {
                (false, true) => {
                    self.new_line(indent_level + 1);
                }
                (false, false) => (),
                (true, _) => {
                    self.print_inline_comment_and_whitespace(
                        dot_token.span().start(),
                        indent_level,
                        preserve_blank_lines,
                    );
                    is_first_attr = false;
                }
            }
            self.write(".");
            match name {
                HtmlNameOrMarkup::HtmlName(html_name) => {
                    self.print_html_name(&html_name);
                    self.print_attr_comment(html_name.span().end());
                }
                HtmlNameOrMarkup::Markup(markup) => self.print_markup(markup, indent_level, true),
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
            if should_wrap {
                self.new_line(indent_level + 1);
            } else {
                self.write(" ");
            }
            self.print_html_attribute_name(&name);
            match attr_type {
                AttributeType::Normal { value, .. } => {
                    self.write("=");
                    let attr_indent = if should_wrap {
                        indent_level + 1
                    } else {
                        indent_level
                    };
                    self.print_markup(value, attr_indent, true)
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
                self.print_block(block, indent_level);
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

    // Returns true if a comment was inserted
    fn print_attr_comment(&mut self, loc: LineColumn) -> bool {
        if !self.is_trailing(loc) {
            return false;
        }

        // LineColumn.line is 1-indexed
        let token_end_byte = self.source.byte_of_line(loc.line - 1) + loc.column;
        let next_line_start_byte = self.source.byte_of_line(loc.line);

        if let Some(comment) = self
            .source
            .byte_slice(token_end_byte..next_line_start_byte)
            .to_string()
            .split_once("//")
            .map(|(_, txt)| txt)
            .map(str::trim_end)
            .map(str::to_string)
        {
            self.write("  //");
            if !comment.is_empty() {
                if !comment.starts_with(" ") {
                    self.write(" ");
                }
                self.write(&comment);
            }
            return true;
        }

        false
    }

    fn print_inline_comment_and_whitespace(
        &mut self,
        loc: LineColumn,
        indent_level: usize,
        preserve_blank_lines: bool,
    ) {
        let mut cursor_line = loc.line - 1; // LineColumn.line is 1-indexed
        if cursor_line == 0 {
            // line is already the top of the document
            return;
        }

        if !self.is_leading(loc) {
            return;
        }

        // Keep whitespace
        if preserve_blank_lines
            && self
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
                if !comment.starts_with(" ") {
                    self.write(" ");
                }
                self.write(&comment);
            }
            self.new_line(indent_level);
        }
    }

    // Check if a Markup location is leading a line or not
    // Prevents inline comments and whitespace
    // from being printed more than once
    fn is_leading(&self, loc: LineColumn) -> bool {
        // LineColumn.line is 1-indexed
        let line = self.source.line(loc.line - 1);
        // is start of the line ?
        line.byte_slice(..loc.column).to_string().trim().is_empty()
    }

    // Check if a Markup location is trainling a line or not
    // Prevents attrs comments from being printed more than once
    fn is_trailing(&self, loc: LineColumn) -> bool {
        // LineColumn.line is 1-indexed
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

    fn print_block_comments(&mut self, delim_span: DelimSpan, indent_level: usize) {
        // LineColumn.line is 1-indexed
        let start_line = delim_span.span().start().line - 1;
        let end_line = delim_span.span().end().line - 1;

        for line_idx in (start_line + 1)..end_line {
            let line = self.source.line(line_idx);
            if let Some((_, comment_part)) = line.to_string().split_once("//") {
                self.new_line(indent_level);
                self.write("//");
                let comment = comment_part.trim_end();
                if !comment.is_empty() {
                    if !comment.starts_with(" ") {
                        self.write(" ");
                    }
                    self.write(comment);
                }
            }
        }
    }

    fn block_contains_comments(&self, delim_span: DelimSpan) -> bool {
        // LineColumn.line is 1-indexed
        let start_line = delim_span.span().start().line - 1;
        let end_line = delim_span.span().end().line - 1;

        if start_line == end_line {
            // closed brackets, let attr_comment handle it
            return false;
        }

        (start_line..=end_line).any(|line| {
            self.source
                .line(line)
                .to_string()
                .split_once("//")
                .is_some()
        })
    }
}

fn extract_inline_comment(line: RopeSlice) -> Option<String> {
    let line_string = line.to_string();
    if line_string.trim().starts_with("//") {
        line_string
            .split_once("//")
            .map(|(_, txt)| txt)
            .map(str::trim_end)
            .map(str::to_string)
    } else {
        None
    }
}
