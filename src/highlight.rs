/// Core syntax highlighting functions
use syntect::{
    highlighting::{HighlightIterator, HighlightState, Highlighter, Style, Theme},
    parsing::{ParseState, ScopeStack, SyntaxReference, SyntaxSet},
};

pub(crate) fn highlight_line<'a>(
    line: &'a str,
    syntax_set: &SyntaxSet,
    highlighter: &Highlighter,
    parse_state: &mut ParseState,
    highlight_state: &mut HighlightState,
    use_bg_color: bool,
) -> String {
    let ops = parse_state.parse_line(line, syntax_set).unwrap();
    let regions: Vec<(Style, &str)> =
        HighlightIterator::new(highlight_state, &ops, line, highlighter).collect();
    let mut line = syntect::util::as_24_bit_terminal_escaped(&regions, use_bg_color);
    line.push_str("\x1b[0m");
    line
}

pub(crate) fn highlight_lines<'a>(
    source: &'a str,
    syntax_set: &'a SyntaxSet,
    syntax: &'a SyntaxReference,
    theme: &'a Theme,
    use_bg_color: bool,
) -> impl Iterator<Item = String> + 'a {
    let highlighter = Highlighter::new(theme);
    let mut parse_state = ParseState::new(syntax);
    let mut highlight_state = HighlightState::new(&highlighter, ScopeStack::new());
    // parse source and highlight span contents
    source.lines().map(move |line| {
        highlight_line(
            line,
            syntax_set,
            &highlighter,
            &mut parse_state,
            &mut highlight_state,
            use_bg_color,
        )
    })
}
