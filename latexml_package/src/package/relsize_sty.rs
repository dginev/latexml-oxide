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

  // relsize.sty:263-310: `\mathlarger`/`\mathsmaller` read their atom, gather the `\limits`,
  // `\nolimits`, `\displaylimits` and scripts that follow it (`\rs@collect@decor`), and size
  // only that, in a group. Perl's (L45-46) are the open declarations `\relsize{±1}` reading an
  // optional step, so the size ran on over the rest of the formula (KPE #373). The choice step
  // is Perl's `\relsize{±1}` where relsize picks a `\mathchoice` of styles.
  RawTeX!(r"\DeclareRobustCommand\mathsmaller[1]{\bgroup
  \let\rs@makechoice\rs@makesmallerchoice
  \def\rs@mathatom{#1}%
  \futurelet\@tempa\rs@collect@decor}
\DeclareRobustCommand\mathlarger[1]{\bgroup
  \let\rs@makechoice\rs@makelargerchoice
  \def\rs@mathatom{#1}%
  \futurelet\@tempa\rs@collect@decor}
\def\rs@collect@decor{%
  \let\@tempb\rs@makechoice
  \ifx\@tempa\limits \let\@tempb\rs@collect@one@decor \fi
  \ifx\@tempa\displaylimits \let\@tempb\rs@collect@one@decor \fi
  \ifx\@tempa\nolimits \let\@tempb\rs@collect@one@decor \fi
  \if\noexpand\@tempa^\let\@tempb\rs@collect@two@decor \fi
  \if\noexpand\@tempa_\let\@tempb\rs@collect@two@decor \fi
  \ifx\@tempa\sp \let\@tempb\rs@collect@two@decor \fi
  \ifx\@tempa\sb \let\@tempb\rs@collect@two@decor \fi
  \@tempb}
\def\rs@collect@one@decor#1{%
  \expandafter\def\expandafter\rs@mathatom\expandafter{\rs@mathatom#1}%
  \futurelet\@tempa\rs@collect@decor}
\def\rs@collect@two@decor#1#2{\expandafter
  \def\expandafter\rs@mathatom\expandafter{\rs@mathatom#1{\rs@sstyle{#2}}}%
  \futurelet\@tempa\rs@collect@decor}
\let\rs@sstyle\@firstofone
\def\rs@makesmallerchoice{\relsize{-1}\rs@mathatom\egroup}
\def\rs@makelargerchoice{\relsize{+1}\rs@mathatom\egroup}");
});
