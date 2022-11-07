---
layout: post
title: Iterator Matching into Variable Sized Slices in Rust
date: 2022-11-05T04:00:00-05:00
---

For the rewrite of my blog engine, I had a thought: a lot of work has gone into
making programs print nice and pretty things into my terminal, but there's no
good way for me to get that output represented on a website in a convenient
format. I could take a screenshot, but that doesn't really feel _clean_ to me.
I could just take the raw output of the command and put that as text, but that
isn't pretty at all. I want a solution where I can take a command, run it in my
terminal, and have it able to be Just Showed Up on my site.

## How Terminals Display Colored Text

**Note:** If you only care about some funky Rust slice traversal, you can feel
free to skip this section.

Terminals use a system called [ANSI escape codes]; in particular, they're using
the "Select Graphics Rendition" code. This is going to be the primary point of
this article. I will do my best to summarize it in a very basic fashion but
it's worth checking out the Wikipedia and the `console_codes(4)` man page if
you want to learn more. It's a very interesting topic.

To start working on this, let's find a program that we can tell to output color
codes at any time. Given this post is going to be about Rust, I think `cargo`
is the most appropriate example. We can also use the program `xxd` to inspect
the data of the file and look at the hex representation of the color codes.

```sh
# Redirect stderr to stdout so we can capture the text to output.txt
cargo --color=always new --bin umbrella 2>&1 | sponge umbrella/output.txt
cat umbrella/output.txt
xxd umbrella/output.txt
```

<opaque-ansi-output source="output.txt" relative></opaque-ansi-output>

We can see that the first character to be included is a `'\u{1b}'` character,
followed by a `'['`. The first character, hereafter referred to as the "escape"
character, is present for almost every ANSI escape code. However, the second
character is only present for what are called "Control Sequences" (CS or CSI).
Most of the operations done on your terminal will be through CSI codes. This
includes things like moving around your cursor, clearing the screen, and
changing properties for text such as boldness, underline, and color.

The item after the CSI component, for the first element a `"0"`, is a parameter
for the CSI sequence. Parameters are usually a `u8` but there is no reason a
parameter couldn't be a `u16` or something larger. For the purpose of
formatting the output of a command, we only care about the 8-bit values. In
this case, the value 0 actually represents a "reset". From there, we can see
that it moves to a `1` (bold), a 32 (green text), and then the text "Created".
It then resets the terminal to its standard settings and continues writing the
rest of the output.

The last item in this sequence is the character `'m'`. This defines the type of
the CSI code to be a "Set Graphic Rendition" (SGR) code. Because the type of
the CSI sequence is defined at the end of the parameter list, it is therefore
required to parse an _entire_ parameter list before determining what CSI
sequence we're in. This has lead to poor optimizations, where CSI sequences
that only require a couple parameters could be fed up to 16 (according to the
`console_codes(4)` manual page).

This is not a good design, but it's a design that has existed for quite a long
time and will probably exist for a lot longer. Parsing this sequence into a
valid type is worth an article itself, but for now I will mention that I've
[forked the `ansi-parser` crate][ansi-parser] and will be using that later in
the article. I can pass it a string and it will give me a list of either `Text`
or `AnsiSequence`.

## Moving to HTML

Console codes are interesting because they can be arbitrarily turned on or off
without managing state between them. However, while some browsers could
_probably_ support this kind of shenanigan, for this use case we will try to
write valid HTML. This means that we should collect all graphics settings at
once, then write the block of text formatted using those graphics settings,
then - while continuing to use those graphics settings - add or remove some
additional settings and get ready to write the next chunk.

This can be easily done in practice by maintaining a `GraphicsModeState` and
updating it when reaching an `AnsiSequence`, or outputting HTML tags for that
state when reaching a `Text`.

```rust
use ansi_parser::{AnsiParser, AnsiSequence, Output};

#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub(crate) enum SgrColor {
    #[default]
    Reset,
    Console(u8),
    ExpandedConsole(u8),
    True(u8, u8, u8),
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

    color: SgrColor,
    background_color: SgrColor,
}

fn main() {
    let ansi_text = std::fs::read_to_string("output.txt");

    let mut state = GraphicsModeState::default();

    for block in ansi_text.ansi_parse().into_iter() {
        match block {
            Output::Escape(AnsiSequence::SetGraphicsMode(mode)) => {
                state = state.clone_from_scan(&mode[..]);
            }
            Output::TextBlock(text) => {
                let (opening_tags, closing_tags) = state.build_tags();
                let text = html_escape::encode_text(text);
                print!("{opening_tags}{text}{closing_tags}");
            }
            _ => {} // Other modes are irrelevant
        }
    }
}
```

We have the fundamental requirements of our software laid out, but we are
missing a few functions:

* `GraphicsModeState::clone_from_scan(&self, &[u8]) -> GraphicsModeState`
* `GraphicsModeState::build_tags(&self) -> (String, String)`

We're going to tackle the less difficult of these first.

## Generating HTML Tags

```rust
static COLORS: [&'static str; 8] = [
    "black", "red", "green", "yellow", "blue", "purple", "cyan", "gray",
];

impl GraphicsModeState {
    fn build_tags(&self) -> (String, String) {
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
}
```

This assumes that you have a CSS stylesheet that has the relevant variables for
the terminal colors. This is left as an exercise to the reader.

The bulk of the function is relatively simple: we're generating HTML opening
and closing tags for each possible option in the `GraphicsModeState`, then
returning, first, the concatenated tags; second, the concatenated reversed list
of tags. This way, they're closed in a "last tag created, first tag closed"
fashion like HTML expects.

## The `AnsiSequence::SetGraphicsMode` Variant

Back when things were simpler, terminals only had access to a total of 32
colors, if you counted 8 normal colors for the foreground, 8 bright colors for
the foreground, and 16 colors of a similar fashion for the background. In
practice, this requires 17 codes: `30-37` were reserved for the foreground,
`40-47` were reserved for the background, and `1` would sometimes be
interpreted as "increase frequency". This means that for every possible
operation that could be done to change the state, you would only need one
parameter.

We could start implementing this now, but it would be futile, as eventually
graphics cards would implement a 256-color lookup table. This is a lot more
colors than could be adequately represented by a `u8`, therefore requiring the
creation of a parameter that would designate the next parameter as the 8bit
color value. This now means that, if we wanted to simply `match` over any
valid sequence, we would now need to match two values. However, in the interest
of future-proofing the system, they also made the value require an additional
parameter to designate that it was from the 256 color set. At this point, we
now have 3 values.

However, it gets worse: as the rise of 24 bit color computing arose, eventually
the demand for 24 bit colors in the terminal grew as well. With 24 bits, and
parameters only accepting 8 bits, we are now stuck with *five* parameters. One
to designate a color sequence, one to designate it's 24bit color, and the three
other parameters to represent red, green, and blue.

## Pattern Matching Slices

Since `cargo` gives us one operation per CSI sequence, we can actually match
over this pretty easily:

```rust
impl GraphicsModeState {
    // omitted: fn build_tags()

    fn clone_from_scan(&self, input: &[u8]) -> Self {
        let mut state = self.clone();

        match input {
            [0] => state = GraphicsMode::default(),
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
            _ => (),
        }

        state
    }
}
```

We can actually run this code now and see that it does give a valid output:

```
% cargo add --git https://github.com/RyanSquared/ansi-parser-rs
% cargo add --git html_escape
% cargo run
   Compiling umbrella v0.1.0 (/home/ryan/builds/enigma/projects/umbrella)
    Finished dev [unoptimized + debuginfo] target(s) in 0.18s
     Running `target/debug/umbrella`
<strong><span style="color: var(--color-green)">     Created</span></strong> binary (application) `umbrella` package
```

However, as previously mentioned, we must be able to take multiple sets of
parameters. What happens if someone wants to reset the terminal, but also apply
a color at the same time? Currently, we'll get a color reset, but that's it.
This example command will run properly in our terminal:

```sh
echo -e '\e[0;32mHello\e[0;46mWorld\e[0m' | tee output.txt
```

<opaque-ansi-output source="output-echo-multiple-chained.txt" relative></opaque-ansi-output>

But if we run this through our program, at its current stage, we get this:

<opaque-ansi-output source="output-echo-multiple-chained-fail.txt" relative></opaque-ansi-output>

```html
<span style="color: var(--color-green)">Hello</span>World
```

This is due to the fact we're only matching over one value in `input`.  In this
case, we are only detecting the reset character, not the character after it.
Luckily, we can iterate through the slice, deciding to take extra parameters if
needed by using the [`Iterator::next_chunk()`] method:

```rust
let iter = input.iter();
while let Some(code) = iter.next() {
    match code {
        0 => state = GraphicsModeState::default(),
        1 => state.bold = true,
        3 => state.italic = true,
        4 => state.underline = true,
        9 => state.strikethrough = true,
        n @ 30..=37 => state.color = SgrColor::Console(n - 30),
        n @ 40..=47 => state.background_color = SgrColor::Console(n - 40),
        38 => {
            if let Ok([_, n]) = iter.next_chunk() {
                //                   ^^^^^^^^^^
                state.color = SgrColor::ExpandedConsole(*n);
            }
        }
    }
}
```

... Wait, what's that red squiggly line in my editor?

<opaque-ansi-output source="next-chunk-error.txt" relative></opaque-ansi-output>

Ah. I guess not.

That's fine. I'll just... Loop over the slice and increment it automagically
using a macro over the `match` block, until `next_chunk` is stable. I hope that
it comes sooner than the next CSI code comes out and I have to look at this
code again.

```rust
macro_rules! iter_over {
    ($input:expr; $([$($t:pat_param),*] => $s:expr,)+) => {
        let mut input = $input;
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
    // omitted: fn build_tags()

    fn clone_from_scan(&self, input: &[u8]) -> Self {
        let mut state = self.clone();

        iter_over! {
            &input[..];
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
}
```

This code looks _miraculously_ like the code I wrote earlier. By design :)

When expanded, the code looks like this:

```rust
loop {
    input = match input {
        [0, input @ ..] => {
            state = GraphicsModeState::default();
            input
        }
        [1, input @ ..] => {
            state.bold = true;
            input
        }
        [3, input @ ..] => {
            state.italic = true;
            input
        }
        [4, input @ ..] => {
            state.underline = true;
            input
        }
        [9, input @ ..] => {
            state.strikethrough = true;
            input
        }
        [n @ 30..=37, input @ ..] => {
            state.color = SgrColor::Console(n - 30);
            input
        }
        [n @ 40..=47, input @ ..] => {
            state.background_color = SgrColor::Console(n - 40);
            input
        }
        [38, 5, n, input @ ..] => {
            state.color = SgrColor::ExpandedConsole(*n);
            input
        }
        [48, 5, n, input @ ..] => {
            state.background_color = SgrColor::ExpandedConsole(*n);
            input
        }
        [38, 2, r, g, b, input @ ..] => {
            state.color = SgrColor::True(*r, *g, *b);
            input
        }
        [48, 2, r, g, b, input @ ..] => {
            state.background_color = SgrColor::True(*r, *g, *b);
            input
        }
        [39, input @ ..] => {
            state.color = SgrColor::Reset;
            input
        }
        [49, input @ ..] => {
            state.background_color = SgrColor::Reset;
            input
        }
        [_, input @ ..] => input,
        [] => break,
    };
}
```

The significance of this code is, the input is _automatically_ incremented.
There is no possible case where an infinite loop happens, and there's always an
exit code. For future proofing, I've even added an option to skip over codes
that I don't know about.

We can run this code again and see that it now correctly formats the output:

<opaque-ansi-output source="output-echo-multiple-chained.txt" relative></opaque-ansi-output>

```html
<span style="color: var(--color-green)">Hello</span><span style="background-color: var(--color-cyan)">World</span>
```

I hope that `iter_over!` isn't useful for that long, and that `next_chunk()`
gets stabilized soon, but until then, I think it's a pretty nifty macro.

I have included the source of this example in the blog repository. If you'd
like to run the example yourself, you can do so. It should be as simple as
`cargo run`.

[ANSI escape codes]: https://en.wikipedia.org/wiki/ANSI_escape_code
[ansi-parser]: https://github.com/RyanSquared/ansi-parser-rs
[`Iterator::next_chunk()`]: https://doc.rust-lang.org/std/iter/trait.Iterator.html#method.next_chunk
