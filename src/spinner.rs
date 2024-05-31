use colored::{Color, ColoredString, Colorize};
use termspin::spinner::{from_array, FromArray};

pub fn dots() -> FromArray<10, ColoredString> {
    from_array(["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"].map(|dot| dot.color(Color::Blue)))
}
