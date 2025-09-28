use anyhow::Result;
use assert_cmd::cargo;
use assert_fs::prelude::*;
use predicates::prelude::*;
use pretty_assertions::assert_eq;

static IN_FILE: &str = r#"
use maud::{DOCTYPE, html, Markup};

/// A basic header with a dynamic `page_title`.
fn header(page_title: &str) -> Markup {
    html!{(DOCTYPE) meta charset="utf-8";title{(page_title)}}
}

/// A static footer.
fn footer() -> Markup {
    html!{footer{a href="rss.atom"{"RSS Feed"}}}
}

/// The final Markup, including `header` and `footer`.
///
/// Additionally takes a `greeting_box` that's `Markup`, not `&str`.
pub fn page(title: &str, greeting_box: Markup) -> Markup {
    html!{(header(title)) h1{(title)}(greeting_box)(footer())}
}
"#;

static OUT_FILE: &str = r#"
use maud::{DOCTYPE, html, Markup};

/// A basic header with a dynamic `page_title`.
fn header(page_title: &str) -> Markup {
    html! {
        (DOCTYPE)
        meta charset="utf-8";
        title { (page_title) }
    }
}

/// A static footer.
fn footer() -> Markup {
    html! {
        footer {
            a href="rss.atom" { "RSS Feed" }
        }
    }
}

/// The final Markup, including `header` and `footer`.
///
/// Additionally takes a `greeting_box` that's `Markup`, not `&str`.
pub fn page(title: &str, greeting_box: Markup) -> Markup {
    html! {
        (header(title))
        h1 { (title) }
        (greeting_box)
        (footer())
    }
}
"#;

#[test]
fn format_file_from_argument() -> Result<()> {
    let file = assert_fs::NamedTempFile::new("sample.rs")?;
    file.write_str(IN_FILE)?;

    // When
    let mut cmd = cargo::cargo_bin_cmd!();
    cmd.arg(file.path());

    // Then
    cmd.assert().success();
    assert_eq!(std::fs::read_to_string(&file)?, OUT_FILE);

    Ok(())
}

#[test]
fn format_multiple_files_from_argument() -> Result<()> {
    // Given
    let file_1 = assert_fs::NamedTempFile::new("sample_1.rs")?;
    file_1.write_str(IN_FILE)?;
    let file_2 = assert_fs::NamedTempFile::new("sample_2.rs")?;
    file_2.write_str(IN_FILE)?;

    // When
    let mut cmd = cargo::cargo_bin_cmd!();
    cmd.arg(file_1.path()).arg(file_2.path());

    // Then
    cmd.assert().success();
    assert_eq!(std::fs::read_to_string(&file_1)?, OUT_FILE);
    assert_eq!(std::fs::read_to_string(&file_2)?, OUT_FILE);

    Ok(())
}

#[test]
fn format_dir_from_argument() -> Result<()> {
    // Given
    let directory = assert_fs::TempDir::new()?;
    let file_1 = directory.child("sample_1.rs");
    file_1.write_str(IN_FILE)?;
    let file_2 = directory.child("sample_2.rs");
    file_2.write_str(IN_FILE)?;

    // When
    let mut cmd = cargo::cargo_bin_cmd!();
    cmd.arg(directory.path());

    // Then
    cmd.assert().success();
    assert_eq!(std::fs::read_to_string(&file_1)?, OUT_FILE);
    assert_eq!(std::fs::read_to_string(&file_2)?, OUT_FILE);

    Ok(())
}

#[test]
fn format_file_from_stdin() -> Result<()> {
    // Given
    let file = assert_fs::NamedTempFile::new("stdin")?;
    file.write_str(IN_FILE)?;

    // When
    let mut cmd = cargo::cargo_bin_cmd!();
    cmd.arg("-s").pipe_stdin(file)?;

    // Then
    cmd.assert()
        .success()
        .stdout(predicate::str::diff(OUT_FILE));

    Ok(())
}

static CUSTOM_MACRO_IN_FILE: &str = r#"
use hypertext::prelude::*;

fn header(page_title: &str) -> Markup {
    maud!{(DOCTYPE) meta charset="utf-8";title{(page_title)}}
}

fn footer() -> Markup {
    hyperscript::maud!{footer{a href="rss.atom"{"RSS Feed"}}}
}

fn sidebar() -> Markup {
    html!{div class="sidebar"{p{"This should not be formatted"}}}
}

pub fn page(title: &str, greeting_box: Markup) -> Markup {
    maud!{(header(title)) h1{(title)}(greeting_box)(footer())(sidebar())}
}
"#;

static CUSTOM_MACRO_OUT_FILE: &str = r#"
use hypertext::prelude::*;

fn header(page_title: &str) -> Markup {
    maud! {
        (DOCTYPE)
        meta charset="utf-8";
        title { (page_title) }
    }
}

fn footer() -> Markup {
    hyperscript::maud! {
        footer {
            a href="rss.atom" { "RSS Feed" }
        }
    }
}

fn sidebar() -> Markup {
    html!{div class="sidebar"{p{"This should not be formatted"}}}
}

pub fn page(title: &str, greeting_box: Markup) -> Markup {
    maud! {
        (header(title))
        h1 { (title) }
        (greeting_box)
        (footer())
        (sidebar())
    }
}
"#;

#[test]
fn format_file_with_custom_macro_names() -> Result<()> {
    let file = assert_fs::NamedTempFile::new("sample.rs")?;
    file.write_str(CUSTOM_MACRO_IN_FILE)?;

    let mut cmd = cargo::cargo_bin_cmd!();
    cmd.arg("--macro-names")
        .arg("maud,hyperscript::maud")
        .arg(file.path());

    cmd.assert().success();
    assert_eq!(std::fs::read_to_string(&file)?, CUSTOM_MACRO_OUT_FILE);

    Ok(())
}

#[test]
fn format_stdin_with_custom_macro_names() -> Result<()> {
    let file = assert_fs::NamedTempFile::new("stdin")?;
    file.write_str(CUSTOM_MACRO_IN_FILE)?;

    let mut cmd = cargo::cargo_bin_cmd!();
    cmd.arg("-s")
        .arg("--macro-names")
        .arg("maud,hyperscript::maud")
        .pipe_stdin(file)?;

    cmd.assert()
        .success()
        .stdout(predicate::str::diff(CUSTOM_MACRO_OUT_FILE));

    Ok(())
}

#[test]
fn format_file_with_custom_macro_names_short_arg() -> Result<()> {
    let file = assert_fs::NamedTempFile::new("sample.rs")?;
    file.write_str(CUSTOM_MACRO_IN_FILE)?;

    let mut cmd = cargo::cargo_bin_cmd!();
    cmd.arg("-m").arg("maud,hyperscript::maud").arg(file.path());

    cmd.assert().success();
    assert_eq!(std::fs::read_to_string(&file)?, CUSTOM_MACRO_OUT_FILE);

    Ok(())
}

static LONG_LINE_IN_FILE: &str = r#"
use maud::{html, Markup};

fn test() -> Markup {
    html!{div class="very-long-class-name" id="super-long-id-name"{p data-attr="value"{"Content"}}}
}
"#;

static LONG_LINE_OUT_FILE_SHORT_LENGTH: &str = r#"
use maud::{html, Markup};

fn test() -> Markup {
    html! {
        div class="very-long-class-name"
            id="super-long-id-name"
        {
            p data-attr="value" { "Content" }
        }
    }
}
"#;

static LONG_LINE_OUT_FILE_LONG_LENGTH: &str = r#"
use maud::{html, Markup};

fn test() -> Markup {
    html! {
        div class="very-long-class-name" id="super-long-id-name" {
            p data-attr="value" { "Content" }
        }
    }
}
"#;

#[test]
fn format_file_with_short_line_length() -> Result<()> {
    let file = assert_fs::NamedTempFile::new("sample.rs")?;
    file.write_str(LONG_LINE_IN_FILE)?;

    let mut cmd = cargo::cargo_bin_cmd!();
    cmd.arg("--line-length").arg("50").arg(file.path());

    cmd.assert().success();
    assert_eq!(
        std::fs::read_to_string(&file)?,
        LONG_LINE_OUT_FILE_SHORT_LENGTH
    );

    Ok(())
}

#[test]
fn format_file_with_long_line_length() -> Result<()> {
    let file = assert_fs::NamedTempFile::new("sample.rs")?;
    file.write_str(LONG_LINE_IN_FILE)?;

    let mut cmd = cargo::cargo_bin_cmd!();
    cmd.arg("--line-length").arg("200").arg(file.path());

    cmd.assert().success();
    assert_eq!(
        std::fs::read_to_string(&file)?,
        LONG_LINE_OUT_FILE_LONG_LENGTH
    );

    Ok(())
}

#[test]
fn format_stdin_with_line_length() -> Result<()> {
    let file = assert_fs::NamedTempFile::new("stdin")?;
    file.write_str(LONG_LINE_IN_FILE)?;

    let mut cmd = cargo::cargo_bin_cmd!();
    cmd.arg("-s")
        .arg("--line-length")
        .arg("50")
        .pipe_stdin(file)?;

    cmd.assert()
        .success()
        .stdout(predicate::str::diff(LONG_LINE_OUT_FILE_SHORT_LENGTH));

    Ok(())
}
