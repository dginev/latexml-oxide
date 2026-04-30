use crate::prelude::*;

// Perl relsize.sty.ltxml L23-24, L30-31: the same four chained s/…/…/
// substitutions to collapse `++`/`--`/`+-`/`-+` prefixes before parsing.
fn relsize_normalize_sign(input: &str) -> String {
  let s = input.trim();
  if let Some(rest) = s.strip_prefix("++") {
    return rest.to_string();
  }
  if let Some(rest) = s.strip_prefix("--") {
    return rest.to_string();
  }
  if let Some(rest) = s.strip_prefix("+-") {
    return format!("-{}", rest);
  }
  if let Some(rest) = s.strip_prefix("-+") {
    return format!("-{}", rest);
  }
  s.to_string()
}

#[rustfmt::skip]
LoadDefinitions!({
  // Perl relsize.sty.ltxml L20-32: `\relsize{s}` multiplies the current
  // font scale by 1.2^s; `\relscale{s}` multiplies by the literal s.
  // Both normalize duplicated sign prefixes (`++`, `--`, `+-`, `-+`)
  // before parsing, so users can chain `\relsize{+\s}` with a stored
  // signed expansion. Previous stub was a no-op DefMacro — calls like
  // `\larger` / `\smaller` (expanding to `\relsize{+1}` / `\relsize{-1}`)
  // silently dropped the scale, leaving text at base size.
  DefPrimitive!("\\relsize{}", sub[(size)] {
    let s = relsize_normalize_sign(&size.to_string());
    if let Ok(n) = s.trim().parse::<f64>() {
      merge_font(fontmap!(scale => 1.2_f64.powf(n)));
    }
    Ok(Vec::new())
  });
  DefPrimitive!("\\relscale{}", sub[(size)] {
    let s = relsize_normalize_sign(&size.to_string());
    if let Ok(n) = s.trim().parse::<f64>() {
      merge_font(fontmap!(scale => n));
    }
    Ok(Vec::new())
  });

  DefMacro!("\\textscale{}{}", "\\begingroup\\relscale{#1}#2\\endgroup");

  DefMacro!("\\larger Optional:1",         "\\relsize{+#1}");
  DefMacro!("\\smaller Optional:1",        "\\relsize{-#1}");
  DefMacro!("\\textlarger Optional:1 {}",  "{\\relsize{+#1}#2}");
  DefMacro!("\\textsmaller Optional:1 {}", "{\\relsize{-#1}#2}");

  DefMacro!("\\RSpercentTolerance", None);
  DefMacro!("\\RSsmallest",         "999pt");
  DefMacro!("\\RSlargest",          "1pt");

  DefMacro!("\\mathlarger Optional:1",  "\\relsize{+#1}");
  DefMacro!("\\mathsmaller Optional:1", "\\relsize{-#1}");
});
