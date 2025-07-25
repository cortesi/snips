# snips

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
    // This code block will be automatically updated.
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

## Installation

```bash
# Once published, you'll be able to install with:
cargo install snips
```

## Commands

  * `snips process [FILES]...`

      * Processes files and writes changes directly to disk.

  * `snips check [FILES]...`

      * Checks if files are in sync. Exits with a non-zero status code if changes are needed.

  * `snips diff [FILES]...`

      * Shows a colored diff of pending changes without modifying any files.
