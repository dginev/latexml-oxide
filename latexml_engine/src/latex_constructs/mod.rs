use std::collections::VecDeque;

use latexml_core::{
  alignment::template::TemplateConfig,
  common::{error::emit_warn, xml::is_descendant_or_self},
  digested::DigestedData,
};

///**********************************************************************
/// Rust port of LaTeXML's `latex_constructs.pool.ltxml`.
///
/// Organized following
///  "`LaTeX`: A Document Preparation System"
///   by Leslie Lamport
///   2nd edition
/// Addison Wesley, 1994
/// Appendix C. Reference Manual
///**********************************************************************
/// NOTE: This will be loaded after `TeX.pool`, so it inherits.
///**********************************************************************
use crate::base_utilities::{already_reported, insert_frontmatter};
use crate::{
  prelude::*,
  tex_box::{FramedOptions, framed_properties},
  tex_tables::alignment_bindings,
};

// digested_to_text moved to base_utilities.rs (PR #2767: needed by
// digest_front_matter for the creator before-separators).

// Mirrors Perl `Package.pm` (`split(/\s*,\s*/, $options)`) — strips
// whitespace on BOTH sides of each comma so option names are normalized
// when LaTeX line-wraps the option list (e.g. `[twocolumn,amsmath\n
// ,amssymb]`). Without leading `\s*` we'd get `"amsmath\n"` and the
// declared option callback wouldn't fire — silently turning the option
// into an unused-global.
static OPTS_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"\s*,\s*").unwrap());

// Perl `\ensuremathfollows` already-math test (latex_constructs.pool.ltxml
// L2083/L2098): `$MATHENVS = 'displaymath|equation*?|eqnarray*?'` and the guard
// `$csname !~ /^Math|\(|\[|(?:$MATHENVS)/o`. Kept verbatim (not hand-expanded)
// so the automath wrapper matches Perl exactly.
static AUTOMATH_ALREADY_MATH: Lazy<Regex> =
  Lazy::new(|| Regex::new(r"^Math|\\\(|\\\[|(?:displaymath|equation*?|eqnarray*?)").unwrap());

// Perl `\documentclass` (latex_constructs.pool.ltxml L78) wraps the
// raw option string in `TrimmedCommaList(...)` — i.e. comma-split AND
// strip whitespace from EACH element including the first/last.
// OPTS_REGEX only strips whitespace around the comma delimiters, so a
// leading-space bracket `[ amsmath, amssymb]` produces a first
// element ` amsmath` (still has leading space) which fails to match
// any DeclareOption. Wrap every OPTS_REGEX.split site with this
// helper to mirror TrimmedCommaList exactly.
// Witness: 2210.07776 (`\documentclass[ amsmath,amssymb,...]
// {revtex4-1}` — leading space prevented amsmath option from firing,
// so amsmath/amsbsy never loaded, so `\boldsymbol` was undefined).
// Only brace-depth-0 commas split, as LaTeX's own `\@for` walk of
// `\@classoptionslist` (latex.ltx `\@process@ptions`) and l3keys' clist
// parsing do: `thesis={type=dr,dr=rernat}` is ONE option (DEMO-TUDaPhD;
// Perl's `split(/\s*,\s*/)` cuts it in two — a Perl-only defect).
fn split_trim_options(s: &str) -> Vec<String> {
  let mut out = Vec::new();
  let mut depth = 0usize;
  let mut start = 0;
  for (i, c) in s.char_indices() {
    match c {
      '{' => depth += 1,
      '}' => depth = depth.saturating_sub(1),
      ',' if depth == 0 => {
        out.push(s[start..i].trim().to_string());
        start = i + 1;
      },
      _ => {},
    }
  }
  out.push(s[start..].trim().to_string());
  out.retain(|o| !o.is_empty());
  out
}
static SEMIVERBATIM_CHARS: [char; 4] = ['%', '\\', '{', '}'];
static NOTE_TEXT_END: Lazy<Regex> = Lazy::new(|| Regex::new("^(\\w+?)text$").unwrap());
static NOTE_MARK_END: Lazy<Regex> = Lazy::new(|| Regex::new("^(\\w+?)mark$").unwrap());

/// Defensive sanitizer for section-type identifiers (the `{section}` arg of
/// `\@@numbered@section` etc.). Some upstream paths — notably figure-block +
/// section sequencing in BoxedEPS-loading papers (Cluster A / math0010095) —
/// allow a trailing `\par` (or other CS) token to pollute the section type
/// string. The type identifier should always be a bare letter sequence
/// (`section`, `subsection`, `appendix` …); strip a single trailing CS if
/// present so downstream `\csname @<type>...@ID\endcsname`-style construction
/// stays well-formed.
/// The section-type NAME of a digested `{}` argument: its reverted tokens,
/// not its typeset string. Digestion decodes letters through the current
/// font encoding, so under LGR (babel greek, textalpha) the digested `section`
/// read back as `σεςτιον` and `\csname the…\endcsname` built `\theσεςτιον`
/// (greek-fontenc manuals, teubner, toptesi-it: 56 nested-section errors per
/// doc). Guard: `perfect_kernel_batch54::section_type_name_is_not_font_decoded`.
fn section_type_name(stype: &Digested) -> String {
  let name = stype
    .revert()
    .map(|t| t.to_string())
    .unwrap_or_else(|_| stype.to_string());
  strip_trailing_cs(name.trim())
}

fn strip_trailing_cs(stype: &str) -> String {
  // Detect a trailing `\<letters>` token attached to the identifier. We strip
  // exactly the well-known CS sentinels so we don't mis-mangle exotic but
  // legitimate type names.
  for tail in ["\\par", "\\@startsection@hook", "\\relax"] {
    if let Some(stripped) = stype.strip_suffix(tail) {
      return stripped.to_string();
    }
  }
  stype.to_string()
}

/// The standard sectioning element for a `\@startsection` level, per
/// latex.ltx's class conventions (classes.dtx: part −1, chapter 0, section 1,
/// subsection 2, subsubsection 3, paragraph 4, subparagraph 5).
fn section_element_for_level(level: i64) -> &'static str {
  match level {
    i64::MIN..=-1 => "ltx:part",
    0 => "ltx:chapter",
    1 => "ltx:section",
    2 => "ltx:subsection",
    3 => "ltx:subsubsection",
    4 => "ltx:paragraph",
    _ => "ltx:subparagraph",
  }
}

/// Is `stype` one of the sectioning tag names the LaTeXML schema knows?
fn is_known_section_type(stype: &str) -> bool {
  matches!(
    stype,
    "part"
      | "chapter"
      | "section"
      | "subsection"
      | "subsubsection"
      | "paragraph"
      | "subparagraph"
      | "appendix"
      | "bibliography"
      | "index"
  )
}

/// The element `\@@numbered@section` / `\@@unnumbered@section` open for a
/// section type. Mirrors Perl `latex_constructs.pool.ltxml:599-607`: a known
/// schema tag is used as-is, `app` → `ltx:appendix`; otherwise Perl warns
/// `malformed` and falls back to `ltx:section`. Before that fallback we consult
/// the `SECTION_ELEMENT` mapping `\@startsection` records for a type it does
/// not know, keyed by the heading LEVEL (`\DeclareSectionCommand`-defined
/// headings — KOMA `\DeclareNewSectionCommand[level=2,…]{task}`, witness
/// tudaexercise — carry their level only there), so the heading opens the
/// element of its level instead of a warned `ltx:section`.
fn section_element_for_type(stype: &str, numbered: bool) -> String {
  section_element_for_type_maybe(stype).unwrap_or_else(|| {
    let kind = if numbered { "numbered" } else { "unnumbered" };
    Warn!(
      "malformed",
      s!("ltx:{stype}"),
      s!("Tried to open an unknown tag ltx:{stype} for {kind} section")
    );
    "ltx:section".to_string()
  })
}

/// `section_element_for_type` without the unknown-type warning: `None` when
/// the type is unknown (the caller falls back to `ltx:section`).
fn section_element_for_type_maybe(stype: &str) -> Option<String> {
  if is_known_section_type(stype) {
    Some(s!("ltx:{stype}"))
  } else if stype == "app" {
    Some("ltx:appendix".to_string())
  } else {
    // OXIDIZED_DESIGN #175 (level-keyed element; KNOWN_PERL_ERRORS #118).
    lookup_mapping("SECTION_ELEMENT", stype).map(|m| m.to_string())
  }
}

/// Mirror of Perl `latex_constructs.pool.ltxml:2569-2574`'s
/// `getShortSource =~ /^plain/` check. Two locator shapes count as
/// "from plain":
///   1. A short source starting with "plain" — e.g. `plain.tex`, `plain.dump.txt` — matching Perl's
///      regex one-for-one.
///   2. The Rust-only sentinel `<embedded:plain>` produced by dump-loaded definitions
///      (`Locator::get_short_source` does not basename-strip this string because it contains a
///      `:`).
///
/// We deliberately do NOT use looser `contains("plain.")` /
/// `contains("/plain")` checks — they would match unrelated
/// paths such as `complainttext.tex` or `…/plainness/foo.sty`.
fn is_plain_definition_source(locator: Locator) -> bool {
  if locator.get_short_source("").starts_with("plain") {
    return true;
  }
  with(locator.get_source(), |s| s == "<embedded:plain>")
}

/// Mirror of Perl `isDefinableLaTeX` (latex_constructs.pool.ltxml:2569-2574).
/// Returns `(definable, plain_origin)`:
///   * `definable` — Perl's bool result. The CS is either undefined, or its prior definition came
///     from the plain pool (allowed to be overridden by LaTeX-pool `\newcommand`).
///   * `plain_origin` — Rust-only flag. True when the prior definition came from plain. Callers use
///     it to bypass any `<cs>:locked` guard installed on plain-pool CSes (Rust-specific lock
///     mechanism not present in Perl). False when the CS was undefined (no prior locator → no lock
///     to bypass).
fn is_definable_latex(cs: &Token) -> Result<(bool, bool)> {
  if is_definable(cs) {
    return Ok((true, false));
  }
  // Autoload triggers (installed by `def_autoload` in tex.rs) appear
  // defined but should not block `\newcommand` — Perl's analogous
  // `DefAutoload` entries live in `OmniBus.cls.ltxml`, which isn't
  // loaded for typical papers, so the user's `\newcommand` sees the
  // CS as undefined and succeeds. Mirror that by treating the
  // trigger as redefinable. (`plain_origin` stays false: there is
  // no `<cs>:locked` guard to bypass for an autoload trigger.)
  if has_value(&s!("{}:autoload", cs.to_string())) {
    return Ok((true, false));
  }
  let plain = lookup_definition(cs)?
    .is_some_and(|prev| prev.get_locator().is_some_and(is_plain_definition_source));
  Ok((plain, plain))
}

//======================================================================
// LaTeX helper functions (moved from latex_functions.rs)
// Perl: inline in latex_constructs.pool.ltxml
//======================================================================

pub fn start_appendices(kind: &str) { begin_appendices(kind) }

pub fn begin_appendices(counter: &str) {
  let the_ctr = s!("\\the{counter}");
  let the_ctr_id = s!("\\the{counter}@ID");
  let cs_ctr = T_CS!(s!("\\{counter}"));
  let_i(
    &T_CS!("\\lx@save@theappendex"),
    &T_CS!(&the_ctr),
    Some(Scope::Global),
  );
  let_i(
    &T_CS!("\\lx@save@theappendex@ID"),
    &T_CS!(&the_ctr_id),
    Some(Scope::Global),
  );
  let_i(&T_CS!("\\lx@save@appendix"), &cs_ctr, Some(Scope::Global));
  let_i(
    &T_CS!("\\lx@save@@appendix"),
    &T_CS!("\\@appendix"),
    Some(Scope::Global),
  );
  assign_mapping(
    "BACKMATTER_ELEMENT",
    "ltx:appendix",
    Some(s!("ltx:{counter}")),
  );
  let has_chapter = lookup_definition(&T_CS!("\\c@chapter"))
    .ok()
    .flatten()
    .is_some();
  if has_chapter && counter != "chapter" {
    let _ = new_counter(
      counter,
      "chapter",
      Some(NewDefault!(NewCounterOptions, idprefix => "A")),
    );
    let expansion: String = s!("\\thechapter.\\Alph{{{counter}}}");
    let _ = def_macro(
      T_CS!(the_ctr),
      None,
      Some(ExpansionBody::from(expansion)),
      Some(NewDefault!(ExpandableOptions, scope => Some(Scope::Global))),
    );
  } else {
    let _ = new_counter(
      counter,
      "document",
      Some(NewDefault!(NewCounterOptions, idprefix => "A")),
    );
    let expansion: String = s!("\\Alph{{{counter}}}");
    let _ = def_macro(
      T_CS!(the_ctr),
      None,
      Some(ExpansionBody::from(expansion)),
      Some(NewDefault!(ExpandableOptions, scope => Some(Scope::Global))),
    );
  }
  let _ = assign_register(
    &s!("\\c@{counter}"),
    RegisterValue::Number(Number::new(0)),
    None,
    Vec::new(),
  );
  assign_mapping("counter_for_type", "appendix", Some(counter.to_string()));
  let_i(&cs_ctr, &T_CS!("\\@@appendix"), Some(Scope::Global));
  let_i(
    &T_CS!("\\@appendix"),
    &T_CS!("\\relax"),
    Some(Scope::Global),
  );
}

pub fn end_appendices() {
  if let Some(counter_stored) = lookup_mapping("BACKMATTER_ELEMENT", "ltx:appendix") {
    let counter_full = counter_stored.to_string();
    let counter = counter_full.strip_prefix("ltx:").unwrap_or(&counter_full);
    let the_ctr = s!("\\the{counter}");
    let the_ctr_id = s!("\\the{counter}@ID");
    let_i(
      &T_CS!(the_ctr),
      &T_CS!("\\lx@save@theappendex"),
      Some(Scope::Global),
    );
    let_i(
      &T_CS!(the_ctr_id),
      &T_CS!("\\lx@save@theappendex@ID"),
      Some(Scope::Global),
    );
    let_i(
      &T_CS!(s!("\\{counter}")),
      &T_CS!("\\lx@save@appendix"),
      Some(Scope::Global),
    );
    let_i(
      &T_CS!("\\@appendix"),
      &T_CS!("\\lx@save@@appendix"),
      Some(Scope::Global),
    );
  }
}

pub fn make_note_tags(
  counter: &str,
  mark_opt: Option<&Digested>,
  tag_opt: Option<Cow<Digested>>,
) -> Result<SymHashMap<Stored>> {
  // Cluster A reprise (hep-ph0204075): strip a trailing \par that the {}
  // parameter reader sometimes pulls into the counter identifier. Without
  // this, the s!("\\the{counter}") and s!("\\ext@{counter}") paths below
  // produce CSes named `\thefootnote\par` / `\ext@footnote\par`.
  let counter = strip_trailing_cs(counter);
  let counter = counter.as_str();
  if let Some(tag) = tag_opt {
    let mut props = ref_step_id(counter)?;
    let mark = match mark_opt {
      None => tag.clone(),
      Some(mark) => Cow::Borrowed(mark),
    };
    props.insert("mark", mark.into());
    props.insert(
      "tags",
      digest(Tokens!(
        T_BEGIN!(),
        T_CS!("\\def"),
        T_CS!(s!("\\the{counter}")),
        T_BEGIN!(),
        tag.revert()?,
        T_END!(),
        T_CS!("\\def"),
        T_CS!(s!("\\typerefnum@{counter}")),
        T_BEGIN!(),
        T_CS!(s!("\\{counter}typerefname")),
        T_SPACE!(),
        tag.revert()?,
        T_END!(),
        T_CS!("\\lx@make@tags"),
        T_BEGIN!(),
        T_OTHER!(counter),
        T_END!(),
        T_END!()
      ))?
      .into(),
    );
    Ok(props)
  } else {
    let mut props = ref_step_counter(counter, false)?;
    let mark = Stored::Digested(match mark_opt {
      None => digest_text(Tokens!(T_CS!(s!("\\the{counter}"))))?,
      Some(mark) => mark.clone(),
    });
    props.insert("mark", mark);
    Ok(props)
  }
}

pub fn relocate_footnote(document: &mut Document, node: &mut Node) -> Result<()> {
  if let Some(caps) = NOTE_TEXT_END.captures(&node.get_attribute("role").unwrap_or_default()) {
    let notetype = caps.get(1).map_or("", |m| m.as_str());
    if let Some(mark) = node.get_attribute("mark") {
      for mut marknote in document.findnodes(
        &format!(".//ltx:note[@role='{notetype}mark'][@mark='{mark}']"),
        None,
      ) {
        if is_descendant_or_self(&marknote, node) {
          continue;
        }
        relocate_footnote_aux(document, notetype, &mut marknote, node)?;
      }
    }
  } else if let Some(caps) = NOTE_MARK_END.captures(&node.get_attribute("role").unwrap_or_default())
  {
    let notetype = caps.get(1).map_or("", |m| m.as_str());
    if let Some(mark) = node.get_attribute("mark") {
      for mut textnote in document.findnodes(
        &format!(".//ltx:note[@role='{notetype}text'][@mark='{mark}']"),
        None,
      ) {
        if is_descendant_or_self(node, &textnote) {
          continue;
        }
        relocate_footnote_aux(document, notetype, node, &mut textnote)?;
      }
    }
  }
  Ok(())
}

fn relocate_footnote_aux(
  document: &mut Document,
  notetype: &str,
  marknote: &mut Node,
  textnote: &mut Node,
) -> Result<()> {
  document.append_clone(marknote, textnote.get_child_nodes())?;
  document.set_attribute(marknote, "role", notetype)?;
  if let Some(labels) = textnote.get_attribute("labels") {
    document.generate_id(marknote, "")?;
    document.set_attribute(marknote, "labels", &labels)?;
  }
  document.safe_unlink(textnote.clone());
  Ok(())
}

pub fn only_preamble(cs: &str) -> Result<()> {
  // Legal while `inPreamble`. `\begin{document}` keeps `inPreamble=1` through the
  // begindocument hooks and only clears it AFTER them (matching real latex.ltx,
  // which disables the \@onlypreamble commands via `\@preamblecmds` (L9522) only
  // after firing the begindocument hook (L9512)), so a `\RequirePackage`/
  // `\usepackage` deferred to `\AtBeginDocument` is still legal — no separate hook
  // flag needed. Paragraph-breaking inside `\AtBeginDocument` (upstream #2754) is
  // handled independently by `\par`'s no-op-in-vertical-mode rule, NOT by clearing
  // `inPreamble` early (upstream #2846 is thus unnecessary; KNOWN_PERL_ERRORS #43).
  if !lookup_bool_sym(pin!("inPreamble")) {
    Error!(
      "unexpected",
      cs,
      s!("The current command '{cs}' can only appear in the preamble")
    );
  }
  Ok(())
}

pub fn tabular_bindings(
  mut template: Template,
  mut properties: SymHashMap<Stored>,
  mut xml_attributes: HashMap<String, String>,
) -> Result<()> {
  for col in template.get_columns_mut() {
    if let Some(ref after) = col.after
      && after
        .unlist_ref()
        .iter()
        .any(|t| t.with_str(|s| s.contains("intercol")))
    {
      col.has_intercol_after = true;
    }
  }
  for col in template.get_repeated_mut() {
    if let Some(ref after) = col.after
      && after
        .unlist_ref()
        .iter()
        .any(|t| t.with_str(|s| s.contains("intercol")))
    {
      col.has_intercol_after = true;
    }
  }
  if !properties.contains_key("guess_headers")
    && let Some(v) = lookup_value("GUESS_TABULAR_HEADERS")
  {
    properties.insert("guess_headers", v);
  }
  if !xml_attributes.contains_key("colsep") {
    let sep_opt = lookup_dimension("\\tabcolsep");
    if let Some(sep) = sep_opt
      && sep.value_of()
        != lookup_dimension("\\lx@default@tabcolsep")
          .unwrap()
          .value_of()
    {
      xml_attributes.insert(String::from("colsep"), sep.to_attribute());
    }
  }
  if !xml_attributes.contains_key("rowsep") {
    let astr = do_expand(T_CS!("\\arraystretch"))?.to_string();
    if astr != "1"
      && let Ok(astr_f) = astr.parse::<f64>()
      && astr_f != 1.0
    {
      let rowsep = Dimension::from_str(&s!("{}em", astr_f - 1.0))?;
      xml_attributes.insert(String::from("rowsep"), rowsep.to_attribute());
    }
  }
  if !properties.contains_key("strut") {
    properties.insert("isLaTeX", Stored::Bool(true));
    if let Ok(Some(bs)) = lookup_register("\\baselineskip", Vec::new()) {
      properties.insert("strut", bs.into());
    }
  }
  alignment_bindings(template, String::from("text"), properties, xml_attributes);
  let_i(&T_CS!("\\\\"), &T_CS!("\\@tabularcr"), None);
  let_i(&T_CS!("\\lx@intercol"), &T_CS!("\\lx@text@intercol"), None);
  let_i(&T_CS!("\\tabularnewline"), &T_CS!("\\\\"), None);
  for name in [
    "@row@before",
    "@row@after",
    "@column@before",
    "@column@after",
  ] {
    let cs = T_CS!(s!("\\lx@alignment{name}"));
    let cs_def = lookup_definition(&cs)?.unwrap();
    let mut expansion = cs_def.get_expansion().cloned().unwrap_or_default();
    expansion.push(T_CS!(s!("\\@tabular{name}")));
    def_macro(cs, None, expansion, None)?;
  }
  Ok(())
}

/// Port of Perl's `latexChangeCase` function.
/// Applies Unicode case conversion (not TeX uccode/lccode tables) to tokens.
/// Converts CC_SPACE to T_SPACE (matching latex3 behavior).
/// Handles \protect + excluded CS tokens (text_case_exclude mapping).
fn lx_change_case_tokens(req_case: &str, tokens: &Tokens) -> Result<Vec<Token>> {
  let mouth = Mouth::new("", None)?;
  open_mouth(mouth, false);
  unread(tokens.clone());
  let result = lx_read_and_change_case(req_case)?;
  close_mouth(true)?;
  Ok(result)
}

fn lx_read_and_change_case(req_case: &str) -> Result<Vec<Token>> {
  let mut result = vec![];
  let mut in_math = false;
  let mut is_upper = req_case == "upper" || req_case == "sentence" || req_case == "title";
  loop {
    let tok = match read_x_token(Some(false), false, None)? {
      None => break,
      Some(t) => t,
    };
    let cc = tok.get_catcode();
    if cc == Catcode::MATH {
      in_math = !in_math;
      result.push(tok);
    } else if in_math {
      // Math content is preserved verbatim (no case change). One hazard: a
      // robust command nested in the math — e.g. `$\MakeUppercase{C}$` — is
      // expanded by read_x_token above to `\protect\MakeUppercase ` (the
      // robust real macro). If we let the following `\MakeUppercase ` body
      // expand here, its OWN definition's literal `$` tokens (from
      // `\def\({$}\let\)\(`) get read into this loop and miscount the CC_MATH
      // toggle, desynchronising `in_math` and leaking math mode into the
      // surrounding digestion (witnessed in amsart frontmatter:
      // `\@add@frontmatter@now Attempt to end mode text in math`). On
      // `\protect` inside math, grab the following token WITHOUT expansion and
      // shield it with `\dont_expand` so the caller's `\edef\reserved@a{...}`
      // keeps it literal; it is then digested as ordinary math later. This
      // mirrors Perl, whose robust-command `\protect` survives the outer
      // `\edef` via `\noexpand`. Plain math symbols (`\alpha`, …) are not
      // `\protect`-prefixed, so normal math is unaffected.
      if cc == Catcode::CS && tok.with_str(|s| s == "\\protect") {
        if let Some(next) = read_token()? {
          result.push(tok);
          result.push(T_CS!("\\dont_expand"));
          result.push(next);
        } else {
          result.push(tok);
        }
      } else {
        result.push(tok);
      }
    } else if cc == Catcode::LETTER || cc == Catcode::OTHER {
      let new_str: String = tok.with_str(|s| {
        if is_upper {
          s.chars().flat_map(|c| c.to_uppercase()).collect()
        } else {
          s.chars().flat_map(|c| c.to_lowercase()).collect()
        }
      });
      let changed = tok.with_str(|s| s != new_str.as_str());
      let new_tok = if changed {
        Token::new(new_str, cc)
      } else {
        tok
      };
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
      if let Some(next_tok) = read_token()? {
        // Perl: $cs->getString (full CS name). Munged-robust CSes carry a
        // trailing space — canonicalise to NO trailing space for the
        // exclude lookup (matches \AddToNoCaseChangeList storage format),
        // and to "CS + trailing space" for the case-mapping lookup
        // (matches `\lx@prepare@case@mapping` storage format, which is
        // `$lower->getString . ' '` in Perl).
        let next_key_bare = next_tok.with_str(|s| s.trim_end().to_string());
        let next_key_case = format!("{} ", next_key_bare);
        if lookup_mapping("text_case_exclude", &next_key_bare).is_some() {
          let opt = read_optional(None)?;
          let arg = read_arg(ExpansionLevel::Off)?;
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
        } else {
          match lookup_mapping(
            if is_upper {
              "text_uppercase"
            } else {
              "text_lowercase"
            },
            &next_key_case,
          ) {
            Some(changed) => {
              if let Stored::Token(changed_tok) = changed {
                result.push(changed_tok);
              } else {
                result.push(tok);
                result.push(next_tok);
              }
              if req_case == "sentence" || req_case == "title" {
                is_upper = false;
              }
            },
            _ => {
              // Fall-through: not in exclude list, not in case-mapping. Pass
              // both `\protect` and the munged CS through, but mark the CS
              // un-expandable via `\dont_expand` so the OUTER `\edef`'s
              // `Partial` body-reader doesn't re-invoke it. Without
              // `\dont_expand`, the captured tokens go through `\edef` body
              // expansion which would re-trigger the robust macro (whose
              // body contains another `\edef\reserved@a{...}`), mangling
              // the saved tokens and dropping content during the outer
              // `\reserved@a` invocation. Driver: nested
              // `\MakeLowercase{\MakeUppercase{...}}`.
              result.push(tok);
              result.push(T_CS!("\\dont_expand"));
              result.push(next_tok);
            },
          }
        }
      }
    } else {
      result.push(tok);
    }
  }
  Ok(result)
}

const PM_ORDINAL_SUFFICES: &[&str] = &["th", "st", "nd", "rd", "th", "th", "th", "th", "th", "th"];
const FNSYMBOLS: &[&str] = &[
  "*",
  "\u{2020}",
  "\u{2021}",
  "\u{00A7}",
  "\u{00B6}",
  "\u{2225}",
  "**",
  "\u{2020}\u{2020}",
  "\u{2021}\u{2021}",
];

//**********************************************************************
// C.6 Displayed Paragraphs
//**********************************************************************
/// Perl: setupAligningContext — saves [node, lastChild] for deferred class application.
fn setup_aligning_context(doc: &mut Document) {
  if let Some(node) = doc.get_element() {
    // Save node and its current last child so we only apply to NEW children later
    assign_value("ALIGNING_NODE", Stored::Node(node.clone()), None);
    match node.get_last_child() {
      Some(last) => {
        assign_value("ALIGNING_PREV_CHILD", Stored::Node(last), None);
      },
      _ => {
        assign_value("ALIGNING_PREV_CHILD", Stored::None, None);
      },
    }
  }
}
/// Perl: applyAligningContext — applies align/class to children added AFTER \centering.
fn apply_aligning_context(document: &mut Document, align: &str, class: &str) -> Result<()> {
  // with_value avoids two Stored envelope clones; Node is Rc-backed so we
  // still pay a Rc::clone inside the closure but skip the enum match work.
  let node_opt = with_value("ALIGNING_NODE", |v| match v {
    Some(Stored::Node(node)) => Some(node.clone()),
    _ => None,
  });
  if let Some(node) = node_opt {
    let previous_opt = with_value("ALIGNING_PREV_CHILD", |v| match v {
      Some(Stored::Node(prev)) => Some(prev.clone()),
      _ => None,
    });
    let children = node.get_child_nodes();
    let mut past_previous = previous_opt.is_none(); // if no previous, apply to all
    for mut child in children {
      if !past_previous {
        if let Some(ref prev) = previous_opt
          && child == *prev
        {
          past_previous = true;
        }
        continue;
      }
      if child.get_type() == Some(NodeType::ElementNode) {
        set_align_or_class(document, &mut child, align, class)?;
      }
    }
  }
  // Release the saved node handles: every `assign_value` here PUSHES onto the
  // key's binding stack (build-time assignments sit outside digestion groups,
  // so nothing ever pops them), and each retained `Stored::Node` pins its
  // whole document — under streaming, a pinned handle into a SPILLED subtree
  // even blocks the `xmlFreeNode` the spill relies on. Overwrite with `None`
  // so the C trees can go; the (empty) stack entries themselves are the cheap
  // part (dhat: 288 B each).
  assign_value("ALIGNING_NODE", Stored::None, None);
  assign_value("ALIGNING_PREV_CHILD", Stored::None, None);
  Ok(())
}

/// Real LaTeX's `\verb`/`{verbatim}` do `\let\do\@makeother\dospecials`:
/// every char REGISTERED in `\dospecials` — including chars packages like
/// csquotes/babel/underscore made ACTIVE and added exactly so verbatim
/// neutralizes them (csquotes.sty L1521-1524) — becomes catcode OTHER.
/// LaTeXML's static SPECIALS list misses those dynamic registrations, so an
/// active auto-quote char inside a `\verb` body FIRED its csquotes meaning
/// (biblatex.tex L6575 `\verb|<|` → `\csq@qopen` → sfcodes probe + a
/// truncated `\csq@fixkern` \expandafter chain; 94 errors; Perl shares —
/// pdflatex clean). Expand `\dospecials` and de-fang each listed char.
fn apply_dospecials() {
  // ONE-level expansion: the list body is `\do \<char>` pairs; full
  // expansion would invoke `\do` itself with whatever meaning it holds.
  let expansion = match lookup_expandable(&T_CS!("\\dospecials"), None) {
    Ok(Some(defn)) => defn.invoke(true),
    _ => return,
  };
  if let Ok(expansion) = expansion {
    for t in expansion.unlist_ref().iter() {
      if !t.get_catcode().is_active_or_cs() {
        continue;
      }
      // The list alternates `\do` and single-char tokens (`\do\ \do\\…`);
      // chars appear as single-char CS or ACTIVE tokens.
      t.with_cs_name(|name| {
        // The stored CS text keeps its leading escape char (`\<` → "\\<").
        let name = name.strip_prefix('\\').unwrap_or(name);
        let mut chars = name.chars();
        if let Some(c) = chars.next()
          && chars.next().is_none()
        {
          assign_catcode(c, Catcode::OTHER, Some(Scope::Local));
        }
      });
    }
  }
}

fn before_digest_verbatim() -> Result<Vec<Digested>> {
  bgroup();
  apply_dospecials();
  let mut stuff = Vec::new();
  if let Some(b) = lookup_tokens("@environment@verbatim@atbegin") {
    stuff.push(digest(b.unlist())?);
  }
  AssignValue!("current_environment", "verbatim");
  DefMacro!("\\@currenvir", "verbatim");
  MergeFont!(family => "typewriter");
  Ok(stuff)
}

fn after_digest_verbatim(starred: bool, whatsit: &mut Whatsit) -> Result<()> {
  // makes you wonder if the `get_font` API should be working with Rc<Font> in the first place...
  let font: Option<Rc<Font>> = whatsit.get_font()?.map(|ft| Rc::new((*ft).clone()));
  let loc = whatsit.get_locator();
  let (end, space) = if starred {
    ("\\end{verbatim*}", '\u{2423}')
  } else {
    ("\\end{verbatim}", ' ')
  };
  let mut lines: Vec<_> = Vec::new();
  while let Some(next_line) = read_raw_line() {
    let mut line = next_line.as_str();
    let mut exiting = false;
    if let Some((final_line, remaining)) = line.split_once(end) {
      line = final_line;
      // The rest of the `\end{verbatim}` line is re-read LAZILY, as raw
      // source, the way TeX does (latex.ltx:15438 `\@xverbatim` is delimited
      // by the catcode-12 `\end{verbatim}` string and `\end` runs `\endgroup`
      // BEFORE the remainder is tokenized, tex.web §332). Perl's
      // latex_constructs.pool:1777 (and the former port) pre-tokenized it,
      // which handed a `\verb` on that line frozen tokens: its reader
      // (`read_verb_invocation`) activates the delimiter and scans raw chars,
      // so the frozen `|` never matched, the scan ran off the end and
      // re-tokenized the rest of the DOCUMENT under `\dospecials` (ddphonism
      // :87 `\end{verbatim} produces the same as \verb|\dmatrix{…}|.` — every
      // later `{`/`}`/`\` literal, three lists never closed; Perl identical,
      // KPE #165). The lazy mouth supplies the line's own end-of-line.
      // Guard: `perfect_kernel_batch54::verb_on_the_endverbatim_line_scans_raw`.
      if remaining.trim().is_empty() {
        unread_one(T_CR!());
      } else {
        open_mouth(Mouth::new(remaining, None)?, true);
      }
      exiting = true;
    }
    // The raw chars will still have to be decoded (but not space!!). A TAB
    // keeps catcode 10 under `\dospecials` (^^I is not in the list), so it is
    // a space in verbatim, not OT1 slot 9 `Ψ` (Perl :1773 decodes it; KPE #165).
    let mut decoded_line: String = String::new();
    for c in line.chars() {
      if c == ' ' || c == '\t' {
        decoded_line.push(space);
      } else {
        let decoded_c = font::decode_string(pin_char(c), Some("OT1_typewriter"), true);
        with(decoded_c, |c_str| decoded_line.push_str(c_str));
      }
    }
    decoded_line.push('\n');
    lines.push(pin(decoded_line));
    if exiting {
      break;
    }
  }
  if let Some(last_line) = lines.last()
    && *last_line == pin!("\n")
  {
    lines.pop();
  }
  // Note last line ends up as Whatsit's "trailer"
  if let Some(b) = lookup_tokens("@environment@verbatim@atend") {
    lines.push(pin(digest(b)?.to_string()));
  }
  egroup()?;
  lines.push(pin_static(end));
  let boxes = lines
    .into_iter()
    .map(|line| {
      Tbox::new(
        line,
        font.clone(),
        loc,
        Token {
          text: line,
          code: Catcode::OTHER,
          #[cfg(feature = "token-locators")]
          loc: 0,
        }
        .into(),
        SymHashMap::default(),
      )
      .into()
    })
    .collect();
  whatsit.set_body(boxes);
  Ok(())
}

//======================================================================
// C.7.1 Math Mode Environments
//======================================================================
// # This provides {equation} with the capabilities for tags, nonumber, etc
// # even though stock LaTeX provides no means to override them.
// #   preset => boolean
// #   postset => boolean
// #   deferretract=>boolean
pub fn prepare_equation_counter(options: SymHashMap<Stored>) {
  // Guard: ensure the equation counter exists — normally created by article.cls,
  // but standalone classes (jpsj2, appolb, etc.) may not define it.
  if lookup_definition(&T_CS!("\\theequation@ID"))
    .ok()
    .flatten()
    .is_none()
  {
    let _ = new_counter(
      "equation",
      "section",
      Some(NewDefault!(NewCounterOptions, idprefix => "E")),
    );
  }
  assign_value(
    "EQUATION_NUMBERING",
    Stored::HashStored(options),
    Some(Scope::Global),
  );
}
pub fn before_equation() -> Result<()> {
  let mut has_preset = false;
  let mut is_numbered = false;
  maybe_peek_label()?;
  let ctr = with_value_mut("EQUATION_NUMBERING", |val_opt| {
    if let Some(Stored::HashStored(numbering)) = val_opt {
      numbering.insert("in_equation", true.into());
      is_numbered = matches!(numbering.get("numbered"), Some(&Stored::Bool(true)));
      has_preset = numbering.contains_key("preset");
      match numbering.get("counter") {
        Some(Stored::String(v)) => to_string(*v),
        Some(other) => {
          emit_warn(
            "internal",
            "state",
            &format!("eq counter should be stored as string, was instead: {other:?}"),
          );
          String::from("equation")
        },
        _ => String::from("equation"),
      }
    } else {
      String::from("equation")
    }
  });
  if has_preset {
    let mut tags = if is_numbered {
      ref_step_counter(&ctr, false)?
    } else {
      ref_step_id(&ctr)?
    };
    tags.insert("preset", true.into());
    assign_value("EQUATIONROW_TAGS", tags, Some(Scope::Global));
  } else {
    assign_value(
      "EQUATIONROW_TAGS",
      Stored::HashStored(SymHashMap::default()),
      Some(Scope::Global),
    );
  }
  let_i(
    &T_CS!("\\lx@end@display@math"),
    &T_CS!("\\lx@eDM@in@equation"),
    None,
  );
  let_i(
    &T_CS!("\\lx@begin@display@math"),
    &T_CS!("\\lx@bDM@in@equation"),
    None,
  );
  Ok(())
}
pub fn after_equation(whatsit: Option<&mut Whatsit>) -> Result<()> {
  // Phase 1: Gather all needed data from state (immutable borrows only)
  enum EqAction {
    Retract,
    Postset,
    TagsUpdate,
    None,
  }
  let mut action = EqAction::None;
  let mut is_aligned = false;
  let mut is_numbered_for_postset = false;
  let mut ctr = String::from("equation");
  with_value("EQUATION_NUMBERING", |eq_num_opt| {
    if let Some(Stored::HashStored(numbering)) = eq_num_opt {
      is_aligned = matches!(numbering.get("aligned"), Some(&Stored::Bool(true)));
      is_numbered_for_postset = matches!(numbering.get("numbered"), Some(&Stored::Bool(true)));
      with_value("EQUATIONROW_TAGS", |tags_opt| {
        if let Some(Stored::HashStored(tags)) = tags_opt {
          ctr = tags
            .get("counter")
            .map_or_else(|| numbering.get("counter"), Some)
            .map(ToString::to_string)
            .unwrap_or_else(|| String::from("equation"));
          if !matches!(tags.get("noretract"), Some(&Stored::Bool(true)))
            && (matches!(tags.get("retract"), Some(&Stored::Bool(true)))
              || (matches!(numbering.get("retract"), Some(&Stored::Bool(true)))
                && matches!(numbering.get("preset"), Some(&Stored::Bool(true)))
                && matches!(tags.get("preset"), Some(&Stored::Bool(true)))))
          {
            action = EqAction::Retract;
          } else if matches!(numbering.get("postset"), Some(&Stored::Bool(true)))
            && !matches!(tags.get("reset"), Some(&Stored::Bool(true)))
          {
            action = EqAction::Postset;
          } else if !matches!(tags.get("reset"), Some(&Stored::Bool(true)))
            && matches!(numbering.get("numbered"), Some(&Stored::Bool(true)))
          {
            action = EqAction::TagsUpdate;
          }
        }
      });
    }
  });
  // Phase 2: Act on gathered data (borrows released, safe to mutate state)
  match action {
    EqAction::Retract => {
      retract_equation();
    },
    EqAction::Postset => {
      let new_tags = if is_numbered_for_postset {
        ref_step_counter(&ctr, false)?
      } else {
        ref_step_id(&ctr)?
      };
      assign_value(
        "EQUATIONROW_TAGS",
        Stored::HashStored(new_tags),
        Some(Scope::Global),
      );
    },
    EqAction::TagsUpdate => {
      let invoked_tags = build_invocation(T_CS!("\\lx@make@tags"), vec![Some(Tokens::new(
        Explode!(ctr),
      ))])?;
      let stored_tags_update = Stored::Digested(digest(invoked_tags)?);
      with_value_mut("EQUATIONROW_TAGS", |tags_opt| {
        if let Some(Stored::HashStored(tags)) = tags_opt {
          tags.insert("tags", stored_tags_update);
        }
      });
    },
    EqAction::None => {},
  }
  // Phase 3: Reset in_equation flag
  with_value_mut("EQUATION_NUMBERING", |eq_num_opt| {
    if let Some(Stored::HashStored(numbering)) = eq_num_opt {
      numbering.insert("in_equation", Stored::Bool(false));
    }
  });
  // Phase 4: Install tags in $whatsit or current Row, as appropriate.
  #[allow(clippy::manual_unwrap_or_default)]
  let props = match remove_value("EQUATIONROW_TAGS") {
    Some(Stored::HashStored(hs)) => hs,
    _ => SymHashMap::default(),
  };
  if is_aligned {
    // Perl: propagate id/tags to current alignment row.
    // In Perl, these get stored as $$row{id}, $$row{tags} on the row object.
    // Store on the current alignment row so each row retains its own props.
    if let Some(alignment_digested) = lookup_alignment()
      && let Some(alignment_cell) = alignment_digested.alignment_cell()
    {
      let mut alignment = alignment_cell.borrow_mut();
      if let Some(row) = alignment.current_row_mut() {
        for (key, val) in &props {
          row.properties.insert(to_string(*key), val.clone());
        }
      }
    }
  } else if let Some(w) = whatsit {
    w.set_properties(props);
  }
  Ok(())
}
/// Perl: latex_constructs.pool.ltxml lines 2025-2035
fn retract_equation() {
  // Phase 1: Gather data (immutable borrows)
  let (ctr, is_preset, is_numbered) = with_value("EQUATION_NUMBERING", |eq_num_opt| {
    let numbering = match eq_num_opt {
      Some(Stored::HashStored(n)) => n,
      _ => return (String::from("equation"), false, false),
    };
    let is_numbered = matches!(numbering.get("numbered"), Some(&Stored::Bool(true)));
    with_value("EQUATIONROW_TAGS", |tags_opt| {
      let tags = match tags_opt {
        Some(Stored::HashStored(t)) => t,
        _ => return (String::from("equation"), false, is_numbered),
      };
      let ctr = tags
        .get("counter")
        .map_or_else(|| numbering.get("counter"), Some)
        .map(ToString::to_string)
        .unwrap_or_else(|| String::from("equation"));
      let is_preset = matches!(tags.get("preset"), Some(&Stored::Bool(true)));
      (ctr, is_preset, is_numbered)
    })
  });
  // Phase 2: Mutate state (borrows released)
  if is_preset {
    // counter (or ID counter) was stepped, so decrement it.
    let counter_name = if is_numbered {
      ctr.clone()
    } else {
      s!("UN{}", ctr)
    };
    let _ = add_to_counter(&counter_name, Number::new(-1));
  }
  if let Ok(mut new_tags) = ref_step_id(&ctr) {
    new_tags.insert("reset", true.into());
    assign_value(
      "EQUATIONROW_TAGS",
      Stored::HashStored(new_tags),
      Some(Scope::Global),
    );
  }
}
/// Perl: latex_constructs.pool.ltxml lines 2287-2325
/// eqnarrayBindings — creates alignment with equationgroup/equation/_Capture_ hooks
pub fn eqnarray_bindings() -> Result<()> {
  // Ensure @equationgroup counter exists — it's normally created by article.cls,
  // but standalone classes (appolb, jpsj2, etc.) may not define it.
  if lookup_definition(&T_CS!("\\the@equationgroup@ID"))?.is_none() {
    NewCounter!("@equationgroup", "document", idprefix => "EG", idwithin => "section");
  }

  // Perl: 3-column template: col1=right, col2=center, col3=left
  let col1 = Cell {
    before: Some(Tokens::new(vec![
      T_CS!("\\hfil"),
      T_MATH!(),
      T_CS!("\\displaystyle"),
    ])),
    after: Some(Tokens::new(vec![T_MATH!()])),
    empty: true,
    ..Cell::default()
  };
  let col2 = Cell {
    before: Some(Tokens::new(vec![
      T_CS!("\\hfil"),
      T_MATH!(),
      T_CS!("\\displaystyle"),
    ])),
    after: Some(Tokens::new(vec![T_MATH!(), T_CS!("\\hfil")])),
    empty: true,
    ..Cell::default()
  };
  let col3 = Cell {
    before: Some(Tokens::new(vec![T_MATH!(), T_CS!("\\displaystyle")])),
    after: Some(Tokens::new(vec![T_MATH!(), T_CS!("\\hfil")])),
    empty: true,
    ..Cell::default()
  };
  let template = Template::new(TemplateConfig {
    columns: Some(vec![col1, col2, col3]),
    ..TemplateConfig::default()
  });
  let mut xml_attrs = HashMap::default();
  xml_attrs.insert(String::from("class"), String::from("ltx_eqn_eqnarray"));
  // Perl: colsep => LookupDimension('\arraycolsep')->multiply(2)
  // LookupDimension (not lookup_register) so a document that `\def`s
  // `\arraycolsep` into a plain macro reads its body silently, matching Perl —
  // `lookup_register` would emit a spurious `expected:register` warning there.
  let acol = lookup_dimension_cs("\\arraycolsep", false).unwrap_or_default();
  let colsep = acol.pt_value(None) * 2.0;
  if colsep > 0.0 {
    xml_attrs.insert(String::from("colsep"), s!("{}pt", colsep));
  }
  // Perl: my $cur_jot = LookupDimension('\jot');
  //   if ($cur_jot && $cur_jot->valueOf != LookupDimension('\lx@default@jot')->valueOf)
  //     { $attributes{rowsep} = $cur_jot; }
  let cur_jot = lookup_dimension_cs("\\jot", false).unwrap_or_default();
  if cur_jot.value_of()
    != lookup_dimension_cs("\\lx@default@jot", false)
      .unwrap_or_default()
      .value_of()
  {
    xml_attrs.insert(String::from("rowsep"), cur_jot.to_string());
  }
  let mut properties = SymHashMap::default();
  properties.insert("preserve_structure", Stored::Bool(true));
  // Use custom alignment hooks for equationgroup/equation/_Capture_
  // Perl: my %attr = RefStepID('@equationgroup') — but Perl runs it inside the
  // container-open hook, i.e. at ABSORB time, after ALL digestion: every
  // group's id gets the LAST section's prefix (S3.EGx1 for a group whose own
  // rows are S2.E*). Minting HERE, at digest time, keeps the group's section
  // prefix consistent with its rows — and makes the eager and streaming
  // (interleaved) pipelines agree byte-for-byte. Sanctioned divergence from
  // Perl, user ruling 2026-07-29: OXIDIZED_DESIGN #91.
  let group_id: Option<String> = ref_step_id("@equationgroup")
    .ok()
    .and_then(|props| props.get("id").map(ToString::to_string));
  let alignment = Alignment::new(AlignmentConfig {
    template: Some(template),
    open_container: Rc::new(move |document, mut props| {
      if let Some(id) = &group_id {
        props.insert(String::from("xml:id"), id.clone());
      }
      props.insert(String::from("class"), String::from("ltx_eqn_eqnarray"));
      document
        .open_element("ltx:equationgroup", Some(props), None)
        .map(Some)
    }),
    close_container: Rc::new(|document| document.close_element("ltx:equationgroup")),
    open_row: Rc::new(|document, mut props| {
      // Perl: $$row{id} and $$row{tags} are passed via props from be_absorbed.
      // The id was stored on the row during after_equation.
      if let Some(id) = props.remove("id") {
        props.insert(String::from("xml:id"), Stored::from(id.to_string()));
      }
      // Extract tags (Digested) before converting to string props
      let tags_digested = props.remove("tags");
      let str_props: HashMap<String, String> =
        props.into_iter().map(|(k, v)| (k, v.to_string())).collect();
      document.open_element("ltx:equation", Some(str_props), None)?;
      // If we have digested tags, absorb them into the opened element
      if let Some(Stored::Digested(d)) = tags_digested {
        document.absorb(&d, None)?;
      }
      Ok(())
    }),
    close_row: Rc::new(|document| document.close_element("ltx:equation")),
    open_column: Rc::new(|document, props| {
      document
        .open_element("ltx:_Capture_", Some(props), None)
        .map(Some)
    }),
    close_column: Rc::new(|document| document.close_element("ltx:_Capture_")),
    is_math: true,
    properties,
    xml_attributes: xml_attrs,
  });
  assign_alignment(alignment, None);
  // NOTE: Perl's eqnarrayBindings does NOT set Let(T_MATH, '\lx@dollar@in@mathmode').
  // eqnarray creates the alignment directly (not through alignmentBindings),
  // so the $ tokens in its template use \lx@dollar@default — same as amsRearrangeableBindings.
  let_i(&T_CS!("\\\\"), &T_CS!("\\lx@alignment@newline"), None);
  let_i(&T_CS!("\\lx@intercol"), &T_CS!("\\lx@math@intercol"), None);
  let_i(
    &T_CS!("\\lx@alignment@row@before"),
    &T_CS!("\\eqnarray@row@before"),
    None,
  );
  let_i(
    &T_CS!("\\lx@alignment@row@after"),
    &T_CS!("\\eqnarray@row@after"),
    None,
  );
  // Perl: Let('\lx@eqnarray@save@label', '\lx@label');
  // Save the canonical \lx@label (NOT the mutable \label) as
  // \lx@eqnarray@save@label — global so the noalign-deferred
  // `\lx@eqnarray@save@label{#1}` expansion still resolves if it fires AFTER
  // the eqnarray group pops (witness 2404.19499 align case). Saving \lx@label
  // (immutable canonical) rather than \label avoids the self-recursion when
  // this binding re-runs while \label is already \lx@eqnarray@label (nested
  // align/gather, 2008.13358).
  let_i(
    &T_CS!("\\lx@eqnarray@save@label"),
    &T_CS!("\\lx@label"),
    Some(Scope::Global),
  );
  // Perl: Let('\label', '\lx@eqnarray@label');
  // Redirect \label to the noalign version so it runs at the equation (row) level
  let_i(&T_CS!("\\label"), &T_CS!("\\lx@eqnarray@label"), None);
  Ok(())
}

/// Perl: rearrangeEqnarray (latex_constructs.pool.ltxml L2356-2445)
/// Analyzes column patterns in eqnarray and rearranges into MathFork structures.
fn rearrange_eqnarray(document: &mut Document, equationgroup: &mut Node) -> Result<()> {
  use crate::base_xmath::{equationgroup_join_cols, equationgroup_join_rows};

  struct EqRow {
    node:     Node,
    cols:     Vec<Node>,
    has_l:    bool,
    has_m:    bool,
    has_r:    bool,
    numbered: bool,
    labelled: bool,
  }

  // Scan the equations (rows)
  let mut rows: Vec<EqRow> = Vec::new();
  let equation_nodes: Vec<Node> = document.findnodes("ltx:equation", Some(equationgroup));
  for rownode in equation_nodes {
    let cells: Vec<Node> = document.findnodes("ltx:_Capture_", Some(&rownode));
    let has_l = cells.first().is_some_and(|c| c.get_first_child().is_some());
    let has_m = cells.get(1).is_some_and(|c| c.get_first_child().is_some());
    let has_r = cells.get(2).is_some_and(|c| c.get_first_child().is_some());
    let numbered = !document.findnodes("ltx:tags", Some(&rownode)).is_empty();
    // OXIDIZED_DESIGN #54: Perl checks hasAttribute('label') (singular), but
    // LaTeXML only ever sets the plural 'labels' attribute (LaTeXML-common.rnc
    // L134) — so the author's documented "Separately numbered AND labeled? must
    // keep separate" safeguard below is dead code in Perl, collapsing distinctly
    // \label-ed continuation rows onto one number. We honor the intent (and match
    // pdfTeX) by reading the real 'labels' attribute. See KNOWN_PERL_ERRORS #46.
    let labelled = rownode.get_attribute("labels").is_some();
    rows.push(EqRow {
      node: rownode,
      cols: cells,
      has_l,
      has_m,
      has_r,
      numbered,
      labelled,
    });
  }

  let n_l = rows.iter().filter(|r| r.has_l).count();
  let n_m = rows.iter().filter(|r| r.has_m).count();
  let n_r = rows.iter().filter(|r| r.has_r).count();

  // Only a single column was used
  if (n_l > 0 && n_m == 0 && n_r == 0)
    || (n_l == 0 && n_m > 0 && n_r == 0)
    || (n_l == 0 && n_m == 0 && n_r > 0)
  {
    let keepcol = if n_l > 0 {
      0
    } else if n_m > 0 {
      1
    } else {
      2
    };
    // Remove empty columns (in reverse order to preserve indices)
    for c in (0..3).rev() {
      if c == keepcol {
        continue;
      }
      for row in rows.iter() {
        if let Some(col) = row.cols.get(c) {
          document.safe_unlink(col.clone());
        }
      }
    }
    // Check if any column begins with a RELOP → join rows
    let begins_with_relop = rows.iter().any(|row| {
      row
        .cols
        .get(keepcol)
        .and_then(|c| {
          c.get_child_elements()
            .into_iter()
            .next()
            .and_then(|first| first.get_attribute("role").map(|r| r == "RELOP"))
        })
        .unwrap_or(false)
    });

    if begins_with_relop {
      let nodes: Vec<Node> = rows.into_iter().map(|r| r.node).collect();
      equationgroup_join_rows(document, equationgroup, nodes)?;
    } else {
      for mut row in rows {
        equationgroup_join_cols(document, 1, &mut row.node)?;
      }
    }
    return Ok(());
  }

  // All 3 columns case — analyze continuation patterns
  let mut eqs: Vec<Vec<Node>> = Vec::new();
  let mut numbered = false;

  for row in &rows {
    let class;
    if row.has_l {
      class = "new";
    } else if row.has_m {
      if eqs.is_empty() {
        class = "odd";
      } else if numbered && row.numbered {
        class = "new";
      } else {
        class = "continue";
      }
    } else if row.has_r {
      if eqs.is_empty() || (numbered && row.numbered && row.labelled) {
        class = "odd";
      } else {
        class = "continue";
      }
    } else {
      // All columns empty
      class = "remove";
    }

    if class == "remove" {
      document.safe_unlink(row.node.clone());
    } else if class == "new" || class == "odd" {
      numbered = row.numbered;
      eqs.push(vec![row.node.clone()]);
    } else {
      // "continue"
      numbered |= row.numbered;
      if let Some(last) = eqs.last_mut() {
        last.push(row.node.clone());
      }
    }
  }

  // Now rearrange
  for eqset in eqs {
    equationgroup_join_rows(document, equationgroup, eqset)?;
  }
  Ok(())
}

fn clean_class_name(name: &str) -> String {
  name
    .trim()
    .chars()
    .filter(|c| c.is_alphanumeric())
    .collect()
}

fn stored_string_list(keys: &[&str]) -> Stored {
  let deque: VecDeque<Stored> = keys.iter().map(|k| Stored::from(k.to_string())).collect();
  Stored::VecDequeStored(deque)
}

fn init_savable_theorem_parameters(keys: Vec<&str>) {
  assign_value(
    "SAVABLE_THEOREM_PARAMETERS",
    stored_string_list(&keys),
    Some(Scope::Global),
  );
}

pub fn get_savable_keys() -> Vec<String> {
  match lookup_value("SAVABLE_THEOREM_PARAMETERS") {
    Some(Stored::VecDequeStored(keys)) => keys.iter().map(|k| k.to_string()).collect(),
    _ => vec![
      "\\thm@bodyfont".into(),
      "\\thm@headpunct".into(),
      "\\thm@styling".into(),
      "\\thm@headstyling".into(),
      "thm@swap".into(),
    ],
  }
}

pub fn set_savable_theorem_parameters(keys: Vec<&str>) {
  assign_value(
    "SAVABLE_THEOREM_PARAMETERS",
    stored_string_list(&keys),
    Some(Scope::Global),
  );
}

pub fn save_theorem_style(name: &str, saved: Vec<(String, Stored)>) {
  let key = s!("THEOREM_{name}_PARAMETERS");
  let deque: VecDeque<Stored> = saved
    .into_iter()
    .flat_map(|(k, v)| vec![Stored::from(k), v])
    .collect();
  assign_value(&key, Stored::VecDequeStored(deque), Some(Scope::Global));
}

pub fn use_theorem_style(name: &str) {
  let savable_keys = get_savable_keys();
  let params_key = s!("THEOREM_{name}_PARAMETERS");
  if let Some(Stored::VecDequeStored(params)) = lookup_value(&params_key) {
    let params_vec: Vec<Stored> = params.into_iter().collect();
    let mut i = 0;
    while i + 1 < params_vec.len() {
      let key = params_vec[i].to_string();
      let val = params_vec[i + 1].clone();
      if savable_keys.iter().any(|k| k == &key) {
        if key.starts_with('\\') {
          let tokens = match val {
            Stored::Tokens(t) => t,
            Stored::Bool(_) => {
              // bool stored for a register key — skip
              i += 2;
              continue;
            },
            // Values round-tripping through tokens — use internal cattable so
            // any `\lx@…` names re-tokenize as single CS (not `\lx`+`@…`).
            // `assembled`: the `Stored::Tokens` case is taken by the arm above,
            // so what reaches here is a scalar (string/number) rendering.
            _ => mouth::tokenize_internal(TeXString::assembled(val.to_string())),
          };
          let _ = assign_register(&key, RegisterValue::Tokens(tokens), None, vec![]);
        } else {
          assign_value(&key, val, None);
        }
      }
      i += 2;
    }
  }
}

pub fn define_new_theorem(
  flag: Option<Tokens>,
  thmset: Tokens,
  otherthmset: Option<Tokens>,
  typ: Option<Tokens>,
  within: Option<Tokens>,
  // Perl `\spnewtheorem` (llncs/sv) carries a per-theorem body font
  // (`afterDigestBegin => sub { Digest($bodyfont); }`, llncs.cls.ltxml L144):
  // e.g. `proof`/`case`/`example` pass `\rmfamily` = upright, overriding the
  // amsthm default `\thm@bodyfont` (`\itshape`). `\newtheorem`/thmtools pass
  // `None` (their body font comes from `\theoremstyle`).
  bodyfont: Option<Tokens>,
) -> Result<()> {
  let thmset_str = thmset.to_string();
  let classname = clean_class_name(&thmset_str);
  let listname = {
    let mut ln = s!("theorem:{thmset_str}");
    ln.retain(|c| !c.is_whitespace());
    ln = ln.replace('\'', "prime");
    ln = ln.replace('?', "question");
    ln = ln.replace('#', "hash");
    ln
  };
  let otherthmset_str = otherthmset
    .as_ref()
    .map(|t| t.to_string())
    .filter(|s| !s.is_empty());
  let has_type = typ.as_ref().is_some_and(|t| !t.is_empty());
  let is_starred = flag.is_some();

  let within_str = if let Some(ref w) = within {
    let ws = digest_literal(w.clone())?.to_string();
    if ws.is_empty() { None } else { Some(ws) }
  } else {
    None
  };

  let counter = otherthmset_str
    .clone()
    .unwrap_or_else(|| thmset_str.clone());
  let counter = counter.replace(' ', ".");

  // If counter != thmset, record mapping
  if counter != thmset_str {
    AssignMapping!("counter_for_type", &thmset_str => &counter);
    DefMacro!(
      T_CS!(s!("\\the{thmset_str}")),
      None,
      Some(ExpansionBody::Tokens(Tokens::new(vec![T_CS!(s!("\\the{counter}"))]))),
      scope => Some(Scope::Global)
    );
  }

  let numbering = {
    let reg = LookupRegister!("\\thm@numbering");
    if let RegisterValue::Tokens(t) = reg {
      t.to_string()
    } else {
      "\\arabic".into()
    }
  };

  let is_starred = is_starred || numbering.is_empty();

  if otherthmset_str.is_none() {
    let idprefix = s!("Thm{}", classname.replace('*', "."));
    let c_counter = s!("\\c@{counter}");
    if !is_defined(&c_counter) {
      let within_ref = within_str.as_deref().unwrap_or("");
      NewCounter!(&counter, within_ref, idprefix => &idprefix);
    }
    // Define \the<counter>
    if !numbering.is_empty() {
      let the_counter_body = if let Some(ref w) = within_str {
        s!("\\csname the{w}\\endcsname\\@thmcountersep{numbering}{{{counter}}}")
      } else {
        s!("{numbering}{{{counter}}}")
      };
      DefMacro!(
        T_CS!(s!("\\the{counter}")),
        None,
        Some(ExpansionBody::Tokens(mouth::tokenize_internal(
          TeXString::assembled(the_counter_body)
        ))),
        scope => Some(Scope::Global)
      );
    }
  }

  // Save current theorem style params for this theorem name
  let savable_keys = get_savable_keys();
  let mut saved_params: Vec<(String, Stored)> = Vec::new();
  for key in &savable_keys {
    if key.starts_with('\\') {
      let reg = LookupRegisterOrDefault!(key);
      let tokens = match reg {
        RegisterValue::Tokens(t) => t,
        _ => Tokens!(),
      };
      // Perl `\spnewtheorem`'s `afterDigestBegin => Digest($bodyfont)` applies
      // a per-theorem body font that overrides the amsthm default. Stage it
      // into the saved `\thm@bodyfont` so `use_theorem_style` restores it at
      // body-digest start (proof/case/... => `\rmfamily` = upright).
      let tokens = if key == "\\thm@bodyfont" {
        bodyfont
          .as_ref()
          .filter(|t| !t.is_empty())
          .cloned()
          .unwrap_or(tokens)
      } else {
        tokens
      };
      saved_params.push((key.clone(), Stored::Tokens(tokens)));
    } else {
      let val = lookup_value(key).unwrap_or(Stored::None);
      saved_params.push((key.clone(), val));
    }
  }
  save_theorem_style(&thmset_str, saved_params);

  // Define \lx@name@<thmset>
  let thmname_cs = s!("\\lx@name@{thmset_str}");
  if has_type {
    let type_tokens = typ.clone().unwrap();
    DefMacro!(
      T_CS!(&thmname_cs),
      None,
      Some(ExpansionBody::Tokens(type_tokens)),
      scope => Some(Scope::Global)
    );
  } else {
    DefMacro!(
      T_CS!(&thmname_cs),
      None,
      Some(ExpansionBody::Tokens(Tokens!())),
      scope => Some(Scope::Global)
    );
  }

  // Read swap value
  let swap = lookup_value("thm@swap")
    .map(|v| match v {
      Stored::Int(n) => n != 0,
      Stored::Bool(b) => b,
      _ => false,
    })
    .unwrap_or(false);

  // Define \fnum@<thmset>
  let fnum_cs = s!("\\fnum@{thmset_str}");
  let fnum_tokens = if is_starred || counter.is_empty() {
    Tokens::new(vec![T_CS!(&thmname_cs)])
  } else if swap {
    let mut toks = vec![T_CS!(s!("\\the{counter}"))];
    if has_type {
      toks.push(T_SPACE!());
    }
    toks.push(T_CS!(&thmname_cs));
    Tokens::new(toks)
  } else {
    let mut toks = vec![T_CS!(&thmname_cs)];
    if has_type {
      toks.push(T_SPACE!());
    }
    toks.push(T_CS!(s!("\\the{counter}")));
    Tokens::new(toks)
  };
  DefMacro!(
    T_CS!(&fnum_cs),
    None,
    Some(ExpansionBody::Tokens(fnum_tokens)),
    scope => Some(Scope::Global)
  );

  // Define \format@title@<thmset>
  let format_title_cs = s!("\\format@title@{thmset_str}");
  let headformatter = LookupRegisterOrDefault!("\\thm@headformatter");
  let headformatter_tokens = match headformatter {
    RegisterValue::Tokens(t) => t,
    _ => Tokens!(),
  };

  let format_cs_token = T_CS!(&format_title_cs);
  if !headformatter_tokens.is_empty() {
    // amsthm-style head formatter
    let mut fmt_toks = vec![T_CS!("\\the"), T_CS!("\\thm@headfont")];
    fmt_toks.extend(headformatter_tokens.unlist());
    fmt_toks.push(T_BEGIN!());
    if has_type {
      fmt_toks.extend(typ.unwrap().unlist());
    }
    fmt_toks.push(T_END!());
    fmt_toks.push(T_CS!(s!("\\the{counter}")));
    fmt_toks.push(T_BEGIN!());
    fmt_toks.push(T_PARAM!());
    fmt_toks.push(T_OTHER!("1"));
    fmt_toks.push(T_END!());
    fmt_toks.push(T_CS!("\\the"));
    fmt_toks.push(T_CS!("\\thm@headpunct"));

    let params = parse_parameters("{}", &format_cs_token, true)?;
    DefMacro!(
      format_cs_token,
      params,
      Some(ExpansionBody::Tokens(Tokens::new(fmt_toks))),
      scope => Some(Scope::Global)
    );
  } else {
    // Standard format
    let note_part = if has_type {
      "\\ifx.#1.\\else\\space\\the\\thm@notefont(#1)\\fi"
    } else {
      "#1"
    };
    let fmt_str = s!(
      // The `{}` after `\endcsname` is OXIDIZED_DESIGN #85 — see `\lx@fnum@@`
      // in `base_utilities.rs`. A theorem set's `\fnum@<set>` reaches the same
      // trap: an author redefinition that takes an argument (to eat LaTeX's
      // separator token) otherwise scans past the hook and swallows the tag
      // group's closing brace.
      "{{\\the\\thm@headfont\\lx@tag{{\\csname fnum@{thmset_str}\\endcsname{{}}}}{{{note_part}}}\\the\\thm@headpunct}}"
    );
    let params = parse_parameters("{}", &format_cs_token, true)?;
    DefMacro!(
      format_cs_token,
      params,
      Some(ExpansionBody::Tokens(mouth::tokenize_internal(
        TeXString::assembled(fmt_str)
      ))),
      scope => Some(Scope::Global)
    );
  }

  // Define the environment.
  //
  // The env-trigger key MUST match what `\begin{<name>}`/`\end{<name>}`
  // look up, which is `ToString(Expand(<name>))` (see `\begin{}` above and
  // Perl LaTeX.pool L164). `\newenvironment` likewise registers under
  // `Expand!(name).to_string()`. But `thmset_str` (used for classname /
  // listname / counter / ids) is the *raw* `thmset.to_string()` — matching
  // Perl `defineNewTheorem` L3054 `ToString($thmset)`. These two agree for
  // Perl because Perl drops invalid input bytes (an undeclared Latin-1 `é`
  // never becomes a char, so raw == expanded == "df"). We preserve the
  // `é` (better output: café stays café), so for a theorem env whose name
  // carries an *active* accented char — e.g. `\newtheorem{déf}` in a
  // Latin-1 source under `[T1]{fontenc}`, where t1enc.dfu's
  // `\DeclareUnicodeCharacter{00E9}` makes `é` active → `\'e` →
  // `\lx@applyaccent…` — the raw name ("déf") and the `\begin`-expanded
  // name ("d\lx@applyaccent\'{e}f") diverge, and `\begin{déf}` fails to
  // find the env ("environment is not defined"). Expanding the trigger key
  // (only the key — classname/ids stay raw & clean) restores the match.
  // No-op for ASCII names. Witness: arXiv:1509.06785.
  let thmset_for_env = Expand!(thmset).to_string();

  // Hand-written replacement closure (compile_replacement! only works with literals)
  let inlist_val = s!("thm {listname}");
  let class_val = s!("ltx_theorem_{classname}");
  let compiled_replacement: Option<ReplacementClosure> = Some(Rc::new(
    move |document: &mut Document, _args: &Vec<Option<Digested>>, props: &SymHashMap<Stored>| {
      let mut av_props: HashMap<String, String> = HashMap::default();
      if let Some(stored) = props.get("id") {
        av_props.insert("xml:id".into(), stored.to_string());
      }
      av_props.insert("inlist".into(), inlist_val.clone());
      av_props.insert("class".into(), class_val.clone());
      let this_font_opt = match props.get("font") {
        Some(Stored::Font(f)) => Some(Cow::Borrowed(&**f)),
        Some(Stored::FontDirective(FontDirective::Asset(fa))) => Some(Cow::Borrowed(&**fa)),
        Some(Stored::FontDirective(FontDirective::Closure(code))) => Some(Cow::Owned(code(None)?)),
        _ => None,
      };
      if let Some(this_font) = this_font_opt {
        document.open_element("ltx:theorem", Some(av_props), Some(&this_font))?;
      } else {
        document.open_element("ltx:theorem", Some(av_props), None)?;
      }
      // #tags
      if let Some(stored_digested) = props.get("tags") {
        let digested_opt: Option<Digested> = stored_digested.into();
        if let Some(ref digested) = digested_opt {
          document.absorb(digested, None)?;
        }
      }
      // <ltx:title font='#titlefont' _force_font='true'>#title</ltx:title>
      let mut title_av: HashMap<String, String> = HashMap::default();
      if let Some(stored) = props.get("titlefont") {
        title_av.insert("font".into(), stored.to_string());
      }
      title_av.insert("_force_font".into(), "true".into());
      let title_font_opt = match props.get("titlefont") {
        Some(Stored::Font(f)) => Some(Cow::Borrowed(&**f)),
        Some(Stored::FontDirective(FontDirective::Asset(fa))) => Some(Cow::Borrowed(&**fa)),
        Some(Stored::FontDirective(FontDirective::Closure(code))) => Some(Cow::Owned(code(None)?)),
        _ => None,
      };
      if let Some(title_font) = title_font_opt {
        document.open_element("ltx:title", Some(title_av), Some(&title_font))?;
      } else {
        document.open_element("ltx:title", Some(title_av), None)?;
      }
      if let Some(stored_digested) = props.get("title") {
        let digested_opt: Option<Digested> = stored_digested.into();
        if let Some(ref digested) = digested_opt {
          document.absorb(digested, None)?;
        }
      }
      document.close_element("ltx:title")?;
      // #body
      if let Some(stored_digested) = props.get("body") {
        let digested_opt: Option<Digested> = stored_digested.into();
        if let Some(ref digested) = digested_opt {
          document.absorb(digested, None)?;
        }
      }
      // #trailer — `set_body` (whatsit.rs) pops the LAST captured box off the
      // body and stashes it as the `trailer`. For a well-formed
      // `\begin{thm}…\end{thm}` that trailer is the content-less `\end{thm}`
      // whatsit (empty replacement → no-op absorb). But when a `\newtheorem`
      // command is used BARE in a brace group — `{\lem … }` with no `\end{lem}`
      // (common for inference/lemma figures) — the body capture over-captures
      // the *following* document content (the next `{\cor …}`, sections, …) and
      // it all lands in that last box, so the trailer holds real content. The
      // replacement absorbed only `#body`, silently dropping it (witness
      // 1905.00186: ~90 % of the document lost). Absorbing the trailer recovers
      // it (Perl keeps the content; both engines over-capture identically).
      if let Some(stored_digested) = props.get("trailer") {
        let digested_opt: Option<Digested> = stored_digested.into();
        if let Some(ref digested) = digested_opt {
          document.absorb(digested, None)?;
        }
      }
      Ok(())
    },
  ));

  // `thmset_for_before` is for the before_digest closure; clone needed
  // because `thmset_str` is moved into `thmset_for_tags` below.
  let thmset_for_before = thmset_str.clone();
  // `thmset_for_tags` and `counter_for_tags` are the last uses of
  // thmset_str and counter — move instead of clone.
  let thmset_for_tags = thmset_str;
  let counter_for_tags = counter;
  let is_starred_for_props = is_starred;
  let has_type_for_props = has_type;

  let mut options = ConstructorOptions {
    mode: Some("internal_vertical".into()),
    scope: Some(Scope::Global),
    ..Default::default()
  };

  // before_digest
  let before_digest_closure: BeforeDigestClosure = Rc::new(move || {
    use_theorem_style(&thmset_for_before);
    let digested = digest(mouth::tokenize_internal("\\normalfont\\the\\thm@prework"))?;
    Ok(vec![digested])
  });
  options.before_digest.push(before_digest_closure);

  // after_digest_begin
  let after_digest_begin_closure: DigestionClosure = Rc::new(move |whatsit| {
    let name_opt = whatsit.get_arg(1);
    let name_str = name_opt
      .map(|n| n.revert().map(|t| t.to_string()).unwrap_or_default())
      .unwrap_or_default();
    let digest_str = s!("\\the\\thm@bodyfont\\the\\thm@styling\\def\\lx@thistheorem{{{name_str}}}");
    let digested = digest(mouth::tokenize_internal(TeXString::assembled(digest_str)))?;
    Ok(vec![digested])
  });
  options.after_digest_begin.push(after_digest_begin_closure);

  // before_digest_end
  let before_digest_end_closure: BeforeDigestClosure = Rc::new(move || {
    let digested = digest(mouth::tokenize_internal(
      "\\thm@doendmark\\the\\thm@postwork",
    ))?;
    Ok(vec![digested])
  });
  options.before_digest_end.push(before_digest_end_closure);

  // after_construct
  let after_construct_closure: ConstructionClosure =
    Rc::new(move |document: &mut Document, _whatsit: &Whatsit| {
      document.maybe_close_element("ltx:theorem")?;
      Ok(())
    });
  options.after_construct.push(after_construct_closure);

  // properties — capture thmset_for_tags / counter_for_tags by move.
  let props_closure: PropertiesClosure = Rc::new(
    #[allow(clippy::ptr_arg)]
    move |args: &Vec<Option<Digested>>| {
      let mut props = SymHashMap::default();

      if !counter_for_tags.is_empty() {
        if is_starred_for_props {
          let ctr_props = ref_step_id(&counter_for_tags)?;
          for (k, v) in ctr_props.iter() {
            props.insert_sym(*k, v.clone());
          }
          // For starred theorems with a type, create tags without the counter number
          if has_type_for_props {
            let tag_tokens = Tokens::new(vec![
              T_BEGIN!(),
              T_CS!("\\let"),
              T_CS!(s!("\\the{}", counter_for_tags)),
              T_CS!("\\@empty"),
              T_CS!("\\lx@make@tags"),
              T_BEGIN!(),
            ]);
            let mut full_toks = tag_tokens.unlist();
            full_toks.extend(
              mouth::tokenize_internal(TeXString::assembled(thmset_for_tags.clone())).unlist(),
            );
            full_toks.push(T_END!());
            full_toks.push(T_END!());
            let tags = digest(Tokens::new(full_toks))?;
            props.insert("tags", tags.into());
          }
        } else {
          let ctr_props = ref_step_counter(&thmset_for_tags, false)?;
          for (k, v) in ctr_props.iter() {
            props.insert_sym(*k, v.clone());
          }
        }
      }

      // Compute title
      let format_title_cs = s!("\\format@title@{}", thmset_for_tags);
      let mut title_tokens = vec![
        T_BEGIN!(),
        T_CS!("\\the"),
        T_CS!("\\thm@headstyling"),
        T_CS!(&format_title_cs),
        T_BEGIN!(),
      ];
      if let Some(Some(arg)) = args.first() {
        title_tokens.extend(arg.revert()?.unlist());
      }
      title_tokens.push(T_END!());
      title_tokens.push(T_END!());

      let title = digest_text(Tokens::new(title_tokens))?;
      let titlefont = title.get_font()?.map(|f| (*f).clone());
      props.insert("title", title.into());
      if let Some(f) = titlefont {
        props.insert("titlefont", Stored::Font(Rc::new(f)));
      }

      Ok(props)
    },
  );
  options.properties = props_closure;

  // Use the OptionalUndigested parameter
  let env_cs = T_CS!(s!("\\begin{{{thmset_for_env}}}"));
  let paramlist = parse_parameters("OptionalUndigested", &env_cs, true)?;
  def_environment(thmset_for_env, paramlist, compiled_replacement, options);

  Ok(())
}

/// Perl: beforeFloat (latex_constructs.pool.ltxml L3430-3438)
/// Sets \@captype, adjusts \hsize for single/double column floats.
/// `preincrement`: if Some("figure"), pre-increments the parent float counter
///   on first subfloat entry (before main caption), storing result for later use.
pub fn before_float(float_type: &str, preincrement: Option<&str>) {
  before_float_ex(float_type, preincrement, false);
}
/// Extended version with `double` flag for `*` variants (span both columns).
pub fn before_float_ex(float_type: &str, preincrement: Option<&str>, double: bool) {
  def_macro(
    T_CS!("\\@captype"),
    None,
    Tokens::new(ExplodeText!(float_type)),
    None,
  )
  .ok();
  // Perl #2775: rebind \\ to \lx@newline in floats to prevent
  // alignment-token early-return when floats are inside tabulars.
  Let!("\\\\", "\\lx@newline");
  // Perl: AssignRegister('\hsize' => LookupDimension($options{double} ? '\textwidth' :
  // '\columnwidth'));
  let dim_name = if double {
    "\\textwidth"
  } else {
    "\\columnwidth"
  };
  let dim_val = lookup_dimension(dim_name).unwrap_or_default();
  assign_register("\\hsize", dim_val.into(), None, Vec::new()).ok();
  // Perl: if (my $main = $options{preincrement}) {
  //   if (($type ne (LookupValue('LAST_FLOATTYPE') || ''))
  //     && !IfCondition('\iflx@donecaption')) {
  //     AssignValue('PREINCREMENTED_' . $main => { RefStepCounter($main) }, 'global'); } }
  if let Some(main_counter) = preincrement {
    let last_type = lookup_value("LAST_FLOATTYPE")
      .map(|s| s.to_string())
      .unwrap_or_default();
    let done_caption = if_condition(&T_CS!("\\iflx@donecaption"))
      .unwrap_or(None)
      .unwrap_or(false);
    if float_type != last_type
      && !done_caption
      && let Ok(props) = ref_step_counter(main_counter, false)
    {
      let prekey = s!("PREINCREMENTED_{main_counter}");
      assign_value(&prekey, props, Some(Scope::Global));
    }
  }
}
/// Perl: afterFloat (latex_constructs.pool.ltxml L3440-3448)
/// Rescues caption counters into the whatsit properties.
pub fn after_float(whatsit: &mut Whatsit) {
  let captype = digest(T_CS!("\\@captype"))
    .map(|d| d.to_string())
    .unwrap_or_default();
  // Perl: AssignValue('PREINCREMENTED_' . $type => undef, 'global');
  let prekey = s!("PREINCREMENTED_{captype}");
  remove_value(&prekey);
  // Perl L3389: $whatsit->setProperty(floatwidth => LookupRegister('\hsize'));
  // Capture \hsize as it stands at the END of the float's digestion (its
  // column/textwidth). `arrange_panels` reads this back from the box in the
  // afterClose hook to size the per-row layout — NOT the ambient \hsize at
  // construction time, which may have been restored to an unrelated value.
  if let Ok(Some(hsize)) = lookup_register("\\hsize", Vec::new()) {
    // geometry SVG-scope (geometry_sty / OXIDIZED_DESIGN #99): when the page
    // geometry defined a wider text width than the class default (`\Gm@tw`) and
    // this float spans the full text width, size the PANEL ARRANGEMENT to that
    // width. The panels are geometry-sized SVG pictures (their `\linewidth` was
    // raised inside the picture), so the per-row overflow threshold must be on
    // the same basis or two `0.495\linewidth` boxes that sit side-by-side in the
    // PDF would wrap to separate rows. Only the arrangement threshold changes —
    // the float's HTML content keeps the class-default width.
    let mut floatwidth = hsize.clone();
    if let (Ok(Some(gm_tw)), Ok(Some(doctw))) = (
      lookup_register("\\Gm@tw", Vec::new()),
      lookup_register("\\Gm@doctw", Vec::new()),
    ) {
      // Full-column float only: compare against the class-default column width
      // \Gm@doctw (captured at geometry load), not the live \textwidth — a
      // minipage sets \textwidth=\linewidth locally, so the live compare would
      // mis-fire inside one.
      let (h, g, d) = (hsize.value_of(), gm_tw.clone().value_of(), doctw.value_of());
      if h == d && g > h {
        floatwidth = gm_tw;
      }
    }
    whatsit.set_property("floatwidth", Stored::from(floatwidth));
  }
  rescue_caption_counters(&captype, whatsit);
  assign_value(
    "LAST_FLOATTYPE",
    Stored::String(pin(&captype)),
    Some(Scope::Global),
  );
}
/// Is `qname` a "panel break" element (Perl `figure_panel_break_names`,
/// L3245-3251) — a break/caption/metadata child that flushes the current row
/// rather than being a panel itself?
fn is_panel_break_name(qname: SymStr) -> bool {
  with(qname, |name| {
    matches!(
      name,
      "ltx:break"
        | "ltx:caption"
        | "ltx:toccaption"
        | "ltx:title"
        | "ltx:toctitle"
        | "ltx:subtitle"
        | "ltx:creator"
        | "ltx:contact"
        | "ltx:date"
        | "ltx:tags"
        | "ltx:classification"
        | "ltx:acknowledgements"
        | "ltx:resource"
        | "ltx:navigation"
    )
  })
}

/// Perl `%standalone_panel_names` (L3227-3229): block-level elements expected to
/// sit alone on their own row within a figure.
fn is_standalone_panel_name(qname: SymStr) -> bool {
  with(qname, |name| {
    matches!(
      name,
      "ltx:p"
        | "ltx:listing"
        | "ltx:math"
        | "ltx:itemize"
        | "ltx:enumerate"
        | "ltx:quote"
        | "ltx:theorem"
        | "ltx:proof"
        | "ltx:description"
        | "ltx:equation"
        | "ltx:equationgroup"
        | "ltx:verbatim"
    )
  })
}

/// Width (in scaled points) of the box that produced `node`, or 0 if unknown.
/// Perl: `$document->getNodeBox($child)->getWidth->valueOf()`. When the node has
/// no tracked box width (e.g. a minipage/parbox, whose set width rides on the
/// element's `width` ATTRIBUTE rather than a box property in our tree), fall back
/// to that attribute — otherwise the panel reads as zero-width and gets spuriously
/// merged into its neighbour (figure_grids minipage grids).
fn panel_width(document: &Document, node: &Node) -> f64 {
  let box_width = document
    .get_node_box(node)
    .and_then(|b| b.get_width(None).ok().flatten())
    .map(|r| r.value_of() as f64)
    .unwrap_or(0.0);
  if box_width > 0.0 {
    return box_width;
  }
  node
    .get_attribute("width")
    .and_then(|w| Dimension::spec_to_f64(&w).ok())
    .unwrap_or(0.0)
}

/// Width (scaled points) of the sole `ltx:graphics` descendant of a figure/table
/// panel, or `None` unless there is exactly one. `\subcaptionbox` and subfig
/// `\subfloat` panels carry no explicit `{width}`, so their box is sized to the
/// full float `\hsize` — which hides the panel's real content width from
/// [`arrange_panels`] and stacks the panels one-per-row. Sizing such a panel to
/// its lone graphic lets siblings share a row, the way an explicit-width
/// `{subfigure}{0.48\linewidth}` already does (#6903). Ambiguous panels (0 or >1
/// graphic) return `None`, so the caller keeps the box width unchanged.
fn sole_graphic_width(document: &Document, node: &Node) -> Option<f64> {
  let qname = document::get_node_qname(node);
  if qname != pin!("ltx:figure") && qname != pin!("ltx:table") {
    return None;
  }
  let graphics_qname = pin!("ltx:graphics");
  let mut found: Option<Node> = None;
  let mut stack = node.get_child_elements();
  while let Some(n) = stack.pop() {
    if document::get_node_qname(&n) == graphics_qname {
      if found.is_some() {
        return None; // >1 graphic — don't guess which sizes the panel
      }
      found = Some(n);
    } else {
      stack.extend(n.get_child_elements());
    }
  }
  document
    .get_node_box(&found?)
    .and_then(|b| b.get_width(None).ok().flatten())
    .map(|r| r.value_of() as f64)
    .filter(|w| *w > 0.0)
}

/// Insert a `<ltx:break class="ltx_break"/>` immediately before `child`.
fn insert_break_before(document: &Document, child: &mut Node) -> Result<Node> {
  let ns = child.get_namespace();
  let mut break_node = Node::new("break", ns, document.get_document())
    .map_err(|e| format!("could not create ltx:break node: {e:?}"))?;
  break_node.set_attribute("class", "ltx_break").ok();
  child
    .add_prev_sibling(&mut break_node)
    .map_err(|e| format!("could not insert ltx:break: {e:?}"))?;
  Ok(break_node)
}

/// Perl L3231: `($whatsit->getProperty('floatwidth') || Dimension('345pt'))->valueOf()`.
/// The row-layout threshold is the float's captured `\hsize` (see `after_float`),
/// read from the box the afterClose hook receives; 345pt when absent.
fn float_width_of(whatsit: Option<&Digested>) -> f64 {
  whatsit
    .and_then(|w| w.get_property("floatwidth"))
    .and_then(|s| Option::<RegisterValue>::from(s.as_ref()))
    .map(|r| r.value_of() as f64)
    .filter(|w| *w > 0.0)
    .unwrap_or(345.0 * 65536.0)
}

/// Faithful port of Perl's `arrange_panels_and_breaks`
/// (latex_constructs.pool.ltxml L3229-3349): partition a figure/table/float's
/// children into rows, inserting `<ltx:break>` where the accumulated panel WIDTH
/// would overflow the float width — mirroring the PDF's per-row arrangement
/// rather than trusting the source's line/paragraph structure. Author-deposited
/// breaks and captions/metadata flush the current row. Panels are marked with
/// `ltx_figure_panel` only when the figure has more than one.
fn arrange_panels(document: &mut Document, node: &mut Node, float_width: f64) -> Result<()> {
  // Perl L3233: 0.03125x min width => at most 32 panels per row.
  let min_panel_width = 0.03125 * float_width;

  let note_qname = pin!("ltx:note");
  let caption_qname = pin!("ltx:caption");
  let block_qname = pin!("ltx:block");

  let mut current_width: f64 = 0.0;
  let mut all_panels: Vec<Node> = Vec::new();
  // Bookkeeping triples for the current row: (node, qname, width).
  let mut row: Vec<(Node, SymStr, f64)> = Vec::new();

  for mut child in node.get_child_elements() {
    let child_name = document::get_node_qname(&child);

    // Perl L3260-3267: move a top-level ltx:note to the nearest caption sibling.
    if child_name == note_qname {
      let sibling_caption = node
        .get_child_elements()
        .into_iter()
        .find(|c| document::get_node_qname(c) == caption_qname);
      if let Some(mut cap) = sibling_caption {
        child.unlink_node();
        cap.add_child(&mut child).ok();
      }
      continue;
    }

    if is_panel_break_name(child_name) {
      // Perl L3269-3275: a break/caption/meta flushes the current row.
      current_width = 0.0;
      row.clear();
      continue;
    }

    // Perl L3277-3284: a standalone block on its own row — break first.
    if is_standalone_panel_name(child_name) && !row.is_empty() {
      insert_break_before(document, &mut child)?;
      current_width = 0.0;
      row.clear();
    }

    let mut child_width = panel_width(document, &child);
    // #6903: a subcaptionbox / subfig \subfloat panel is sized to the full float
    // \hsize (it has no explicit `{width}`), so it would take its own row. When
    // it wraps a single narrower graphic, size it to that graphic so sibling
    // panels share a row — an explicit-width `{subfigure}{W}` already does.
    if child_width >= float_width
      && let Some(inner) = sole_graphic_width(document, &child)
      && inner < child_width
    {
      child_width = inner;
    }

    if !row.is_empty() && (current_width + child_width > float_width) {
      // Perl L3287-3295: row overflow — break before this child, start a new row.
      insert_break_before(document, &mut child)?;
      document.add_class(&mut child, "ltx_figure_panel")?;
      all_panels.push(child.clone());
      row = vec![(child, child_name, child_width)];
      current_width = child_width;
    } else if let Some((mut prev_node, prev_name, prev_width0)) = row.pop() {
      // Perl L3296-3330: try to merge into the previous panel, else append.
      let prev_width = if prev_width0 > 0.0 {
        prev_width0
      } else {
        panel_width(document, &prev_node)
      };
      let big_disparity = prev_width > 0.0
        && child_width > 0.0
        && (prev_width.max(child_width) / prev_width.min(child_width) > 8.0);
      // #2709: the merge groups small/disparate content into an `ltx:block`, but
      // the schema forbids a float (`ltx:figure`/`ltx:table`, or any future
      // `ltx:float`) as a block child — wrapping one yields an invalid
      // `<block><figure/></block>`. Ask the MODEL (not a hard-coded name list)
      // whether the block can hold what would go into it, per merge branch; if
      // not, keep the panels as siblings. Minipage grids stay valid block content
      // and are unaffected.
      let merge_is_valid = if prev_name == block_qname {
        model::can_contain_sym(block_qname, child_name)
      } else if child_name == block_qname {
        model::can_contain_sym(block_qname, prev_name)
      } else {
        model::can_contain_sym(block_qname, prev_name)
          && model::can_contain_sym(block_qname, child_name)
      };
      if merge_is_valid
        && (child_width == 0.0 || big_disparity || (prev_width + child_width < min_panel_width))
      {
        // Perl L3312-3325: contain the two pieces in a single ltx:block panel.
        let merged_width = prev_width + child_width;
        if prev_name == block_qname {
          child.unlink_node();
          prev_node.add_child(&mut child).ok();
          row.push((prev_node, prev_name, merged_width));
        } else if child_name == block_qname {
          prev_node.unlink_node();
          child.add_child(&mut prev_node).ok();
          all_panels.push(child.clone());
          row.push((child, child_name, merged_width));
        } else if let Some(block) = document.wrap_nodes("ltx:block", vec![prev_node, child])? {
          all_panels.pop();
          all_panels.push(block.clone());
          row.push((block, block_qname, merged_width));
        }
      } else {
        // Perl L3327-3330: keep the previous panel, append this one as a sibling.
        row.push((prev_node, prev_name, prev_width));
        all_panels.push(child.clone());
        row.push((child, child_name, child_width));
      }
      current_width += child_width;
    } else {
      // Perl L3331-3333: no previous panel in the row — just add this child.
      if child_width > 0.0 {
        all_panels.push(child.clone());
      }
      if is_standalone_panel_name(child_name) {
        // Perl L3334-3342: a standalone panel as the sole row content flushes the
        // row and forces a break before the next sibling (unless that sibling is
        // itself a break/caption/meta), so subsequent content starts a new row.
        if let Some(mut trailer) = child.get_next_sibling() {
          let trailer_name = document::get_node_qname(&trailer);
          if !is_panel_break_name(trailer_name) {
            insert_break_before(document, &mut trailer)?;
          }
        }
      } else {
        row.push((child, child_name, child_width));
      }
      // Perl L3343: $current_width += $child_width runs for both sub-branches.
      // The row is already empty here (so current_width is 0), meaning this just
      // seeds the accumulator with the child's width — matching Perl's 0 + width.
      current_width += child_width;
    }
  }

  // Perl L3346-3348: only mark panels when the figure is complex (>1 panel).
  if all_panels.len() > 1 {
    for panel in &mut all_panels {
      document.add_class(panel, "ltx_figure_panel")?;
    }
  }
  Ok(())
}
/// Perl: collapseFloat (latex_constructs.pool.ltxml L3493-3520)
/// If a figure/table/float contains exactly one inner float child,
/// and they don't BOTH have captions, collapse the inner into the outer.
fn collapse_float(document: &mut Document, float: &mut Node) -> Result<()> {
  let caption_qname = pin!("ltx:caption");
  let figure_qname = pin!("ltx:figure");
  let table_qname = pin!("ltx:table");
  let float_qname = pin!("ltx:float");
  // Find inner float/figure/table children
  let mut inners: Vec<Node> = Vec::new();
  for child in float.get_child_elements() {
    let qname = document::get_node_qname(&child);
    if qname == figure_qname || qname == table_qname || qname == float_qname {
      inners.push(child);
    }
  }
  if inners.len() != 1 {
    return Ok(());
  }
  let inner = inners.into_iter().next().unwrap();
  // Check captions: collapse only if they don't BOTH have captions
  let outer_has_caption = float
    .get_child_elements()
    .iter()
    .any(|c| document::get_node_qname(c) == caption_qname);
  let inner_has_caption = inner
    .get_child_elements()
    .iter()
    .any(|c| document::get_node_qname(c) == caption_qname);
  if outer_has_caption && inner_has_caption {
    return Ok(());
  }
  // Copy inner's attributes to outer (except xml:id)
  let attrs = inner.get_attributes();
  for (name, value) in &attrs {
    // get_attributes() may return the key as "id" (local name) or "xml:id" (prefixed)
    if name != "xml:id" && name != "id" {
      document.set_attribute(float, name, value)?;
    }
  }
  // If inner has caption, promote inner's xml:id to outer
  if inner_has_caption {
    let inner_id = inner
      .get_attribute("xml:id")
      .or_else(|| inner.get_attribute_ns("id", "http://www.w3.org/XML/1998/namespace"));
    if let Some(id) = inner_id {
      // Unrecord the outer's old ID and remove the attribute before setting the new one
      if let Some(old_id) = float.get_attribute_ns("id", "http://www.w3.org/XML/1998/namespace") {
        document.unrecord_id(&old_id);
      }
      float.remove_attribute("xml:id").ok();
      document.unrecord_id(&id);
      document.set_attribute(float, "xml:id", &id)?;
    }
  }
  // Replace inner element with its children (unwrap inner)
  let children: Vec<Node> = inner.get_child_nodes();
  for mut child in children {
    child.unlink_node();
    float.add_child(&mut child).ok();
  }
  document.safe_unlink(inner);
  Ok(())
}

/// Perl: tabbingBindings() — sets up alignment with repeated template and rebinds control chars
fn tabbing_bindings() -> Result<()> {
  // Template: repeated column with before=\lx@text@intercol, after=\hfil\lx@text@intercol
  let col = Cell {
    before: Some(Tokens::new(vec![T_CS!("\\lx@text@intercol")])),
    after: Some(Tokens::new(vec![
      T_CS!("\\hfil"),
      T_CS!("\\lx@text@intercol"),
    ])),
    empty: true,
    ..Cell::default()
  };
  let template = Template::new(TemplateConfig {
    repeated: vec![col],
    ..TemplateConfig::default()
  });

  let mut xml_attrs = HashMap::default();
  xml_attrs.insert(String::from("class"), String::from("ltx_tabbing"));

  let alignment = Alignment::new(AlignmentConfig {
    template:        Some(template),
    open_container:  Rc::new(|document, props| {
      document
        .open_element("ltx:tabular", Some(props), None)
        .map(Some)
    }),
    close_container: Rc::new(|document| document.close_element("ltx:tabular")),
    open_row:        Rc::new(|document, props| {
      let str_props: HashMap<String, String> =
        props.into_iter().map(|(k, v)| (k, v.to_string())).collect();
      document
        .open_element("ltx:tr", Some(str_props), None)
        .and(Ok(()))
    }),
    close_row:       Rc::new(|document| document.close_element("ltx:tr")),
    open_column:     Rc::new(|document, props| {
      document.open_element("ltx:td", Some(props), None).map(Some)
    }),
    close_column:    Rc::new(|document| document.close_element("ltx:td")),
    is_math:         false,
    properties:      SymHashMap::default(),
    xml_attributes:  xml_attrs,
  });
  assign_alignment(alignment, None);

  // Rebind control characters within tabbing
  // Perl: Let("\\=", '\@tabbing@tabset') etc.
  // latex.ltx:10005 `\@tabacckludge#1` = `\@changed@cmd\csname\string#1
  // \endcsname\relax` recovers the ENCODING-level accent by name, so `\a=`,
  // `\a<`, `\a>` still accent although `\=`/`\<`/`\>` are tab operators
  // inside tabbing (encguide `\a=o`; greek-fontenc test-lgrenc/textalpha-doc
  // `\a<`; Perl pool:3572 saves only `'` and `` ` ``). Save every rebound
  // accent BEFORE rebinding it. Guard:
  // `perfect_kernel_batch54::tabbing_accent_kludge_recovers_rebound_accents`.
  let_i(&T_CS!("\\@tabbing@="), &T_CS!("\\="), None);
  let_i(&T_CS!("\\@tabbing@<"), &T_CS!("\\<"), None);
  let_i(&T_CS!("\\@tabbing@>"), &T_CS!("\\>"), None);
  let_i(&T_CS!("\\="), &T_CS!("\\@tabbing@tabset"), None);
  let_i(&T_CS!("\\>"), &T_CS!("\\@tabbing@nexttab"), None);
  let_i(&T_CS!("\\\\"), &T_CS!("\\@tabbing@newline"), None);
  let_i(&T_CS!("\\kill"), &T_CS!("\\@tabbing@kill"), None);
  let_i(&T_CS!("\\+"), &T_CS!("\\@tabbing@increment"), None);
  let_i(&T_CS!("\\-"), &T_CS!("\\@tabbing@decrement"), None);
  let_i(&T_CS!("\\<"), &T_CS!("\\@tabbing@untab"), None);
  // Save accent definitions before rebinding \' and \`
  let_i(&T_CS!("\\@tabbing@'"), &T_CS!("\\'"), None);
  let_i(&T_CS!("\\@tabbing@`"), &T_CS!("\\`"), None);
  let_i(&T_CS!("\\a"), &T_CS!("\\@tabbing@accent"), None);
  // Rebind \' and \` to tabbing-specific (flush right / hfil)
  let_i(&T_CS!("\\'"), &T_CS!("\\@tabbing@flushright"), None);
  let_i(&T_CS!("\\`"), &T_CS!("\\@tabbing@hfil"), None);
  let_i(&T_CS!("\\pushtabs"), &T_CS!("\\@tabbing@pushtabs"), None);
  let_i(&T_CS!("\\poptabs"), &T_CS!("\\@tabbing@poptabs"), None);

  Ok(())
}

pub fn note_backmatter_element(whatsit: &mut Whatsit, backelement: &str) {
  if let Some(val) = lookup_mapping("BACKMATTER_ELEMENT", backelement) {
    whatsit.set_property("backmatterelement", val);
    whatsit.set_property("backmatterself", pin(backelement));
  }
}

/// Where a backmatter element (`ltx:bibliography`, `ltx:index`, a section
/// declared backmatter) goes: at the point its `BACKMATTER_ELEMENT` stand-in
/// (`ltx:section` by default) would be inserted — UNLESS no open node can
/// reach that stand-in without closing a `_noautoclose` container, in which
/// case the element is placed where IT is legal. A `\bibliography` inside a
/// beamer frame (an `ltx:subsection` that never auto-closes) is the case:
/// asking for `ltx:section` there erred `<ltx:section> isn't allowed in
/// <ltx:p>` and left the bibliography inside the `<p>` (metropolis demo,
/// simpleplus/simpledarkblue/pure-minimalistic samples, gotham), while the
/// subsection itself may hold an `ltx:bibliography` — which is also what
/// beamer typesets: the references on that slide. Perl's
/// adjustBackmatterElement (pool:3843) has no fallback. Guard:
/// `perfect_kernel_batch54::bibliography_inside_a_beamer_frame_stays_in_the_frame`.
fn backmatter_insertion_target(document: &Document, asif: &str, element: &str) -> String {
  if !document.is_openable(asif) && document.is_openable(element) {
    element.to_string()
  } else {
    asif.to_string()
  }
}

pub fn adjust_backmatter_element(document: &mut Document, whatsit: &Whatsit) -> Result<()> {
  let asif_opt = match whatsit.get_property("backmatterelement").as_deref() {
    Some(Stored::String(asif_sym)) => Some(to_string(*asif_sym)),
    _ => None,
  };
  // Note: We allocate a string here, since
  // it looks like arena::with can deadlock with find_insertion_point
  // we may need a find_insertion_point_sym to avoid that...
  let element = match whatsit.get_property("backmatterself").as_deref() {
    Some(Stored::String(sym)) => to_string(*sym),
    _ => "ltx:bibliography".to_string(),
  };
  if let Some(asif) = asif_opt {
    let target = backmatter_insertion_target(document, &asif, &element);
    let point = document.find_insertion_point(&target, None)?;
    document.set_node(&point);
  }
  Ok(())
}

// Do this before digesting the body of a bibliography
// Perl: beforeDigestBibliography in latex_constructs.pool.ltxml L3900
pub fn before_digest_bibliography() -> Result<()> {
  AssignValue!("inPreamble" => false);
  Digest!("\\@lx@inbibliographytrue")?;
  def_macro_noop("\\bibliographystyle{}")?;
  def_macro_noop("\\bibliography {}")?;
  // avoid \let-based redefinitions of the ending.
  Let!("\\endthebibliography", "\\saved@endthebibliography");
  ResetCounter!("@bibitem");
  Ok(())
}

// Since SOME people seem to write bibliographies w/o \bibitem,
// just blank lines between apparent entries,
// Making \par do a \bibitem{} works, but screws up valid
// bibliographies with blank lines!
// So, let's do some redirection!
fn setup_pseudo_bibitem() -> Result<()> {
  // Capture the REAL meanings, but only once per arming. Perl
  // (latex_constructs.pool.ltxml:setupPseudoBibitem L4028-4032) captures
  // unconditionally; if this runs while the redirection below is still
  // installed, all three saves capture the *redirectors* instead of the
  // originals — `\save@bibitem` becomes `\restoring@bibitem`, whose body ends
  // in `\bibitem`, which `\let\bibitem\save@bibitem` has just pointed back at
  // `\restoring@bibitem`. That is an unconditional infinite expansion loop
  // (`\let \bibitem \save@bibitem \let \par \save@par \let \\
  // \save@backbackslash \bibitem` forever), not a slow document.
  //
  // Perl has the identical hole — `\thebibliography \endthebibliography
  // \thebibliography \bibitem{b}` hangs Perl 0.8.8 too (>400 s on an 8-line
  // file that converts in <2 s otherwise); see KNOWN_PERL_ERRORS #57. It stays
  // latent upstream only because Perl's biblatex binding never defines
  // `\printbibliography`, so Perl never reads a real `.bbl` this way.
  //
  // We do reach it: a biber `.bbl` carries one `\datalist` per sorting scheme
  // (apa emits `nyt/apasortcite//…` *and* `nyt/global//…`), and every
  // `\enddatalist` expands to a whole `\thebibliography…\endthebibliography`.
  // Neither of those is an environment — no group — so the first arming is
  // still in force when the second `\thebibliography` opens. Witness:
  // arXiv 2605.17646 (`Fatal:Timeout:TokenLimit`, 1e9 tokens), guarded by
  // `tests/cluster_regressions/biblatex_two_datalists`.
  if !x_equals(&T_CS!("\\bibitem"), &T_CS!("\\restoring@bibitem")) {
    Let!("\\save@bibitem", "\\bibitem");
    Let!("\\save@par", "\\par");
    Let!("\\save@backbackslash", "\\\\");
  }
  Let!("\\bibitem", "\\restoring@bibitem");
  Let!("\\par", "\\par@in@bibliography");
  Let!("\\\\", "\\par@in@bibliography");
  Let!("\\vskip", "\\vskip@in@bibliography");
  // Moreover some people use \item instead of \bibitem
  Let!("\\item", "\\item@in@bibliography");
  // And protect from redefinitions.
  Let!("\\newblock", "\\lx@bibnewblock");
  // Risky, but when bibliography immediatesly starts with text (no implied \par)
  if let Some(token) = read_non_space()? {
    unread_one(token);
    if !token.is_executable() {
      unread_one(T_CS!("\\par"));
    }
  }
  Ok(())
}
// This sub does things that would commonly be needed when starting a bibliography
// setting the ID, etc...
pub fn begin_bibliography(whatsit: &mut Whatsit) -> Result<()> {
  begin_bibliography_clean(whatsit)?;
  // Fix for missing \bibitems!
  setup_pseudo_bibitem()
}

pub fn begin_bibliography_clean(whatsit: &mut Whatsit) -> Result<()> {
  // Check if \bibsection is defined and try to decipher it.
  // Expecting something like \section*{sometext}
  // Perl: beginBibliography_clean in latex_constructs.pool.ltxml
  let mut bibtitle: Option<Tokens> = None;
  if let Some(bs) = lookup_definition(&T_CS!("\\bibsection"))?
    && bs.is_expandable()
    && let Some(ExpansionBody::Tokens(expansion_toks)) = bs.get_expansion()
  {
    let mut tokens = expansion_toks.clone().unlist();
    if !tokens.is_empty() {
      let bibunitmap: &[(&str, &str)] = &[
        ("\\part", "ltx:part"),
        ("\\chapter", "ltx:chapter"),
        ("\\section", "ltx:section"),
        ("\\subsection", "ltx:subsection"),
        ("\\subsubsection", "ltx:subsubsection"),
        ("\\paragraph", "ltx:paragraph"),
        ("\\subparagraph", "ltx:subparagraph"),
      ];
      let first_cs = tokens.remove(0).to_string();
      if let Some((_, unit)) = bibunitmap.iter().find(|(cs, _)| *cs == first_cs) {
        assign_mapping("BACKMATTER_ELEMENT", "ltx:bibliography", Some(pin(unit)));
        // Strip * if present
        if !tokens.is_empty() && tokens[0].text == pin!("*") {
          tokens.remove(0);
        }
        if !tokens.is_empty() {
          // Perl L4052 flags a TODO right here: "Check for balanced? or
          // just take balanced begining?" — i.e. the bib-section title is
          // the sectional unit's *argument* (the leading brace group), not
          // every trailing token. Perl nonetheless takes all of @t, which
          // is fine until \bibsection is a parameterized renewal such as
          //   \renewcommand\bibsection[1]{\section*{\refname}\small #1}
          // (witness 1702.01165). After the unit+star strip that leaves
          // `{\refname}\small #1`; digesting all of it pushes the page/font
          // directive `\small` AND the bare parameter token `#1` — an
          // ARG-catcode token that errors "should never reach Stomach!".
          // Take only the leading balanced {...} group as the title (the
          // unit argument); fall back to all tokens when there is no
          // leading group (Perl's behavior for un-braced titles). This
          // realizes the Perl author's own "take balanced beginning" note
          // and drops trailing page/font junk LaTeXML never renders. See
          // docs/parity/OXIDIZED_DESIGN.md (bib-section title = leading group).
          let title_toks = if tokens[0].get_catcode() == Catcode::BEGIN {
            let mut depth = 0i32;
            let mut end = tokens.len();
            for (i, t) in tokens.iter().enumerate() {
              match t.get_catcode() {
                Catcode::BEGIN => depth += 1,
                Catcode::END => {
                  depth -= 1;
                  if depth == 0 {
                    end = i + 1;
                    break;
                  }
                },
                _ => {},
              }
            }
            tokens[..end].to_vec()
          } else {
            tokens
          };
          bibtitle = Some(Tokens::new(title_toks));
        }
      }
    }
  }

  note_backmatter_element(whatsit, "ltx:bibliography");
  // Try to compute a reasonable, but unique ID;
  // relative to the document's ID, if any.
  // But also, if there are multiple bibliographies,
  let bibnumber = 1 + lookup_int("n_bibliographies");
  assign_value("n_bibliographies", bibnumber, Some(Scope::Global));
  let mut docid: String = Expand!(T_CS!("\\thedocument@ID")).to_string();
  if !docid.is_empty() {
    docid += ".";
  }
  let bibid = s!("{}bib{}", docid, radix::radix_alpha(bibnumber - 1));
  DefMacro!(T_CS!("\\thebibliography@ID"), None, T_OTHER!(&bibid), scope => Some(Scope::Global));
  // Perl L3939 — child ID prefixes (e.g., `\the@bibitem@ID`) chain off the
  // parent counter's `@ID` macro. With parent counter `@lx@bibliography`,
  // child IDs derive from `\the@lx@bibliography@ID`. Mirror Perl exactly.
  DefMacro!(T_CS!("\\the@lx@bibliography@ID"), None, T_OTHER!(&bibid), scope => Some(Scope::Global));
  whatsit.set_property("id", bibid);
  let title_opt = if let Some(bt) = bibtitle {
    Some(Digest!(bt)?)
  } else {
    match DigestIf!("\\refname")? {
      Some(v) => Some(v),
      None => DigestIf!("\\bibname")?,
    }
  };
  if let Some(title) = title_opt {
    if let Some(titlefont) = title.get_font()? {
      whatsit.set_property("titlefont", titlefont);
    }
    whatsit.set_property("title", title);
  }
  if let Some(bs) = lookup_value("BIBSTYLE") {
    whatsit.set_property("bibstyle", bs);
  }
  if let Some(cs) = lookup_value("CITE_STYLE") {
    whatsit.set_property("citestyle", cs);
  }
  // NB: Perl's `beginBibliography` does NOT populate `#sort` here (only the
  // `\bibstyle` DefConstructor sets the `sort` attribute, and only on a
  // bibliography node that already exists — the bibunits path). We match that:
  // the main `<ltx:bibliography>` carries `bibstyle`/`citestyle` but no `sort`,
  // byte-identical to Perl. MakeBibliography derives citation-order numbering
  // from the `bibstyle` name (html_feedback #6294), so no `sort` attribute is
  // needed on the core node.
  // And prepare for the likely nonsense that appears within bibliographies
  ResetCounter!("enumiv");
  Ok(())
}

// Perl: $BIBSTYLES hash — maps bib style names to (citestyle, sort) pairs
// (latex_constructs.pool.ltxml L3953-3961). `sort => 'false'` is bibtex's own
// "leave in citation order" flag; MakeBibliography honors it (html_feedback
// #6294) to number the References the way the `.bst` — and the PDF — do.
fn lookup_bibstyle_params(style: &str) -> Option<(&'static str, &'static str)> {
  match style {
    "plain" => Some(("numbers", "true")),
    "unsrt" => Some(("numbers", "false")),
    "alpha" => Some(("AY", "true")),
    "abbrv" => Some(("numbers", "true")),
    "plainnat" => Some(("numbers", "true")),
    "unsrtnat" => Some(("numbers", "false")),
    "alphanat" => Some(("AY", "true")),
    "abbrvnat" => Some(("numbers", "true")),
    // Surpass-Perl: the real `ieeetr.bst`/`IEEEtran.bst` are UNSORTED (number by
    // first citation). Perl's base table omits them and `IEEEtran.cls.ltxml`
    // L331 even maps `IEEEtran → sort='true'` — both alphabetize, contradicting
    // the `.bst` and the published PDF. Map them to citation order to match the
    // ground-truth PDF (witness arXiv 2510.05438). See OXIDIZED_DESIGN.
    "ieeetr" => Some(("numbers", "false")),
    "IEEEtran" => Some(("numbers", "false")),
    _ => None,
  }
}

// Perl: setBibstyle($style) — set BIBSTYLE, CITE_STYLE, CITE_SORT
pub fn set_bibstyle(style: &str) {
  assign_value("BIBSTYLE", pin(style), None);
  if let Some((cs, so)) = lookup_bibstyle_params(style) {
    assign_value("CITE_STYLE", pin(cs), None);
    assign_value("CITE_SORT", pin(so), None);
  }
}

/// Perl: addIndexPhraseKey — sets the `key` attribute on index/glossary phrase
/// nodes from their text content, applying CleanIndexKey normalization.
fn add_index_phrase_key(node: &mut Node) -> Result<()> {
  if node.get_attribute("key").is_none() {
    let text = node.get_content();
    let key = clean_index_key(&text);
    if !key.is_empty() {
      node.set_attribute("key", &key)?;
    }
  }
  Ok(())
}
/// Perl: doIndexItem — open/close index list levels.
fn do_index_item(document: &mut Document, level: i64) -> Result<()> {
  if document.is_closeable("ltx:indexrefs").is_some() {
    document.close_element("ltx:indexrefs")?;
  }
  if document.is_closeable("ltx:indexphrase").is_some() {
    document.close_element("ltx:indexphrase")?;
  }
  let current_level = lookup_int("INDEXLEVEL");
  let mut l = current_level;
  while l < level {
    document.open_element("ltx:indexlist", None, None)?;
    l += 1;
  }
  while l > level {
    document.close_element("ltx:indexlist")?;
    l -= 1;
  }
  assign_value("INDEXLEVEL", Stored::Int(l), Some(Scope::Local));
  if level > 0 {
    document.open_element("ltx:indexentry", None, None)?;
    document.open_element("ltx:indexphrase", None, None)?;
  }
  Ok(())
}

/// Perl: CleanIndexKey — trim whitespace, remove trailing punctuation.
/// Additionally strips ONE outer brace pair spanning the whole key — the
/// wrapper process_index_phrases adds to protect `]`-bearing sort keys
/// (never strips a pair that closes mid-string, so `{a}{b}` survives).
fn clean_index_key(key: &str) -> String {
  let mut key = key.trim();
  if key.len() >= 2 && key.starts_with('{') && key.ends_with('}') {
    let inner = &key[1..key.len() - 1];
    let mut depth = 0i32;
    if inner.chars().all(|c| {
      match c {
        '{' => depth += 1,
        '}' => depth -= 1,
        _ => {},
      }
      depth >= 0
    }) && depth == 0
    {
      key = inner.trim();
    }
  }
  key.trim_end_matches(['.', ',', ';']).to_string()
}
/// Perl: process_index_phrases — expand \index{a!b@c|see{d}} into
/// \@index{\@indexphrase{a}\@indexphrase[c]{b}} etc.
///
/// Port of latex_constructs.pool.ltxml L4528-4591
/// #354 surpass (OXIDIZED_DESIGN #119): a `\verb`/`\verb*` inside `\index`.
/// `\index` reads its argument `SanitizedVerbatim`, which re-tokenizes it —
/// collapsing `\verb`'s raw body back into control sequences (`\delta`, not
/// `\`,`d`,…) and leaving `\verb` with no mouth to scan a delimiter from. In
/// both engines this yielded an empty `<verbatim/>` with the body leaking out
/// mis-tokenized, and a `|` delimiter additionally collided with the makeindex
/// encap separator. Consume each whole `\verb<D>body<D>` run HERE — before
/// expansion and before the `!`/`@`/`|` split can see the delimiter — and emit
/// `\@internal@text@verb{star}{D}{body}` (a non-expandable constructor, so the
/// run also survives the `\protected@write` expansion below) so the body
/// renders as typewriter.
fn absorb_index_verb_runs(toks: &[Token]) -> Vec<Token> {
  let mut out: Vec<Token> = Vec::with_capacity(toks.len());
  let mut i = 0;
  while i < toks.len() {
    let tok = toks[i];
    i += 1;
    if tok != T_CS!("\\verb") {
      out.push(tok);
      continue;
    }
    let mut starred = false;
    if i < toks.len() && toks[i] == T_OTHER!("*") {
      starred = true;
      i += 1;
    }
    if i >= toks.len() {
      out.push(tok);
      continue;
    }
    // A control sequence in the delimiter slot is expanded to the character
    // it stands for, as `\verb`'s own delimiter scan (a `read_x_token`) would:
    // doc.sty's `\SpecialMacroIndex` writes `\verb\verbatimchar…\verbatimchar`
    // (l3doc.cls:2151 `\verbatimchar` = `&` — an unexpanded `&` in the phrase
    // is a stray alignment tab; ltx-talk, postnotes, pythonimmediate), and
    // amsldoc.cls:87-114 `\index{foo@\string\verb\string"bar}` reaches
    // `\string"` = `"` with no closing `"` (amsldoc-it/-vn): the body then runs
    // to the end of the ENTRY, never past it (`readBalanced ran out of input`
    // when `\verb` itself was expanded; Perl never expands the entry).
    let (delim, rest): (Token, Vec<Token>) = if toks[i].get_catcode() == Catcode::CS {
      let tail = Tokens::new(toks[i..].to_vec());
      let expanded: Result<(Option<Token>, Vec<Token>)> =
        reading_from_mouth(Mouth::new("", None).expect("empty mouth"), move || {
          unread(tail);
          let first = read_x_token(None, false, None)?;
          let mut remaining = Vec::new();
          while let Some(t) = read_token()? {
            remaining.push(t);
          }
          Ok((first, remaining))
        });
      match expanded {
        Ok((Some(d), remaining)) => (d, remaining),
        _ => {
          // Not a verbatim invocation at all: keep `\verb` as index text.
          out.extend(Explode!("\\verb"));
          if starred {
            out.push(T_OTHER!("*"));
          }
          continue;
        },
      }
    } else {
      (toks[i], toks[i + 1..].to_vec())
    };
    let delim_s = delim.with_str(|d| d.to_string());
    let mut j = 0;
    while j < rest.len() && rest[j].with_str(|d| d != delim_s.as_str()) {
      j += 1;
    }
    // The re-tokenized body collapsed `\verb`'s raw chars back into control
    // sequences; `untex` + `Explode!` restores them to catcode-OTHER literals
    // so the digested `#3` renders as typewriter text instead of re-expanding
    // (which is exactly the `\delta`→math-δ leak this fixes).
    let body_str = Tokens::new(rest[..j].to_vec()).untex();
    let after = if j < rest.len() { j + 1 } else { j }; // consume the closing delimiter
    out.push(T_CS!("\\@internal@text@verb"));
    out.push(T_BEGIN!());
    if starred {
      out.push(T_OTHER!("*"));
    }
    out.push(T_END!());
    out.push(T_BEGIN!());
    out.push(delim);
    out.push(T_END!());
    out.push(T_BEGIN!());
    out.extend(Explode!(body_str));
    out.push(T_END!());
    // The remainder may hold further runs; it is a fresh slice now.
    out.extend(absorb_index_verb_runs(&rest[after..]));
    return out;
  }
  out
}

fn process_index_phrases(tokens: Tokens) -> Result<Tokens> {
  let token_list = tokens.unlist();
  if token_list.is_empty() {
    return Ok(Tokens::new(vec![]));
  }
  // Real `\index` (latex.ltx:17720-17725 `\@wrindex`) writes the entry with
  // `\protected@write`, i.e. the argument is EXPANDED (robust/`\protected`
  // commands deferred) before makeindex ever sees the `@`/`!`/`|` separators.
  // Packages build entries out of macros — tcolorbox's documentation library
  // writes `\kvtcb@doc@sortindex\idx@actual\tcbIndexPrintComC{…}` where
  // `\idx@actual` IS the `@` (tcbdocumentation.code.tex:147/495) — so a split
  // over unexpanded tokens sees no separator, digests the sort key as text,
  // and every `_` in it becomes `Script _ can only appear in math mode`
  // (tagpdf manual: 92 lines; perfect-kernel repro `\begin{docCommand}
  // {tag_if_active:TF}{}`). Perl's process_index_phrases (pool:4376-4397)
  // shares the omission; expanding is the faithful `\@wrindex` behaviour.
  // The `\verb` runs are absorbed first so their bodies stay verbatim.
  // `\protected@write` (latex.ltx:9551) does `\let\protect\@unexpandable@protect`
  // before its `\edef`, so a `\protect`ed macro is FROZEN into the entry and
  // runs only when the entry is typeset. Expanding with `\protect`=`\relax`
  // ran manyind.sty:100/119's `\protect\def\nwletre{…}`/`\protect\nxtletre`
  // at write time — the `\def` never bound its name (`undefined \nwletre`)
  // and `\proc@letter`'s caller-closing `\fi` (manyind.sty:148) surfaced as a
  // stray `\fi` (mindsample; Perl, which never expands here, is clean).
  // Guard: `perfect_kernel_batch54::index_entry_defers_protected_macros`.
  let toks = absorb_index_verb_runs(&token_list);
  // A control SYMBOL in the entry is never expanded here. After `\@sanitize`
  // (latex.ltx:1778) real `\@wrindex` writes a `\string`ed control symbol as
  // two characters, makeindex drops the sort key entirely and `\printindex`
  // re-reads only the display — so amsldoc.cls:84-89's sort key `\*` for
  // `\cn{\\*}` is never executed. `SanitizedVerbatim`'s re-tokenization
  // welds those two characters back into the live `\*` (amsldoc.cls:213
  // `\def\*#1`), which ate the rest of the entry (itamsldoc, amsldoc-vi;
  // Perl shares it, PLANS P73). Freezing every control symbol with
  // `\noexpand` keeps `\*` inert (the key is a string anyway) while accents
  // in the display (`M\"uller`) are constructors and unaffected; separators
  // still come from control WORDS (`\idx@actual`). Guard:
  // `perfect_kernel_batch54::index_sanitized_backslash_symbol_stays_literal`.
  let mut frozen: Vec<Token> = Vec::with_capacity(toks.len());
  for t in toks {
    if t.get_catcode() == Catcode::CS
      && t.with_str(|s| {
        let mut chars = s.chars();
        chars.next() == Some('\\')
          && chars.next().is_some_and(|c| !c.is_alphabetic())
          && chars.next().is_none()
      })
    {
      frozen.push(T_CS!("\\noexpand"));
    }
    frozen.push(t);
  }
  push_frame();
  let_i(
    &T_CS!("\\protect"),
    &T_CS!("\\@unexpandable@protect"),
    Some(Scope::Local),
  );
  let expanded = do_expand_partially(Tokens::new(frozen));
  pop_frame()?;
  let token_list = expanded?.unlist();
  // Add terminal ! if not present
  let mut toks = token_list;
  if toks
    .last()
    .map(|t| t.with_str(|s| s != "!"))
    .unwrap_or(true)
  {
    toks.push(T_OTHER!("!"));
  }
  let mut expansion: Vec<Token> = Vec::new();
  let mut phrase: Vec<Token> = Vec::new();
  let mut sortas: Vec<Token> = Vec::new();
  let mut style: Option<String> = None;
  let mut i = 0;
  // Separator chars (`"`/`@`/`!`/`|`) act ONLY at brace depth 0. A flat
  // scan (Perl latex_constructs.pool.ltxml L4326-4350 — Perl shares this
  // byte-identically) cuts through nested groups: packdoc.sty L328/L331
  // writes `\index{#2@\PDElement{#1}{#2}\csuse{packdoc@#1@IndexRemark}}`,
  // whose in-group `@`s shredded the phrase and emitted UNBALANCED braces
  // into the live stream — one mode error + one orphaned indexphrase per
  // `\OptionInd` (algxpar-doc 162+149 errs, numerica; pdflatex clean —
  // real makeindex splits the out-of-band .idx STRING where imbalance
  // cannot corrupt the document). KNOWN_PERL_ERRORS #83.
  let mut depth: i32 = 0;
  while i < toks.len() {
    let tok = toks[i];
    match tok.get_catcode() {
      Catcode::BEGIN => depth += 1,
      Catcode::END => depth -= 1,
      _ => {},
    }
    if depth > 0 || tok.get_catcode() == Catcode::END && depth == 0 {
      // Inside a group (or the closing brace returning to depth 0):
      // plain phrase material, never a separator.
      phrase.push(tok);
      i += 1;
      continue;
    }
    let s = tok.with_str(|s| s.to_string());
    i += 1;
    if s == "\"" && i < toks.len() {
      // Escaped character: take next token literally
      phrase.push(toks[i]);
      i += 1;
    } else if s == "@" {
      // Sort key: everything before @ is the sort key
      while phrase
        .last()
        .map(|t| t.with_str(|s| s.trim().is_empty()))
        .unwrap_or(false)
      {
        phrase.pop();
      }
      // The sort key is a makeindex STRING: real `\index` reads it under
      // `\@sanitize` (latex.ltx:17705-17711) and it is never typeset, so
      // `_`/`^`/`&`/`#`/`$`/`~` in it are literal characters (tcolorbox
      // `docCommand{tag_if_active:TF}` → sortindex `tag_if_active:TF`;
      // `\index{a_b@\texttt{a\_b}}`). Digesting the key with their live
      // catcodes raised `Script _ can only appear in math mode`; Perl
      // (`\@indexphrase[]` digests too) shares that. Neutralize them here.
      sortas = phrase
        .drain(..)
        .map(|t| match t.get_catcode() {
          Catcode::MATH
          | Catcode::ALIGN
          | Catcode::PARAM
          | Catcode::SUPER
          | Catcode::SUB
          | Catcode::ACTIVE => T_OTHER!(t.with_str(|s| s.to_string())),
          _ => t,
        })
        .collect();
    } else if s == "!" || s == "|" {
      // End of phrase
      while phrase
        .last()
        .map(|t| t.with_str(|s| s.trim().is_empty()))
        .unwrap_or(false)
      {
        phrase.pop();
      }
      if !phrase.is_empty() {
        expansion.push(T_CS!("\\@indexphrase"));
        if !sortas.is_empty() {
          // Brace-protect the sort key: a raw `]` inside it (examdoc
          // `\indc{gradetable[v]}`, pgfornament `pgfornament[<options>]`)
          // truncates the constructor's `[]` re-parse at the first `]`,
          // spilling the display phrase as illegal <ltx:indexmark> children
          // (Perl shares the flat re-parse — KNOWN_PERL_ERRORS #83 sibling).
          // clean_index_key strips the one wrapping pair symmetrically.
          expansion.push(T_OTHER!("["));
          expansion.push(T_BEGIN!());
          expansion.append(&mut sortas);
          expansion.push(T_END!());
          expansion.push(T_OTHER!("]"));
        }
        expansion.push(T_BEGIN!());
        expansion.append(&mut phrase);
        expansion.push(T_END!());
      }
      sortas.clear();
      if s == "|" {
        // Collect remaining tokens as style/see/seealso
        if i < toks.len()
          && toks
            .last()
            .map(|t| t.with_str(|s| s == "!"))
            .unwrap_or(false)
        {
          // Remove terminal ! stopbit
          toks.pop();
        }
        let extra: String = toks[i..]
          .iter()
          .map(|t| t.with_str(|s| s.to_string()))
          .collect();
        if extra.starts_with("see{") || extra.starts_with("see {") {
          // \@indexsee{content} — pass the ORIGINAL tokens (Perl:
          // @tokens[3..]) rather than a re-Exploded string, so macros
          // and math inside the see-phrase keep their catcodes and
          // expand normally (e.g. \index{X|see{\foo $n$-cube}}).
          // The brace tokens ride along as the constructor's argument
          // delimiters.
          expansion.push(T_CS!("\\@indexsee"));
          expansion.extend(toks[i + 3..].iter().cloned());
        } else if extra.starts_with("seealso{") || extra.starts_with("seealso {") {
          expansion.push(T_CS!("\\@indexseealso"));
          expansion.extend(toks[i + 7..].iter().cloned());
        } else if extra.starts_with("seeonly{") || extra.starts_with("seeonly {") {
          // DEVIATION from Perl LaTeXML, which lets `|seeonly{...}`
          // fall through to the style branch below: the raw string
          // (catcode-mangled) ends up as a font attribute and renders
          // as a garbage ltx_font_* class (98 occurrences in the AoBF
          // book, 181 validation errors). The makeindex idiom means
          // "print ONLY the see-reference, no locators" — treating it
          // as \@indexsee both renders it correctly and suppresses
          // this mark's locator refs (Scan skips referrer recording
          // when ltx:indexsee children are present).
          expansion.push(T_CS!("\\@indexsee"));
          expansion.extend(toks[i + 7..].iter().cloned());
        } else if extra == "(" {
          style = Some("rangestart".to_string());
        } else if extra == ")" {
          style = Some("rangeend".to_string());
        } else if !extra.is_empty() {
          // Style name (e.g., textbf → bold)
          style = Some(match extra.as_str() {
            "textbf" | "bf" => "bold".to_string(),
            "textit" | "it" | "emph" => "italic".to_string(),
            "textrm" | "rm" => String::new(),
            other => other.to_string(),
          });
        }
        break; // Consumed everything after |
      }
    } else if phrase.is_empty() && s.trim().is_empty() {
      // Skip leading whitespace
    } else {
      phrase.push(tok);
    }
  }
  // Wrap in \@index[style]{...}
  let mut result = vec![T_BEGIN!(), T_CS!("\\normalfont"), T_CS!("\\@index")];
  if let Some(ref sty) = style
    && !sty.is_empty()
  {
    result.push(T_OTHER!("["));
    result.extend(Explode!(sty));
    result.push(T_OTHER!("]"));
  }
  result.push(T_BEGIN!());
  result.extend(expansion);
  result.push(T_END!());
  result.push(T_END!());
  Ok(Tokens::new(result))
}

/// Convert TeX points to CSS pixels using DPI setting (default 100).
/// Perl: $$self[0] / 65536 * DPI / 72.27
fn px_value(pt: f64) -> f64 {
  // DPI default is 100 in LaTeXML (state::lookupValue('DPI') || 100)
  let dpi = lookup_value("DPI")
    .and_then(|v| {
      if let Stored::Number(n) = v {
        Some(n.0 as f64)
      } else {
        None
      }
    })
    .unwrap_or(100.0);
  // Round to 2 decimal places (Perl default precision)
  (pt * dpi / 72.27 * 100.0).round() / 100.0
}
/// Format a px value, dropping trailing ".0" for integers
fn fmt_px(v: f64) -> String {
  if v == v.round() && v.abs() < 1e10 {
    format!("{}", v as i64)
  } else {
    format!("{v}")
  }
}

/// Perl: %unicode_enclosed_alphanumerics table
/// Maps single chars (0-9, a-z, A-Z) and numbers 10-20 to their circled Unicode equivalents.
fn unicode_enclosed_alphanumeric(text: &str) -> Option<String> {
  let ch = match text {
    "0" => '\u{24EA}',
    "1" => '\u{2460}',
    "2" => '\u{2461}',
    "3" => '\u{2462}',
    "4" => '\u{2463}',
    "5" => '\u{2464}',
    "6" => '\u{2465}',
    "7" => '\u{2466}',
    "8" => '\u{2467}',
    "9" => '\u{2468}',
    "10" => '\u{2469}',
    "11" => '\u{246A}',
    "12" => '\u{246B}',
    "13" => '\u{246C}',
    "14" => '\u{246D}',
    "15" => '\u{246E}',
    "16" => '\u{246F}',
    "17" => '\u{2470}',
    "18" => '\u{2471}',
    "19" => '\u{2472}',
    "20" => '\u{2473}',
    "a" => '\u{24D0}',
    "b" => '\u{24D1}',
    "c" => '\u{24D2}',
    "d" => '\u{24D3}',
    "e" => '\u{24D4}',
    "f" => '\u{24D5}',
    "g" => '\u{24D6}',
    "h" => '\u{24D7}',
    "i" => '\u{24D8}',
    "j" => '\u{24D9}',
    "k" => '\u{24DA}',
    "l" => '\u{24DB}',
    "m" => '\u{24DC}',
    "n" => '\u{24DD}',
    "o" => '\u{24DE}',
    "p" => '\u{24DF}',
    "q" => '\u{24E0}',
    "r" => '\u{24E1}',
    "s" => '\u{24E2}',
    "t" => '\u{24E3}',
    "u" => '\u{24E4}',
    "v" => '\u{24E5}',
    "w" => '\u{24E6}',
    "x" => '\u{24E7}',
    "y" => '\u{24E8}',
    "z" => '\u{24E9}',
    "A" => '\u{24B6}',
    "B" => '\u{24B7}',
    "C" => '\u{24B8}',
    "D" => '\u{24B9}',
    "E" => '\u{24BA}',
    "F" => '\u{24BB}',
    "G" => '\u{24BC}',
    "H" => '\u{24BD}',
    "I" => '\u{24BE}',
    "J" => '\u{24BF}',
    "K" => '\u{24C0}',
    "L" => '\u{24C1}',
    "M" => '\u{24C2}',
    "N" => '\u{24C3}',
    "O" => '\u{24C4}',
    "P" => '\u{24C5}',
    "Q" => '\u{24C6}',
    "R" => '\u{24C7}',
    "S" => '\u{24C8}',
    "T" => '\u{24C9}',
    "U" => '\u{24CA}',
    "V" => '\u{24CB}',
    "W" => '\u{24CC}',
    "X" => '\u{24CD}',
    "Y" => '\u{24CE}',
    "Z" => '\u{24CF}',
    _ => return None,
  };
  Some(ch.to_string())
}

#[rustfmt::skip]
/// Perl `%makebox_alignment` (#2829): l/r/c/s → left/right/center/stretched.
fn makebox_alignment(key: &str) -> &'static str {
  match key {
    "l" => "left",
    "r" => "right",
    "c" => "center",
    "s" => "stretched",
    _ => "",
  }
}

// The `LoadDefinitions!` body is split into per-section modules (Lamport
// Appendix-C order), called below in the original sequence: the dump is a
// definition-ORDER snapshot, so the order is load-bearing (verified by a
// bitwise dump diff at the split, 2026-09-03).
mod sect01;
mod sect02;
mod sect03;
mod sect04;
mod sect05;
mod sect06;
mod sect07;
mod sect08;
mod sect09;
mod sect10;
mod sect11;
mod sect12;
mod sect13;

LoadDefinitions!({
  // Perl `latex_constructs.pool.ltxml` L19-38 — force-reload of
  // `plain_constructs` and `math_common`. By the time
  // `latex_constructs` runs, both pools were already loaded during the
  // plain-format chain (`tex.rs::LoadFormat('plain')`), and several of
  // their definitions have since been clobbered by `latex_base` and
  // earlier `latex_constructs` activity. Perl explicitly clears the
  // `_loaded` flags and re-runs `LoadPool('plain_constructs')` (L21)
  // followed by `LoadPool('math_common')` (L38) to re-establish those
  // pools' definitions on top of LaTeX-side changes.
  //
  // Rust note: since commit `8dfcb12f7`, `InnerPool!(...)` honors
  // `<name>.pool_loaded` (mirror of Perl `LoadPool`'s
  // `<name>.pool.ltxml_loaded` guard, with the Rust suffix
  // convention). The two `assign_value(... Stored::None)` resets
  // below are therefore load-bearing — without them, `InnerPool!`
  // would skip the re-run.
  //
  // Perl interleaves a handful of defs (font reset, `\hline`,
  // `\f@encoding`, `\par→\lx@normal@par`, etc.) between L21 and L38;
  // Rust collapses both reloads here at the top because the
  // intervening defs are positioned later in this file (or in
  // `plain_constructs.rs`) and are agnostic to whether `math_common`
  // is reloaded before or after them.
  assign_value(
    "plain_constructs.pool_loaded",
    Stored::None,
    Some(Scope::Global),
  );
  assign_value("math_common.pool_loaded", Stored::None, Some(Scope::Global));
  // The reloads MUST run with state unlocked. By the time we get here,
  // the first plain-format pass has already locked common math CSes
  // (e.g. `\prime`, `\active@math@prime`) via their `locked => true`
  // DefMath/DefMacro entries. Without an unlocked frame, the second
  // pass sees `\prime:locked` and silently drops the redefinition,
  // leaving the dump-loaded `\mathchardef\prime="0230` mathchar in
  // place — which renders as digit `0` (char 0x30 in fam 2) instead
  // of U+2032 ′. Mirror Perl's LoadPool flow which reloads via the
  // top-level binding scope where re-locks are allowed.
  local_state_unlocked(true);
  InnerPool!(plain_constructs);
  InnerPool!(math_common);
  expire_state_unlocked();

  // Perl latex_constructs.pool.ltxml:36 — `Let('\par', '\lx@normal@par')`.
  // After the dump load (which installs `\par` as the heavy expl3 chain
  // `\para_end:` body via Lt-aliases), Perl re-Lets `\par` to the engine's
  // plain `\lx@normal@par` Constructor here. So at document-body time,
  // `\par` is a Constructor (no body residue), not the chain. Without
  // this re-Let in Rust, dump-path `\par` stays the expl3 chain;
  // `leave_horizontal()`'s implicit `\par` invocation then leaves chain
  // residue (`\tex_unskip:D \mode_if_horizontal:TF{...}\tex_par:D ...`)
  // in the gullet pushback, corrupting subsequent numeric/token reads —
  // breaks box_test, ifthen_test, aftergroup_test, halign_test,
  // vmode_test, IEEE_test, plainfonts_test (each NODUMP-passes,
  // DUMP-only-fails, all share the same `\par` chain residue root cause).
  Let!("\\par", "\\lx@normal@par");

  // Perl latex_constructs.pool.ltxml L42-43 — early DefAccent duplicates
  // for `\k` (Ogonek) and `\r` (Ring above). Per Perl: "These really
  // shouldn't be here, but somehow aren't getting defined in the right
  // place???". The dump installs `\k`/`\r` as raw t1enc Expandables
  // (`\T1-cmd \k \T1\k`); without this re-install they stay broken
  // until `\k` is reinstalled at L8820 in math section. Mirror Perl by
  // restoring the DefAccent right after dump at C.1 region.
  DefAccent!("\\k", '\u{0328}', "\u{02DB}", below => true); // COMBINING OGONEK & OGONEK
  DefAccent!("\\r", '\u{030A}', "\u{02DA}"); // COMBINING RING ABOVE & non-combining

  // Perl latex_constructs.pool.ltxml L25-26 — restore default fonts after
  // dump load. Without this, dump-time font residue (CJK shimming etc.)
  // leaks into document body. Mirror Perl `assignValue('font' => textDefault, 'global')`.
  assign_value(
    "font",
    Stored::Font(Rc::new(Font::text_default())),
    Some(Scope::Global),
  );
  assign_value(
    "mathfont",
    Stored::Font(Rc::new(Font::math_default())),
    Some(Scope::Global),
  );

  // \&, \#, \%, \$, \_ math-mode dispatch — the dump captures these as raw
  // CharDef registers losing plain_base.rs's `\ifmmode` dispatch. Tried
  // re-DefMacro to `\ifmmode\lx@math@amp\else\lx@text@amp\fi` but the
  // `\lx@(text|math)@*` targets live in plain_base.rs (DefPrimitive
  // closures) which is SKIPPED on dump path — re-DefMacro produces
  // <ERROR class="undefined"/>. Proper fix needs relocating the
  // `\lx@text@amp`/`\lx@math@amp` family from plain_base.rs to
  // plain_constructs.rs (which runs in both paths) before adding the
  // re-DefMacro here. Deferred — dump's CharDef-38 works in text mode,
  // ifthen_test remaining failure is the only known math-mode issue.

  sect01::load()?;
  sect02::load()?;
  sect03::load()?;
  sect04::load()?;
  sect05::load()?;
  sect06::load()?;
  sect07::load()?;
  sect08::load()?;
  sect09::load()?;
  sect10::load()?;
  sect11::load()?;
  sect12::load()?;
  sect13::load()?;
});

/// Read a `\verb`-style verbatim argument from the current input — the
/// optional `*`, the delimiter, and the body up to the next delimiter — and
/// return the tokens that typeset it: `[\lx@use@visiblespace]
/// \@internal@verb{star}{delim}{body}`, WITHOUT the enclosing hidden group,
/// so a wrapper (newverbs' `\newverbcommand{\cmd}{before}{after}`) can put
/// its own tokens inside the same group. `None` when no delimiter was found
/// (an `expected:delimiter` error has been reported).
pub fn read_verb_invocation() -> Result<Option<Vec<Token>>> {
  begin_semiverbatim(Some(&SEMIVERBATIM_CHARS));
  apply_dospecials();
  // Do NOT (necessarily) skip spaces after \verb!!!
  assign_catcode(' ', Catcode::ACTIVE, None);
  let mut init = None;
  let mut skipped_space = false;
  // As of texlive 2021, DO skip spaces before delimiter (even tho we've changed catcodes)
  // but if we do skip spaces, * can be the delimiter
  let space_sym = pin!(" ");
  while let Some(maybe_init) = read_token()? {
    if maybe_init.get_sym() == space_sym {
      skipped_space = true;
    } else {
      init = Some(maybe_init);
      break;
    }
  }
  let mut starred = false;
  if let Some(ref init_token) = init
    && *init_token == T_OTHER!("*")
    && !skipped_space
  {
    starred = true;
    while let Some(maybe_init) = read_token()? {
      if maybe_init.get_sym() != space_sym {
        init = Some(maybe_init);
        break;
      }
    }
  }
  if let Some(init_token) = init {
    let init_ch = init_token.with_str(|is| is.chars().next().unwrap());
    assign_catcode(init_ch, Catcode::ACTIVE, None);
    let delim = Tokens!(T_ACTIVE!(init_ch));
    let body = read_until(&delim)?.unwrap_or_default();
    end_semiverbatim()?;
    let mut result = Vec::new();
    if starred {
      result.push(T_CS!("\\lx@use@visiblespace"));
    }
    result.extend(
      Invocation!(T_CS!("\\@internal@verb"), vec![
        if starred {
          Tokens!(T_OTHER!("*"))
        } else {
          Tokens!()
        },
        Tokens!(init_token),
        body
      ])
      .unlist(),
    );
    Ok(Some(result))
  } else {
    // typically something read too far got \verb and the content is somewhere else..?
    Error!(
      "expected",
      "delimiter",
      "Verbatim argument lost\n Bindings for preceding code is probably broken"
    );
    end_semiverbatim()?;
    Ok(None)
  }
}

/// True when some open ancestor can hold `qname` (`ltx:caption` /
/// `ltx:toccaption`) — i.e. the caption really is inside a float. The
/// transient capture wrappers (`ltx:_CaptureBlock_`, `insert_block`'s
/// holder for a `{center}` body inside a figure; `ltx:_Capture_`) admit every
/// block element and are unwrapped into their parent afterwards, so they
/// neither count nor stop the walk: a `marginfigure` (`lrbox` + minipage,
/// used from a `\marginpar`) reaches `ltx:note` → `ltx:p` → document and
/// answers no; `{center}` inside `{figure}` reaches the figure. See
/// OXIDIZED_DESIGN #182.
fn caption_can_float(document: &Document, qname: &str) -> bool {
  let Some(mut node) = document.get_element() else {
    return false;
  };
  loop {
    if !node.get_name().starts_with('_') && document::can_contain(&node, qname) {
      return true;
    }
    match node.get_parent() {
      Some(parent) if parent.get_type() == Some(NodeType::ElementNode) => node = parent,
      _ => return false,
    }
  }
}

/// Absorb a formatted caption title without its `\lx@tag` pieces — the
/// float's counter label, which only a float's `ltx:caption` may carry
/// (OXIDIZED_DESIGN #182). Lists are walked so a tag nested beside the title
/// text is dropped without losing the text.
pub(crate) fn absorb_without_tags(document: &mut Document, piece: &Digested) -> Result<()> {
  match piece.data() {
    DigestedData::Whatsit(w) => {
      if w.borrow().get_definition().get_cs_name().contains("lx@tag") {
        return Ok(());
      }
      document.absorb(piece, None)?;
    },
    DigestedData::List(l) => {
      let boxes = l.borrow().boxes.clone();
      for inner in boxes {
        absorb_without_tags(document, &inner)?;
      }
    },
    _ => {
      document.absorb(piece, None)?;
    },
  }
  Ok(())
}
