use anyhow::{Context, Result};
use crop::Rope;

mod collect;
mod format;
mod line_length;
mod print;
mod unparse;
mod vendor;

#[cfg(test)]
mod testing;

use vendor::ast;

pub use format::FormatOptions;

pub fn try_fmt_file(source: &str, options: &format::FormatOptions) -> Result<String> {
    let (processed_source, ignore_info) = format::preprocess_source_for_ignore(source);

    let ast = syn::parse_file(&processed_source).context("Failed to parse source")?;
    let rope = Rope::from(processed_source);
    let (mut rope, macros) = collect::collect_macros_from_file(&ast, rope, &options.macro_names);
    let formatted_processed = format::format_source(&mut rope, macros, options);

    // Reinsert ignored lines if any
    if ignore_info.is_empty() {
        Ok(formatted_processed)
    } else {
        Ok(format::reinsert_ignored_lines_in_source(
            &formatted_processed,
            &ignore_info,
        ))
    }
}
