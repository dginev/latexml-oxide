//! TeX Logic
//!
//! Core TeX Implementation for LaTeXML

use crate::prelude::*;
LoadDefinitions!({
  //%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
  // Logic Family of primitive control sequences
  //%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
  //======================================================================
  // Basic logic
  //----------------------------------------------------------------------
  // \iftrue           c  is a conditional which is always true.
  // \iffalse          c  is a conditional which is always false.
  // \else             c  begins the false part of a conditional.
  // \fi               c  is the concluding command of a conditional.
  // \or               c  separates cases in an \ifcase conditional.
  DefConditional!("\\iftrue", { true });
  DefConditional!("\\iffalse", { false });
  DefConditional!("\\else"); // BUILT-IN to Definition
  DefConditional!("\\or"); // BUILT-IN to Definition
  DefConditional!("\\fi"); // BUILT-IN to Definition

  //======================================================================
  // Token testing
  //----------------------------------------------------------------------
  // \if               c  tests if two tokens have the same character codes (i.e., values 0-256).
  // \ifx              c  tests if two tokens are the same.
  // \ifcat            c  tests if two tokens have the same category codes (i.e., values 0-16).
  DefParameterType!(ExpandedIfToken, sub[_inner, _extra] {
    let token_opt = read_x_token(Some(false), true, None)?;
    match token_opt {
      Some(t) => t,
      None => {
        Error!("expected", "ExpandedIfToken",
          "conditional expected a token argument, came back empty. Falling back to \\lx@empty");
        T_CS!("\\lx@empty")
      }}
  });

  DefConditional!("\\if ExpandedIfToken ExpandedIfToken", sub[(left,right)] {
    left.get_charcode() == right.get_charcode()
  });
  DefConditional!("\\ifx Token Token", sub[(left,right)] { x_equals(&left, &right) });
  DefConditional!("\\ifcat ExpandedIfToken ExpandedIfToken", sub[(left,right)] {
    left.get_catcode() == right.get_catcode()
  });

  //======================================================================
  // Number testing
  //----------------------------------------------------------------------
  // \ifnum            c  compares two integers.
  // \ifodd            c  tests for an odd integer.
  // \ifcase           c  begins a multi-case conditional.
  // Perl (2026-03-18): Relation parameter type = skip spaces + readXToken (for <, =, >)
  DefConditional!("\\ifnum Number Relation Number", sub[(u,rel,v)] {
    compare(u.value_of(), rel, v.value_of())
  });
  DefConditional!("\\ifodd Number", sub[(u)] {
    u.value_of() % 2 == 1
  });
  DefConditional!("\\ifcase Number");

  //======================================================================
  // Dimension testing
  //----------------------------------------------------------------------
  // \ifdim            c  compares two dimensions.
  DefConditional!("\\ifdim Dimension Relation Dimension", sub[(u,rel,v)] {
    compare(u.value_of(), rel, v.value_of())
  });

  //======================================================================
  // Box testing
  //----------------------------------------------------------------------
  // \ifhbox           c  is true if a box register contains an \hbox.
  // \ifvbox           c  is true if a box register contains a \vbox.
  // \ifvoid           c  is true if a box register is void.
  //
  // Perl TeX_Logic.pool.ltxml L111-113: \ifvoid / \ifhbox / \ifvbox.
  DefConditional!("\\ifvoid Number", sub[(arg)] { classify_box(arg)?.is_empty() });
  DefConditional!("\\ifhbox Number", sub[(arg)] { classify_box(arg)? == "hbox" });
  DefConditional!("\\ifvbox Number", sub[(arg)] { classify_box(arg)? == "vbox" });

  //======================================================================
  // Mode testing
  //----------------------------------------------------------------------
  // \ifhmode          c  is true if TeX is in horizontal or restricted horizontal mode.
  // \ifinner          c  is true if TeX is in internal vertical, restricted horizontal, or
  // nondisplay math mode. \ifmmode          c  is true if TeX is in math or display math mode.
  // \ifvmode          c  is true if TeX is in vertical or internal vertical mode.

  DefConditional!("\\ifvmode", {
    let mode = lookup_string_from_sym(pin!("MODE"));
    mode.ends_with("vertical")
  });
  DefConditional!("\\ifhmode", {
    let mode = lookup_string_from_sym(pin!("MODE"));
    mode.ends_with("horizontal")
  });
  DefConditional!("\\ifinner", {
    let mode = lookup_string_from_sym(pin!("MODE"));
    matches!(
      mode.as_str(),
      "restricted_horizontal" | "internal_vertical" | "math"
    )
  });
  // Perl: LookupValue('MODE') =~ /math$/
  DefConditional!("\\ifmmode", {
    let mode = lookup_string_from_sym(pin!("MODE"));
    mode.ends_with("math")
  });

  //======================================================================
  // I/O testing
  //----------------------------------------------------------------------
  // \ifeof c tests for the end of a file .
  DefConditional!("\\ifeof Number", sub[(port)] {
    with_value(&s!("input_file:{}", port), |val_opt|
      if let Some(Stored::Mouth(mouth)) = val_opt {
        mouth.borrow().at_eof()
      } else {
        true
      })
  });
});

fn compare(u: i64, rel: Token, v: i64) -> bool {
  // NOTE: One would expect this to be best written as an advanced match state::ent
  // however, due to the shallow comparison of Cow<str> the Cow::Borrowed("<") and
  // Cow::Owned("<") variants will NOT be equal via a destructuring match.
  // However, since we've defined our own PartialEq trait over Token, an equality comparison
  // will produce the right behavior
  if rel == T_OTHER!("<") || rel == T_CS!("\\@@<") {
    u < v
  } else if rel == T_OTHER!("=") {
    u == v
  } else if rel == T_OTHER!(">") || rel == T_CS!("\\@@>") {
    u > v
  } else {
    let message = s!(
      "Expected a relational token for comparision. Got {:?} (cc {:?})",
      rel,
      rel.get_catcode()
    );
    let err = || {
      Error!("expected", "<relationaltoken>", message);
      Ok(())
    };
    err().ok();
    false
  }
}
