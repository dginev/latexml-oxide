use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: \xspace reads the next token. If it's NOT in a set of
  // "no-space" characters (.,:;!?/')-~), insert a space.
  // This makes \No\xspace produce "No " before words but "No." before periods.
  DefPrimitive!("\\xspace", {
    let next = gullet::read_token()?;
    let is_no_space = next.as_ref().map(|t| {
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
