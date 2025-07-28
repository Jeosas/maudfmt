use crop::Rope;
use syn::{
    File, Macro, Meta,
    spanned::Spanned,
    visit::{self, Visit},
};

pub struct MaudMacro<'a> {
    pub macro_: &'a Macro,
    pub indent: Indent,
    pub macro_name: String,
}

pub struct Indent {
    pub tabs: usize,
    pub spaces: usize,
}

struct MacroVisitor<'a> {
    macros: Vec<MaudMacro<'a>>,
    source: Rope,
    macro_names: &'a Vec<String>,
    skip_count: usize,
}

impl<'ast> Visit<'ast> for MacroVisitor<'ast> {
    fn visit_macro(&mut self, node: &'ast Macro) {
        let should_format = self
            .macro_names
            .iter()
            .any(|macro_name| &get_macro_full_path(node) == macro_name);

        if should_format && self.skip_count == 0 {
            let span_line = node.span().start().line;
            let line = self.source.line(span_line - 1);

            let indent_chars: Vec<_> = line
                .chars()
                .take_while(|&c| c == ' ' || c == '\t')
                .collect();

            let tabs = indent_chars.iter().filter(|&&c| c == '\t').count();
            let spaces = indent_chars.iter().filter(|&&c| c == ' ').count();

            self.macros.push(MaudMacro {
                macro_: node,
                indent: Indent { tabs, spaces },
                macro_name: get_macro_full_path(node),
            })
        }

        // Delegate to the default impl to visit any nested functions.
        visit::visit_macro(self, node);
    }

    // attributes can occur on stmts and items - we need to make sure the stack is reset when we exit
    // this means we save the skipped length and set it back to its original length
    fn visit_stmt(&mut self, i: &'ast syn::Stmt) {
        let skipped_len = self.skip_count;
        syn::visit::visit_stmt(self, i);
        self.skip_count = skipped_len;
    }

    fn visit_item(&mut self, i: &'ast syn::Item) {
        let skipped_len = self.skip_count;
        syn::visit::visit_item(self, i);
        self.skip_count = skipped_len;
    }

    fn visit_attribute(&mut self, i: &'ast syn::Attribute) {
        // we need to communicate that this stmt is skipped up the tree
        if attr_is_rustfmt_skip(i) {
            self.skip_count += 1;
        }

        syn::visit::visit_attribute(self, i);
    }
}

/// Check if an attribute is a rustfmt skip attribute
fn attr_is_rustfmt_skip(i: &syn::Attribute) -> bool {
    match &i.meta {
        Meta::Path(path) => {
            path.segments.len() == 2
                && matches!(i.style, syn::AttrStyle::Outer)
                && path.segments[0].ident == "rustfmt"
                && path.segments[1].ident == "skip"
        }
        _ => false,
    }
}

fn get_macro_full_path(mac: &Macro) -> String {
    mac.path
        .segments
        .iter()
        .map(|path| path.ident.to_string())
        .collect::<Vec<String>>()
        .join("::")
}

pub fn collect_macros_from_file<'a>(
    file: &'a File,
    source: Rope,
    macro_names: &'a Vec<String>,
) -> (Rope, Vec<MaudMacro<'a>>) {
    let mut macro_visitor = MacroVisitor {
        macros: Vec::new(),
        source,
        macro_names,
        skip_count: 0,
    };
    macro_visitor.visit_file(file);

    (macro_visitor.source, macro_visitor.macros)
}
