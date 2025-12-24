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

fn main() {
    println!("Hello, world!");
}
