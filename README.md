# snips

![Discord](https://img.shields.io/discord/1381424110831145070?style=flat-square&logo=rust&link=https%3A%2F%2Fdiscord.gg%2FfHmRmuBDxF)
[![Crates.io](https://img.shields.io/crates/v/snips.svg)](https://crates.io/crates/snips)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

**Keep your code snippets in sync with your source files. Effortlessly.**

`snips` is a command-line tool that prevents your documentation from becoming
stale. It works by creating a direct link between your source code and your
Markdown files, ensuring your code examples are always up-to-date.

-----

## How It Works

`snips` uses two simple markers to work its magic:

1.  **In your source code**, you define a snippet with special comments:

    ```rust
    // In examples/example.rs
    pub fn example() {
        // snips-start: main_feature
        println!("This is the code I want in my docs!");
        // snips-end: main_feature
    }
    ```

2.  **In your Markdown file**, you reference that snippet using an HTML
    comment:

````markdown
<!-- snips: examples/example.rs#main_feature -->
```rust
println!("This is the code I want in my docs!");
```
````

Run `snips process`, and the tool will inject the source code, automatically
handling indentation and language detection.

-----

## Features

  * **Named Snippets**: Pull specific blocks of code from any source file.
  * **Whole-File Insertion**: Embed an entire source file with a simple marker
    (`<!-- snips: path/to/file.rs -->`).
  * **CI/CD Friendly**: The `snips check` command fails if your docs are out of
    sync, making it perfect for CI pipelines.
  * **Language Agnostic**: Works with any programming language that supports
    comments.

-----

## Community

Want to contribute? Have ideas or feature requests? Come tell us about it on
[Discord](https://discord.gg/fHmRmuBDxF).

-----

## Installation

```bash
cargo install snips
```

## Commands

  * `snips render [FILES]...`

      * Processes files and writes changes directly to disk.

  * `snips check [FILES]...`

      * Checks if files are in sync. Exits with a non-zero status code if changes are needed.

  * `snips diff [FILES]...`

      * Shows a colored diff of pending changes without modifying any files.
