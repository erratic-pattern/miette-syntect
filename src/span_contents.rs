use miette::{SpanContents, SourceSpan};
use syntect::highlighting::Style;


/// A miette SpanContents highlighted via syntect
pub struct HighlightedSpanContents<'src> {
    pub(crate) span: SourceSpan,
    pub(crate) regions: Vec<(Style, &'src str)>
}

impl<'d, 'src> SpanContents<'d> for HighlightedSpanContents<'src> {
    fn column(&self) -> usize {
        todo!()
    }
    fn data(&self) -> &'d [u8] {
        todo!()
    }

    fn span(&self) -> &miette::SourceSpan {
        todo!()
    }

    fn line(&self) -> usize {
        todo!()
    }

    fn line_count(&self) -> usize {
        todo!()
    }
}