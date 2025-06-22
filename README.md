# Maudfmt

An opinionated yet customizable [Maud](https://github.com/lambda-fairy/maud) formatter.

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
      prepend_args = { "-s" },
      args = {},  -- add any config you wish
    },
  },
  formatters_by_ft = {
    rust = { "rustfmt", "maudfmt", lsp_format = "fallback" },
  },
})
```

## Acknowledgment

Special thanks to the creators and contributors of the following repos for their awesome work and inspiration:

- [lambda-fairy/maud](https://github.com/lambda-fairy/maud)
- [DioxusLabs/dioxus](https://github.com/DioxusLabs/dioxus)
- [bram209/leptosfmt](https://github.com/bram209/leptosfmt)
