use std::env::args;

use crate::url::Url;

mod url;

/// Strips HTML tags from a string and prints the remaining text to the console.
///
/// This function implements a basic state-machine parser that toggles printing
/// based on whether the current character is inside an HTML tag.
///
/// # Arguments
/// * `body` - A string slice containing the raw HTML content to be processed.
///
/// # Behavior
/// - Characters between `<` and `>` (inclusive) are ignored.
/// - All other characters are printed to standard output.
/// - A newline is appended at the end of the output.
///
/// # Warning
/// This is a naive parser. It does not handle:
/// - HTML entities (e.g., `&nbsp;` or `&amp;`).
/// - Script or Style blocks (it will print the CSS/JS code itself).
/// - Comments or nested angle brackets.
fn show_text_without_tags(body: &str) {
    let mut in_tag = false;
    let mut chars = body.chars();
    while let Some(c) = chars.next() {
        match c {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => print!("{}", c),
            _ => {}
        }
    }
    println!();
}
/// High-level orchestrator that fetches a URL and renders its text content.
///
/// This function handles the result of the network request:
/// - On success: Passes the response body to the tag-stripping renderer.
/// - On failure: Prints a formatted error message to the standard error stream.
///
/// # Arguments
/// * `url` - A validated [`Url`] struct ready to perform a request.
///
/// # Note
/// This function consumes the `Url` object. If you need to reuse the URL after
/// loading, you should modify the signature to take a reference (`&Url`).
fn load(url: Url) {
    match url.request() {
        Ok(body) => {
            show_text_without_tags(&body);
        }
        Err(e) => {
            eprintln!("Error loading URL: {}", e);
        }
    }
}

fn main() {
    let url = args().skip(1).next().expect("No URL provided.");
    load(Url::new(&url).unwrap());
}
