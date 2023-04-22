use std::borrow::Cow;

use miette::{MietteSpanContents, SourceCode, SourceSpan};

use syntect::{
    highlighting::{HighlightIterator, HighlightState, Highlighter, Style, Theme},
    parsing::{ParseState, ScopeStack, SyntaxReference, SyntaxSet},
};

#[derive(Debug)]
pub struct HighlightedSource<'s> {
    // reference to incoming source
    source: &'s str,
    // output source with or without highlight
    data: Cow<'s, str>,
}

impl<'s> HighlightedSource<'s> {
    fn no_color(source: &'s str) -> Self {
        Self {
            source,
            data: Cow::Borrowed(source),
        }
    }
    fn with_color(
        source: &'s str,
        syntax_set: &SyntaxSet,
        syntax_ref: &SyntaxReference,
        theme: &Theme,
        use_bg_color: bool,
    ) -> Self {
        let highlighter = Highlighter::new(theme);
        // parse source and highlight span contents
        let mut highlight_state = HighlightState::new(&highlighter, ScopeStack::new());
        let mut parse_state = ParseState::new(syntax_ref);
        let mut data = source
            .lines()
            .map(|line| {
                let ops = parse_state.parse_line(line, syntax_set).unwrap();

                let regions: Vec<(Style, &str)> =
                    HighlightIterator::new(&mut highlight_state, &ops, line, &highlighter)
                        .collect();
                syntect::util::as_24_bit_terminal_escaped(&regions, use_bg_color)
            })
            .collect::<Vec<String>>()
            .join("\n");
        // clear colors at end
        data.push_str("\x1b[0m");
        Self {
            source,
            data: Cow::Owned(data),
        }
    }

    // fn use_color(&self) -> bool {
    //     self.force_color.unwrap_or_else(|| {
    //         env::var_os("NO_COLOR") != Some("0".into())
    //             && env::var_os("NO_GRAPHICS") != Some("0".into())
    //     })
    // }

    // fn use_bg_color(&self) -> bool {
    //     self.use_bg_color.unwrap_or(false)
    // }
}

impl<'s> SourceCode for HighlightedSource<'s> {
    fn read_span<'a>(
        &'a self,
        span: &SourceSpan,
        context_lines_before: usize,
        context_lines_after: usize,
    ) -> Result<Box<dyn miette::SpanContents<'a> + 'a>, miette::MietteError> {
        let span_contents =
            self.source
                .read_span(span, context_lines_before, context_lines_after)?;
        Ok(Box::new(MietteSpanContents::new(
            self.data.as_bytes(),
            *span_contents.span(),
            span_contents.line(),
            span_contents.column(),
            span_contents.line_count(),
        )))
    }
}

// TODO: HighlightedSourceFile with syntax set handling, name handling, and buffered reading
// impl<'s> HighlightedSource<'s>  {
//     pub fn from_file<P: AsRef<Path>>(path: P, syntax_set: &'syn SyntaxSet, theme: &'syn Theme) -> Result<Self, io::Error> {
//         let path = path.as_ref();
//         let f = File::open(path)?;
//         let syntax = syntax_set.find_syntax_for_file(path)?
//             .unwrap_or_else(|| syntax_set.find_syntax_plain_text());
//         Ok(Self::new(f, syntax_set, syntax, theme).with_name(path.to_string_lossy()))
//     }
// }

#[cfg(test)]
mod test {

    use miette::{Diagnostic, MietteHandlerOpts, Report, SourceOffset, SourceSpan};
    use syntect::{highlighting::ThemeSet, parsing::SyntaxSet};
    use thiserror::Error;

    use crate::HighlightedSource;

    #[derive(Error, Debug, Diagnostic)]
    #[error("test error")]
    #[diagnostic()]
    struct BasicReport<'s> {
        #[source_code]
        src: HighlightedSource<'s>,
        #[label("test label")]
        span: SourceSpan,
    }

    #[test]
    fn basic_report_rendering() {
        miette::set_hook(Box::new(|_| {
            Box::new(
                MietteHandlerOpts::new()
                    .force_graphical(true)
                    .context_lines(0)
                    .build(),
            )
        }))
        .unwrap();
        let code = r#"
            fn main() {
                println!(\"Hello, World!\");
            }
        "#;
        let ss = SyntaxSet::load_defaults_newlines();
        let sr = ss.find_syntax_by_extension("rs").unwrap();
        let ts = ThemeSet::load_defaults();
        let theme = ts.themes.values().next().unwrap();
        let loc = SourceOffset::from_location(code, 2, 0);
        let report = BasicReport {
            src: HighlightedSource::with_color(code, &ss, sr, theme, false),
            span: SourceSpan::new(loc, SourceOffset::from(loc.offset() + 10)),
        };
        println!("{:?}", Report::new(report))

        // println!("{:?}", report);
    }
}
