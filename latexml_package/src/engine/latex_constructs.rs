//! latex_constructs — LaTeX semantic definitions (constructors, environments)
//!
//! Perl: latex_constructs.pool.ltxml (6014 lines)
//! Loaded AFTER latex_dump in the Perl loading order.
//! Contains DefConstructor, DefEnvironment, Tag!, and other semantic
//! definitions that build on the basic infrastructure from latex_base.
//!
//! In our Rust port, these are organized by Lamport chapter (latex_ch*.rs files).
use crate::prelude::*;

/// Port of Perl's `latexChangeCase` function.
/// Applies Unicode case conversion (not TeX uccode/lccode tables) to tokens.
/// Converts CC_SPACE to T_SPACE (matching latex3 behavior).
/// Handles \protect + excluded CS tokens (text_case_exclude mapping).
fn lx_change_case_tokens(req_case: &str, tokens: &Tokens) -> Result<Vec<Token>> {
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
      result.push(T_SPACE!());
      if req_case == "title" {
        is_upper = true;
      }
    } else if cc == Catcode::CS && tok.with_str(|s| s == "\\protect") {
      if let Some(next_tok) = gullet::read_token()? {
        let next_key = next_tok.with_str(|s| s.trim_end().to_string());
        if lookup_mapping("text_case_exclude", &next_key).is_some() {
          let opt = gullet::read_optional(None)?;
          let arg = gullet::read_arg(ExpansionLevel::Off)?;
          result.push(tok);
          result.push(next_tok);
          if let Some(opt_tokens) = opt {
            let converted = lx_change_case_tokens(req_case, &opt_tokens)?;
            result.push(T_OTHER!("["));
            result.extend(converted);
            result.push(T_OTHER!("]"));
          }
          result.push(T_BEGIN!());
          result.extend(arg.unlist());
          result.push(T_END!());
        } else if let Some(changed) =
          lookup_mapping(if is_upper { "text_uppercase" } else { "text_lowercase" }, &next_key)
        {
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

#[rustfmt::skip]
LoadDefinitions!({
  // C.1 Commands and Environments
  InnerPool!(latex_ch1_documentclass);
  InnerPool!(latex_ch1_environments);
  InnerPool!(latex_ch1_fragile_commands);
  InnerPool!(latex_ch1_break_command);

  // C.2 The Structure of the Document
  InnerPool!(latex_ch2_document);

  // C.3 Sentences and Paragraphs
  InnerPool!(latex_ch3_sentences_and_paragraphs);

  // C.4 Sectioning and Table of Contents
  InnerPool!(latex_ch4_sectioning_and_toc);

  // C.5 Classes, Packages and Page Styles
  InnerPool!(latex_ch5_packages);
  InnerPool!(latex_ch5_page_styles);
  InnerPool!(latex_ch5_title_page_and_abstract);

  // C.6 Displayed Paragraphs
  InnerPool!(latex_ch6_displayed_paragraphs);
  InnerPool!(latex_ch6_quotations_and_verse);
  InnerPool!(latex_ch6_list_making_environments);
  InnerPool!(latex_ch6_list_and_trivlist_environments);
  InnerPool!(latex_ch6_verbatim);

  // C.7 Mathematical Formulas
  InnerPool!(latex_ch7_math_mode_environments);
  InnerPool!(latex_ch7_math_common_structures);
  InnerPool!(latex_ch7_math_common_delimiters);
  InnerPool!(latex_ch7_math_mode_changing_style);

  // C.8 Definitions, Numbering and Programming
  InnerPool!(latex_ch8_defining_commands);
  InnerPool!(latex_ch8_defining_environments);
  InnerPool!(latex_ch8_theoremlike_environments);
  InnerPool!(latex_ch8_numbering);

  // C.9 Figures and Other Floating Bodies
  InnerPool!(latex_ch9_figures_and_tables);
  InnerPool!(latex_ch9_marginal_notes);

  // C.10 Lining It Up in Columns
  InnerPool!(latex_ch10_tabbing_environment);
  InnerPool!(latex_ch10_array_and_tabular);

  // C.11 Moving Information Around
  InnerPool!(latex_ch11_moving_information);
  InnerPool!(latex_ch11_splitting_the_input);
  InnerPool!(latex_ch11_index_and_glossary);
  InnerPool!(latex_ch11_terminal_io);

  // C.12-C.13 Line/Page Breaking, Boxes
  InnerPool!(latex_ch12_line_and_page_breaking);
  InnerPool!(latex_ch13_boxes);

  // C.14-C.15 Pictures, Fonts, Symbols
  InnerPool!(latex_ch14_pictures_and_color);
  InnerPool!(latex_ch15_font_selection);
  InnerPool!(latex_ch15_special_symbol);

  // Semi-documented commands (L3 hooks, expl3 stubs)
  InnerPool!(latex_semi_undocumented);

  // Perl latex_constructs.pool.ltxml L5937-5938:
  // LaTeX now includes textcomp by default.
  RequirePackage!("textcomp");

  //======================================================================
  // Perl latex_constructs.pool.ltxml L5941-5993: Case-changing
  // (was in latex_other_in_appendices.rs, which has no Perl equivalent)
  //======================================================================

  DefMacro!(
    "\\@uclclist",
    r"\oe\OE\o\O\ae\AE\dh\DH\dj\DJ\l\L\ng\NG\ss\SS\th\TH"
  );

  DefPrimitive!("\\lx@prepare@case@mapping", {
    assign_mapping("text_uppercase", "\\i ", Some(T_LETTER!("I")));
    assign_mapping("text_uppercase", "\\j ", Some(T_LETTER!("J")));
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

  DefPrimitive!("\\AddToNoCaseChangeList DefToken", sub[(cs)] {
    let key = cs.with_str(|s| s.trim_end().to_string());
    assign_mapping("text_case_exclude", &key, Some(true));
  });

  DefMacro!("\\NoCaseChange {}", "#1", robust => true);

  DefMacro!("\\lx@latex@changecase {} GeneralText", sub[(case, tokens)] {
    let req_case = Expand!(case).to_string().to_lowercase();
    Ok(Tokens::new(lx_change_case_tokens(&req_case, &tokens)?))
  });

  TeX!(
    r"\AddToNoCaseChangeList{\NoCaseChange}%
\AddToNoCaseChangeList{\label}%
\AddToNoCaseChangeList{\ref}%
\AddToNoCaseChangeList{\cite}%
\AddToNoCaseChangeList{\ensuremath}%
\AddToNoCaseChangeList{\@ensuremath}%
\AddToNoCaseChangeList{\thanks}%"
  );

  // Perl L5966-5993: \MakeUppercase, \MakeLowercase, \MakeTitlecase
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

  // Perl L5913,5916: fixltx2e defaults
  DefMacro!("\\eminnershape", None, None);
  DefMacro!("\\TextOrMath{}{}", "\\ifmmode#2\\else#1\\fi");
});
