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

#[cfg(test)]
mod test {
    use crate::testing::*;

    test_default!(
        elements_with_contents,
        r#"
        html! { h1 { "Poem" } p { strong { "Rock," } " you are a rock."}}
        "#,
        r#"
        html! {
            h1 { "Poem" }
            p {
                strong { "Rock," }
                " you are a rock."
            }
        }
        "#
    );

    test_default!(
        void_element,
        r#"
        html! {
          p { "Rock, you are a rock." br; "Gray, you are gray," br;
            "Like a rock, which you are." br; "Rock." } }
        "#,
        r#"
        html! {
            p {
                "Rock, you are a rock."
                br;
                "Gray, you are gray,"
                br;
                "Like a rock, which you are."
                br;
                "Rock."
            }
        }
        "#
    );

    test_default!(
        custom_elements_and_attributes,
        r#"
        html! {
          article data-index="12345"{h1 { "My blog"}tag-cloud {"pinkie pie pony cute"}}}
        "#,
        r#"
        html! {
            article data-index="12345" {
                h1 { "My blog" }
                tag-cloud { "pinkie pie pony cute" }
            }
        }
        "#
    );

    test_default!(
        non_empty_attributes,
        r#"
        html! { ul { li { a href="about:blank" { "Apple Bloom" } }
        li class="lower-middle" { "Sweetie Belle" }
        li dir="rtl" { "Scootaloo " small { "(also a chicken)" } } } }
        "#,
        r#"
        html! {
            ul {
                li {
                    a href="about:blank" { "Apple Bloom" }
                }
                li class="lower-middle" { "Sweetie Belle" }
                li dir="rtl" {
                    "Scootaloo "
                    small { "(also a chicken)" }
                }
            }
        }
        "#
    );

    test_default!(
        empty_attributes,
        r#"
        html! { form { input type="checkbox" name="cupcakes" checked;
        " " label for="cupcakes" { "Do you like cupcakes?" } } }
        "#,
        r#"
        html! {
            form {
                input type="checkbox" name="cupcakes" checked;
                " "
                label for="cupcakes" { "Do you like cupcakes?" }
            }
        }
        "#
    );

    test_default!(
        classes_and_ids,
        r#"
        html! { input#cannon .big.scary.bright-red type="button" value="Launch Party Cannon"; }
        "#,
        r#"
        html! {
            input #cannon.big.scary.bright-red type="button" value="Launch Party Cannon";
        }
        "#
    );

    test_default!(
        quoted_class_and_ids,
        r#"
        html!{div   #"quoted-id"   ."col-sm-2" { "Bootstrap column!" } }
        "#,
        r#"
        html! {
            div #"quoted-id"."col-sm-2" { "Bootstrap column!" }
        }
        "#
    );

    test_default!(
        implicit_div,
        r#"
        html! { #main { "Main content!" .tip { 
        "Storing food in a refrigerator can make it 20% cooler." } } }
        "#,
        r#"
        html! {
            #main {
                "Main content!"
                .tip { "Storing food in a refrigerator can make it 20% cooler." }
            }
        }
        "#
    );

    test_default!(
        splice_in_attributes,
        r#"
        html!{p title=  (secret_message){"Nothing to see here, move along."}}
        "#,
        r#"
        html! {
            p title=(secret_message) { "Nothing to see here, move along." }
        }
        "#
    );

    test_default!(
        splice_concatenation,
        r#"
        html!{a href={(GITHUB)"/lambda-fairy/maud"}{"Fork me on GitHub"}}
        "#,
        r#"
        html! {
            a href={ (GITHUB) "/lambda-fairy/maud" } { "Fork me on GitHub" }
        }
        "#
    );

    test_default!(
        splice_classes_and_ids,
        r#"
        html!{aside #(name){p.{ "color-"(severity)}{"This is the worst! Possible! Thing!"}}}
        "#,
        r#"
        html! {
            aside #(name) {
                p.{ "color-" (severity) } { "This is the worst! Possible! Thing!" }
            }
        }
        "#
    );

    test_default!(
        toggle_empty_attributes,
        r#"
        html!{p contenteditable[allow_editing]{"Edit me, I " em{"dare"}" you."}}
        "#,
        r#"
        html! {
            p contenteditable[allow_editing] {
                "Edit me, I "
                em { "dare" }
                " you."
            }
        }
        "#
    );

    test_default!(
        toggle_classes,
        r#"
        html!{p.cute[cuteness>50]{"Squee!"}}
        "#,
        r#"
        html! {
            p.cute[cuteness > 50] { "Squee!" }
        }
        "#
    );

    test_default!(
        toggle_optional_attributes,
        r#"
        html!{p title=[Some("Good password")]{"Correct horse"}}
        "#,
        r#"
        html! {
            p title=[Some("Good password")] { "Correct horse" }
        }
        "#
    );

    test_small_line!(
        line_length_element_id,
        r##"
        html! {
        random-element#big-id-that-should-wrap {}
        }
        "##,
        r##"
        html! {
            random-element
                #big-id-that-should-wrap {}
        }
        "##
    );

    test_small_line!(
        line_length_class,
        r##"
        html! {
        random-element.class1.class2.class3 {}
        }
        "##,
        r##"
        html! {
            random-element
                .class1
                .class2
                .class3 {}
        }
        "##
    );

    test_small_line!(
        line_length_attrs_empty,
        r##"
        html! {
        random-element data-something-long {}
        }
        "##,
        r##"
        html! {
            random-element
                data-something-long {}
        }
        "##
    );

    test_small_line!(
        line_length_attrs_empty_toggle,
        r##"
        html! {
        random-element data-something[true] {}
        }
        "##,
        r##"
        html! {
            random-element
                data-something[true] {}
        }
        "##
    );

    test_small_line!(
        line_length_attrs_normal,
        r##"
        html! {
        random-element data-something="foo" {}
        }
        "##,
        r##"
        html! {
            random-element
                data-something="foo" {}
        }
        "##
    );

    test_small_line!(
        line_length_attrs_optional,
        r##"
        html! {
        random-element data-something=[toggle] {}
        }
        "##,
        r##"
        html! {
            random-element
                data-something=[toggle] {}
        }
        "##
    );

    test_small_line!(
        line_length_element_body_no_expand,
        r##"
        html! {
            p { 
                "one line" 
            }
        }
        "##,
        r##"
        html! {
            p { "one line" }
        }
        "##
    );

    // NOTE: literal length is left to the user to deal with
    test_small_line!(
        line_length_element_body_expand_one_el,
        r##"
        html! {
            p { "one line very very long omg" }
        }
        "##,
        r##"
        html! {
            p {
                "one line very very long omg"
            }
        }
        "##
    );

    test_small_line!(
        line_length_element_body_no_expand_multi_el,
        r##"
        html! {
            p { 
                "one"
                "line"
            }
        }
        "##,
        r##"
        html! {
            p { "one" "line" }
        }
        "##
    );

    test_small_line!(
        line_length_element_body_expand_multi_el,
        r##"
        html! {
            p { "one very" "chunky line" }
        }
        "##,
        r##"
        html! {
            p {
                "one very"
                "chunky line"
            }
        }
        "##
    );

    test_small_line!(
        indented_multi_line_attribute_value,
        r#"
        html! {
            div test={ "This is a long multi-line attribute." "This is another line in the long attribute value." } {
                p { "hi" }
            }
        }
        "#,
        r#"
        html! {
            div
                test={
                    "This is a long multi-line attribute."
                    "This is another line in the long attribute value."
                } {
                p { "hi" }
            }
        }
        "#
    );

    test_default!(
        quoted_attributes,
        r#"
        html! {
            p "class"="bold" { "text" }
        }
        "#,
        r#"
        html! {
            p class="bold" { "text" }
        }
        "#
    );

    test_default!(
        quoted_attributes_special_chars,
        r#"
        html! {
            p "@click.window"="console.log('click')" "x-on:click"="test" ":class"="bold" { "click" }
        }
        "#,
        r#"
        html! {
            p "@click.window"="console.log('click')" x-on:click="test" ":class"="bold" { "click" }
        }
        "#
    );

    test_default!(
        multiline_attribute_toggle_expression,
        r#"
        html! {
            input checked[example_rust_condition().unwrap().map(|x| x.to_string()).unwrap_or_default() == some_long_testing_variable_name];
        }
        "#,
        r#"
        html! {
            input
                checked[
                    example_rust_condition()
                        .unwrap()
                        .map(|x| x.to_string())
                        .unwrap_or_default() == some_long_testing_variable_name
                ];
        }
        "#
    );

    test_default!(
        multiline_attribute_toggle_block,
        r#"
        html! {
            input checked
                disabled[{let x = example_rust_condition().unwrap().map(|x| x.to_string()).unwrap_or_default() == some_long_testing_variable_name; let _y = example_rust_condition().unwrap().map(|x| x.to_string()).unwrap_or_default() == some_long_testing_variable_name; x}];
        }
        "#,
        r#"
        html! {
            input
                checked
                disabled[{
                    let x = example_rust_condition()
                        .unwrap()
                        .map(|x| x.to_string())
                        .unwrap_or_default() == some_long_testing_variable_name;
                    let _y = example_rust_condition()
                        .unwrap()
                        .map(|x| x.to_string())
                        .unwrap_or_default() == some_long_testing_variable_name;
                    x
                }];
        }
        "#
    );
}
