use quote::quote;
use syn::{
    spanned::Spanned as _,
    token::{Dot, Pound},
};

use crate::{
    line_length::{block_len, element_attrs_len},
    print::Printer,
    vendor::ast::{
        Attribute, AttributeType, Element, ElementBody, HtmlName, HtmlNameFragment,
        HtmlNameOrMarkup, HtmlNamePunct, Toggler,
    },
};

impl<'a, 'b> Printer<'a, 'b> {
    pub fn print_element_with_contents(
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
                self.print_attr_comment(toggler.bracket_token.span.open().span().end());
                self.print_toggle_expr(toggler.cond, indent_level);
                self.write("]");
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
                    self.print_attr_comment(toggler.bracket_token.span.open().span().end());
                    self.print_toggle_expr(toggler.cond, indent_level);
                    self.write("]");
                    self.print_attr_comment(toggler.bracket_token.span.close().span().end());
                }
                AttributeType::Empty(maybe_toggler) => {
                    if let Some(toggler) = maybe_toggler {
                        self.write("[");
                        self.print_attr_comment(toggler.bracket_token.span.open().span().end());
                        self.print_toggle_expr(toggler.cond, indent_level);
                        self.write("]");
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
}
