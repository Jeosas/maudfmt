# Maudfmt

An opinionated yet customizable [Maud](https://github.com/lambda-fairy/maud) formatter.

## Install

`cargo install maudfmt`

or for trying out unreleased features:

`cargo install --git https://github.com/jeosas/maudfmt.git`

## Usage

### Format files as arguments

- Providing files

```
maudfmt ./build.rs ./src/main.rs
```

- Providing a directory

```
maudfmt ./src
```

- Providing a glob

```
maudfmt ./{src, tests}/**/*
```

### Format files through stdin

```
cat ./src/main.rs | maudfmt -s
```

### Options

<!-- help start -->

```console
$ maudfmt --help
An opinionated yet customizable Maud formatter.

Usage: maudfmt [OPTIONS] [FILE]...

Arguments:
  [FILE]...  A space separated list of file, directory or glob

Options:
  -s, --stdin    Format stdin and write to stdout
  -h, --help     Print help
  -V, --version  Print version
```

<!-- help end -->

## IDE Setup

### vim - conform.nvim

```lua
require("conform").setup({
  formatters = {
    maudfmt = {
      command = "maudfmt",
      args = { "-s" },  -- add any config you wish
    },
  },
  formatters_by_ft = {
    rust = { "rustfmt", "maudfmt" },
  },
})
```

## Tips and Tricks

### Magic comments

_maudfmt_ automatically manages exanding and collapsing blocks depending on line length.

In some cases, would might prefer to expand a block even if it fits on a single line.
To do this, you can use _magic comments_:

- on the opening bracket line:

```
p { //
   "Small text"
}
```

- inside the block itself

```
p {
   // either on an empty line
   "Small text" // or as a trailing comment
}
```

> doesn't matter if there is an actual comment, the `//` comment marker is enough.

## Acknowledgment

Special thanks to the creators and contributors of the following projects for their awesome work and inspiration

- [lambda-fairy/maud](https://github.com/lambda-fairy/maud)
- [bram209/leptosfmt](https://github.com/bram209/leptosfmt)
- [DioxusLabs/dioxus](https://github.com/DioxusLabs/dioxus)

## A note on non-doc comments

Currently this formatter does not support non-doc comments in code blocks (`Splices`).
It uses `prettyplease` for formatting rust code, and `prettyplease` does not support this.
This means that you _can_ use non-doc comments throughout your view macro, as long as they don't reside within code blocks.

> A bit more context: `prettyplease` uses `syn` to parse rust syntax. According to https://doc.rust-lang.org/reference/comments.html#non-doc-comments
> non-doc comments _are interpreted as a form of whitespace_ by the parser; `syn` basically ignores/skips these comments and does not include them in the syntax tree.
