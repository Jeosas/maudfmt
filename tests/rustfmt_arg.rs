use anyhow::Result;
use assert_cmd::Command;
use assert_fs::prelude::*;
use predicates::prelude::*;
use pretty_assertions::assert_eq;

static IN_FILE: &str = r#"use maud::{DOCTYPE, html, Markup};

/// A basic header with a dynamic `page_title`.
fn header(page_title: &str) -> Markup {html!{(DOCTYPE) meta charset="utf-8";title{(page_title)}} }

/// A static footer.
fn footer() -> Markup { html!{footer{a href="rss.atom"{"RSS Feed"}}} }

/// The final Markup, including `header` and `footer`.
///
/// Additionally takes a `greeting_box` that's `Markup`, not `&str`.
pub fn page(title: &str, greeting_box: Markup) -> Markup { html!{(header(title)) h1{(title)}(greeting_box)(footer())} }
"#;

static OUT_FILE: &str = r#"use maud::{html, Markup, DOCTYPE};

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
fn execute_rustfmt_on_files() -> Result<()> {
    dbg!(&IN_FILE);
    let file = assert_fs::NamedTempFile::new("sample.rs")?;
    file.write_str(IN_FILE)?;

    // When
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;
    cmd.arg("--rustfmt").arg(file.path());

    // Then
    cmd.assert().success();
    assert_eq!(std::fs::read_to_string(&file)?, OUT_FILE);

    Ok(())
}

#[test]
fn execute_rustfmt_on_stdin() -> Result<()> {
    // Given
    let file = assert_fs::NamedTempFile::new("stdin")?;
    file.write_str(IN_FILE)?;

    // When
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;
    cmd.arg("--rustfmt").arg("-s").pipe_stdin(file)?;

    // Then
    cmd.assert()
        .success()
        .stdout(predicate::str::diff(OUT_FILE));

    Ok(())
}
