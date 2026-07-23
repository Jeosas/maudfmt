use std::sync::LazyLock;

use rustywind_core::{PlainClassList, app::RustyWind, pattern_sorter};

static SORTER: LazyLock<RustyWind> = LazyLock::new(RustyWind::default);

/// Sorts a whitespace-separated string of classes, e.g. the value of a `class="..."` attribute.
/// Returns the input unchanged if it isn't a plain class list (shouldn't happen for a literal
/// string pulled straight out of source, but the validation is fallible so we fall back safely).
pub fn sort_class_string(classes: &str) -> String {
    match PlainClassList::parse(classes) {
        Ok(list) => SORTER.sort_class_list(list),
        Err(_) => classes.to_string(),
    }
}

/// Reorders `items` by the Tailwind sort order of their class name, as returned by `class_name`.
/// Leaves `items` untouched if any name is `None` (e.g. a dynamically spliced class name), since
/// the resulting order would otherwise be arbitrary.
pub fn sort_by_class_name<T>(items: Vec<T>, class_name: impl Fn(&T) -> Option<String>) -> Vec<T> {
    let Some(names) = items.iter().map(&class_name).collect::<Option<Vec<_>>>() else {
        return items;
    };
    let refs: Vec<&str> = names.iter().map(String::as_str).collect();
    let sorted = pattern_sorter::sort_classes(&refs);

    let mut slots: Vec<Option<T>> = items.into_iter().map(Some).collect();
    sorted
        .into_iter()
        .map(|class| {
            let index = refs
                .iter()
                .position(|candidate| std::ptr::eq(*candidate, class))
                .expect("sorted class must originate from the input refs");
            slots[index]
                .take()
                .expect("each input consumed exactly once")
        })
        .collect()
}
