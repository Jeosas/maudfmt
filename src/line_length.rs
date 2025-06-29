use syn::{
    Expr,
    spanned::Spanned as _,
    token::{Dot, Pound},
};

use crate::ast::*;

// returns None if content should be on multiple lines
pub fn markup_len<E: Into<Element>>(markup: &Markup<E>) -> Option<usize> {
    match markup {
        Markup::Lit(html_lit) => {
            let span = html_lit.lit.span();
            let start = span.start();
            let end = span.end();

            if start.line != end.line {
                None
            } else {
                Some(end.column - start.column)
            }
        }
        Markup::Splice { expr, .. } => expr_len(expr),
        Markup::Element(_) => None,
        Markup::Block(block) => block_len(block),
        Markup::ControlFlow(_) => None,
        Markup::Semi(_semi) => Some(1),
    }
}

pub fn element_attrs_len(
    name: &Option<HtmlName>,
    id_name: &Option<(Pound, HtmlNameOrMarkup)>,
    classes: &Vec<(Dot, HtmlNameOrMarkup, Option<Toggler>)>,
    named_attrs: &Vec<(HtmlName, AttributeType)>,
    body: &ElementBody,
) -> Option<usize> {
    let mut element_len = 0usize;
    let mut is_first_attr = true;

    // name
    if let Some(html_name) = name {
        match html_name_len(html_name) {
            Some(value) => element_len += value,
            None => return None,
        }
        is_first_attr = false;
    }

    // id
    if let Some((_, name)) = id_name {
        if !is_first_attr {
            // (space)
            element_len += 1;
        } else {
            is_first_attr = false;
        }
        // (pound)
        element_len += 1;
        match html_name_or_markup_len(name) {
            Some(value) => element_len += value,
            None => return None,
        }
    }

    // classes
    for (_, name, maybe_toggler) in classes {
        if is_first_attr {
            is_first_attr = false;
        }
        // (dot)
        element_len += 1;
        match html_name_or_markup_len(name) {
            Some(value) => element_len += value,
            None => return None,
        }
        if let Some(toggler) = maybe_toggler {
            // (open bracket)
            element_len += 1;
            match expr_len(&toggler.cond) {
                Some(value) => element_len += value,
                None => return None,
            }
            // (close bracket)
            element_len += 1;
        }
    }

    // other attributes
    for (name, attr_type) in named_attrs {
        // (space)
        element_len += 1;
        match html_name_len(name) {
            Some(value) => element_len += value,
            None => return None,
        }
        match attr_type {
            AttributeType::Normal { value, .. } => {
                // (eq)
                element_len += 1;
                match markup_len(value) {
                    Some(value) => element_len += value,
                    None => return None,
                }
            }
            AttributeType::Optional { toggler, .. } => {
                // (eq) + (open bracket)
                element_len += 2;
                match expr_len(&toggler.cond) {
                    Some(value) => element_len += value,
                    None => return None,
                }
                // (close bracket)
                element_len += 1;
            }
            AttributeType::Empty(maybe_toggler) => {
                if let Some(toggler) = maybe_toggler {
                    // (open bracket)
                    element_len += 1;
                    match expr_len(&toggler.cond) {
                        Some(value) => element_len += value,
                        None => return None,
                    }
                    // (close bracket)
                    element_len += 1;
                }
            }
        }
    }

    match body {
        ElementBody::Void(_) => {
            // (semi)
            element_len += 1;
        }
        ElementBody::Block(_) => {
            // always add open body brace at minimum
            // (space) + (open brace)
            element_len += 2;
        }
    }

    Some(element_len)
}

pub fn block_len<E: Into<Element>>(Block { markups, .. }: &Block<E>) -> Option<usize> {
    let mut element_len = 0usize;

    // (open brace) + (space)
    element_len += 2;

    for markup in &markups.markups {
        match markup_len(markup) {
            Some(value) => element_len += value,
            None => return None,
        }
        // (space)
        element_len += 1;
    }

    // (close brace)
    element_len += 1;

    Some(element_len)
}

pub fn html_name_or_markup_len(html_or_markup: &HtmlNameOrMarkup) -> Option<usize> {
    match &html_or_markup {
        HtmlNameOrMarkup::HtmlName(html_name) => html_name_len(html_name),
        HtmlNameOrMarkup::Markup(markup) => markup_len(markup),
    }
}

pub fn html_name_len(html_name: &HtmlName) -> Option<usize> {
    let span = html_name.span();
    let start = span.start();
    let end = span.end();

    if start.line != end.line {
        None
    } else {
        Some(end.column - start.column)
    }
}

pub fn expr_len(expr: &Expr) -> Option<usize> {
    let span = expr.span();
    let start = span.start();
    let end = span.end();

    if start.line != end.line {
        None
    } else {
        Some(end.column - start.column)
    }
}
