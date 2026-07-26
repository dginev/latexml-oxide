//! A forward cursor over `&str` whose position is always a char boundary.
//!
//! # Why this exists
//!
//! `&str` carries exactly one invariant the compiler enforces — *valid UTF-8*.
//! A `usize` used to index it carries **none**. So every `&s[a..b]` is an
//! unchecked assertion that both ends are char boundaries, and when the
//! assertion is wrong the program does not return an error, it **panics**.
//!
//! That combination is unusually hostile here. This is a Perl port, and Perl
//! strings are sequences of *characters*: `pos`, `substr` and `\G` have no
//! boundary concept at all. Every hand-rolled `as_bytes()` + `i += 1` scanner
//! translated from Perl therefore introduces an invariant the original never
//! had, silently, with no type-level trace — and passes every ASCII fixture
//! forever. Witness 2605.22125: a `.bib` title containing `\“` aborted the whole
//! document, and the code had been live for months.
//!
//! # The rule the bug teaches
//!
//! The panic happens at the slice, but the defect is always in the **advance**:
//!
//! | advance | safe? |
//! |---|---|
//! | scan to an ASCII delimiter | always — an ASCII byte is never a UTF-8 continuation byte |
//! | by `char::len_utf8()` | always |
//! | a fixed count past an unclassified byte | **never** |
//!
//! So the fix is not to check slices, it is to remove the ability to advance
//! wrongly. This cursor exposes no byte-count advance at all, which makes
//! [`slice_from`](crate::util::char_cursor::CharCursor::slice_from) infallible and the whole class
//! unrepresentable in code written against it.
//!
//! Rust guidelines `anti-index-over-iter` / `perf-iter-over-index`: prefer the
//! iterator std already provides (`char_indices`) over manual indexing. This is
//! a thin, self-documenting wrapper over exactly that — not a new abstraction.
//!
//! # Cost
//!
//! A `Peekable<CharIndices>` and the source reference; no allocation, one pass,
//! the same traversal a byte walker made. `char_indices` also decodes each
//! character once instead of re-decoding at every slice.

/// Forward cursor over a `&str`, positioned only ever at char boundaries.
///
/// # Examples
///
/// ```
/// use latexml_core::util::char_cursor::CharCursor;
///
/// // Take a run of ASCII letters, then whatever single character follows —
/// // even a 3-byte one. A byte walker would split it; this cannot.
/// let mut cur = CharCursor::new("word“tail");
/// let start = cur.pos();
/// cur.take_while(char::is_alphanumeric);
/// assert_eq!(cur.slice_from(start), "word");
/// assert_eq!(cur.next(), Some('“'));
/// ```
pub struct CharCursor<'a> {
  src:  &'a str,
  iter: std::iter::Peekable<std::str::CharIndices<'a>>,
  /// Byte offset of the next character, or `src.len()` at the end. Always a
  /// char boundary: it only ever comes from `CharIndices`.
  pos:  usize,
}

impl<'a> CharCursor<'a> {
  /// Start at the beginning of `src`.
  #[inline]
  pub fn new(src: &'a str) -> Self {
    Self {
      src,
      iter: src.char_indices().peekable(),
      pos: 0,
    }
  }

  /// Byte offset of the next character — a valid boundary, and the mark to
  /// hand to [`slice_from`](Self::slice_from).
  #[inline]
  pub fn pos(&self) -> usize { self.pos }

  /// The next character, without consuming it.
  #[inline]
  pub fn peek(&mut self) -> Option<char> { self.iter.peek().map(|&(_, c)| c) }

  /// The character *after* the next one, without consuming anything.
  ///
  /// This is the `i + 1 < len` lookahead a byte walker spells out by hand, with
  /// no arithmetic on indices — the arithmetic is where the bug lives.
  #[inline]
  pub fn peek_second(&mut self) -> Option<char> {
    let mut probe = self.iter.clone();
    probe.next();
    probe.next().map(|(_, c)| c)
  }

  /// Consume and return the next character, advancing by its full width.
  #[inline]
  #[allow(clippy::should_implement_trait)] // deliberately not `Iterator`: see below
  pub fn next(&mut self) -> Option<char> {
    let (i, c) = self.iter.next()?;
    self.pos = i + c.len_utf8();
    Some(c)
  }

  /// Consume characters while `pred` holds.
  #[inline]
  pub fn take_while(&mut self, mut pred: impl FnMut(char) -> bool) {
    while self.peek().is_some_and(&mut pred) {
      self.next();
    }
  }

  /// `true` once the input is exhausted.
  #[inline]
  pub fn is_done(&mut self) -> bool { self.peek().is_none() }

  /// The text between a previous [`pos`](Self::pos) mark and the current
  /// position.
  ///
  /// Infallible — both ends came from `CharIndices`, so both are char
  /// boundaries. That is the entire point of the type.
  ///
  /// # Panics
  ///
  /// Only if `mark` did not come from this cursor's [`pos`](Self::pos), or is
  /// ahead of the current position. Both are caller bugs, not input-dependent.
  #[inline]
  pub fn slice_from(&self, mark: usize) -> &'a str { &self.src[mark..self.pos] }

  /// The remaining, unconsumed text.
  #[inline]
  pub fn rest(&self) -> &'a str { &self.src[self.pos..] }
}

// NOTE: deliberately NOT an `Iterator` impl. `Iterator` would hand callers
// `by_ref().take_while(..)`, `zip`, `enumerate` and friends, all of which
// consume the item that fails the predicate — the cursor's whole job is that
// `peek`/`pos` stay in lockstep so a mark remains meaningful.

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn positions_are_always_char_boundaries() {
    // One representative per UTF-8 encoded width, interleaved with ASCII so
    // marks land on both sides of every multi-byte character.
    let src = "a é b “ c 𝔄 d";
    let mut cur = CharCursor::new(src);
    while !cur.is_done() {
      assert!(
        src.is_char_boundary(cur.pos()),
        "cursor stopped at byte {} which is not a boundary of {src:?}",
        cur.pos()
      );
      // The invariant that matters: slicing at any reachable position is safe.
      let _ = cur.slice_from(0);
      let _ = cur.rest();
      cur.next();
    }
    assert_eq!(cur.pos(), src.len());
  }

  #[test]
  fn slice_from_returns_exactly_the_consumed_text() {
    let mut cur = CharCursor::new("“quoted” rest");
    let start = cur.pos();
    cur.take_while(|c| c != ' ');
    assert_eq!(cur.slice_from(start), "“quoted”");
    assert_eq!(cur.rest(), " rest");
  }

  #[test]
  fn peek_does_not_advance_and_peek_second_looks_past_it() {
    let mut cur = CharCursor::new("𝔄b");
    assert_eq!(cur.peek(), Some('𝔄'));
    assert_eq!(cur.peek(), Some('𝔄'), "peek must not consume");
    assert_eq!(cur.pos(), 0, "peek must not advance");
    assert_eq!(cur.peek_second(), Some('b'));
    assert_eq!(cur.pos(), 0, "peek_second must not advance");
    assert_eq!(cur.next(), Some('𝔄'));
    assert_eq!(cur.pos(), 4, "a 4-byte char advances by 4");
  }

  #[test]
  fn empty_and_exhausted_are_well_behaved() {
    let mut cur = CharCursor::new("");
    assert!(cur.is_done());
    assert_eq!(cur.next(), None);
    assert_eq!(cur.pos(), 0);
    assert_eq!(cur.slice_from(0), "");

    let mut cur = CharCursor::new("é");
    assert_eq!(cur.next(), Some('é'));
    assert_eq!(cur.next(), None, "past the end stays None");
    assert_eq!(cur.pos(), 2, "position does not run past the end");
  }
}
