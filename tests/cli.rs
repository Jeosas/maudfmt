use anyhow::Result;
use assert_cmd::Command;
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
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;
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
    let mut cmd = Command::cargo_bin("maudfmt")?;
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
    let mut cmd = Command::cargo_bin("maudfmt")?;
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
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;
    cmd.arg("-s").pipe_stdin(file)?;

    // Then
    cmd.assert()
        .success()
        .stdout(predicate::str::diff(OUT_FILE));

    Ok(())
}
