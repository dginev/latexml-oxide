//**********************************************************************
// Other stuff
//**********************************************************************
// Some stuff that got missed in the appendices ?

use crate::prelude::*;

/// Port of Perl's `latexChangeCase` function.
/// Applies Unicode case conversion (not TeX uccode/lccode tables) to tokens.
/// Converts CC_SPACE to T_SPACE (matching latex3 behavior).
/// Handles \protect + excluded CS tokens (text_case_exclude mapping).
fn lx_change_case_tokens(req_case: &str, tokens: &Tokens) -> Result<Vec<Token>> {
  // Match Perl's readingFromMouth($tokens, ...) behavior:
  // create an empty mouth, then unread the tokens so they're read from the pushback.
  // This avoids the endline-char trailing space that would appear if we converted
  // tokens to a string and created a Mouth from that string.
  let mouth = Mouth::new("", None)?;
  gullet::open_mouth(mouth, false);
  gullet::unread(tokens.clone());
  let result = lx_read_and_change_case(req_case)?;
  gullet::close_mouth(true)?;
  Ok(result)
}

fn lx_read_and_change_case(req_case: &str) -> Result<Vec<Token>> {
  let mut result = vec![];
  let mut in_math = false;
  let mut is_upper = req_case == "upper" || req_case == "sentence" || req_case == "title";
  loop {
    let tok = match gullet::read_x_token(Some(false), false, None)? {
      None => break,
      Some(t) => t,
    };
    let cc = tok.get_catcode();
    if cc == Catcode::MATH {
      in_math = !in_math;
      result.push(tok);
    } else if in_math {
      result.push(tok);
    } else if cc == Catcode::LETTER || cc == Catcode::OTHER {
      // Compute new case outside the with_str borrow to avoid arena RefCell conflict
      let new_str: String = tok.with_str(|s| {
        if is_upper {
          s.chars().flat_map(|c| c.to_uppercase()).collect()
        } else {
          s.chars().flat_map(|c| c.to_lowercase()).collect()
        }
      });
      let changed = tok.with_str(|s| s != new_str.as_str());
      let new_tok = if changed { Token::new(new_str, cc) } else { tok };
      result.push(new_tok);
      if req_case == "sentence" || req_case == "title" {
        is_upper = false;
      }
    } else if cc == Catcode::SPACE {
      result.push(T_SPACE!()); // HACK: match Perl/latex3 latexChangeCase
      if req_case == "title" {
        is_upper = true;
      }
    } else if cc == Catcode::CS && tok.with_str(|s| s == "\\protect") {
      // Handle \protect + next token: check text_case_exclude
      if let Some(next_tok) = gullet::read_token()? {
        // Trim trailing space to handle both "\NoCaseChange " (DeclareRobustCommand) and
        // "\@ensuremath" (manual definition) patterns
        let next_key = next_tok.with_str(|s| s.trim_end().to_string());
        if lookup_mapping("text_case_exclude", &next_key).is_some() {
          // Protected: read optional and required arg, preserve unchanged
          let opt = gullet::read_optional(None)?;
          let arg = gullet::read_arg(ExpansionLevel::Off)?;
          result.push(tok);       // \protect
          result.push(next_tok);  // \cs_space
          if let Some(opt_tokens) = opt {
            // Optional arg gets case-changed too (per Perl)
            let converted = lx_change_case_tokens(req_case, &opt_tokens)?;
            result.push(T_OTHER!("["));
            result.extend(converted);
            result.push(T_OTHER!("]"));
          }
          result.push(T_BEGIN!());
          result.extend(arg.unlist()); // required arg unchanged
          result.push(T_END!());
        } else if let Some(changed) =
          lookup_mapping(if is_upper { "text_uppercase" } else { "text_lowercase" }, &next_key)
        {
          // Map this CS to its case-changed form
          if let Stored::Token(changed_tok) = changed {
            result.push(changed_tok);
          } else {
            result.push(tok);
            result.push(next_tok);
          }
          if req_case == "sentence" || req_case == "title" {
            is_upper = false;
          }
        } else {
          result.push(tok);
          result.push(next_tok);
        }
      }
    } else {
      result.push(tok);
    }
  }
  Ok(result)
}


LoadDefinitions!({
  // Case-change infrastructure — Perl: latex_constructs.pool.ltxml L5941-5993
  // (Base definitions moved to latex_base.rs)

  DefMacro!(
    "\\@uclclist",
    r"\oe\OE\o\O\ae\AE\dh\DH\dj\DJ\l\L\ng\NG\ss\SS\th\TH"
  );
  // PORT of Perl's \lx@prepare@case@mapping / prepareCaseMapping:
  // Sets up text_uppercase / text_lowercase mappings for \protect+CS handling.
  // Since lx_change_case_tokens uses read_x_token to expand non-protected macros,
  // simple CS tokens like \i, \ae expand automatically. This maps the
  // robust-prefixed forms like "\i " (with trailing space) used when \protect is
  // the preceding token.
  DefPrimitive!("\\lx@prepare@case@mapping", {
    assign_mapping("text_uppercase", "\\i ", Some(T_LETTER!("I")));
    assign_mapping("text_uppercase", "\\j ", Some(T_LETTER!("J")));
    // Expand \@uclclist and read pairs
    let pairs_tokens = Expand!(Tokens!(T_CS!("\\@uclclist")));
    let pairs: Vec<Token> = pairs_tokens.unlist();
    let mut i = 0;
    while i + 1 < pairs.len() {
      let lower = pairs[i];
      let upper = pairs[i + 1];
      let lower_key = lower.with_str(|s| format!("{} ", s));
      let upper_key = upper.with_str(|s| format!("{} ", s));
      assign_mapping("text_uppercase", &lower_key, Some(upper));
      assign_mapping("text_lowercase", &upper_key, Some(lower));
      i += 2;
    }
  });

  // \AddToNoCaseChangeList{cs} - marks cs as protected from case-changing.
  // Key is stored as the raw CS name (trimmed, no trailing space) so it
  // works for both DeclareRobustCommand inner names ("\NoCaseChange ") and
  // manually-defined inner names like "\@ensuremath".
  DefPrimitive!("\\AddToNoCaseChangeList DefToken", sub[(cs)] {
    let key = cs.with_str(|s| s.trim_end().to_string());
    assign_mapping("text_case_exclude", &key, Some(true));
  });

  // \NoCaseChange{} - marks its argument as excluded from case change
  DefMacro!("\\NoCaseChange {}", "#1", robust => true);

  // \lx@latex@changecase{case}{text} - Port of Perl's latexChangeCase
  DefMacro!("\\lx@latex@changecase {} GeneralText", sub[(case, tokens)] {
    let req_case = Expand!(case).to_string().to_lowercase();
    Ok(Tokens::new(lx_change_case_tokens(&req_case, &tokens)?))
  });

  // Pre-register common excluded commands (matching Perl's latex_constructs.pool.ltxml).
  // We also register internal Rust forms (e.g. \@ensuremath) since our \ensuremath
  // expands to \protect\@ensuremath rather than \protect\ensuremath_space.
  TeX!(
    r"\AddToNoCaseChangeList{\NoCaseChange}%
\AddToNoCaseChangeList{\label}%
\AddToNoCaseChangeList{\ref}%
\AddToNoCaseChangeList{\cite}%
\AddToNoCaseChangeList{\ensuremath}%
\AddToNoCaseChangeList{\@ensuremath}%
\AddToNoCaseChangeList{\thanks}%"
  );

  // \MakeUppercase, \MakeLowercase, \MakeTitlecase - port of Perl latex_constructs.pool.ltxml
  TeX!(
    r"\DeclareRobustCommand{\MakeUppercase}[1]{{%
  \lx@prepare@case@mapping%
  \def\({$}\let\)\(%
  \def\i{I}\def\j{J}%
  \let\UTF@two@octets@noexpand\@empty
  \let\UTF@three@octets@noexpand\@empty
  \let\UTF@four@octets@noexpand\@empty
  \edef\reserved@a{\lx@latex@changecase{upper}{#1}}%
  \reserved@a
}}
\DeclareRobustCommand{\MakeLowercase}[1]{{%
  \lx@prepare@case@mapping%
  \def\({$}\let\)\(%
  \let\UTF@two@octets@noexpand\@empty
  \let\UTF@three@octets@noexpand\@empty
  \let\UTF@four@octets@noexpand\@empty
  \edef\reserved@a{\lx@latex@changecase{lower}{#1}}%
  \reserved@a
}}
\DeclareRobustCommand{\MakeTitlecase}[1]{{%
  \lx@prepare@case@mapping%
  \def\({$}\let\)\(%
  \let\UTF@two@octets@noexpand\@empty
  \let\UTF@three@octets@noexpand\@empty
  \let\UTF@four@octets@noexpand\@empty
  \edef\reserved@a{\lx@latex@changecase{sentence}{#1}}%
  \reserved@a
}}
\protected@edef\MakeUppercase#1{\MakeUppercase{#1}}
\protected@edef\MakeLowercase#1{\MakeLowercase{#1}}
\protected@edef\MakeTitlecase#1{\MakeTitlecase{#1}}"
  );

  // Perl latex_constructs L5913,5916: fixltx2e defaults  
  DefMacro!("\\eminnershape", None, None);
  DefMacro!("\\TextOrMath{}{}", "\\ifmmode#2\\else#1\\fi");
});
