use crop::Rope;
use syn::{
    File, Macro,
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
}

impl<'ast> Visit<'ast> for MacroVisitor<'ast> {
    fn visit_macro(&mut self, node: &'ast Macro) {
        let should_format = self
            .macro_names
            .iter()
            .any(|macro_name| &get_macro_full_path(node) == macro_name);

        if should_format {
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
    };
    macro_visitor.visit_file(file);

    (macro_visitor.source, macro_visitor.macros)
}
