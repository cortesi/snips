//! Example program demonstrating snippet extraction.

/// Code that will be embedded into documentation via snips markers.
pub fn example() {
    // snips-start: main_feature
    println!("This is the code I want in my docs!");
    // snips-end: main_feature
}

/// Minimal entry point that calls the example snippet.
fn main() {
    example();
}
