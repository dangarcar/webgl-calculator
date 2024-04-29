use std::{iter::once, ops::Range};

use tex_parser::ast::Token;

use crate::error;

#[derive(Debug)]
pub(super) struct Term {
    pub range: Range<usize>,
    pub subtract: bool,
}

/// Returns the groups in reverse order
pub(super) fn get_terms(tokens: &[Token]) -> error::Result<Vec<Term>> {
    Ok(tokens.iter().enumerate()
        .filter_map(|(i, e)| {
            match e.punctuation() {
                Some(p) => if p.ch == '+' || p.ch == '-' { Some((i as i64, p.ch)) } else { None },
                None => None
            }
        }) //Get the plus and minus
        .rev()
        .chain(once((-1, '+'))) //Suppose an initial +
        .scan(tokens.len() as i64, |last, (i, ch)| {
            let end = *last;
            *last = i;
            Some(Term {
                range: (i+1) as usize .. end as usize,
                subtract: ch == '-'
            })
        })
        .filter(|e| !e.range.is_empty())
        .collect()
    )
}