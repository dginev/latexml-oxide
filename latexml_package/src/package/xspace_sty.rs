use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: \xspace reads the next token. If it's NOT in a set of
  // "no-space" tokens, insert a space. The no-space set includes:
  // T_BEGIN, T_END, punctuation (.,:;!?/')-~), and certain CS tokens.
  // This makes \No\xspace produce "No " before words but "No." before periods.
  // Perl kind is DefMacro with a gullet-reading sub body; Rust DefPrimitive
  // achieves the same via gullet::read_token + unread (WISDOM #41 variant —
  // gullet-interaction pattern rather than pure state mutation).
  DefPrimitive!("\\xspace", {
    let next = gullet::read_token()?;
    let is_no_space = next.as_ref().map(|t| {
      let cc = t.get_catcode();
      // T_BEGIN ({) and T_END (}) suppress space (Perl: @XSPACES includes T_BEGIN, T_END)
      if matches!(cc, Catcode::BEGIN | Catcode::END) {
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
      gullet::unread(Tokens!(tok));
    }
    if !is_no_space {
      // Insert a space token
      gullet::unread(Tokens!(T_SPACE!()));
    }
  });
});
