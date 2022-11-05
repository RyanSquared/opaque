use ansi_parser::{AnsiParser, AnsiSequence, Output};

#[cfg(feature = "tracing")]
use tracing::debug;

#[derive(Default, Debug, Clone, PartialEq, Eq)]
enum SgrColor {
    #[default]
    Reset,
    Console(u8),
    ExpandedConsole(u8),
    True(u8, u8, u8),
}

#[derive(Default, Debug, Clone, PartialEq, Eq)]
enum StatePath {
    #[default]
    NextMode,
    // Determines whether or not it's Expanded or True
    ForegroundMatch,
    BackgroundMatch,
    // 5, n
    ForegroundExpanded,
    // 2, r, g, b
    ForegroundTrueOne,
    ForegroundTrueTwo,
    ForegroundTrueThree,
    // 5, n
    BackgroundExpanded,
    // 2, r, g, b
    BackgroundTrueOne,
    BackgroundTrueTwo,
    BackgroundTrueThree,
}

#[derive(Default, Debug, Clone, PartialEq, Eq)]
struct GraphicsModeState {
    // reset is interpreted as resetting everything to default
    // the methods defined here are taken from:
    // https://en.wikipedia.org/wiki/ANSI_escape_code#SGR_(Select_Graphic_Rendition)_parameters
    // and have been selected in accordance with their compatibility with static HTML
    bold: bool,
    italic: bool,
    underline: bool,
    strikethrough: bool,

    // SgrColor is a tagged enum, match that when in Foreground or Background color
    color: SgrColor,
    background_color: SgrColor,

    path: StatePath,
}

static COLORS: [&'static str; 8] = [
    "black", "red", "green", "yellow", "blue", "purple", "cyan", "gray",
];

impl GraphicsModeState {
    fn get_tags(&self) -> (String, String) {
        if self == &Self::default() {
            return ("".to_string(), "".to_string());
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
                closing_tags.push("</span>".to_string())
            }
            SgrColor::ExpandedConsole(n) => {
                let span = format!("<span style=\"color: var(--terminal-color-{})\">", n);
                opening_tags.push(span);
                closing_tags.push("</span>".to_string())
            }
            SgrColor::True(r, g, b) => {
                let span = format!("<span style=\"color: rgb({r}, {g}, {b})\">");
                opening_tags.push(span);
                closing_tags.push("</span>".to_string())
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
                closing_tags.push("</span>".to_string())
            }
            SgrColor::ExpandedConsole(n) => {
                let span = format!("<span style=\"background-color: var(--terminal-color-{n})\">");
                opening_tags.push(span);
                closing_tags.push("</span>".to_string())
            }
            SgrColor::True(r, g, b) => {
                let span = format!("<span style=\"background-color: rgb({r}, {g}, {b})\">");
                opening_tags.push(span);
                closing_tags.push("</span>".to_string())
            }
            _ => (),
        }

        (
            opening_tags.join(""),
            closing_tags.into_iter().rev().collect::<Vec<_>>().join(""),
        )
    }

    fn get_next_state(self, graphics_mode: ansi_parser::Vec<u8, 5>) -> Self {
        graphics_mode.iter().fold(self, |state, new_mode| {
            match state.path {
                StatePath::ForegroundMatch => match new_mode {
                    5 => GraphicsModeState {
                        path: StatePath::ForegroundExpanded,
                        ..state
                    },
                    2 => GraphicsModeState {
                        path: StatePath::ForegroundTrueOne,
                        ..state
                    },
                    _ => state,
                },
                StatePath::BackgroundMatch => match new_mode {
                    5 => GraphicsModeState {
                        path: StatePath::BackgroundExpanded,
                        ..state
                    },
                    2 => GraphicsModeState {
                        path: StatePath::BackgroundTrueOne,
                        ..state
                    },
                    _ => state,
                },
                StatePath::ForegroundExpanded => {
                    return GraphicsModeState {
                        path: StatePath::NextMode,
                        color: SgrColor::ExpandedConsole(*new_mode),
                        ..state
                    }
                }
                StatePath::BackgroundExpanded => GraphicsModeState {
                    path: StatePath::NextMode,
                    background_color: SgrColor::ExpandedConsole(*new_mode),
                    ..state
                },
                StatePath::ForegroundTrueOne => GraphicsModeState {
                    path: StatePath::ForegroundTrueTwo,
                    color: SgrColor::True(*new_mode, 0, 0),
                    ..state
                },
                StatePath::ForegroundTrueTwo => {
                    if let SgrColor::True(r, _, b) = state.color {
                        return GraphicsModeState {
                            path: StatePath::ForegroundTrueThree,
                            color: SgrColor::True(r, *new_mode, b),
                            ..state
                        };
                    }
                    panic!("reached a ForegroundTrue state without an SgrColor");
                }
                StatePath::ForegroundTrueThree => {
                    if let SgrColor::True(r, g, _) = state.color {
                        return GraphicsModeState {
                            path: StatePath::NextMode,
                            color: SgrColor::True(r, g, *new_mode),
                            ..state
                        };
                    }
                    panic!("reached a ForegroundTrue state without an SgrColor");
                }
                StatePath::BackgroundTrueOne => GraphicsModeState {
                    path: StatePath::BackgroundTrueTwo,
                    background_color: SgrColor::True(*new_mode, 0, 0),
                    ..state
                },
                StatePath::BackgroundTrueTwo => {
                    if let SgrColor::True(r, _, b) = state.background_color {
                        return GraphicsModeState {
                            path: StatePath::BackgroundTrueThree,
                            background_color: SgrColor::True(r, *new_mode, b),
                            ..state
                        };
                    };
                    panic!("reached a BackgroundTrue state without an SgrColor");
                }
                StatePath::BackgroundTrueThree => {
                    if let SgrColor::True(r, g, _) = state.background_color {
                        return GraphicsModeState {
                            path: StatePath::NextMode,
                            background_color: SgrColor::True(r, g, *new_mode),
                            ..state
                        };
                    }
                    panic!("reached a BackgroundTrue state without an SgrColor");
                }
                StatePath::NextMode => {
                    // Parse the next color like normal
                    match new_mode {
                        // Instead of using GraphicsModeState::default(), this can be
                        // picked up by the optimizer to reuse the existing, soon to be
                        // dropped, GraphicsModeState
                        0 => GraphicsModeState {
                            ..Default::default()
                        },
                        1 => GraphicsModeState {
                            bold: true,
                            ..state
                        },
                        3 => GraphicsModeState {
                            italic: true,
                            ..state
                        },
                        4 => GraphicsModeState {
                            underline: true,
                            ..state
                        },
                        9 => GraphicsModeState {
                            strikethrough: true,
                            ..state
                        },
                        30..=37 => GraphicsModeState {
                            color: SgrColor::Console(new_mode - 30),
                            ..state
                        },
                        40..=47 => GraphicsModeState {
                            background_color: SgrColor::Console(new_mode - 40),
                            ..state
                        },
                        38 => GraphicsModeState {
                            path: StatePath::ForegroundMatch,
                            ..state
                        },
                        48 => GraphicsModeState {
                            path: StatePath::BackgroundMatch,
                            ..state
                        },
                        39 => GraphicsModeState {
                            color: SgrColor::Reset,
                            ..state
                        },
                        49 => GraphicsModeState {
                            background_color: SgrColor::Reset,
                            ..state
                        },
                        _ => state,
                    }
                }
            }
        })
    }
}

#[cfg_attr(feature = "tracing", tracing::instrument(skip(input)))]
pub fn rewrite_ansi_to_html(input: &str) -> String {
    #[cfg(feature = "tracing")]
    debug!("parsing ANSI escape codes");

    let parsed: Vec<Output> = input
        .ansi_parse()
        .filter(|value| {
            // Trash all Escapes that aren't stylish
            match value {
                Output::Escape(AnsiSequence::SetGraphicsMode(_)) => true,
                Output::TextBlock(_) => true,
                _ => false,
            }
        })
        .collect();

    let mut state = GraphicsModeState::default();
    let mut output = vec![];

    output.push("<div class=\"code\"><pre><code>".to_string());

    #[cfg(feature = "tracing")]
    debug!("converting ANSI escape code and text chunks to HTML");

    for block in parsed.into_iter() {
        match block {
            Output::Escape(AnsiSequence::SetGraphicsMode(mode)) => {
                state = state.get_next_state(mode);
            }
            Output::TextBlock(text) => {
                let (opening_tags, closing_tags) = state.get_tags();
                let text = html_escape::encode_text(text);
                output.push(format!("{opening_tags}{text}{closing_tags}"));
            }
            _ => {}
        }
    }

    output.push("</code></pre></div>".to_string());
    output.join("")
}
