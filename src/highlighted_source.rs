use miette::{SourceCode, SourceSpan};
use syntect::{parsing::{SyntaxReference, SyntaxSet, ParseState, ScopeStack}, highlighting::{Theme, Highlighter, HighlightIterator, HighlightState}};
use crate::HighlightedSpanContents;

pub struct HighlightedSource<'src, 'ss, 't> {
    highlighter: Highlighter<'t>,
    syntax_set: &'ss SyntaxSet,
    syntax_ref: &'ss SyntaxReference,
    source: &'src str,
    use_bg_color: bool,
}

impl<'src, 'ss, 't> HighlightedSource<'src, 'ss, 't>  {
    pub fn new(source: &'src str, syntax_set: &'ss SyntaxSet, syntax_ref: &'ss SyntaxReference, theme: &'t Theme, ) -> Self {
        Self { source, highlighter: Highlighter::new(theme), syntax_set, syntax_ref, use_bg_color: true }
    }

    pub fn use_bg_color(mut self, flag: bool) -> Self {
        self.use_bg_color = flag;
        self
    }
}

impl<'src, 'ss, 't>  SourceCode for HighlightedSource<'src, 'ss, 't>  {
    fn read_span<'a>(
            &'a self,
            span: &SourceSpan,
            _context_lines_before: usize,
            _context_lines_after: usize,
        ) -> Result<Box<dyn miette::SpanContents<'a> + 'a>, miette::MietteError> {
            let mut highlight_state = HighlightState::new(&self.highlighter, ScopeStack::new());
            let mut parse_state = ParseState::new(self.syntax_ref);
            let mut regions = Vec::new();
            for line in self.source.lines() {
                    let ops = parse_state.parse_line(line, self.syntax_set).unwrap();
                    // TODO: only highlight code in the region
                    let iter =
                        HighlightIterator::new(&mut highlight_state, &ops[..], line, &self.highlighter);
                    regions.extend(iter);
            };

            Ok(Box::new(HighlightedSpanContents { regions, span: *span }))
    }
}

// TODO: HighlightedSourceFile with syntax set handling, name handling, and buffered reading
// impl<'src, 'ss, 't> HighlightedSource<'src, 'ss, 't>  {
//     pub fn from_file<P: AsRef<Path>>(path: P, syntax_set: &'ss SyntaxSet, theme: &'t Theme) -> Result<Self, io::Error> {
//         let path = path.as_ref();
//         let f = File::open(path)?;
//         let syntax = syntax_set.find_syntax_for_file(path)?
//             .unwrap_or_else(|| syntax_set.find_syntax_plain_text());
//         Ok(Self::new(f, syntax_set, syntax, theme).with_name(path.to_string_lossy()))
//     }
// }