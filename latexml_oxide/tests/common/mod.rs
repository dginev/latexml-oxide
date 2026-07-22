//! Helpers shared by the binary-driven integration tests (each `tests/*.rs` is
//! its own crate, so this is included with `mod common;`).

/// Drop ANSI SGR sequences so a `grep`-style assertion on captured stderr can
/// never silently match zero (CLAUDE.md "canvas signal integrity"). The logger
/// TTY-gates its colors, so a redirected stream is already color-free — this is
/// the defensive belt, kept because a false "no errors" is the one failure mode
/// we must never have.
pub fn strip_ansi(s: &str) -> String {
  let mut out = String::with_capacity(s.len());
  let mut chars = s.chars().peekable();
  while let Some(c) = chars.next() {
    if c == '\x1b' {
      for d in chars.by_ref() {
        if d.is_ascii_alphabetic() {
          break;
        }
      }
    } else {
      out.push(c);
    }
  }
  out
}
