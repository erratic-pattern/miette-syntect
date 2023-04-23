use std::{borrow::Cow, fs::File, io::Read, path::Path};

use miette::{MietteSpanContents, SourceCode, SourceSpan};

use syntect::{
    highlighting::{Theme, ThemeSet},
    parsing::{Scope, SyntaxReference, SyntaxSet},
};

use crate::{highlight::highlight_lines, source_context::context_info};

// trait object for syntects family of find_syntax_* functions
type SyntaxFinder = Box<dyn for<'a> FnOnce(&'a SyntaxSet) -> Option<&'a SyntaxReference>>;

pub struct HighlightedSourceBuilder<'s, 'ss, 't> {
    source: Cow<'s, str>,
    syntax_set: Option<Cow<'ss, SyntaxSet>>,
    find_syntax_fn: Option<SyntaxFinder>,
    theme: Option<Cow<'t, Theme>>,
    force_color: Option<bool>,
    use_bg_color: Option<bool>,
    name: Option<String>,
}

impl<'s, 'ss, 't> HighlightedSourceBuilder<'s, 'ss, 't> {
    pub fn from_string<S: Into<Cow<'s, str>>>(source: S) -> Self {
        let source = source.into();
        Self {
            source,
            syntax_set: None,
            find_syntax_fn: None,
            theme: None,
            force_color: None,
            use_bg_color: None,
            name: None,
        }
    }

    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, std::io::Error> {
        let path = path.as_ref();
        let mut source = String::new();
        let mut file = File::open(path)?;
        file.read_to_string(&mut source)?;
        let mut builder = Self::from_string(Cow::Owned(source));
        if let Some(path) = path.to_str() {
            builder.name = Some(path.to_string());
            builder.find_syntax_fn = Some(Box::new({
                let name = path.to_string();
                move |ss| ss.find_syntax_for_file(name).ok().flatten()
            }));
        }

        Ok(builder)
    }

    pub fn syntax_set<SS: Into<Cow<'ss, SyntaxSet>>>(mut self, syntax_set: SS) -> Self {
        self.syntax_set = Some(syntax_set.into());
        self
    }

    pub fn theme<T: Into<Cow<'t, Theme>>>(mut self, theme: T) -> Self {
        self.theme = Some(theme.into());
        self
    }

    pub fn name<N: Into<String>>(mut self, name: N) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn force_color(mut self, flag: bool) -> Self {
        self.force_color = Some(flag);
        self
    }

    pub fn use_bg_color(mut self, flag: bool) -> Self {
        self.use_bg_color = Some(flag);
        self
    }

    pub fn default_syntax_set_newlines(mut self) -> Self {
        self.syntax_set = Some(Cow::Owned(SyntaxSet::load_defaults_newlines()));
        self
    }

    pub fn default_syntax_set_nonewlines(mut self) -> Self {
        self.syntax_set = Some(Cow::Owned(SyntaxSet::load_defaults_nonewlines()));
        self
    }

    pub fn find_syntax_by_extension(mut self, ext: &str) -> Self {
        let ext = ext.to_string();
        self.find_syntax_fn = Some(Box::new(move |ss| ss.find_syntax_by_extension(&ext)));
        self
    }

    pub fn find_syntax_by_name(mut self, name: &str) -> Self {
        let name = name.to_string();
        self.find_syntax_fn = Some(Box::new(move |ss| ss.find_syntax_by_name(&name)));
        self
    }

    pub fn find_syntax_by_scope(mut self, scope: Scope) -> Self {
        self.find_syntax_fn = Some(Box::new(move |ss| ss.find_syntax_by_scope(scope)));
        self
    }

    pub fn build(self) -> HighlightedSource {
        let ss = self
            .syntax_set
            .unwrap_or_else(|| Cow::Owned(SyntaxSet::load_defaults_newlines()));
        let find_syntax_fn = self
            .find_syntax_fn
            .unwrap_or_else(|| Box::new(|ss| ss.find_syntax_by_first_line(&self.source)));
        let theme = self.theme.unwrap_or_else(|| {
            Cow::Owned(
                ThemeSet::load_defaults()
                    .themes
                    .into_values()
                    .next()
                    .unwrap(),
            )
        });
        let syntax = find_syntax_fn(&ss).unwrap_or_else(|| ss.find_syntax_plain_text());
        let use_bg_color = self.use_bg_color.unwrap_or(false);
        HighlightedSource::new(&self.source, &ss, syntax, &theme, use_bg_color, self.name)
    }
}

#[derive(Debug)]
pub struct HighlightedSource {
    // reference to incoming source
    highlighted_source: String,
    name: Option<String>,
}

impl HighlightedSource {
    fn new(
        source: &str,
        syntax_set: &SyntaxSet,
        syntax_ref: &SyntaxReference,
        theme: &Theme,
        use_bg_color: bool,
        name: Option<String>,
    ) -> Self {
        let mut highlighted_source =
            highlight_lines(source, syntax_set, syntax_ref, theme, use_bg_color)
                .collect::<Vec<String>>()
                .join("\n");
        // parse source and highlight span contents
        Self {
            highlighted_source,
            name,
        }
    }
}

impl SourceCode for HighlightedSource {
    fn read_span<'a>(
        &'a self,
        span: &SourceSpan,
        context_lines_before: usize,
        context_lines_after: usize,
    ) -> Result<Box<dyn miette::SpanContents<'a> + 'a>, miette::MietteError> {
        let span_contents = context_info(
            self.highlighted_source.as_bytes(),
            span,
            context_lines_before,
            context_lines_after,
        )?;
        Ok(Box::new(span_contents))
    }
}

#[cfg(test)]
mod test {

    use miette::{Diagnostic, MietteHandlerOpts, Report, SourceOffset, SourceSpan};

    use thiserror::Error;

    use crate::{highlighted_source::HighlightedSourceBuilder, HighlightedSource};

    #[derive(Error, Debug, Diagnostic)]
    #[error("test error")]
    #[diagnostic()]
    struct BasicReport {
        #[source_code]
        src: HighlightedSource,
        #[label("test label")]
        span: SourceSpan,
    }

    #[test]
    fn default_report_zero_context_lines() {
        miette::set_hook(Box::new(|_| {
            Box::new(
                MietteHandlerOpts::new()
                    .force_graphical(true)
                    .context_lines(1)
                    .build(),
            )
        }))
        .unwrap();
        let code = r#"fn main() {
    println!("Hello, World!");
}"#;
        let src = HighlightedSourceBuilder::from_string(code)
            .find_syntax_by_extension("rs")
            .build();

        let loc = SourceOffset::from_location(code, 3, 4);
        println!("{:?}", loc);
        let span = (loc, 4.into()).into();
        let report = BasicReport { src, span };
        println!("{:?}", Report::new(report))

        // println!("{:?}", report);
    }
}
