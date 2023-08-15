use ansi_parser::{AnsiParser, AnsiSequence, Output};

#[cfg(feature = "tracing")]
use tracing::debug;

/// The three color options for SGR color codes and a default option.
#[derive(Default, Debug, Clone, PartialEq, Eq)]
enum SgrColor {
    #[default]
    Reset,
    /// A value of 0..=7 to represent one of 8 standard terminal colors.
    Console(u8),
    /// A value of 0..=255 to represent one of 256 standard terminal colors.
    ExpandedConsole(u8),
    /// Three values of 0..=255 to represent a "true" RGB color value.
    True(u8, u8, u8),
}

/// The continuous state of the next block of text. A new set of SGR parameters does not imply that
/// the state should be reset unless the 0 (reset) parameter has been explicitly used.
#[derive(Default, Debug, Clone, PartialEq, Eq)]
#[allow(clippy::struct_excessive_bools)]
struct GraphicsModeState {
    // reset is interpreted as resetting everything to default
    // the methods defined here are taken from:
    // https://en.wikipedia.org/wiki/ANSI_escape_code#SGR_(Select_Graphic_Rendition)_parameters
    // and have been selected in accordance with their compatibility with static HTML
    bold: bool,
    italic: bool,
    underline: bool,
    strikethrough: bool,

    color: SgrColor,
    background_color: SgrColor,
}

/// The standard terminal colors, approximated from experience with various terminals.
static COLORS: [&str; 8] = [
    "black", "red", "green", "yellow", "blue", "purple", "cyan", "gray",
];

/// Iterate over a slice using a variable-argument matching pattern. The iterator moves after the
/// end of the slice, so matching two values moves the slice to the right by two.
///
/// # Examples
///
/// ```rust
/// let slice = ["a", "b", "c", "d"];
/// let mut output = vec![];
/// opaque_ansi::iter_over! {
///     slice;
///     [a @ "a"] => output.push(vec![a]),
///     [b @ "b", c @ "c"] => output.push(vec![b, c]),
///     [d @ "d"] => output.push(vec![d]),
/// }
/// # assert_eq!(vec![vec![&"a"], vec![&"b", &"c"], vec![&"d"]], output);
/// ```
#[macro_export]
macro_rules! iter_over {
    ($input:expr; $([$($t:pat_param),*] => $s:expr,)+) => {
        let mut input = &$input[..];
        loop {
            input = match input {
                $(
                    [$($t),*, input @ ..] => { $s; input }
                ),+
                [_, input @ ..] => input,
                [] => break,
            }
        }
    }
}

impl GraphicsModeState {
    /// Returns a copy of Self, setting values based off SGR parameters from the input.
    fn clone_from_scan(&self, input: &[u8]) -> Self {
        let mut state = self.clone();

        iter_over! {
            input;
            [0] => state = GraphicsModeState::default(),
            [1] => state.bold = true,
            [3] => state.italic = true,
            [4] => state.underline = true,
            [9] => state.strikethrough = true,
            [n @ 30..=37] => state.color = SgrColor::Console(n - 30),
            [n @ 40..=47] => state.background_color = SgrColor::Console(n - 40),
            [38, 5, n] => state.color = SgrColor::ExpandedConsole(*n),
            [48, 5, n] => state.background_color = SgrColor::ExpandedConsole(*n),
            [38, 2, r, g, b] => state.color = SgrColor::True(*r, *g, *b),
            [48, 2, r, g, b] => state.background_color = SgrColor::True(*r, *g, *b),
            [39] => state.color = SgrColor::Reset,
            [49] => state.background_color = SgrColor::Reset,
        }

        state
    }

    /// Returns opening and closing HTML tags representing the formatting of the text. All
    /// non-truecolor colors require a stylesheet to be provided.
    /// [`SgrColor::Console`] values are represented using a `--color-{c}` CSS variable, using
    /// the value from [`COLORS`].
    /// [`SgrColor::ExpandedConsole`] values are represented using a `--terminal-color-{n}` CSS
    /// variable.
    /// [`SgrColor::True`] values are represented using `rgb({r}, {g}, {b})`.
    /// When the state matches the "default" state, empty strings are returned.
    fn build_tags(&self) -> (String, String) {
        if self == &Self::default() {
            return (String::new(), String::new());
        }

        let mut opening_tags = vec![];
        let mut closing_tags = vec![];

        if self.bold {
            opening_tags.push("<strong>".to_string());
            closing_tags.push("</strong>".to_string());
        }

        if self.italic {
            opening_tags.push("<em>".to_string());
            closing_tags.push("</em>".to_string());
        }

        if self.underline {
            opening_tags.push("<u>".to_string());
            closing_tags.push("</u>".to_string());
        }

        if self.strikethrough {
            opening_tags.push("<s>".to_string());
            closing_tags.push("</s>".to_string());
        }

        match self.color {
            SgrColor::Console(n @ 0..=7) => {
                let span = format!(
                    "<span style=\"color: var(--color-{})\">",
                    COLORS[n as usize]
                );
                opening_tags.push(span);
                closing_tags.push("</span>".to_string());
            }
            SgrColor::ExpandedConsole(n) => {
                let span = format!("<span style=\"color: var(--terminal-color-{n})\">");
                opening_tags.push(span);
                closing_tags.push("</span>".to_string());
            }
            SgrColor::True(r, g, b) => {
                let span = format!("<span style=\"color: rgb({r}, {g}, {b})\">");
                opening_tags.push(span);
                closing_tags.push("</span>".to_string());
            }
            _ => (),
        }

        match self.background_color {
            SgrColor::Console(n @ 0..=7) => {
                let span = format!(
                    "<span style=\"background-color: var(--color-{})\">",
                    COLORS[n as usize]
                );
                opening_tags.push(span);
                closing_tags.push("</span>".to_string());
            }
            SgrColor::ExpandedConsole(n) => {
                let span = format!("<span style=\"background-color: var(--terminal-color-{n})\">");
                opening_tags.push(span);
                closing_tags.push("</span>".to_string());
            }
            SgrColor::True(r, g, b) => {
                let span = format!("<span style=\"background-color: rgb({r}, {g}, {b})\">");
                opening_tags.push(span);
                closing_tags.push("</span>".to_string());
            }
            _ => (),
        }

        (
            opening_tags.join(""),
            closing_tags.into_iter().rev().collect::<String>(),
        )
    }
}

/// Render an ANSI terminal output to HTML, using [SGR] parameters to generate formatting.
///
/// [SGR]: https://en.wikipedia.org/wiki/ANSI_escape_code#SGR_(Select_Graphic_Rendition)_parameters
#[cfg_attr(feature = "tracing", tracing::instrument(skip(input)))]
pub fn rewrite_ansi_to_html(input: &str) -> String {
    #[cfg(feature = "tracing")]
    debug!("parsing ANSI escape codes");

    let parsed: Vec<Output> = input
        .ansi_parse()
        .filter(|value| {
            // Trash all Escapes that aren't stylish
            matches!(
                value,
                Output::Escape(AnsiSequence::SetGraphicsMode(_)) | Output::TextBlock(_)
            )
        })
        .collect();

    let mut state = GraphicsModeState::default();
    let mut output = vec![];

    output.push("<pre class=\"ansi_output\"><code>".to_string());

    #[cfg(feature = "tracing")]
    debug!("converting ANSI escape code and text chunks to HTML");

    for block in parsed {
        match block {
            Output::Escape(AnsiSequence::SetGraphicsMode(mode)) => {
                state = state.clone_from_scan(&mode[..]);
            }
            Output::TextBlock(text) => {
                let (opening_tags, closing_tags) = state.build_tags();
                let text = html_escape::encode_text(text);
                output.push(format!("{opening_tags}{text}{closing_tags}"));
            }
            Output::Escape(_) => {}
        }
    }

    output.push("</code></pre>".to_string());
    output.join("")
}

#[cfg(test)]
mod tests {
    use super::*;
    use console::Style;

    #[test]
    fn parsing_basic_colors() {
        // names are slightly different but it's :ok_hand:
        let colors = [
            Style::new().black(),
            Style::new().red(),
            Style::new().green(),
            Style::new().yellow(),
            Style::new().blue(),
            Style::new().magenta(),
            Style::new().cyan(),
            Style::new().white(),
        ];
        let mut state = GraphicsModeState::default();
        for (i, color) in colors.iter().enumerate() {
            let color_text = color.apply_to(" ").to_string();
            let color_code = color_text.ansi_parse().next().unwrap();
            let Output::Escape(AnsiSequence::SetGraphicsMode(color_code)) = color_code else {
                unreachable!();
            };
            state = state.clone_from_scan(&color_code);
            // TODO: assert_matches!()
            assert!(
                matches!(state.color, SgrColor::Console(n) if (n as usize) == i),
                "{:?} doesn't equal {}",
                state.color,
                i,
            );
        }
    }

    #[test]
    fn parsing_expanded_colors() {
        let colors = (0..=255).map(|c| Style::new().color256(c));
        let mut state = GraphicsModeState::default();
        for (i, color) in colors.enumerate() {
            let color_text = color.apply_to(" ").to_string();
            let color_code = color_text.ansi_parse().next().unwrap();
            let Output::Escape(AnsiSequence::SetGraphicsMode(color_code)) = color_code else {
                unreachable!();
            };
            state = state.clone_from_scan(&color_code);
            // TODO: assert_matches!()
            assert!(
                matches!(state.color, SgrColor::ExpandedConsole(n) if (n as usize) == i),
                "{:?} doesn't equal {}",
                state.color,
                i,
            );
        }
    }

    #[test]
    fn parsing_styling() {
        // console-rs doesn't implement strikethrough :(
        let styled_text = Style::new()
            .bold()
            .italic()
            .underlined()
            .apply_to(" ")
            .to_string();
        let mut style_codes: Vec<_> = styled_text.ansi_parse().collect();
        let reset = style_codes.pop().expect("no reset code"); // remove the reset code
        let expected_state = GraphicsModeState {
            bold: true,
            italic: true,
            underline: true,
            ..Default::default()
        };
        let mut state = GraphicsModeState::default();
        for code in style_codes {
            state = match code {
                Output::Escape(AnsiSequence::SetGraphicsMode(code)) => state.clone_from_scan(&code),
                _ => state,
            }
        }
        assert_eq!(state, expected_state);
        if let Output::Escape(AnsiSequence::SetGraphicsMode(code)) = reset {
            state = state.clone_from_scan(&code);
            assert_eq!(state, GraphicsModeState::default());
        }
    }

    #[test]
    fn rewriting_ansi_to_html() {
        let input = include_str!("test_data/input");
        let output = include_str!("test_data/output");
        assert_eq!(rewrite_ansi_to_html(input), output);
    }
}
