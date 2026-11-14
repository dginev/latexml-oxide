use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: \xspace reads the next token. If it's NOT in a set of
  // "no-space" tokens, insert a space. The no-space set includes:
  // T_BEGIN, T_END, punctuation (.,:;!?/')-~), and certain CS tokens.
  // This makes \No\xspace produce "No " before words but "No." before periods.
  // Perl kind is DefMacro with a gullet-reading sub body; Rust DefPrimitive
  // reads + unreads at stomach time. WISDOM #44: this is NOT a universal
  // equivalence — `\xspace` inside `\edef` would behave differently (Perl
  // expands at \edef time; Rust freezes the CS). Safe in practice because
  // `\xspace` call sites are post-expansion text-mode position.
  DefPrimitive!("\\xspace", {
    let next = read_token()?;
    let is_no_space = next.as_ref().map(|t| {
      let cc = t.get_catcode();
      // T_BEGIN ({) and T_END (}) suppress space (Perl: @XSPACES includes T_BEGIN, T_END)
      if matches!(cc, Catcode::BEGIN | Catcode::END) {
        return true;
      }
      // A PENDING SPACE TOKEN suppresses too — xspace.sty:49 lists `\@sptoken`
      // (LaTeX's `\let` alias of a catcode-10 space) among the exceptions, so
      // `\bazA[x] and` (space token survives after a `]`-delimited argument, or
      // after amsgen's non-space-skipping `\new@ifnextchar`) gets ONE space.
      // Perl's @XSPACES compares against the literal CS `\@sptoken`, never a
      // space token, so it doubles the space (pdflatex-probed: single space).
      // Witness: glossaries `\gls{potato} and` (structure/glossary golden).
      if cc == Catcode::SPACE {
        return true;
      }
      // CS tokens: \/, \ , \xspace, \space, \@sptoken, \@xobeysp
      if cc == Catcode::CS {
        return t.with_str(|s| matches!(s,
          "\\/"|"\\ "|"\\xspace"|"\\space"|"\\@sptoken"|"\\@xobeysp"
        ));
      }
      // Punctuation and special characters
      t.with_str(|s| matches!(s,
        "." | "," | ":" | ";" | "!" | "?" | "/" | "'" | ")" | "-" | "~"
        | "\u{00A0}" // non-breaking space
      ))
    }).unwrap_or(false);
    if let Some(tok) = next {
      unread(Tokens!(tok));
    }
    if !is_no_space {
      // Insert a space token
      unread(Tokens!(T_SPACE!()));
    }
  });

  // xspace.sty L57-60 `\xspaceaddexceptions{chars}` adds chars to the
  // dynamic exception list. Our `\xspace` primitive above uses a
  // hardcoded suppress-list; the dynamic list mechanism isn't modeled.
  // Stub as no-op so raw-loaded sibling packages that call this don't
  // emit undefined-CS errors. Author's added exceptions silently
  // won't take effect (cosmetic — potential extra space, never
  // missing-space). Companion `\xspaceremoveexception` too.
  def_macro_noop("\\xspaceaddexceptions{}")?;
  def_macro_noop("\\xspaceremoveexception{}")?;
});
