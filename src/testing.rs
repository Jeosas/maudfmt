use std::sync::LazyLock;

use crate::format::FormatOptions;

pub static DEFAULT_OPTIONS: LazyLock<FormatOptions> = LazyLock::new(FormatOptions::default);
pub static SMALL_LINE_OPTIONS: LazyLock<FormatOptions> = LazyLock::new(|| FormatOptions {
    line_length: 40,
    ..Default::default()
});

macro_rules! test_default {
    ($title: ident, $content: literal, $expected: literal ) => {
        #[test]
        fn $title() {
            // check formatter works as expected
            assert_eq!(
                crate::try_fmt_file($content, &DEFAULT_OPTIONS).expect("should be able to parse"),
                String::from($expected)
            );
            // check that `$expected` is a valid maud macro
            crate::try_fmt_file($expected, &DEFAULT_OPTIONS)
                .expect("expected should be parsable and valid maud");
        }
    };
}

macro_rules! test_small_line {
    ($title: ident, $content: literal, $expected: literal ) => {
        #[test]
        fn $title() {
            // check formatter works as expected
            assert_eq!(
                crate::try_fmt_file($content, &SMALL_LINE_OPTIONS)
                    .expect("should be able to parse"),
                String::from($expected)
            );
            // check that `$expected` is a valid maud macro
            crate::try_fmt_file($expected, &SMALL_LINE_OPTIONS)
                .expect("expected should be parsable and valid maud");
        }
    };
}

pub(crate) use test_default;
pub(crate) use test_small_line;
