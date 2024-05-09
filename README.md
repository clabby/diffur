# `diffur`

A small TUI that assists in diffing two bodies of text easily. Rather than creating two files and passing them to `diff`,
this tool handles the tempfile creation and cleanup, as well quick launch hotkeys for editing the files and executing `diff`.

https://github.com/clabby/diffur/assets/8406232/b8211eca-d77e-4758-822f-c80d8101447f

## `delta`

This tool is meant to be used alongside [`delta`][delta], a custom pager for `diff` and `git`. To install `delta`, run:

```sh
cargo install git-delta
```

## Installation

After `delta` has been installed, `diffur` can be installed by running:

```sh
git clone git@github.com:clabby/diffur.git && cd diffur && cargo install --path .
```

[delta]: https://github.com/dandavison/delta
