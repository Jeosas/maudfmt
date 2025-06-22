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

_maudfmt_ (will soonTM) automatically manage exanding and collapsing blocks depending on line length.

In some cases, would might prefer to expand a block even if it fits on a single line.
To do this, you can use _magic comments_:

- on the opening bracket line:

```
p { //
   "Small text"
}
```

- inside the block itself (soonTM)

```
p {
   // either on an empty line
   "Small text" // or as a trailing comment
}
```

> doesn't matter if there is an accual comment, the `//` comment marker is enough.

## Acknowledgment

Special thanks to the creators and contributors of the following repos for their awesome work and inspiration:

- [lambda-fairy/maud](https://github.com/lambda-fairy/maud)
- [DioxusLabs/dioxus](https://github.com/DioxusLabs/dioxus)
- [bram209/leptosfmt](https://github.com/bram209/leptosfmt)
