use std::collections::VecDeque;

use miette::{MietteError, MietteSpanContents, SourceSpan};

/// A function to find the context info for a report while ignoring ANSI escapes.
/// Adapted from miette's implementation here: https://github.com/zkat/miette/blob/2b4d67d7cd1ced4f0e81e74a0c636a8cfcf2074e/src/source_impls.rs#L13
/// We only check for graphics codes of the form CSI [ n;...m
pub(crate) fn context_info<'a>(
    input: &'a [u8],
    span: &SourceSpan,
    context_lines_before: usize,
    context_lines_after: usize,
) -> Result<MietteSpanContents<'a>, MietteError> {
    let mut offset = 0usize;
    let mut offset_no_ansi = 0usize; // offset, ignoring ansi codes
    let mut line_count = 0usize;
    let mut start_line = 0usize;
    let mut start_column = 0usize;
    let mut before_lines_starts = VecDeque::new();
    let mut current_line_start = 0usize;
    let mut end_lines = 0usize;
    let mut post_span = false;
    let mut post_span_got_newline = false;
    let mut iter = input.iter().copied().peekable();
    while let Some(char) = iter.next() {
        // ANSI escape
        if matches!(char, b'\x1B') {
            let mut ansi_lookahead = iter.clone().enumerate();
            // lookahead for CSI sequence
            if let Some((_, b'[')) = ansi_lookahead.next() {
                // lookahead until we find m after sequence of digits and semicolons
                if let Some((ansi_count, b'm')) = ansi_lookahead
                    .skip_while(|(_, char)| char.is_ascii_digit() || *char == b';')
                    .next()
                {
                    // advance the loop through the ansi sequence
                    offset += ansi_count + 1;
                    for _ in 0..=ansi_count {
                        iter.next();
                    }
                    continue;
                }
            }
        } else if matches!(char, b'\r' | b'\n') {
            line_count += 1;
            if char == b'\r' && iter.next_if_eq(&b'\n').is_some() {
                offset += 1;
                offset_no_ansi += 1;
            }
            if offset_no_ansi < span.offset() {
                // We're before the start of the span.
                start_column = 0;
                before_lines_starts.push_back(current_line_start);
                if before_lines_starts.len() > context_lines_before {
                    start_line += 1;
                    before_lines_starts.pop_front();
                }
            } else if offset_no_ansi >= span.offset() + span.len().saturating_sub(1) {
                // We're after the end of the span, but haven't necessarily
                // started collecting end lines yet (we might still be
                // collecting context lines).
                if post_span {
                    start_column = 0;
                    if post_span_got_newline {
                        end_lines += 1;
                    } else {
                        post_span_got_newline = true;
                    }
                    if end_lines >= context_lines_after {
                        offset += 1;
                        offset_no_ansi += 1;
                        break;
                    }
                }
            }
            current_line_start = offset + 1;
        } else if offset_no_ansi < span.offset() {
            start_column += 1;
        }

        if offset_no_ansi >= (span.offset() + span.len()).saturating_sub(1) {
            post_span = true;
            if end_lines >= context_lines_after {
                offset += 1;
                offset_no_ansi += 1;
                break;
            }
        }

        offset += 1;
        offset_no_ansi += 1;
    }

    println!(
        "len: {}, offset: {} no_ansi: {}, current_line_start: {}, start_column: {}",
        input.len(),
        offset,
        offset_no_ansi,
        current_line_start,
        start_column
    );

    if offset_no_ansi >= (span.offset() + span.len()).saturating_sub(1) {
        let starting_offset = before_lines_starts.front().copied().unwrap_or_else(|| {
            if context_lines_before == 0 {
                span.offset()
            } else {
                0
            }
        });
        Ok(MietteSpanContents::new(
            &input[starting_offset..offset],
            (starting_offset, offset - starting_offset).into(),
            start_line,
            if context_lines_before == 0 {
                start_column
            } else {
                0
            },
            line_count,
        ))
    } else {
        Err(MietteError::OutOfBounds)
    }
}
