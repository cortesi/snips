# snips

![Discord](https://img.shields.io/discord/1381424110831145070?style=flat-square&logo=rust&link=https%3A%2F%2Fdiscord.gg%2FfHmRmuBDxF)
[![Crates.io](https://img.shields.io/crates/v/snips.svg)](https://crates.io/crates/snips)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

**snips keeps code snippets in Mardown documentation in sync with source files.**

`snips` is a command-line tool that prevents your documentation from becoming
stale. It works by creating a direct link between your source code and your
Markdown files, ensuring your code examples are always up-to-date.

-----

## How It Works

`snips` uses two simple markers to work its magic:

1.  **In your source code**, you define a snippet with `snips-start` and
    `snips-end` comments:

    ```rust
    // In examples/example.rs
    pub fn example() {
        // snips-start: main_feature
        println!("This is the code I want in my docs!");
        // snips-end: main_feature
    }
    ```

2.  **In your Markdown file**, reference snippets using an HTML comment:

````markdown
<!-- snips: examples/example.rs#main_feature -->
```rust
println!("This is the code I want in my docs!");
```
````

HTML comments are used to avoid interfering with Markdown rendering - they are
hidden from view in the final output.

Run `snips` to process all Markdown files in the current directory and update
all contained snippets. 

-----

## Features

  * **Named Snippets**: Pull specific blocks of code from any source file.
  * **Whole-File Insertion**: Embed an entire source file with a simple marker
    (`<!-- snips: path/to/file.rs -->`).
  * **CI/CD Friendly**: The `--check` flag exits with non-zero status if docs
    are out of sync, making it perfect for CI pipelines.
  * **Language Agnostic**: Works with any programming language that supports
    comments.
  * **Smart Language Detection**: Automatically detects programming languages
    using the official [GitHub Linguist language specification](https://github.com/github/linguist),
    providing accurate CodeMirror syntax highlighting modes.

-----

## Community

Want to contribute? Have ideas or feature requests? Come tell us about it on
[Discord](https://discord.gg/fHmRmuBDxF).

-----

## Installation

```bash
cargo install snips
```

## Usage

```
snips [OPTIONS] [FILES]...
```

Processes Markdown files, updating embedded snippets from their source files.
When no files are provided, `snips` processes every `.md` and `.markdown` file
in the current directory.

### Options

  * `--check` - Don't write changes, exit with non-zero status if files are out
    of sync. Useful for CI pipelines.

  * `--diff` - Show a colored diff of pending changes without modifying files.

  * `--quiet` - Suppress output.

-----

## Related Projects

`snips` uses the [languages](https://github.com/cortesi/languages) library for 
programming language detection. This library provides access to GitHub's 
Linguist language specification, ensuring accurate language identification and 
CodeMirror mode selection for syntax highlighting in your documentation.
