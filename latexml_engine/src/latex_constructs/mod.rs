use std::collections::VecDeque;

use latexml_core::{
  alignment::template::TemplateConfig,
  common::{
    error::{IgnoreDiagnosticsScope, emit_warn},
    xml::is_descendant_or_self,
  },
  digested::DigestedData,
  parameter::{
    KEEP_STATED_LENGTHS, STATED_LENGTHS, STATED_LENGTHS_WITHHELD, WITHHOLD_STATED_LENGTHS,
    read_braced,
  },
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
use crate::base_utilities::{
  FrontmatterAnchor, already_reported, begin_deposit_insert, begin_deposit_provenance,
  end_deposit_insert, end_deposit_provenance, mark_preamble_residue, place_frontmatter,
  remove_frontmatter_marks, tag_forms,
};
use crate::{
  base_parameter_types::reread_sanitized_entry,
  plain_constructs::tmspace_width,
  prelude::*,
  tex_box::{FramedOptions, framed_properties},
  tex_tables::{
    alignment_bindings, array_strut, array_zeroes_interline, clear_alignment_row_hooks,
    restore_dollar_outside_alignment,
  },
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
/// The NAME of a digested `{}` identifier argument — a section type, a note type (`\lx@note{footnote}`): its
/// reverted tokens, not its typeset string. Digestion decodes letters through the current font encoding, so under
/// LGR (babel greek, textalpha) the digested `section` read back as `σεςτιον` and `\csname the…\endcsname` built
/// `\theσεςτιον` (greek-fontenc manuals, teubner, toptesi-it: 56 nested-section errors per doc); a note's type is
/// digested after `neutralize_font`, in the document's encoding (`footnote` → `φοοτνοτε`, an undefined counter). Guards:
/// `perfect_kernel_batch54::section_type_name_is_not_font_decoded`,
/// `perfect_kernel_batch60::normalfont_note_expands_the_encoding_default`.
pub(crate) fn type_name(stype: &Digested) -> String {
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
pub fn is_definable_latex(cs: &Token) -> Result<(bool, bool)> {
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
  // A fallback class's guess (OmniBus's `\let\abst\abstract`, for a class with no binding): a document that
  // compiles with its real class and `\newcommand`s the name shows that class leaves it undefined, so the
  // document's definition wins (2605.05619's `\newcommand{\abst}[1]{|#1|}`; 59g, OXIDIZED_DESIGN_DIVERGENCES #408).
  // The document's, and its author's own files' (K22: the author's files outrank our guesses; a raw file the author
  // shipped beside the source, `binding::content::reading_author_file` — 0901.4335's `mn2e_journal_style.sty`): a
  // TeX Live class or package loaded beside OmniBus (apa7.cls) yields to it, as raw files yield to bindings.
  // A kernel stub (`DefinitionOrigin::Stub`: aaai's `\equalcontrib`, sect05.rs) yields to a raw file's own definition
  // as well, since the real class or style defines the name (copernicus.cls:1673), and so does what a binding borrows
  // from a package the real one does not load (`DefinitionOrigin::Borrowed`, `require_package_borrowed`: in LaTeX the
  // name does not exist) — cc.cls's ccproof.sty:56 `\newenvironment{proof}` after theorem.sty (1507.02955).
  if is_fallback_meaning(cs) {
    use definition::origin::DefinitionOrigin;
    let origin = definition::origin::current_origin();
    if origin == DefinitionOrigin::Document
      || (origin == DefinitionOrigin::File
        && (reading_author_file()
          || lookup_definition(cs)?
            .is_some_and(|prev| prev.get_origin() == DefinitionOrigin::Stub)
          || fallback_meaning_origin(cs) == Some(DefinitionOrigin::Borrowed)))
    {
      return Ok((true, false));
    }
  }
  // A standard-class derivative read over its binding defines the declared text names the binding assigned as the
  // class's own, as it does in TeX, read in the class's place (`state::standard_class_binding_name`; aip-cp.cls:875
  // `\newcommand\refname{REFERENCES}`, 1506.04050; OXIDIZED_DESIGN_DIVERGENCES #731 rule 13).
  // So does a copy its `\Provides` line routed to its binding, for a declared text name the binding (or one it
  // requires) defined: proto.cls:1731 `\newcommand\apj{\ref@jnl{\it Astrophys.\ J.}}` over the aas_macros binding's
  // "ApJ" (1402.7103; `state::routed_copy_text_name_yields`, rule 14, user ruling 2026-10-21); whether its definition
  // stands for the document is decided by its body when the copy's read ends (`state::leave_copy_provides_frame`).
  // The declarations are global (`state::declare_copy_text_names` assigns them so): a name any binding loaded so far
  // declared counts for every routed copy, not only for one routed to that binding — a copy routed to the IEEEtran
  // binding in a document that also loaded aas_macros may supply `\apj`.
  if standard_class_binding_name(cs) || routed_copy_text_name_yields(cs) {
    return Ok((true, false));
  }
  // Perl reads the definition's source (`=~ /^plain/`); Rust pools carry no such
  // locator, so the plain LAYER is also read from the definition's provenance.
  let plain = lookup_definition(cs)?.is_some_and(|prev| {
    prev.get_origin() == definition::origin::DefinitionOrigin::Plain
      || prev.get_locator().is_some_and(is_plain_definition_source)
  });
  Ok((plain, plain))
}

//======================================================================
// LaTeX helper functions (moved from latex_functions.rs)
// Perl: inline in latex_constructs.pool.ltxml
//======================================================================

pub fn start_appendices(kind: &str) { begin_appendices(kind) }

/// Whether `chapter` is a LaTeX counter — `\c@chapter` AND its reset list `\cl@chapter`, as every `\newcounter`
/// (latex.ltx:10139 `\@definecounter`, our `new_counter`) makes both — so a class has chapters. Perl tests `\c@chapter`
/// alone (latex_constructs.pool.ltxml:690-693), safe while its kernel `\chapter` stayed defined; since
/// OXIDIZED_DESIGN #179 retracts it in a chapterless class, tcilatex's no-chapters branch makes a bare count register
/// (`\newcount\c@chapter`, tcilatex_tex.rs), and the appendix numbered `\thechapter.\Alph{section}` with `\thechapter`
/// undefined (537 of run 336's papers: 1509.06343, quant-ph0609144). Not `\thechapter`: under OmniBus it is an autoload
/// stub that loads book.cls (omnibus_cls.rs). Nor the counter a `\chapter` call TeX refuses makes for LaTeXML's
/// chapter (`kernel_chapter_with_warning`, `\chapter:stand@in@counter`): jmlr's article appendix is "Appendix A".
pub fn has_chapter_counter() -> bool {
  let defined = |name: &str| lookup_definition(&T_CS!(name)).ok().flatten().is_some();
  defined("\\c@chapter") && defined("\\cl@chapter") && !lookup_bool("\\chapter:stand@in@counter")
}

pub fn begin_appendices(counter: &str) {
  // (numbered within chapters where the class has chapters, by `\chapter` — not a `chapter` counter alone, which
  // WileyNJD-v2.cls:2410 and WileyNJDv5.cls:5728 declare for bookmarks with no `\chapter`: "Appendix A", not "0.A",
  // 2211.16474; `state::class_defines_chapter`)
  begin_appendices_within(
    counter,
    has_chapter_counter() && class_defines_chapter() && counter != "chapter",
  )
}

/// [`begin_appendices`] with the numbering chosen by the caller: the units `\the<counter>` within the chapter
/// (`\thechapter.\Alph{<counter>}`) or within the document (`\Alph{<counter>}`). A class with a `\chapter` of its own
/// whose `\appendix` is still article's section-level one (llncs.cls:51 `\LoadClass{article}`, its `\chapter` at :302
/// for proceedings volumes; article.cls:439-442) numbers them within the document: "Appendix A", not "0.A".
pub fn begin_appendices_within(counter: &str, within_chapter: bool) {
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
  if within_chapter {
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

/// A note's size in the line it sits in: its displayed mark (the `mark` property, or plain's
/// `prenote`), nothing without one — Perl's `\lx@note` `sizer => '#mark'`
/// (latex_constructs.pool.ltxml:460), which the port had dropped; extended to every note
/// constructor (OXIDIZED_DESIGN_DIVERGENCES #391). The note's text is an insertion, set out of the
/// line (tex.web §1100 `begin_insert_or_adjust`; in the HTML a popup, `.ltx_note_content`
/// display:none). Measured with its body, a note holding a `\par` added its paragraphs' lines to
/// the line; measured as its `tags`, it counted every registered tag form (cleveref's six: a
/// `\resizebox` of a tabular with a footnote shrank to half); a `\footnotemark` summed its
/// arguments, the counter's name included. Witnesses 2605.18427 S3.T4 (natural width 802.6 →
/// 556.7pt, pdflatex 521.7), 2605.14713 S2.T1 (aa `\tablefoot`: `innerwidth` 2793.7 → 1644.8).
/// Guard `perfect_kernel_batch58::notes_measure_as_their_mark`.
pub fn note_size_in_line(whatsit: &Whatsit) -> Result<(Dimension, Dimension, Dimension)> {
  for key in ["mark", "prenote"] {
    if let Some(Stored::Digested(mark)) = whatsit.get_property(key).as_deref() {
      return mark.compute_size(Default::default());
    }
  }
  Ok(out_of_line_size())
}

/// The unit of a [`symbol_box_size`]: the font's em (quad) or ex (x-height).
#[derive(Clone, Copy, Debug)]
pub enum FontUnit {
  Em,
  Ex,
}

/// A symbol box `width` × `height` `unit`s of the whatsit's font, no depth: the size TeX measures for a class's QED
/// box (a rule, `\openbox`, `\hbox{\rlap{$\sqcap$}$\sqcup$}`), given to a size-less constructor so a cell or box holding
/// only it is not empty — alignment emptiness is by measured size (alignment/normalize.rs, Perl Alignment.pm:458-470),
/// and a size-less whatsit alone in a cell was dropped. The constants are TFM ratios of the glyph font's design size
/// (or the class's em/ex dimensions), applied as multiples of the current font's em or ex — equal at 10pt, where
/// cmsy10's QUAD is 1.000003 of the design size.
pub fn symbol_box_size(
  whatsit: &Whatsit,
  width: f64,
  height: f64,
  unit: FontUnit,
) -> (Dimension, Dimension, Dimension) {
  let font = whatsit
    .get_font()
    .ok()
    .flatten()
    .unwrap_or_else(|| Rc::new(Font::text_default()));
  let scale = match unit {
    FontUnit::Em => font.get_em_width(),
    FontUnit::Ex => font.get_ex_height(),
  } as f64;
  (
    Dimension::new((width * scale) as i64),
    Dimension::new((height * scale) as i64),
    Dimension::new(0),
  )
}

/// llncs.cls:400, svjour3.cls (v3.2) :959, svmult.cls (v5.4) :1027, aa.cls (v5.3-7.0, :805-:914) and mn2e.cls:410
/// `\squareforqed`/`\qedsymbol`
/// `\hbox{\rlap{$\sqcap$}$\sqcup$}`: the glyph box of cmsy10's `\sqcup` (0.666669 × 0.555557 em).
pub fn sqcup_qed_size(whatsit: &Whatsit) -> (Dimension, Dimension, Dimension) {
  symbol_box_size(whatsit, 0.666669, 0.555557, FontUnit::Em)
}

/// The size in its line of material set out of it: a `\footnotetext`/`\endnotetext` (an
/// insertion with no mark there) or a `\marginpar` (a float insertion, tex.web §1100) — nothing.
pub fn out_of_line_size() -> (Dimension, Dimension, Dimension) {
  (Dimension::new(0), Dimension::new(0), Dimension::new(0))
}

/// Whether a `\footnote` here is a minipage's (latex.ltx:16323 `\def\@mpfn{mpfootnote}`): `\@mpfn` names another
/// counter than `footnote`.
pub(crate) fn minipage_footnotes() -> Result<bool> {
  Ok(has_meaning(&T_CS!("\\@mpfn")) && Expand!(Tokens!(T_CS!("\\@mpfn"))).to_string() != "footnote")
}

/// The tag a note of `note_type` is given (`\lx@note`'s `[<tag>]`, the `given` one or none). latex.ltx:17649-17656
/// `\footnote` steps the counter `\@mpfn` names and marks the note `\thempfn` (`\footnote[<n>]` sets that counter to
/// `<n>`); a minipage makes them `mpfootnote` and `\thempfootnote` (:16323, the counter set to 0; sect12.rs
/// `{minipage}`), so a minipage's footnotes are lettered "a", "b", … (:17638) apart from the page's, which go on where
/// they were; the note stays a footnote. `\footnotemark` keeps to `footnote` (:17677), and so does `\footnotetext`,
/// which LaTeX marks `\thempfn` (:17690-17699): in a minipage LaTeX numbers a mark and letters the text answering it
/// (2304.06412's table, `\footnotemark[1]` "1", `\footnotetext[1]` "a"), a mismatch that would leave the two unpaired.
/// Perl (latex_base.pool.ltxml:270-272) numbers a minipage's notes as the page's (KNOWN_PERL_ERRORS #1049). Witnesses
/// 0802.0707, 2305.00493, 2309.00006; guards `perfect_kernel_batch78i::{minipage_footnotes_are_lettered,
/// minipage_footnotetext_pairs_with_its_mark}`.
pub(crate) fn footnote_tag<'a>(
  note_type: &str,
  given: Option<&'a Digested>,
  number: &FootnoteNumber,
) -> Result<Option<Cow<'a, Digested>>> {
  if note_type != "footnote" || !minipage_footnotes()? {
    return match given {
      Some(given) => Ok(Some(Cow::Owned(numbered_footnote_mark(given, number)?))),
      None => Ok(None),
    };
  }
  let mut tokens = TokenizeInternal!(r"\begingroup").unlist();
  match given {
    None => tokens.extend(TokenizeInternal!(r"\stepcounter\@mpfn").unlist()),
    Some(number) => {
      tokens.extend(TokenizeInternal!(r"\csname c@\@mpfn\endcsname").unlist());
      tokens.extend(number.revert()?.unlist());
      tokens.push(T_CS!("\\relax"));
    },
  }
  tokens.extend(TokenizeInternal!(r"\thempfn\endgroup").unlist());
  Ok(Some(Cow::Owned(digest(Tokens::new(tokens))?)))
}

/// The document's class, as its `\documentclass` (or `\documentstyle`) named it (`lx@document@class`, recorded before
/// the class loads): a binding another class's binding loads as its base (revtex4, IEEEtran, elsarticle and llncs
/// load the article binding) is not the document's class, and its own title block is not in force.
pub fn is_document_class(class: &str) -> bool { lookup_string("lx@document@class") == class }

/// The title block's own `\thefootnote`, the one hook a binding declares its title's footnote marks with: defined, the
/// marks of the notes made in the title, the authors and the date are it (`enter_title_footnotes`), and so are the
/// frontmatter observer's Mark labels (`observer/rehearsal.rs` `TITLE_FOOTNOTE_HOOK`, OXIDIZED_DESIGN_DIVERGENCES #837,
/// #851). Article, report and book declare it `\@fnsymbol\c@footnote` (`declare_fnsymbol_title_block`); a class
/// numbering its title's marks otherwise defines its own (revtex4-2.cls:3040-3041 `\frontmatter@thefootnote`). Guard
/// `perfect_kernel_batch78o::declared_title_thefootnote_prints_the_marks_the_observer_labels`.
pub const TITLE_THEFOOTNOTE: &str = "\\lx@title@thefootnote";

/// The title block of a class whose `\maketitle` marks its notes in `\@fnsymbol` — article, report and book (and
/// extarticle, extreport, extbook, copies of them; article.cls:203-205, report.cls:211, book.cls:217, in the
/// `\if@titlepage` false branch: `titlepage`, report's and book's default (:50-51) and an option of all three, keeps
/// `\thefootnote`, `titlepage_default` `Some(..)`), siamltex (siamltex.cls:156-159, no title-page form, `None`) — when
/// that class is the document's (`is_document_class`): the notes of the title, the authors and the date are marked so,
/// and the footnote counter is set back to 0 after `\maketitle` (article.cls:190, :223; siamltex.cls:175), so the body's
/// notes count from 1. The marks are made where the frontmatter digests the fields (`enter_title_footnotes`):
/// `\footnotemark[1]` in an `\author` prints "∗" as the text `\footnotetext[1]` gives after
/// `\renewcommand\thefootnote{\fnsymbol{footnote}}` (2412.20504, 2006.14582; siamltex 0903.0014; KNOWN_PERL_ERRORS
/// #1060). Perl's classes mark them as the body (`\thefootnote`). The class's `\thanks` (and `\footnote`, which its
/// `\maketitle` lets to `\thanks`) is `\footnotemark` and its text, so each steps the footnote counter, under a title
/// page too (article.cls:173, :241, report.cls:179, :247, book.cls:185, :253; latex.ltx:17215 `\thanks`): the
/// block's frontmatter notes step it as the `\maketitle` digests them (`lx_title_counts_footnotes`,
/// `\lx@title@footnote@step`), and `\footnotemark[\arabic{footnote}]` after an author's `\footnote` is that note's "∗"
/// (2201.11183 "Zhenan Fan∗1, Huang Fang∗2"; 2111.13260 "Suhani Vora1 *, Noha Radwan1 *").
pub fn declare_fnsymbol_title_block(class: &str, titlepage_default: Option<bool>) -> Result<()> {
  if !is_document_class(class) {
    return Ok(());
  }
  let mut options = String::new();
  for list in [s!("\\opt@{class}.cls"), s!("\\@classoptionslist")] {
    if let Some(defn) = lookup_definition(&T_CS!(list))?
      && let Some(ExpansionBody::Tokens(body)) = defn.get_expansion()
    {
      options = body.to_string();
      break;
    }
  }
  let mut titlepage = titlepage_default;
  if let Some(titlepage) = titlepage.as_mut() {
    for option in options.split(',').map(str::trim) {
      match option {
        "titlepage" => *titlepage = true,
        "notitlepage" => *titlepage = false,
        _ => {},
      }
    }
  }
  // (its `\thanks` steps the counter under a title page too: article.cls:173, :241, latex.ltx:17215)
  assign_value("lx_title_counts_footnotes", true, Some(Scope::Global));
  if titlepage != Some(true) {
    def_macro(
      T_CS!(TITLE_THEFOOTNOTE),
      None,
      TokenizeInternal!(r"\@fnsymbol\c@footnote"),
      Some(ExpandableOptions {
        scope: Some(Scope::Global),
        ..Default::default()
      }),
    )?;
  }
  AddToMacro!("\\lx@maketitle@body", "\\setcounter{footnote}{0}");
  Ok(())
}

/// Before a note is made in the title block (a field the frontmatter digests: the title, an author, the date) of a
/// class whose title marks its notes in a `\thefootnote` of its own ([`TITLE_THEFOOTNOTE`],
/// `declare_fnsymbol_title_block`): `\thefootnote` is that one for the rest of the field, as in the class's
/// `\maketitle` group.
pub(crate) fn enter_title_footnotes() -> Result<()> {
  // (the title block's contacts too — an affiliation, an address: the class prints them inside its title, under the same
  // marks; revtex's `\affiliation{\footnotemark[2]Lab … \footnotetext[2]{\tt volkov@…}}` answers the `\author{Mikhail S.
  // Volkov\footnotemark[2]}` above it with the same †, hep-th0302032)
  if matches!(
    lookup_string("lx_frontmatter_item").as_str(),
    "ltx:creator" | "ltx:title" | "ltx:date" | "ltx:contact"
  ) && has_meaning(&T_CS!(TITLE_THEFOOTNOTE))
  {
    let_i(&T_CS!("\\thefootnote"), &T_CS!(TITLE_THEFOOTNOTE), None);
  }
  Ok(())
}

/// A footnote's mark given as `[<n>]`: latex.ltx:17652-17657 `\@xfootnote`, :17678-17682 `\@xfootnotemark` and
/// :17694-17699 `\@xfootnotenext` set the counter to `<n>` in a group and mark the note `\thefootnote` there, so under
/// `\renewcommand\thefootnote{\fnsymbol{footnote}}` (or footmisc's `symbol`) `\footnotetext[1]` is "∗", as the
/// `\footnotemark` it answers. A mark that is not a number (`\footnotemark[$\dagger$]`, an error in LaTeX: "Missing
/// number, treated as zero", the note marked 0) is printed as given, as before and as in Perl (KNOWN_PERL_ERRORS #1060,
/// OXIDIZED_DESIGN_DIVERGENCES #851), with no diagnostic: a class may read the mark as text (pasj00.cls:2237-2239 and
/// emulateapj.cls:558-563 `\@xfootnotemark` set `\@thefnmark{#1}`, ecai.cls:480 its author `\footnotemark[\dagger]`),
/// which our locked `\footnotemark` does not see (78u, SYNC). `number` is the one `given_footnote_number` read for the
/// note; a minipage's notes are `footnote_tag`'s.
pub(crate) fn numbered_footnote_mark(
  given: &Digested,
  number: &FootnoteNumber,
) -> Result<Digested> {
  let Some(digits) = number.digits.as_ref() else {
    return digest(given.revert()?);
  };
  let mut tokens = TokenizeInternal!(r"\begingroup\c@footnote").unlist();
  tokens.extend(digits.clone().unlist());
  tokens.extend(TokenizeInternal!(r"\relax\thefootnote\endgroup").unlist());
  digest(Tokens::new(tokens))
}

/// The number a footnote's `[<n>]` gives (`given_footnote_number`).
#[derive(Default)]
pub(crate) struct FootnoteNumber {
  /// Its digits, once expanded (`2`, `\arabic{footnote}`), or a count register's value (`\value{footnote}`); none for
  /// any other mark, and for a note given no `[<n>]`, which is the counter's.
  digits:       Option<Tokens>,
  /// Whether the number moves with the footnote counter: a note given no `[<n>]`, and a `[<n>]` such as
  /// `\value{footnote}`, `\arabic{footnote}` or `\the\c@footnote` (a literal `2`, or `\thealtnum` of a counter of its
  /// own, does not).
  from_counter: bool,
}

/// The number a footnote's `[<n>]` gives (`numbered_footnote_mark`, `footnote_pairing`), read once per note (none for
/// any other type). The notes read `[<n>]` undigested (`OptionalUndigested`, sect03.rs `\lx@note`, `\lx@notemark`,
/// `\lx@notetext`), as TeX reads `\c@footnote #1\relax` — digested on its own, `\value{footnote}` was a register
/// assignment missing its number ("Missing number, treated as zero", the mark printed as nothing). The look is a check,
/// its diagnostics held: a `[<n>]` whose expansion errs (`\footnotemark[\the\relax]`) is no number, and the mark's
/// digestion as given reports the error, once, as pdflatex does (review probes rev78o2 valerr, val). Whether the number
/// is the counter's is read the same way, the counter one ahead for the look.
pub(crate) fn given_footnote_number(
  note_type: &str,
  given: Option<&Digested>,
) -> Result<FootnoteNumber> {
  if note_type != "footnote" {
    return Ok(FootnoteNumber::default());
  }
  let Some(given) = given else {
    return Ok(FootnoteNumber {
      digits:       None,
      from_counter: true,
    });
  };
  let hold = util::logger::DiagnosticsHold::begin();
  let digits = footnote_number_digits(given)?;
  let from_counter = match digits.as_ref() {
    Some(digits) => {
      let saved = counter_value("footnote")?;
      assign_register(
        "\\c@footnote",
        Number::new(saved.value_of() + 1).into(),
        None,
        Vec::new(),
      )?;
      let ahead = footnote_number_digits(given);
      assign_register("\\c@footnote", saved.into(), None, Vec::new())?;
      ahead?.map(|n| n.to_string()) != Some(digits.to_string())
    },
    None => false,
  };
  if !hold.is_empty() {
    hold.discard();
    return Ok(FootnoteNumber::default());
  }
  hold.commit();
  Ok(FootnoteNumber { digits, from_counter })
}

/// `given_footnote_number`'s digits of a `[<n>]`.
fn footnote_number_digits(given: &Digested) -> Result<Option<Tokens>> {
  let expanded = Expand!(given.revert()?);
  let text = expanded.to_string();
  let text = text.trim();
  if !text.is_empty()
    && text
      .trim_start_matches('-')
      .chars()
      .all(|c| c.is_ascii_digit())
  {
    return Ok(Some(Tokens::new(ExplodeText!(text))));
  }
  // (`\c@footnote #1\relax` reads `#1` as TeX's scan_int does, tex.web §440-§445: its leading spaces skipped, as in
  // `[ \numexpr…]`, and an internal integer read whole)
  let toks = expanded.unlist_ref();
  let lead = toks
    .iter()
    .take_while(|t| t.get_catcode() == Catcode::SPACE)
    .count();
  Ok(match &toks[lead..] {
    [register] if lookup_definition(register)?.is_some_and(|d| d.is_register()) => {
      Some(Expand!(Tokens!(T_CS!("\\the"), *register)))
    },
    // (an e-TeX integer expression, which `\c@footnote #1\relax` reads whole: `\footnotetext[\numexpr\value{footnote}-1]`,
    // 2210.14698; Perl and int59a "Missing number, treated as zero", the mark lost, KNOWN_PERL_ERRORS #1135. The
    // expression ends at its own `\relax` (`[\numexpr\value{footnote}-1\relax]`, the common form) or at ours, and the
    // `\relax` left over is unexpandable, so it is not part of the number: `\the\numexpr…\relax\relax` gives "1\relax")
    [first, rest @ ..] if *first == T_CS!("\\numexpr") => {
      let mut the = vec![T_CS!("\\the"), *first];
      the.extend(rest.iter().copied());
      the.push(T_CS!("\\relax"));
      let mut value = Expand!(Tokens::new(the)).unlist();
      while value
        .last()
        .is_some_and(|t| *t == T_CS!("\\relax") || t.get_catcode() == Catcode::SPACE)
      {
        value.pop();
      }
      let value = Tokens::new(value).to_string();
      let value = value.trim();
      (!value.is_empty()
        && value
          .trim_start_matches('-')
          .chars()
          .all(|c| c.is_ascii_digit()))
      .then(|| Tokens::new(ExplodeText!(value)))
    },
    _ => None,
  })
}

/// A footnote's bookkeeping for `relocate_footnote`, kept on the note: the value of the footnote counter it stands for
/// (`_fnvalue`: its `[<n>]`'s number, or the counter as it stepped or stands) — none for a minipage's notes, which
/// count apart, and any other type —, and, in a class that sets the counter back after its `\maketitle`
/// (`lx_title_counts_footnotes`, `declare_fnsymbol_title_block`), whether its number is the counter's (`_fncounted`,
/// `FootnoteNumber::from_counter`): such a note in the title block has the title's count, one after it the body's,
/// which counts from 1 again (`same_count`).
pub(crate) fn footnote_pairing(
  props: &mut SymHashMap<Stored>,
  note_type: &str,
  given: Option<&Digested>,
  number: &FootnoteNumber,
) -> Result<()> {
  if note_type != "footnote" || minipage_footnotes()? {
    return Ok(());
  }
  let value = match given {
    Some(_) => number
      .digits
      .as_ref()
      .and_then(|n| n.to_string().trim().parse::<i64>().ok()),
    None => Some(counter_value("footnote")?.value_of()),
  };
  if let Some(value) = value {
    props.insert("fnvalue", Stored::String(pin(value.to_string())));
  }
  if number.from_counter && lookup_bool("lx_title_counts_footnotes") {
    props.insert("fncounted", Stored::String(pin("true")));
  }
  Ok(())
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
    // LaTeX steps a note with `\stepcounter\@mpfn` (latex.ltx:17649, :17675), not `\refstepcounter`.
    let mut props = ref_step_counter(counter, false)?;
    let mark = Stored::Digested(match mark_opt {
      None => digest_text(Tokens!(T_CS!(s!("\\the{counter}"))))?,
      Some(mark) => mark.clone(),
    });
    props.insert("mark", mark);
    Ok(props)
  }
}

/// Pair a note's mark and text: Perl's `relocateFootnote` (latex_constructs.pool.ltxml:498-510)
/// matches their `mark`s exactly; here spaces are ignored, so a mark printed as set
/// (`\tablenotemark{c, d}`, deluxetable_sty.rs `shown_mark`) pairs with the text given `c,d`. A
/// counter's mark has no space, so a `\footnotemark`'s pairing is Perl's. A mark and a text that print no common
/// mark, one of them in the title block — or neither printing a mark (`title_block_partner`) —, pair by the footnote
/// counter value they stand for (`_fnvalue`, `footnote_pairing`): article's title block marks
/// `\footnotemark[2]` "†" where a `\footnotetext[2]` after `\maketitle` prints "2" (2505.14146; pdflatex prints the
/// two apart, the author pairs them), a class whose title marks are not known (siamltex: 1406.0089, 1503.07490), and
/// `\fnsymbol`'s "∥" against `\@fnsymbol`'s "‖" for 6 (2002.11721). Either way a mark and a text on the two sides of
/// a class's counter reset after `\maketitle` pair only as `same_count` admits. KNOWN_PERL_ERRORS #1060,
/// OXIDIZED_DESIGN_DIVERGENCES #851.
pub fn relocate_footnote(document: &mut Document, node: &mut Node) -> Result<()> {
  if let Some(caps) = NOTE_TEXT_END.captures(&node.get_attribute("role").unwrap_or_default()) {
    let notetype = caps.get(1).map_or("", |m| m.as_str());
    let mut paired = false;
    if let Some(mark) = node.get_attribute("mark") {
      let key = mark.replace(' ', "");
      for mut marknote in document.findnodes(
        &format!(".//ltx:note[@role='{notetype}mark'][translate(@mark,' ','')='{key}']"),
        None,
      ) {
        if is_descendant_or_self(&marknote, node) || !same_count(document, &marknote, node, BY_MARK)
        {
          continue;
        }
        relocate_footnote_aux(document, notetype, &mut marknote, node)?;
        paired = true;
      }
    }
    if !paired && let Some(value) = node.get_attribute(FOOTNOTE_VALUE) {
      let partner = title_block_partner(document, node);
      for mut marknote in document.findnodes(
        &format!(".//ltx:note[@role='{notetype}mark'][@{FOOTNOTE_VALUE}='{value}']{partner}"),
        None,
      ) {
        if is_descendant_or_self(&marknote, node)
          || !same_count(document, &marknote, node, BY_VALUE)
        {
          continue;
        }
        relocate_footnote_aux(document, notetype, &mut marknote, node)?;
      }
    }
  } else if let Some(caps) = NOTE_MARK_END.captures(&node.get_attribute("role").unwrap_or_default())
  {
    let notetype = caps.get(1).map_or("", |m| m.as_str());
    let mut paired = false;
    if let Some(mark) = node.get_attribute("mark") {
      let key = mark.replace(' ', "");
      for mut textnote in document.findnodes(
        &format!(".//ltx:note[@role='{notetype}text'][translate(@mark,' ','')='{key}']"),
        None,
      ) {
        if is_descendant_or_self(node, &textnote) || !same_count(document, node, &textnote, BY_MARK)
        {
          continue;
        }
        relocate_footnote_aux(document, notetype, node, &mut textnote)?;
        paired = true;
      }
    }
    if !paired && let Some(value) = node.get_attribute(FOOTNOTE_VALUE) {
      let partner = title_block_partner(document, node);
      for mut textnote in document.findnodes(
        &format!(".//ltx:note[@role='{notetype}text'][@{FOOTNOTE_VALUE}='{value}']{partner}"),
        None,
      ) {
        if is_descendant_or_self(node, &textnote)
          || !same_count(document, node, &textnote, BY_VALUE)
        {
          continue;
        }
        relocate_footnote_aux(document, notetype, node, &mut textnote)?;
      }
    }
  }
  Ok(())
}

/// The bookkeeping attribute holding the footnote counter value a note stands for (`footnote_pairing`).
const FOOTNOTE_VALUE: &str = "_fnvalue";

/// The bookkeeping attribute of a note numbered from the footnote counter in a class that sets the counter back after
/// its `\maketitle` (`footnote_pairing`).
const FOOTNOTE_COUNTED: &str = "_fncounted";

/// Whether the note is in the title block: in a creator, the document's title (not a section's: 1907.00457's
/// `\section{Results\footnotemark}` and the `\footnotetext` after it are one body note) or the date.
fn in_title_block(document: &mut Document, node: &Node) -> bool {
  !document
    .findnodes(
      "ancestor::ltx:creator | ancestor::ltx:title[parent::ltx:document] | ancestor::ltx:date",
      Some(node),
    )
    .is_empty()
}

/// `same_count`'s pairing by the printed mark.
const BY_MARK: bool = false;
/// `same_count`'s pairing by the counter value.
const BY_VALUE: bool = true;

/// Whether a mark and a text that print the same mark (`BY_MARK`) or stand for the same counter value (`BY_VALUE`) are
/// one note (`relocate_footnote`), in a class that sets the footnote counter back to 0 after its `\maketitle`
/// (article.cls:190, :223, report.cls:196, :229): two notes on the same side of the reset (both in the title block, or
/// neither) count alike, as before; across it, a note numbered from the counter (`_fncounted`: no `[<n>]`,
/// `[\value{footnote}]`, `[\arabic{footnote}]`) has its side's count. So a value pairs across the reset only between two
/// notes that name it (`\footnotemark[2]` "†" and the body's `\footnotetext[2]` "2", 2505.14146), and a printed mark
/// unless both are counted: article's "C†" for `\footnotemark[\value{footnote}]` after two `\thanks` is not the body's
/// `\footnotetext[2]` "2 Body text two", and report's (title page) "B2" for a bare `\footnotemark` after a `\thanks` is
/// not the body's bare `\footnotetext` "2", which answers the body's own mark (pdflatex; review probes rev78o2 art2,
/// ttp3), while a title's bare `\footnotemark` "∗" and the body's `\footnotetext[1]` "∗" under `\fnsymbol` print one
/// note (2507.05750, 1405.7962; astro-ph9712060's `[\thealtnum]` on both sides, a counter of its own).
fn same_count(document: &mut Document, mark: &Node, text: &Node, by_value: bool) -> bool {
  let counted = |node: &Node| node.get_attribute(FOOTNOTE_COUNTED).is_some();
  let (mark_counted, text_counted) = (counted(mark), counted(text));
  if !(mark_counted || text_counted)
    || in_title_block(document, mark) == in_title_block(document, text)
  {
    return true;
  }
  !by_value && !(mark_counted && text_counted)
}

/// The notes a footnote mark or text pairs with by the counter value they stand for (`relocate_footnote`), as an XPath
/// predicate on the partner: a title block's marks (article's `\@fnsymbol`, `declare_fnsymbol_title_block`) and the
/// body's numbers are the two that print no common mark for the same note, so one of the two is in the title block —
/// any note, when `node` is there, else one in a creator, the title or the date; and two notes that print no mark at all
/// (`{\renewcommand\thefootnote{}\footnotemark\footnotetext{…}}`, 2606.03847) print the same, so they pair as a mark and
/// its text printing the same mark do. A body text and a body mark of the same number that print different marks stay
/// apart, as in pdflatex: `\footnotetext[1]{…}` under `\fnsymbol` ("∗ Equal contribution") is not the table's
/// `\footnotemark` "1" (review probe rev78o f2).
fn title_block_partner(document: &mut Document, node: &Node) -> &'static str {
  if in_title_block(document, node) {
    ""
  } else if node
    .get_attribute("mark")
    .is_none_or(|mark| mark.trim().is_empty())
  {
    "[ancestor::ltx:creator or ancestor::ltx:title[parent::ltx:document] or ancestor::ltx:date or normalize-space(@mark)='']"
  } else {
    "[ancestor::ltx:creator or ancestor::ltx:title[parent::ltx:document] or ancestor::ltx:date]"
  }
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
    col.mark_intercol_after();
  }
  for col in template.get_repeated_mut() {
    col.mark_intercol_after();
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
    if let Some(strut) = array_strut() {
      properties.insert("strut", strut);
    }
  }
  if array_zeroes_interline() {
    properties.insert("zero_interline", Stored::Bool(true));
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
/// Handles \protect + excluded CS tokens (text_case_exclude mapping), and the
/// l3text case-change declarations a CS can carry (see [`lx_case_change_cs`]).
fn lx_change_case_tokens(req_case: &str, tokens: &Tokens) -> Result<Vec<Token>> {
  let mut keep = false;
  lx_change_case_in_mouth(req_case, tokens, true, &mut keep, 0)
}

/// Case-change `tokens` from a mouth of their own. `expand` is false for a
/// `\DeclareCaseChangeEquivalent` replacement, which l3text walks with its
/// case loop only, never re-expanding it (`\__text_change_case_replace:nnnn`,
/// expl3-code.tex:36693-36694), as is the material before an excluded
/// command's argument. `keep` carries the sentence/title
/// "rest of the word is left alone" state across such a nested walk.
fn lx_change_case_in_mouth(
  req_case: &str,
  tokens: &Tokens,
  expand: bool,
  keep: &mut bool,
  depth: usize,
) -> Result<Vec<Token>> {
  let mouth = Mouth::new("", None)?;
  open_mouth(mouth, false);
  unread(tokens.clone());
  let result = lx_read_and_change_case(req_case, expand, keep, depth);
  close_mouth(true)?;
  result
}

/// What [`lx_case_change_cs`] did with a CS or active token.
enum CaseChangeCs {
  /// The token (and whatever it consumed) is accounted for in the result.
  Done,
  /// The token carries no case-change declaration; the caller reads it as
  /// Perl's `latexChangeCase` does.
  Plain,
}

/// Push a token the case changer passes through VERBATIM (l3text
/// `\exp_not:n`/`\exp_not:v`). The result is re-read by the `\edef` in
/// `\MakeUppercase`, so a CS or active token is shielded with `\dont_expand`:
/// expanding it there would run it on the already case-changed tokens after
/// it (textalpha's `\LGR@hiatus` composites, `\CaseSwitch` branches).
fn push_case_verbatim(result: &mut Vec<Token>, tok: Token) {
  if tok.get_catcode().is_active_or_cs() {
    result.push(T_CS!("\\dont_expand"));
  }
  result.push(tok);
}

/// l3text's replacement for `\DeclareCaseChangeEquivalent{<cs>}{<replacement>}`
/// (latex.ltx:22400 `\cs_new_eq:NN \DeclareCaseChangeEquivalent
/// \text_declare_case_equivalent:Nn`), stored in the tl
/// `\l__text_case_<\token_to_str:N cs>_tl` (expl3-code.tex:36952-36956) and
/// read at change time (`\__text_change_case_replace:nnnN`,
/// expl3-code.tex:36684-36692). `name` is the token's string, `\>` for `\>`.
fn case_change_equivalent(name: &str) -> Result<Option<Tokens>> {
  // `\cs_if_exist:cTF` — a `\relax` left by a `\csname` probe is not a tl.
  Ok(
    match lookup_definition_stored(&T_CS!(s!("\\l__text_case_{name}_tl")))? {
      Some(Stored::Expandable(tl)) => match tl.get_expansion() {
        Some(ExpansionBody::Tokens(replacement)) => Some(replacement.clone()),
        _ => None,
      },
      _ => None,
    },
  )
}

/// Whether `tok` is `\CaseSwitch` (latex.ltx:22399 `\cs_new_eq:NN \CaseSwitch
/// \text_case_switch:nnnn`): l3text's case loop tests the meaning
/// (expl3-code.tex:36698 `\cs_if_eq:NNTF #4 \text_case_switch:nnnn`).
fn is_case_switch(tok: &Token) -> bool {
  let switch = T_CS!("\\text_case_switch:nnnn");
  has_meaning(&switch) && x_equals(tok, &switch)
}

/// The letter-like mapping of the CS named `name` in `map` (`text_uppercase`
/// or `text_lowercase`, keyed `"<cs> "` by `\lx@prepare@case@mapping`): l3text's
/// `c__text_<case>case_<cs>_tl` constants (expl3-code.tex:36749-36768
/// `\__text_change_case_letterlike:nnnnnN`).
fn case_letterlike(map: &str, name: &str) -> Option<Token> {
  match lookup_mapping(map, &s!("{name} ")) {
    Some(Stored::Token(mapped)) => Some(mapped),
    _ => None,
  }
}

/// l3text's exclusion list (`\AddToNoCaseChangeList`, `\l_text_case_exclude_arg_tl`,
/// expl3-code.tex:36002-36007), the first check of both its passes
/// (`\__text_expand_exclude:N`, 36192-36233; `\__text_change_case_exclude:nnnN`,
/// 36632-36683). The command passes unchanged; everything up to its first `{`
/// (a star, optional arguments: `#5` of `\__text_change_case_exclude:nnnNw`,
/// 36668-36669) is case-changed without expansion, and the braced argument is
/// kept as is (36670-36683). So `\MakeUppercase{\cite[see][p.~5]{k}}` is
/// `\cite[SEE][P.~5]{k}`, and `\cite*{k}` keeps its star. Perl reads one
/// `[opt]` and cases it with an expanding changer. `prefix` is the `\protect`
/// of a robust command, whose output keeps Perl's shape.
fn lx_case_change_excluded(
  req_case: &str,
  tok: Token,
  name: &str,
  prefix: Option<Token>,
  depth: usize,
  result: &mut Vec<Token>,
) -> Result<bool> {
  if lookup_mapping("text_case_exclude", name).is_none() {
    return Ok(false);
  }
  let mut before_arg = Vec::new();
  while let Some(t) = read_token_across_input_ends()? {
    if t.get_catcode() == Catcode::BEGIN {
      unread_one(t);
      break;
    }
    before_arg.push(t);
  }
  match prefix {
    Some(protect) => {
      result.push(protect);
      result.push(tok);
    },
    None => push_case_verbatim(result, tok),
  }
  if !before_arg.is_empty() {
    let mut keep = false;
    result.extend(lx_change_case_in_mouth(
      req_case,
      &Tokens::new(before_arg),
      false,
      &mut keep,
      depth + 1,
    )?);
  }
  let arg = read_arg(ExpansionLevel::Off)?;
  result.push(T_BEGIN!());
  result.extend(arg.unlist());
  result.push(T_END!());
  Ok(true)
}

/// l3text's case-change declarations on a CS met by its case loop, in its
/// order: a `\DeclareCaseChangeEquivalent` replacement (expl3-code.tex:
/// 36684-36695), `\CaseSwitch` (36696-36717), then the letter-like table built
/// from `\@uclclist` (36749-36768). l3text runs them on the output of
/// `\text_expand:n`, in which robust, protected and encoding commands survive
/// unexpanded (36274-36367) while other macros are expanded away. The changer
/// here expands as it reads, so, approximating the two passes, it checks the
/// declarations of EVERY command before expanding it. Without that,
/// `\MakeUppercase{\>\`α}` expanded textalpha's `\>` (equivalent
/// `\CaseSwitch{\>}{\LGR@hiatus}{\>}{\>}`, textalpha.sty:213-214) into
/// `\add@unicode@accent{"0313}{\`}`, whose accent then took `\char` as its
/// argument ("Missing number", `”0313` printed). Witness: greek-fontenc
/// char-list (lualatex).
///
/// In the sentence/title "rest of the word" state (`keep`), l3text copies
/// tokens verbatim (`\__text_change_case_skip:nnw`, 36718-36740), so a declared
/// token is passed through unexpanded rather than acted on. `name` is the
/// token's string without the trailing space of a robust command's inner name.
fn lx_case_change_cs(
  req_case: &str,
  tok: Token,
  name: &str,
  prefix: Option<Token>,
  keep: &mut bool,
  depth: usize,
  result: &mut Vec<Token>,
) -> Result<CaseChangeCs> {
  let titling = req_case == "sentence" || req_case == "title";
  let push_self = |result: &mut Vec<Token>| {
    if let Some(protect) = prefix {
      result.push(protect);
    }
    push_case_verbatim(result, tok);
  };
  if let Some(replacement) = case_change_equivalent(name)? {
    if *keep {
      push_self(result);
    } else if depth >= MAX_CASE_EQUIVALENT_DEPTH {
      Error!(
        "recursion",
        name,
        s!(
          "Case-change equivalent of {name} nests deeper than {MAX_CASE_EQUIVALENT_DEPTH} levels; it is left unchanged"
        )
      );
      push_self(result);
    } else {
      result.extend(lx_change_case_in_mouth(
        req_case,
        &replacement,
        false,
        keep,
        depth + 1,
      )?);
    }
    return Ok(CaseChangeCs::Done);
  }
  if prefix.is_none() && is_case_switch(&tok) {
    // `\CaseSwitch{<normal>}{<upper>}{<lower>}{<title>}`
    let args = [
      read_arg(ExpansionLevel::Off)?,
      read_arg(ExpansionLevel::Off)?,
      read_arg(ExpansionLevel::Off)?,
      read_arg(ExpansionLevel::Off)?,
    ];
    if *keep {
      push_case_verbatim(result, tok);
      for arg in args {
        result.push(T_BEGIN!());
        for t in arg.unlist() {
          push_case_verbatim(result, t);
        }
        result.push(T_END!());
      }
    } else {
      let [_, upper, lower, title] = args;
      let chosen = match req_case {
        "upper" => upper,
        "lower" => lower,
        _ => title,
      };
      for t in chosen.unlist() {
        push_case_verbatim(result, t);
      }
      if titling {
        *keep = true;
      }
    }
    return Ok(CaseChangeCs::Done);
  }
  let is_upper = req_case == "upper" || titling;
  let (map, other_map) = if is_upper {
    ("text_uppercase", "text_lowercase")
  } else {
    ("text_lowercase", "text_uppercase")
  };
  if let Some(mapped) = case_letterlike(map, name) {
    if *keep {
      push_self(result);
    } else {
      push_case_verbatim(result, mapped);
      if titling {
        *keep = true;
      }
    }
    return Ok(CaseChangeCs::Done);
  }
  // Already in the requested case: l3text still counts it as the letter that
  // starts a title-cased word (expl3-code.tex:36758-36766).
  if case_letterlike(other_map, name).is_some() {
    push_self(result);
    if titling {
      *keep = true;
    }
    return Ok(CaseChangeCs::Done);
  }
  Ok(CaseChangeCs::Plain)
}

/// l3text's `\text_declare_expand_equivalent:Nn` replacement, stored in
/// `\l__text_expand_<\token_to_str:N cs>_tl` (expl3-code.tex:36395-36399) and
/// re-scanned by `\text_expand:n` in place of the CS (`\__text_expand_replace:N`,
/// 36339-36354). The kernel
/// declares every text accent and `\AA`…`\th` as themselves under `\exp_not:n`
/// (36401-36418): an accent reaches typesetting with its cased argument, as in
/// LaTeX (`\MakeUppercase{\'e}` typesets `\'E`), instead of being expanded
/// before its argument is cased.
fn case_expand_equivalent(name: &str) -> Result<Option<Tokens>> {
  Ok(
    match lookup_definition_stored(&T_CS!(s!("\\l__text_expand_{name}_tl")))? {
      Some(Stored::Expandable(tl)) => match tl.get_expansion() {
        Some(ExpansionBody::Tokens(replacement)) => Some(replacement.clone()),
        _ => None,
      },
      _ => None,
    },
  )
}

/// l3text's `\__text_expand_testopt:N` (expl3-code.tex:36319-36328): when a
/// command's one-step expansion opens with `\@protected@testopt`, the command
/// is stored unexpanded and `\@protected@testopt`'s other two arguments are
/// dropped, so the optional argument is read where the command is typeset.
/// Every `\newcommand` optional-argument command expands that way
/// (latex.ltx:1249 `\@xargdef`); ours carry the `testopt` flag on their
/// optional parameter instead of the `\@protected@testopt` body. (`\@yargdef`
/// also flags the inner `\\cs`, a plain `[#1]` macro in latex.ltx that l3text
/// would expand; the loop never reaches it, as the outer command is stored
/// first.) Returns the tokens to store, or `None` when `tok` is no such
/// command. Guard: `case_change_equivalents::optional_argument_commands_stay_unexpanded`.
fn case_testopt_store(tok: Token) -> Result<Option<Vec<Token>>> {
  let protected_testopt = T_CS!("\\@protected@testopt");
  if has_meaning(&protected_testopt) && x_equals(&tok, &protected_testopt) {
    // `\@protected@testopt <cmd> <\\cmd> {<default>}` spelled out directly.
    let cmd = read_arg(ExpansionLevel::Off)?;
    read_arg(ExpansionLevel::Off)?;
    read_arg(ExpansionLevel::Off)?;
    return Ok(Some(cmd.unlist()));
  }
  let testopt = lookup_expandable(&tok, None)?.is_some_and(|defn| {
    defn.get_parameters().is_some_and(|params| {
      params
        .get_parameters()
        .first()
        .is_some_and(|param| param.testopt)
    })
  });
  Ok(testopt.then(|| vec![tok]))
}

/// l3text's `\__text_expand_encoding:N` (expl3-code.tex:36330-36338), the next
/// check after `\__text_expand_testopt:N`: an encoding-dispatched text command
/// expands one step to `\<enc>-cmd <cmd> \<enc><cmd>`, the `\<enc>-cmd` being
/// `\@current@cmd` or `\@changed@cmd` (latex.ltx:9865/9871), and
/// `\__text_expand_encoding_escape:NN` keeps `<cmd>` under `\exp_not:n` and
/// drops `\<enc><cmd>`. Our `\DeclareTextCommand` family dispatches without
/// that step (sect08.rs `def_text_command_dispatcher`: `<cmd>` expands straight
/// to `\expandafter\ifx\csname\cf@encoding\string<cmd>\endcsname…`), so a
/// command that expands that way is the one the escape keeps. Expanded
/// instead, textalpha's `ά` (`\ensuregreek{\acctonos\textalpha}`) reached
/// `\greekscript`'s `\def\encodingdefault{LGR}` with `\encodingdefault`
/// expanded to `T1`, a definition named `T` (textalpha-doc, char-list,
/// hyperref-with-greek), and `\@changed@cmd` reached `\@use@text@encoding`'s
/// `\edef\f@encoding` (latex.ltx:9975). Returns the kept tokens, or `None`
/// for any other token. The escape's two arguments are undelimited
/// (`\__text_expand_encoding_escape:NN #1#2`); a command `\let` to a text
/// command keeps the command its dispatch names, as expanding it one step
/// and escaping would (`\let\foo\textalpha` keeps `\textalpha`).
/// Guards: `sweep125_roots::{case_change_keeps_a_text_command,
/// case_change_keeps_encoding_dispatched_command}`.
fn case_encoding_escape(tok: Token) -> Result<Option<Vec<Token>>> {
  let is_dispatch = ["\\@current@cmd", "\\@changed@cmd"]
    .into_iter()
    .any(|name| {
      let dispatch = T_CS!(name);
      has_meaning(&dispatch) && x_equals(&tok, &dispatch)
    });
  if is_dispatch {
    let kept = read_arg(ExpansionLevel::Off)?;
    read_arg(ExpansionLevel::Off)?;
    return Ok(Some(kept.unlist()));
  }
  Ok(text_command_dispatched_name(&tok)?.map(|name| vec![name]))
}

/// The command our text-command encoding dispatch (sect08.rs
/// `def_text_command_dispatcher`) names, when `tok` expands to it: the body
/// opens with `\expandafter\ifx\csname\cf@encoding\string<cmd>`.
fn text_command_dispatched_name(tok: &Token) -> Result<Option<Token>> {
  let Some(defn) = lookup_expandable(tok, None)? else {
    return Ok(None);
  };
  let Some(ExpansionBody::Tokens(body)) = defn.get_expansion() else {
    return Ok(None);
  };
  let head = [
    "\\expandafter",
    "\\ifx",
    "\\csname",
    "\\cf@encoding",
    "\\string",
  ];
  let body = body.unlist_ref();
  let opens_dispatch = body.len() > head.len()
    && head
      .iter()
      .zip(body)
      .all(|(name, t)| t.get_catcode() == Catcode::CS && t.with_str(|s| s == *name));
  Ok(opens_dispatch.then(|| body[head.len()]))
}

/// Whether `tok` means `\unexpanded` (`\exp_not:n`): `\text_expand:n` keeps its
/// argument unexpanded, the case loop still changes it
/// (`\__text_expand_cs_expand:N`/`\__text_expand_unexpanded:w`,
/// expl3-code.tex:36355-36394).
fn is_unexpanded(tok: &Token) -> bool {
  let unexpanded = T_CS!("\\unexpanded");
  has_meaning(&unexpanded) && x_equals(tok, &unexpanded)
}

/// l3text `\__text_expand_explicit:N` (expl3-code.tex:36165-36191): an active
/// character is expanded unless it is protected, or its expansion starts with a
/// babel shorthand's `\active@prefix` or a UTF-8 octet handler; its case is then
/// that of the character (36622-36631), with no case-change declarations. A
/// babel shorthand kept whole composes as typed: babel-greek's `~` in
/// textalpha's `\>~` (greek-fontenc char-list-alphabeta).
fn active_char_kept(tok: &Token) -> Result<bool> {
  let Some(defn) = lookup_expandable(tok, None)? else {
    return Ok(false);
  };
  if defn.is_protected() {
    return Ok(true);
  }
  let head = match defn.get_expansion() {
    Some(ExpansionBody::Tokens(body)) => body.unlist_ref().first().copied(),
    _ => None,
  };
  Ok(head.is_some_and(|head| {
    [
      "\\UTFviii@two@octets",
      "\\UTFviii@three@octets",
      "\\UTFviii@four@octets",
      "\\active@prefix",
    ]
    .into_iter()
    .any(|marker| {
      let marker = T_CS!(marker);
      has_meaning(&marker) && x_equals(&head, &marker)
    })
  }))
}

/// Bound on nested `\DeclareCaseChangeEquivalent` replacements (an equivalent
/// naming its own command would otherwise recurse without end; l3text loops).
const MAX_CASE_EQUIVALENT_DEPTH: usize = 32;

fn lx_read_and_change_case(
  req_case: &str,
  expand: bool,
  keep: &mut bool,
  depth: usize,
) -> Result<Vec<Token>> {
  let mut result = vec![];
  let mut in_math = false;
  let is_upper = req_case == "upper" || req_case == "sentence" || req_case == "title";
  // `sentence`/`title` (`\text_titlecase_first:n`/`_all:n`, which
  // `\MakeTitlecase` is in the kernel) uppercase the first letter (of each
  // word, for `title`) and leave the rest as it is (l3text, TeX Live 2025:
  // `\text_titlecase_all:n{hELLO wORLD}` is `HELLO WORLD`). Perl lowercases the
  // rest (latex_constructs.pool.ltxml:5507-5516 notes the ambiguity).
  loop {
    // Outside math a CS is read unexpanded first, so its case-change
    // declarations are seen before it is expanded (`lx_case_change_cs`), and
    // each expansion step is re-checked (`\__text_expand_cs_expand:N` expands
    // one step and re-enters its loop, expl3-code.tex:36355-36367). The read
    // crosses the end of an `\input` file or `\scantokens` pseudo-file into
    // the argument, as `read_x_token` does.
    let next = if in_math && expand {
      read_x_token(Some(false), false, None)?
    } else {
      read_token_across_input_ends()?
    };
    let mut tok = match next {
      None => break,
      Some(t) => t,
    };
    if !in_math && tok.get_catcode() == Catcode::ACTIVE {
      if active_char_kept(&tok)? || !expand {
        push_case_verbatim(&mut result, tok);
        continue;
      }
      if expand_once_partial(tok)? {
        continue;
      }
      unread_one(tok);
      tok = match read_x_token(Some(false), false, None)? {
        None => break,
        Some(t) => t,
      };
    } else if !in_math && tok.get_catcode() == Catcode::CS && !tok.with_str(|s| s == "\\protect") {
      let name = tok.with_str(|s| s.to_string());
      if lx_case_change_excluded(req_case, tok, &name, None, depth, &mut result)? {
        continue;
      }
      // `\text_expand:n` replaces or shields the token before the case loop
      // ever sees it (expl3-code.tex:36339-36394).
      if expand {
        if let Some(replacement) = case_expand_equivalent(&name)? {
          unread_expansion(replacement);
          continue;
        }
        if is_unexpanded(&tok) {
          let unexpanded = read_arg(ExpansionLevel::Off)?;
          result.extend(lx_change_case_in_mouth(
            req_case,
            &unexpanded,
            false,
            keep,
            depth + 1,
          )?);
          continue;
        }
      }
      match lx_case_change_cs(req_case, tok, &name, None, keep, depth, &mut result)? {
        CaseChangeCs::Done => continue,
        CaseChangeCs::Plain if !expand => {
          push_case_verbatim(&mut result, tok);
          continue;
        },
        CaseChangeCs::Plain => {
          if let Some(stored) = case_testopt_store(tok)? {
            for t in stored {
              push_case_verbatim(&mut result, t);
            }
            continue;
          }
          // The kept command is `\exp_not:n`'d: changed, not expanded, as
          // `\unexpanded` above is.
          if let Some(kept) = case_encoding_escape(tok)? {
            result.extend(lx_change_case_in_mouth(
              req_case,
              &Tokens::new(kept),
              false,
              keep,
              depth + 1,
            )?);
            continue;
          }
          if expand_once_partial(tok)? {
            continue;
          }
          // Unexpandable, undefined or `\let` to a character: the reader
          // Perl uses decides (an undefined CS becomes its error stub).
          unread_one(tok);
          tok = match read_x_token(Some(false), false, None)? {
            None => break,
            Some(t) => t,
          };
        },
      }
    }
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
        if let Some(next) = read_token_across_input_ends()? {
          result.push(tok);
          result.push(T_CS!("\\dont_expand"));
          result.push(next);
        } else {
          result.push(tok);
        }
      } else if expand {
        result.push(tok);
      } else {
        push_case_verbatim(&mut result, tok);
      }
    } else if *keep && (cc == Catcode::LETTER || cc == Catcode::OTHER) {
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
      // the case changes the character, not the token's out-of-band field provenance (`Token::prov`, as `\uppercase`)
      let new_tok = if changed {
        Token {
          prov: tok.prov,
          ..Token::new(new_str, cc)
        }
      } else {
        tok
      };
      result.push(new_tok);
      if req_case == "sentence" || req_case == "title" {
        *keep = true;
      }
    } else if cc == Catcode::SPACE {
      result.push(Token { prov: tok.prov, ..T_SPACE!() });
      if req_case == "title" {
        *keep = false;
      }
    } else if cc == Catcode::CS && tok.with_str(|s| s == "\\protect") {
      if let Some(next_tok) = read_token_across_input_ends()? {
        // Perl: $cs->getString (full CS name). Munged-robust CSes carry a
        // trailing space — canonicalise to NO trailing space for the
        // exclude and equivalent lookups (matches \AddToNoCaseChangeList
        // storage format, and l3text's `\__text_expand_protect:Nw`, which
        // hands the case loop the user-level `\foo` for `\protect\foo `,
        // expl3-code.tex:36308-36318), and to "CS + trailing space" for the
        // case-mapping lookup (matches `\lx@prepare@case@mapping` storage
        // format, which is `$lower->getString . ' '` in Perl).
        let next_key_bare = next_tok.with_str(|s| s.trim_end().to_string());
        if lx_case_change_excluded(
          req_case,
          next_tok,
          &next_key_bare,
          Some(tok),
          depth,
          &mut result,
        )? {
          continue;
        }
        let declared = lx_case_change_cs(
          req_case,
          next_tok,
          &next_key_bare,
          Some(tok),
          keep,
          depth,
          &mut result,
        )?;
        if let CaseChangeCs::Plain = declared {
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
        }
      }
    } else if expand {
      result.push(tok);
    } else {
      push_case_verbatim(&mut result, tok);
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
///
/// Local scope, like Perl: a nested context (`\centering` in a minipage
/// panel inside a centered figure) restores the outer one when the panel's
/// collected frame pops at build time (tests/graphics/graphrot). Under
/// streaming the build step runs inside `state::with_build_floor`, which keeps
/// a base-level binding from popping with a DIGESTION group instead.
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
  if let Some(mut node) = node_opt.clone()
    && is_heading_name(document::get_node_qname(&node))
  {
    // A heading is one paragraph, and TeX's `\centering`/`\raggedright`/`\raggedleft` set
    // `\leftskip`/`\rightskip`/`\parfillskip` for the paragraph they end (latex.ltx:15409-15423):
    // the heading's own, whichever of its inline children follow the switch. Perl sets it on those
    // children (latex_constructs.pool.ltxml:1225-1235) — an `align` on a title's inline `<text>`
    // that centres nothing and blocks the font collapsing onto the title, so a size in the format
    // multiplies with the title's own (a titlesec `{\huge\bfseries\filcenter}` chapter at ≈517%).
    // Centring is the text alignment only (`ltx_align_center`): `ltx_centering` is a centred block,
    // which would stop a run-in heading running in. OXIDIZED_DESIGN_DIVERGENCES #344.
    let class = if class == "ltx_centering" {
      "ltx_align_center"
    } else {
      class
    };
    set_align_or_class(document, &mut node, align, class)?;
  } else if let Some(node) = node_opt {
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
  // so nothing ever pops them — under streaming `state::with_build_floor`
  // restores that invariant), and each retained `Stored::Node` pins its
  // whole document — under streaming, a pinned handle into a SPILLED subtree
  // even blocks the `xmlFreeNode` the spill relies on. Overwrite with `None`
  // so the C trees can go; the (empty) stack entries themselves are the cheap
  // part (dhat: 288 B each).
  assign_value("ALIGNING_NODE", Stored::None, None);
  assign_value("ALIGNING_PREV_CHILD", Stored::None, None);
  Ok(())
}

/// A heading element: its content is the one paragraph an alignment switch inside it sets.
fn is_heading_name(qname: SymStr) -> bool {
  qname == pin!("ltx:title") || qname == pin!("ltx:toctitle") || qname == pin!("ltx:subtitle")
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
  // The body is decoded through the document's 8-bit input encoding (inputenc's
  // active upper-half characters keep their meaning: latex.ltx:15441-15459
  // `\@verbatim` resets only `\dospecials`), as `.bib` lines are. A cp1251
  // listing came out as Latin-1 mojibake (russ_doc, serbian-apostrophe).
  // Guard: `perfect_kernel_batch56::verbatim_decodes_the_input_encoding`.
  while let Some(next_line) = read_raw_line_decoded() {
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
      // Hand `\end{verbatim}` back to the CURRENT `\end` macro instead of
      // consuming it here: latex.ltx:15438 `\def\@xverbatim#1\end{verbatim}
      // {#1\end{verbatim}}` re-executes the terminator, so packages that hook
      // `\begin`/`\end` (knowledge.sty:1671-1685 pushes a scope area at
      // `\begin{verbatim}` and pops it at `\end{verbatim}`) stay balanced.
      // Perl's fused constructor (latex_constructs.pool.ltxml:1734) swallows
      // the terminator, but Perl never defines `\verbatim`/`\endverbatim`, so
      // knowledge's `\KnowledgeConfigureEnvironment?` (:3754, gated on both
      // existing) never registers the area there; our dumped latex.ltx does
      // define them. The fused `\end{verbatim}` (sect06.rs) is a no-op so the
      // kernel `\end` adds nothing: the group already closed below.
      // OXIDIZED_DESIGN_DIVERGENCES #199. Guard:
      // `perfect_kernel_batch56::package_state_prtec_psfragx_knowledge`.
      let mut end_tokens = vec![T_CS!("\\end"), T_BEGIN!()];
      end_tokens.extend(ExplodeText!(if starred { "verbatim*" } else { "verbatim" }));
      end_tokens.push(T_END!());
      unread_expansion(Tokens::new(end_tokens));
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
        Token!(sym line, Catcode::OTHER).into(),
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
      ref_step_counter_by_meaning(&ctr)?
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
        ref_step_counter_by_meaning(&ctr)?
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
/// Perl: latex_constructs.pool.ltxml lines 2025-2035. Two IEEEeqnarray additions (the numbering key `subnumbers=1`),
/// on every path that retracts a row (its end, an immediate `\nonumber` in `IEEEeqnarray*`, a `\tag`): a row whose
/// pre-advance of `equation` reset a sub-equation number ([`SUBEQUATION_ROLLBACK`]) gives it back, as IEEEtrantools
/// steps nothing for a row it leaves unnumbered (`\IEEEnonumber`, IEEEtrantools.sty:1405; `IEEEeqnarray*`'s `e &=& f
/// \nonumber \\ g &=& h \IEEEyessubnumber`, pdflatex "0f", this port "0a" when the retraction dropped the key; witness
/// 1610.06458 `eq:inq3`, PDF 36c); and a row whose counters an `\IEEEyesnumber`/`\IEEEyessubnumber` set
/// ([`EQUATIONROW_HOLDS`]) keeps them, its number passed to the next row ([`EQUATIONROW_CARRY`]).
fn retract_equation() {
  // Phase 1: Gather data (immutable borrows)
  let (ctr, is_preset, is_numbered, rollback, holds) =
    with_value("EQUATION_NUMBERING", |eq_num_opt| {
      let numbering = match eq_num_opt {
        Some(Stored::HashStored(n)) => n,
        _ => return (String::from("equation"), false, false, None, false),
      };
      let is_numbered = matches!(numbering.get("numbered"), Some(&Stored::Bool(true)));
      with_value("EQUATIONROW_TAGS", |tags_opt| {
        let tags = match tags_opt {
          Some(Stored::HashStored(t)) => t,
          _ => return (String::from("equation"), false, is_numbered, None, false),
        };
        let ctr = tags
          .get("counter")
          .map_or_else(|| numbering.get("counter"), Some)
          .map(ToString::to_string)
          .unwrap_or_else(|| String::from("equation"));
        let is_preset = matches!(tags.get("preset"), Some(&Stored::Bool(true)));
        let rollback = match tags.get(SUBEQUATION_ROLLBACK) {
          Some(Stored::Int(rollback)) => Some(*rollback),
          _ => None,
        };
        let holds = matches!(tags.get(EQUATIONROW_HOLDS), Some(&Stored::Bool(true)));
        (ctr, is_preset, is_numbered, rollback, holds)
      })
    });
  // Phase 2: Mutate state (borrows released)
  if holds {
    // (IEEEtrantools' `\global\@IEEEeqnumpreadvfalse`, :1402, :1422: the counters are the row's number, which the
    // next row prints, :2086-2088; or none follows and they stay, :2022-2023)
    let carry = if counter_value("subequation").is_ok_and(|n| n.value_of() > 0) {
      "subequation"
    } else {
      "equation"
    };
    assign_value(
      EQUATIONROW_CARRY,
      Stored::String(pin(carry)),
      Some(Scope::Global),
    );
  } else if is_preset {
    // counter (or ID counter) was stepped, so decrement it.
    let counter_name = if is_numbered {
      ctr.clone()
    } else {
      s!("UN{}", ctr)
    };
    let _ = add_to_counter(&counter_name, Number::new(-1));
  }
  if let Some(rollback) = rollback
    && let Ok(current) = counter_value("subequation")
  {
    let _ = add_to_counter("subequation", Number::new(rollback - current.value_of()));
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
  install_eqnarray_alignment(eqnarray_columns(), 3)?;
  // latex.ltx:15793 (`\@@eqncr`): a row ended by `\\` in the number column has too many columns.
  let_i(&T_CS!("\\\\"), &T_CS!("\\lx@eqnarray@newline"), None);
  Ok(())
}

/// The eqnarray's columns: Perl's three (latex_constructs.pool.ltxml:2231-2236), right, centre and left, math in
/// `\displaystyle`, and latex.ltx's fourth (:15764, `\hb@xt@\z@\bgroup\hss##\egroup`), the equation number's, in
/// text. A row ended by `\cr` may fill it (1804.05059: `&\cr`); `rearrange_eqnarray` drops it where empty. The math
/// columns keep latex.ltx's braces, `$\displaystyle{##}$` (:15760-15763; tex.web §789), as `\lx@template@bgroup`/
/// `\lx@template@egroup` (tex_box.rs): a cell whose `{`/`}` balance only across them (`G_t(h)=}\det(h)-e^{\frac{a}{2}g_t.`,
/// 2207.06932, 1912.10594) is one formula, as in pdflatex; Perl's template has none (KNOWN_PERL_ERRORS #908). Guard
/// `perfect_kernel_batch69c::eqnarray_cell_braces_balance_across_the_template`.
fn eqnarray_columns() -> Vec<Cell> {
  let col1 = Cell {
    before: Some(Tokens::new(vec![
      T_CS!("\\hfil"),
      T_MATH!(),
      T_CS!("\\displaystyle"),
      T_CS!("\\lx@template@bgroup"),
    ])),
    after: Some(Tokens::new(vec![T_CS!("\\lx@template@egroup"), T_MATH!()])),
    empty: true,
    ..Cell::default()
  };
  let col2 = Cell {
    before: Some(Tokens::new(vec![
      T_CS!("\\hfil"),
      T_MATH!(),
      T_CS!("\\displaystyle"),
      T_CS!("\\lx@template@bgroup"),
    ])),
    after: Some(Tokens::new(vec![
      T_CS!("\\lx@template@egroup"),
      T_MATH!(),
      T_CS!("\\hfil"),
    ])),
    empty: true,
    ..Cell::default()
  };
  let col3 = Cell {
    before: Some(Tokens::new(vec![
      T_MATH!(),
      T_CS!("\\displaystyle"),
      T_CS!("\\lx@template@bgroup"),
    ])),
    after: Some(Tokens::new(vec![
      T_CS!("\\lx@template@egroup"),
      T_MATH!(),
      T_CS!("\\hfil"),
    ])),
    empty: true,
    ..Cell::default()
  };
  let number = Cell {
    before: Some(Tokens::new(vec![T_CS!("\\hfil")])),
    empty: true,
    ..Cell::default()
  };
  vec![col1, col2, col3, number]
}

/// [`eqnarray_bindings`] over the given columns: an eqnarray-style alignment whose rows are numbered equations,
/// rearranged at the end by `rearrange_eqnarray` (IEEEtrantools' `\IEEEeqnarray`, whose column specification makes
/// its own preamble, IEEEtrantools.sty:1996 `\@IEEEbuildpreamble`).
pub fn eqnarray_bindings_with_columns(columns: Vec<Cell>) -> Result<()> {
  let ncolumns = columns.len();
  install_eqnarray_alignment(columns, ncolumns)
}

/// The eqnarray alignment over `columns`, of which the first `equation_columns` hold the equations
/// (`rearrange_eqnarray` reads those; any after are the number column's).
fn install_eqnarray_alignment(columns: Vec<Cell>, equation_columns: usize) -> Result<()> {
  // Ensure @equationgroup counter exists — it's normally created by article.cls,
  // but standalone classes (appolb, jpsj2, etc.) may not define it.
  if lookup_definition(&T_CS!("\\the@equationgroup@ID"))?.is_none() {
    NewCounter!("@equationgroup", "document", idprefix => "EG", idwithin => "section");
  }
  // The equation columns' count, for `rearrange_eqnarray`: a row with an extra `&` grows the alignment past the
  // template (`next_column`), and Perl reads the eqnarray's columns 0/1/2 whatever the rows hold.
  let ncolumns = equation_columns.to_string();
  let has_number_column = equation_columns < columns.len();
  let template = Template::new(TemplateConfig {
    columns: Some(columns),
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
  // latex.ltx `\eqnarray`: `$$\halign to\displaywidth` (IEEEtrantools.sty:2013 alike).
  properties.insert(crate::tex_tables::ALIGNMENT_IS_DISPLAY, Stored::Bool(true));
  // The columns past the equations' are the number column (latex.ltx:15764), which `\lx@eqnarray@number@check`
  // guards.
  if has_number_column {
    properties.insert(
      "eqnarray_columns",
      Stored::Number(Number::new(equation_columns as i64)),
    );
  }
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
      props.insert(String::from("_ncolumns"), ncolumns.clone());
      if has_number_column {
        props.insert(String::from("_number_column"), String::from("true"));
      }
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
      // (the next number's reference forms of a labelled unnumbered row, `NEXT_NUMBER_TAGS_PROPERTY`, which
      // `rearrange_eqnarray` keeps or drops)
      let next_number_tags = props.remove(NEXT_NUMBER_TAGS_PROPERTY);
      if next_number_tags.is_some() {
        props.insert(NEXT_NUMBER_ROW_ATTRIBUTE.to_string(), Stored::from("true"));
      }
      let str_props: HashMap<String, String> =
        props.into_iter().map(|(k, v)| (k, v.to_string())).collect();
      document.open_element("ltx:equation", Some(str_props), None)?;
      // If we have digested tags, absorb them into the opened element
      if let Some(Stored::Digested(d)) = tags_digested {
        document.absorb(&d, None)?;
      }
      if let Some(Stored::Digested(d)) = next_number_tags {
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
    node:             Node,
    cols:             Vec<Node>,
    has_l:            bool,
    has_m:            bool,
    has_r:            bool,
    numbered:         bool,
    labelled:         bool,
    tagged:           bool,
    unnumbered_eqncr: bool,
    next_number:      bool,
    keep_next_number: bool,
  }

  // Scan the equations (rows). Perl reads the eqnarray's three columns; an `\IEEEeqnarray` has as many as its
  // specification (`eqnarray_bindings_with_columns`, which records the template's width on the group), read the same
  // way: the first column (`l`), the last (`r`) and any between (`m`) — with three columns, Perl's. With fewer than
  // three there is no `r`: `{rl}`'s second column is the relation's (`m`).
  let ncols = equationgroup
    .get_attribute("_ncolumns")
    .and_then(|n| n.parse::<usize>().ok())
    .unwrap_or(3);
  let _ = equationgroup.remove_attribute("_ncolumns");
  // eqnarray's number column (latex.ltx:15764), which an `\IEEEeqnarray` lacks.
  let number_column = equationgroup.get_attribute("_number_column").is_some();
  let _ = equationgroup.remove_attribute("_number_column");
  let row_cells: Vec<(Node, Vec<Node>)> = document
    .findnodes("ltx:equation", Some(equationgroup))
    .into_iter()
    .map(|rownode| {
      let cells = document.findnodes("ltx:_Capture_", Some(&rownode));
      let cells = if number_column {
        drop_empty_number_cells(document, cells, ncols)
      } else {
        cells
      };
      (rownode, cells)
    })
    .collect();
  let mut rows: Vec<EqRow> = Vec::new();
  for (rownode, cells) in row_cells {
    let used = |c: &Node| c.get_first_child().is_some();
    let has_l = cells.first().is_some_and(used);
    let last_m = if ncols >= 3 { ncols - 1 } else { ncols };
    let has_m = cells.iter().take(last_m).skip(1).any(used);
    let has_r = ncols >= 3 && cells.get(ncols - 1).is_some_and(used);
    // (a labelled unnumbered row's tags are the next number's reference forms, `NEXT_NUMBER_ROW_ATTRIBUTE`: no number
    // of its own)
    let next_number = rownode.get_attribute(NEXT_NUMBER_ROW_ATTRIBUTE).is_some();
    let numbered = !next_number && !document.findnodes("ltx:tags", Some(&rownode)).is_empty();
    // OXIDIZED_DESIGN #54: Perl checks hasAttribute('label') (singular), but
    // LaTeXML only ever sets the plural 'labels' attribute (LaTeXML-common.rnc
    // L134) — so the author's documented "Separately numbered AND labeled? must
    // keep separate" safeguard below is dead code in Perl, collapsing distinctly
    // \label-ed continuation rows onto one number. We honor the intent (and match
    // pdfTeX) by reading the real 'labels' attribute. See KNOWN_PERL_ERRORS #46.
    let labelled = rownode.get_attribute("labels").is_some();
    // a row its author tagged (`… \nonumber \\ && \TCItag{27}`; the alignment's `_tagged` row, `Template::has_explicit_tags`)
    let tagged = rownode.get_attribute(TAGGED_ROW_ATTRIBUTE).is_some();
    let mut rownode = rownode;
    let unnumbered_eqncr = rownode
      .get_attribute(UNNUMBERED_EQNCR_ROW_ATTRIBUTE)
      .is_some();
    let _ = rownode.remove_attribute(UNNUMBERED_EQNCR_ROW_ATTRIBUTE);
    let _ = rownode.remove_attribute(NEXT_NUMBER_ROW_ATTRIBUTE);
    let keep_next_number = rownode
      .get_attribute(KEEP_NEXT_NUMBER_ROW_ATTRIBUTE)
      .is_some();
    let _ = rownode.remove_attribute(KEEP_NEXT_NUMBER_ROW_ATTRIBUTE);
    rows.push(EqRow {
      node: rownode,
      cols: cells,
      has_l,
      has_m,
      has_r,
      numbered,
      labelled,
      tagged,
      unnumbered_eqncr,
      next_number,
      keep_next_number,
    });
  }
  // Every `\label` names the number the next printing `\@@eqncr` prints ([`eqnarray_eqncr_numbering`]): an unnumbered
  // row's labels move to the next numbered row; when none follows (an eqnarray* row, a trailing `\nonumber` row) they
  // name the counter's next value, which no row of the group prints, and stay on their row with that value's
  // reference forms (`NEXT_NUMBER_TAGS_PROPERTY`; 1303.3431's appa3, 1602.03553, 2301.00760). So do they when the next
  // numbered row's number is no longer the one they named (`KEEP_NEXT_NUMBER_ROW_ATTRIBUTE`, IEEEtrantools'
  // `\IEEEyessubnumber`).
  let mut pending: Vec<usize> = Vec::new();
  let mut kept: Vec<usize> = Vec::new();
  for index in 0..rows.len() {
    if rows[index].numbered && !pending.is_empty() {
      let mut carried: Vec<String> = Vec::new();
      for from in pending.drain(..) {
        let row = &mut rows[from];
        if let Some(labels) = row.node.get_attribute("labels") {
          carried.push(labels);
        }
        let _ = row.node.remove_attribute("labels");
        row.labelled = false;
      }
      let row = &mut rows[index];
      if let Some(own) = row.node.get_attribute("labels") {
        carried.push(own);
      }
      document.set_attribute(&mut row.node, "labels", &carried.join(" "))?;
      row.labelled = true;
    } else if rows[index].unnumbered_eqncr && rows[index].labelled && !rows[index].numbered {
      if rows[index].keep_next_number {
        kept.push(index);
      } else {
        pending.push(index);
      }
    }
  }
  // The columns any row uses.
  let used_columns: Vec<usize> = (0..ncols)
    .filter(|&c| {
      rows.iter().any(|row| {
        row
          .cols
          .get(c)
          .is_some_and(|cell| cell.get_first_child().is_some())
      })
    })
    .collect();

  // A single column used, which some row begins with a RELOP: every row joins one equation (below).
  let begins_with_relop = match used_columns[..] {
    [keepcol] => rows.iter().any(|row| {
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
    }),
    _ => false,
  };
  // A labelled row with no numbered row after it keeps its next number's reference forms and counts as numbered and
  // labelled — its own equation, as a separately numbered and labelled row is below, so that no number of the group
  // takes its label — unless every row joins one equation that prints a number of its own (a single column of
  // relations, below), whose number then stands. A row whose labels moved on drops them.
  let printed_number = rows.iter().any(|row| row.numbered);
  for (index, row) in rows.iter_mut().enumerate() {
    if !row.next_number {
      continue;
    }
    if (pending.contains(&index) || kept.contains(&index))
      && row.labelled
      && !(printed_number && begins_with_relop)
    {
      row.numbered = true;
    } else {
      for tags in document.findnodes("ltx:tags", Some(&row.node)) {
        document.safe_unlink(tags);
      }
    }
  }

  // Only a single column was used
  if let [keepcol] = used_columns[..] {
    // Remove empty columns (in reverse order to preserve indices)
    for c in (0..ncols).rev() {
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
    if begins_with_relop {
      let nodes: Vec<Node> = rows.into_iter().map(|r| r.node).collect();
      equationgroup_join_rows(document, equationgroup, nodes)?;
    } else {
      // Each row its own equation, except an empty numbered row under an unnumbered one, which joins it with its
      // number and labels, as below (`\label{e} … \nonumber\\ \end{eqnarray}`: the end's `\@@eqncr` prints it, and
      // the label names it; 1206.6051).
      let mut groups: Vec<(Vec<Node>, bool)> = Vec::new();
      for row in rows {
        let empty = !(row.has_l || row.has_m || row.has_r || row.tagged);
        match groups.last_mut() {
          Some((group, numbered)) if empty && row.numbered && !*numbered => {
            group.push(row.node);
            *numbered = true;
          },
          _ => groups.push((vec![row.node], row.numbered)),
        }
      }
      for (mut group, _) in groups {
        if group.len() > 1 {
          equationgroup_join_rows(document, equationgroup, group)?;
        } else if let Some(node) = group.first_mut() {
          equationgroup_join_cols(document, 1, node)?;
        }
      }
    }
    return Ok(());
  }

  // Several columns used — analyze continuation patterns
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
      // A continuation (`&&+ 2,`) whose row prints a number of its own under a numbered equation stands as its own
      // equation, as a separately numbered relation row does above: latex.ltx's `\@@eqncr` prints each row's number
      // (:15795), and a join keeps only the last, so the first equation's number was lost and a `\label` on it named
      // the continuation's (1310.2253 `eqn:zg`, PDF 11, ours 12). Perl always joins these rows: its rule keeps one
      // separate when it is labelled too (latex_constructs.pool.ltxml:2360-2362), but tests `hasAttribute('label')`,
      // which is never set (OXIDIZED_DESIGN #54, KNOWN_PERL_ERRORS #46); OXIDIZED_DESIGN_DIVERGENCES #888. Repro
      // cross-references/eqnarray_numbered_continuation_row_keeps_its_number.
      if eqs.is_empty() || (numbered && row.numbered) {
        class = "odd";
      } else {
        class = "continue";
      }
    } else if (row.labelled || row.tagged) && row.numbered {
      // All columns empty, but the row holds a `\label`: when `\@@eqncr` prints its number (latex.ltx:15795, only while
      // `\if@eqnsw` holds — the rule is [`eqnarray_eqncr_numbering`]) the label binds to that number. Its number and label
      // join the equation above, or stand as their own when that one has a number already (Perl
      // latex_constructs.pool.ltxml:2366-2369 notes the rows lost; 1011.4399, hep-th9412215, 1305.3072; repro
      // alignment/eqnarray_empty_labelled_row_keeps_number). The labels of an unnumbered row move to the next numbered row
      // (`rearrange_eqnarray`); a label with no numbered row after it stays, its row numbered by the next number's
      // reference forms (`NEXT_NUMBER_TAGS_PROPERTY`), so an empty one is an equation of its own here. So does a row whose only
      // content is its author's tag: Scientific Word's `… \nonumber \\ && \TCItag{27}` numbers the equation above it
      // (pdflatex prints (27) on the tag's row; nlin0106009's 11 such rows; OXIDIZED_DESIGN_DIVERGENCES #591,
      // KNOWN_PERL_ERRORS #682) — in a multi-column eqnarray; a single-column one returned above, where the tagged row
      // stands as its own equation. An auto-numbered empty row (`\\ \\`) is removed, as in Perl.
      class = if eqs.is_empty() || numbered {
        "new"
      } else {
        "continue"
      };
    } else if row.numbered && !eqs.is_empty() && !numbered {
      // All columns empty, numbered, under an unnumbered equation: the number `\@@eqncr` prints on a row of its own,
      // after rows ended by `\cr` (`x &=& y \cr \end{eqnarray}`; `… \cr &&\\`, 0708.0012, nucl-th9807024) or
      // `\nonumber`. It is that equation's, so it joins it with its tags and id (latex.ltx:15789-15797; repro
      // cross-references/eqnarray_cr_then_empty_numbered_row). Perl removes the row and the number with it.
      class = "continue";
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

/// eqnarray numbering as latex.ltx's `\@@eqncr` state (latex.ltx:15750-15797). A row is numbered only if it ends
/// through `\@@eqncr` (`\\`, `\\*`, `\\[len]`, `\end{eqnarray}`; here `\lx@eqnarray@number@check`) and no `\nonumber`
/// came since the previous `\@@eqncr`: `\nonumber` is `\global\@eqnswfalse`, `\@@eqncr` prints `\@eqnnum` and steps the
/// counter only `\if@eqnsw`, then sets it true, and a row ended by a raw `\cr` neither prints nor resets it. So a
/// `\nonumber` in a `\cr` row unnumbers the next row too (2112.13115), and a `\cr` row is unnumbered (1011.0523,
/// nucl-th9903062). `\@currentlabel` is `\p@equation\theequation`, the counter `\eqnarray`'s `\stepcounter` and each
/// printing `\@@eqncr` step, so every `\label` names the number the next printing `\@@eqncr` prints: the labels of an
/// unnumbered row move to the next numbered row in `rearrange_eqnarray` (`UNNUMBERED_EQNCR_ROW_ATTRIBUTE`; a `\label`
/// in a `\nonumber` row, 1206.6051's NA3; in a `\cr` row, 0708.0012), and stay where they are when none follows, with
/// the counter's next value's reference forms ([`eqnarray_next_number_tags`]; 1303.3431's `appa3`, 1602.03553). A
/// number `\@@eqncr` prints on an empty row of its own under an unnumbered one (`x &=& y \cr \end{eqnarray}`,
/// `\cr &&\\`, `\nonumber\\ \end{eqnarray}`) is that equation's (`NUMBER_ROW_PROPERTY`; 0708.0012, nucl-th9807024,
/// 1206.6051). For the environments that opt in with `eqncr=1`: latex.ltx's eqnarray and eqnarray*. Rust-only (Perl
/// numbers each row unless it holds `\nonumber`: OXIDIZED_DESIGN_DIVERGENCES #847). Witnesses also 0806.3762,
/// 0910.0753. Guards `perfect_kernel_batch78n::{eqnarray_row_ended_by_cr_is_unnumbered,
/// eqnarray_empty_numbered_row_numbers_the_equation_above, eqnarray_cr_row_label_names_next_number,
/// eqnarray_nonumber_row_label_names_next_number, eqnarray_nonumber_holds_until_eqncr}`,
/// `perfect_kernel_batch78r::eqnarray_label_with_no_numbered_row_after_it_names_the_next_number`.
pub(crate) fn eqnarray_eqncr_numbering() -> bool {
  with_value(
    "EQUATION_NUMBERING",
    |v| matches!(v, Some(Stored::HashStored(n)) if matches!(n.get("eqncr"), Some(&Stored::Bool(true)))),
  )
}

/// latex.ltx's `\if@eqnsw` ([`eqnarray_eqncr_numbering`]), global as there.
pub(crate) const EQNARRAY_EQNSW: &str = "EQNARRAY_EQNSW";

/// The value `\if@eqnsw` takes where an eqnarray begins and after each `\@@eqncr`: true in latex.ltx (:15754, :15795
/// `\global\@eqnswtrue`); equations.sty's `\if@defeqnsw`, false in its eqnarray* (contrib equations_sty.rs;
/// hep-ph9904268). Local; absent is true.
const EQNARRAY_DEFAULT_EQNSW: &str = "EQNARRAY_DEFAULT_EQNSW";

/// The `\if@eqnsw` an eqnarray row starts with ([`EQNARRAY_DEFAULT_EQNSW`]).
pub(crate) fn eqnarray_default_eqnsw() -> bool {
  !matches!(
    lookup_value(EQNARRAY_DEFAULT_EQNSW),
    Some(Stored::Bool(false))
  )
}

/// Sets, locally, the `\if@eqnsw` the eqnarray rows start with: equations.sty's `\if@defeqnsw` (its eqnarray*'s
/// `\@defeqnswfalse`, equations.sty:128-160), read where an eqnarray begins. Witness hep-ph9904268.
pub fn set_eqnarray_default_eqnsw(numbered: bool) {
  assign_value(
    EQNARRAY_DEFAULT_EQNSW,
    Stored::Bool(numbered),
    Some(Scope::Local),
  );
}

/// `\global\@eqnswtrue` from the document (equations.sty's `\yesnumber`): the eqnarray row being read is numbered by
/// the `\@@eqncr` that ends it (`eqnarray_eqncr_numbering`). Witness hep-ph9904268.
pub fn set_eqnarray_eqnsw(numbered: bool) {
  assign_value(EQNARRAY_EQNSW, Stored::Bool(numbered), Some(Scope::Global));
}

/// Whether the `\@@eqncr` that ended the current eqnarray row printed its number ([`eqnarray_eqncr_numbering`]): set
/// by `\lx@eqnarray@number@check`, taken by `\eqnarray@row@after@`; false or absent for a row ended by a raw `\cr`.
pub(crate) const EQNARRAY_EQNNUM: &str = "EQNARRAY_EQNNUM";

/// The row attribute that marks an eqnarray row its `\@@eqncr` state left unnumbered ([`eqnarray_eqncr_numbering`]):
/// its labels name the next printed number. Set by `\eqnarray@row@after@` (sect07.rs), read and removed by
/// `rearrange_eqnarray`.
pub(crate) const UNNUMBERED_EQNCR_ROW_ATTRIBUTE: &str = "_unnumbered_eqncr";

/// Whether a `\label` was digested in the current eqnarray row (`\lx@eqnarray@label` sets it, `\eqnarray@row@after@`
/// takes it).
pub(crate) const EQNARRAY_ROW_LABELLED: &str = "EQNARRAY_ROW_LABELLED";

/// The row property of a labelled eqnarray row its `\@@eqncr` state left unnumbered: the reference forms (refnum,
/// typerefnum, …; no printed tag) of the number `\@currentlabel` names in it, the counter's next value
/// ([`eqnarray_next_number_tags`]). `rearrange_eqnarray` keeps them when no numbered row follows, so the label names that
/// number.
pub(crate) const NEXT_NUMBER_TAGS_PROPERTY: &str = "next_number_tags";

/// The row tag (`EQUATIONROW_TAGS`) holding the sub-equation number the row's pre-advance of `equation` reset,
/// IEEEtrantools' `\@IEEEsubeqnnumrollback` (IEEEtrantools.sty:1985, :2094, :2109): `\IEEEyessubnumber` backs out to
/// it (`\@IEEEyessubnumber`, :1410-1424), and a retraction of the row restores it. Set by `\eqnarray@row@before@`
/// (sect07.rs) under the numbering key `subnumbers=1`.
pub const SUBEQUATION_ROLLBACK: &str = "subequation_rollback";

/// The row tag (`EQUATIONROW_TAGS`) marking an IEEEeqnarray row whose counters are its number, no pre-advance:
/// `\IEEEyesnumber` and `\IEEEyessubnumber` set them (IEEEtrantools' `\global\@IEEEeqnumpreadvfalse`, :1402, :1422),
/// and a row given its number by [`EQUATIONROW_CARRY`]. Its retraction (a `\nonumber` or `\IEEEnonumber` after them,
/// `\global\@eqnswfalse`, the last one wins) leaves the counters where they are and carries the number to the next row:
/// `\IEEEyessubnumber\nonumber \\ c &=& d`, pdflatex "(1c)" on the second row; `\IEEEyessubnumber\IEEEnonumber`
/// after "(4)", pdflatex "(4a)" on the row after, where this port printed "(4)" again. Witness 2012.02697
/// (`lcmpc.tex:123-136`, `eq:fpar`/`eq:par` 14a/14b as the `.aux`).
pub const EQUATIONROW_HOLDS: &str = "holds_its_counters";

/// The global value naming the counter (`equation`, or `subequation` when the sub-equation number is set) whose
/// current value the next IEEEeqnarray row prints without stepping it: the number of a retracted
/// [`EQUATIONROW_HOLDS`] row. Read and removed by `\eqnarray@row@before@` (sect07.rs), cleared where an equation
/// group's numbering begins (`\@equationgroup@numbering`), as `\@IEEEeqnarray` pre-advances afresh (:1985-1987).
pub const EQUATIONROW_CARRY: &str = "EQUATIONROW_CARRY";

/// The row attribute marking a labelled row left unnumbered whose label keeps the number it named
/// ([`NEXT_NUMBER_TAGS_PROPERTY`]) though a numbered row follows: that row's number is no longer the one the label named
/// (an `\IEEEyessubnumber` backed it out, [`keep_pending_next_numbers`]). Read and removed by `rearrange_eqnarray`.
pub(crate) const KEEP_NEXT_NUMBER_ROW_ATTRIBUTE: &str = "_keep_next_number";

/// Marks the labelled rows left unnumbered since the last numbered row of the current alignment as keeping the number
/// their labels named (`KEEP_NEXT_NUMBER_ROW_ATTRIBUTE`), for a row whose number changes from the one they named:
/// IEEEtrantools' `\label` writes `\@currentlabel` as it is then, the pre-advanced number (`p6`: `c &=& d
/// \label{z1}\IEEEnonumber \\ e &=& f \IEEEyessubnumber`, pdflatex "2" and "1a"; 2012.02697).
pub fn keep_pending_next_numbers() {
  if let Some(alignment_digested) = lookup_alignment()
    && let Some(alignment_cell) = alignment_digested.alignment_cell()
  {
    let mut alignment = alignment_cell.borrow_mut();
    let earlier = alignment.rows().len().saturating_sub(1);
    for row in alignment.rows_mut().iter_mut().take(earlier).rev() {
      if row.is_pseudo() {
        continue;
      }
      if row.properties.contains_key("tags") {
        break;
      }
      if row.properties.contains_key(NEXT_NUMBER_TAGS_PROPERTY) {
        row.properties.insert(
          KEEP_NEXT_NUMBER_ROW_ATTRIBUTE.to_string(),
          Stored::from("true"),
        );
      }
    }
  }
}

/// The row attribute marking the tags of a row as its [`NEXT_NUMBER_TAGS_PROPERTY`] (not a number of its own); read and
/// removed by `rearrange_eqnarray`.
pub(crate) const NEXT_NUMBER_ROW_ATTRIBUTE: &str = "_next_number";

/// The reference forms of the number a `\label` in an eqnarray row names (latex.ltx:15750-15752 `\stepcounter{equation}`
/// and `\def\@currentlabel{\p@equation\theequation}`, stepped again by each printing `\@@eqncr`, :15795): the counter's
/// value while the row is digested, which its preset number holds until the row's end retracts it. Every tag role a
/// reference prints (`tag_forms`), none printed on the row. Witnesses 1602.03553 (`eqn:zeta01`, PDF 5), 2301.00760
/// (54), 1303.3431 (`appa3`).
pub(crate) fn eqnarray_next_number_tags() -> Result<Digested> {
  let counter = with_value("EQUATION_NUMBERING", |v| match v {
    Some(Stored::HashStored(n)) => n.get("counter").map(ToString::to_string),
    _ => None,
  })
  .unwrap_or_else(|| String::from("equation"));
  let mut tokens = vec![T_CS!("\\lx@tags"), T_BEGIN!()];
  for (printed_here, form) in tag_forms(&Tokens::new(Explode!(counter)))? {
    if !printed_here {
      tokens.extend(form.unlist());
    }
  }
  tokens.push(T_END!());
  digest(Tokens::new(tokens))
}

/// The trailing run of empty cells past an eqnarray row's `equation_columns` (latex.ltx's number column, :15764, and
/// any an extra `&` opened) is dropped, and a span reaching into it (a `\lefteqn`'s, over the empty cells after it)
/// ends at the equations' last column: the eqnarray's rows keep Perl's three columns unless a row writes past them.
/// An empty cell before a later one with content stays, so that content keeps its column (`k &=& l & & EXTRA`): each
/// `&` opens the next preamble column whatever the cell before holds (tex.web §791 `fin_col`). A trailing empty cell
/// is an empty unset box, which `fin_align` sets as it does a row that ended before it (§§800-810: a short row is set
/// over the boxes it has), so dropping the run leaves what TeX prints. eqnarray only (`_number_column`): an
/// `\IEEEeqnarray`'s cells are its specification's, kept as written.
fn drop_empty_number_cells(document: &mut Document, cells: Vec<Node>, ncols: usize) -> Vec<Node> {
  let spans: Vec<usize> = cells
    .iter()
    .map(|cell| {
      cell
        .get_attribute("colspan")
        .and_then(|n| n.parse::<usize>().ok())
        .unwrap_or(1)
    })
    .collect();
  let starts: Vec<usize> = spans
    .iter()
    .scan(0, |start, span| {
      let this = *start;
      *start += span;
      Some(this)
    })
    .collect();
  // The kept cells: up to the last that holds content or starts within the equation columns.
  let keep = (0..cells.len())
    .rev()
    .find(|&k| starts[k] < ncols || cells[k].get_first_child().is_some())
    .map_or(0, |k| k + 1);
  let mut kept = Vec::with_capacity(keep);
  for (k, mut cell) in cells.into_iter().enumerate() {
    if k >= keep {
      document.safe_unlink(cell);
      continue;
    }
    // The last kept cell's span ends where the dropped run begins: at the equations' last column.
    if k + 1 == keep && starts[k] < ncols && starts[k] + spans[k] > ncols {
      let width = ncols - starts[k];
      if width > 1 {
        let _ = cell.set_attribute("colspan", &width.to_string());
      } else {
        let _ = cell.remove_attribute("colspan");
      }
    }
    kept.push(cell);
  }
  kept
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

/// The thm-patch hooks `\thmt@<set>@<point>` among `hooks` that are defined, in order, as source text to digest;
/// empty when none is (they exist once thmtools is loaded, thmtools_sty.rs).
fn theorem_hooks(hooks: &[(&str, &str)]) -> String {
  hooks
    .iter()
    .map(|(set, point)| s!("\\thmt@{set}@{point}"))
    .filter(|cs| has_meaning(&T_CS!(cs.as_str())))
    .collect()
}

/// The mapping from a counter to the macro (of one argument, the counter's name) a theorem numbered within it prints
/// in place of `\the<counter>`.
const THEOREM_WITHIN_DISPLAY: &str = "theorem_within_display";

/// A class's `\@xnthm` printing a theorem numbered within `within` with `display{within}` in place of
/// `\the<within>`: IEEEtran.cls:5607-5627 `\@IEEEthmcounterinsection{section}`, so "Theorem 2.1" where `\thesection` is
/// "II" (1911.01952; ieeetran_cls.rs; OXIDIZED_DESIGN_DIVERGENCES #848). `None` removes it.
pub fn set_theorem_within_display(within: &str, display: Option<&str>) {
  assign_mapping(THEOREM_WITHIN_DISPLAY, within, display.map(String::from));
}

/// A package whose own `\@xnthm` replaces the class's (amsthm, ntheorem, theorem.sty) drops every
/// [`set_theorem_within_display`]: its theorems print `\the<within>`.
pub fn clear_theorem_within_display() {
  for within in with_mapping_keys(THEOREM_WITHIN_DISPLAY, |keys| keys) {
    assign_mapping::<String>(THEOREM_WITHIN_DISPLAY, &to_string(within), None);
  }
}

/// Set by a package whose unnumbered theorems still step their counter: ntheorem.sty:824-839 `\@othm` and :870-880
/// `\@thm` run `\refstepcounter{#2}` for every theorem, its `\newtheorem*` (a `nonumber` style, :720-727) only printing
/// no number, so `\newtheorem*{rmk}[thm]{Remark}` takes a number from `thm` (1606.01585: five references one behind).
/// Read where a theorem is declared ([`define_new_theorem`]). Global.
pub const THEOREM_UNNUMBERED_STEPS: &str = "lx@theorem@unnumbered@steps";

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
  // The shared counter is a name read through `\csname`, so a macro naming it expands to that name: latex.ltx
  // :17190-17195 `\@othm` tests `\@ifundefined{c@#2}` and defines `\the#1` as `\@nameuse{the#2}`. smfthm.sty:52-56
  // `\gdef\smf@thm{smfthm}` `\newtheorem{theo}[\smf@thm]{\theoname}` (witnesses 1711.11547, 1902.02049, 2108.12000)
  // and ccthm.sty's `[\cc@thmcnter]` (1507.02955) named a counter `\smf@thm`: "Counter not defined" and an undefined
  // `\the\smf@thm` for every theorem. Perl takes the literal string (latex_constructs.pool.ltxml:2892
  // `ToString($otherthmset)`), KNOWN_PERL_ERRORS #691. The trailing `[within]` is a counter name the same way
  // (latex.ltx `\@xnthm`: `\@addtoreset{#1}{#3}`, `\csname the#3\endcsname`), so it is read through `\csname` too:
  // digested (Perl `DigestLiteral`), `\newtheorem{x}{X}[todo_counter]` raised "Script _ can only appear in math mode"
  // where pdflatex is silent (1307.7314). Guards `perfect_kernel_batch67s::newtheorem_*`.
  let otherthmset_str = otherthmset
    .map(|t| ::latexml_core::gullet::expand_as_csname_text(t).map(|e| e.to_string()))
    .transpose()?
    .filter(|s| !s.is_empty());
  let has_type = typ.as_ref().is_some_and(|t| !t.is_empty());
  let is_starred = flag.is_some();

  let within_str = within
    .map(|t| ::latexml_core::gullet::expand_as_csname_text(t).map(|e| e.to_string()))
    .transpose()?
    .filter(|s| !s.is_empty());

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
    // A theorem declared with a counter of its own steps that counter, even where an earlier `\newtheorem{X}[Y]`
    // made X share Y's (latex.ltx `\@xnthm`: `\newcounter{X}`, `\theX` its own): the shared mapping goes, or its
    // numbers step Y's counter and print X's, "Proposition 1.0" (2407.21129, a paper's `\newtheorem{proposition}
    // {Proposition}[section]` over tac's `[theorem]` one) and "Remark 0" (1205.2947, 1307.5990: arxbj's
    // `\newtheorem{rem}{Remark}` over elsart_support's `[thm]` one). Guards
    // `perfect_kernel_batch77v::{newtheorem_over_shared_counter_theorem, elsart_support_theorem_tag_zero}`.
    assign_mapping::<Stored>("counter_for_type", &thmset_str, None);
    let idprefix = s!("Thm{}", classname.replace('*', "."));
    let c_counter = s!("\\c@{counter}");
    if !is_defined(&c_counter) {
      let within_ref = within_str.as_deref().unwrap_or("");
      NewCounter!(&counter, within_ref, idprefix => &idprefix);
    }
    // Define \the<counter>
    if !numbering.is_empty() {
      let the_counter_body = if let Some(ref w) = within_str {
        // A class's own `\@xnthm` may print the within-counter its own way ([`set_theorem_within_display`]).
        match lookup_mapping(THEOREM_WITHIN_DISPLAY, w) {
          Some(display) => s!("{display}{{{w}}}\\@thmcountersep{numbering}{{{counter}}}"),
          None => s!("\\csname the{w}\\endcsname\\@thmcountersep{numbering}{{{counter}}}"),
        }
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
  let thmset_for_head_hooks = thmset_str.clone();
  let thmset_for_restate = thmset_for_env.clone();
  let thmset_for_foot_hooks = thmset_str.clone();
  // `thmset_for_tags` and `counter_for_tags` are the last uses of
  // thmset_str and counter — move instead of clone.
  let thmset_for_tags = thmset_str;
  let counter_for_tags = counter;
  let is_starred_for_props = is_starred;
  let unnumbered_steps = is_starred && lookup_bool(THEOREM_UNNUMBERED_STEPS);
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
    let mut digest_str =
      s!("\\the\\thm@bodyfont\\the\\thm@styling\\def\\lx@thistheorem{{{name_str}}}");
    // thm-patch.sty:33-62 runs a theorem's `\thmt@<env>@preheadhook`, `\thmt@generic@preheadhook` before its head
    // and `\thmt@<env>@postheadhook`, `\thmt@generic@postheadhook` after it, inside the theorem: the hooks thmtools'
    // `\addtotheorem…hook[<env>]` gathers (thmtools_sty.rs, the only binding that defines them).
    digest_str.push_str(&theorem_hooks(&[
      (&thmset_for_head_hooks, "preheadhook"),
      ("generic", "preheadhook"),
      (&thmset_for_head_hooks, "postheadhook"),
      ("generic", "postheadhook"),
    ]));
    let mut digested = vec![digest(mouth::tokenize_internal(TeXString::assembled(
      digest_str,
    )))?];
    // …and thm-kv's local post-head hook, last (thm-patch.sty:60 `\thmt@local@postheadhook`): the `label=` key's
    // `\label` (thm-kv.sty:485-487, `thmtools_use_keys`)
    if let Some(Stored::Tokens(label)) = lookup_value("thmt_local_label") {
      assign_value("thmt_local_label", Stored::None, None);
      digested.push(digest(Invocation!(T_CS!("\\label"), vec![label]))?);
    }
    // …and thm-restate's `restate=<name>` (thm-restate.sty:198-215, the same hook): the theorem is stored for
    // `\<name>` as `{restatable}` stores it (thm_restate_sty.rs `\lx@thm@restate@store`, with the theorem's other
    // keys for the restated copy). The store reads the body that follows, so it is read from the input, not digested
    // here.
    if let Some(Stored::Tokens(name)) = lookup_value("thmt_local_restate") {
      assign_value("thmt_local_restate", Stored::None, None);
      let keys = match lookup_value("thmt_local_restate_keys") {
        Some(Stored::Tokens(keys)) => keys,
        _ => Tokens!(),
      };
      let env = Tokens::new(Explode!(&thmset_for_restate));
      unread(Invocation!(T_CS!("\\lx@thm@restate@store"), vec![
        env.clone(),
        env,
        name,
        keys
      ]));
    }
    Ok(digested)
  });
  options.after_digest_begin.push(after_digest_begin_closure);

  // before_digest_end. A theorem ended by a plain group closer (`{\Proof …}`)
  // never reaches its `\endX` in TeX, so neither the end mark (ntheorem's
  // `[thmmarks]` symbol) nor the postwork is set (stomach.rs
  // `close_environment_frame`).
  let before_digest_end_closure: BeforeDigestClosure = Rc::new(move || {
    if environment_ended_by_group() {
      return Ok(Vec::new());
    }
    // thm-patch.sty:130-142: the foot hooks, here before the theorem's end code (ditto).
    let foot_hooks = theorem_hooks(&[
      ("generic", "prefoothook"),
      (&thmset_for_foot_hooks, "prefoothook"),
      ("generic", "postfoothook"),
      (&thmset_for_foot_hooks, "postfoothook"),
    ]);
    let digested = digest(mouth::tokenize_internal(TeXString::assembled(s!(
      "{foot_hooks}\\thm@doendmark\\the\\thm@postwork"
    ))))?;
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
      // thm-kv's key=value optional argument (`thmtools_use_keys`), when thmtools is loaded
      let thmuse = match args.first() {
        Some(Some(arg)) if lookup_bool("thmtools_thmuse_keys") => thmtools_use_keys(&arg.revert()?),
        _ => None,
      };
      if let Some(label) = thmuse.as_ref().and_then(|u| u.label.clone()) {
        assign_value("thmt_local_label", Stored::Tokens(label), None);
      }
      if let Some(name) = thmuse.as_ref().and_then(|u| u.restate.clone())
        && has_meaning(&T_CS!("\\lx@thm@restate@store"))
      {
        assign_value("thmt_local_restate", Stored::Tokens(name), None);
        let keys = thmuse
          .as_ref()
          .map(|u| u.restate_keys.clone())
          .unwrap_or_default();
        assign_value(
          "thmt_local_restate_keys",
          Stored::Tokens(Tokens::new(keys)),
          None,
        );
      }
      // A starred restatement (thm_restate_sty.rs `\lx@thm@restate*`) is numbered as the theorem it restates
      // (thm-restate.sty:130-140: `\the<env>` faked from the stored number, the counter not stepped), as
      // `continues=` below numbers a theorem as the one it continues
      let restated = match lookup_value("thmt_restate_number") {
        Some(Stored::Tokens(number)) => {
          assign_value("thmt_restate_number", Stored::None, None);
          Some(number)
        },
        _ => None,
      };
      // `continues=<label>` (thm-kv.sty:511-516 `\thmt@suspendcounter`): the theorem is numbered as the one it
      // continues — its number, in its tags and its title, is a reference to that label — and its counter does not
      // step
      let continued = thmuse
        .as_ref()
        .and_then(|u| u.continues.as_ref())
        .map(|continued| {
          let mut number = vec![T_CS!("\\ref"), T_BEGIN!()];
          number.extend(continued.unlist_ref().iter().copied());
          number.push(T_END!());
          number
        });
      let suspended: Vec<Token> = match restated.map(|number| number.unlist()).or(continued) {
        Some(number) if !is_starred_for_props && !counter_for_tags.is_empty() => {
          let mut toks = Vec::new();
          for name in [&counter_for_tags, &thmset_for_tags] {
            toks.extend([T_CS!("\\def"), T_CS!(s!("\\the{}", name)), T_BEGIN!()]);
            toks.extend(number.iter().copied());
            toks.push(T_END!());
          }
          toks
        },
        _ => Vec::new(),
      };

      if !counter_for_tags.is_empty() {
        if !suspended.is_empty() {
          let ctr_props = ref_step_id(&counter_for_tags)?;
          for (k, v) in ctr_props.iter() {
            props.insert_sym(*k, v.clone());
          }
          let mut toks = vec![T_BEGIN!()];
          toks.extend(suspended.iter().copied());
          toks.extend([T_CS!("\\lx@make@tags"), T_BEGIN!()]);
          toks.extend(
            mouth::tokenize_internal(TeXString::assembled(thmset_for_tags.clone())).unlist(),
          );
          toks.extend([T_END!(), T_END!()]);
          props.insert("tags", digest(Tokens::new(toks))?.into());
        } else if is_starred_for_props {
          if unnumbered_steps {
            step_counter(&counter_for_tags, false)?;
          }
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
          let ctr_props = ref_step_counter_by_meaning(&thmset_for_tags)?;
          for (k, v) in ctr_props.iter() {
            props.insert_sym(*k, v.clone());
          }
        }
      }

      // Compute title
      let format_title_cs = s!("\\format@title@{}", thmset_for_tags);
      let mut title_tokens = vec![T_BEGIN!()];
      title_tokens.extend(suspended);
      title_tokens.extend([
        T_CS!("\\the"),
        T_CS!("\\thm@headstyling"),
        T_CS!(&format_title_cs),
        T_BEGIN!(),
      ]);
      if let Some(thmuse) = thmuse {
        title_tokens.extend(thmuse.note);
      } else if let Some(Some(arg)) = args.first() {
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

/// What thm-kv makes of a theorem's optional argument written as keys (thm-kv.sty:434-437 adds `\thmt@garbleoptarg` to
/// every theorem's pre-head hook, :439-463): its note, the label it sets, the theorem it continues.
struct ThmUse {
  /// The note in place of the argument (`\thmt@newoptarg`, expanded): empty when only `label=` is given.
  note:         Vec<Token>,
  /// `label=<name>` (:485-487): a `\label` after the head (`\thmt@local@postheadhook`).
  label:        Option<Tokens>,
  /// `continues=<label>` (:511-516): the theorem continued, whose number this one takes.
  continues:    Option<Tokens>,
  /// thm-restate's `restate=[<keys>]<name>` (thm-restate.sty:198-215): the name the theorem is stored under for a
  /// later `\<name>`; the bracketed keys apply to the restated copy only.
  restate:      Option<Tokens>,
  /// The keys the restated copy is begun with (thm-restate.sty:217-229 `restate phase 2`): the theorem's other keys,
  /// then the bracketed ones.
  restate_keys: Vec<Token>,
}

/// thm-kv's `\thmt@garbleoptarg` (thm-kv.sty:439-463) over the keys it defines: `label` (:485-487), `name` (:488-496,
/// `[short]long`, the long form the note) and `continues` (:511-516, "<name>, \thmcontinues{<label>}" or, alone,
/// "\thmcontinues{<label>}" — the `\@gobble` the note starts with eats its `{, }`), processed in order as `\kvsetkeys`
/// does. An argument none of whose keys is one of these is no key list (`\ifthmt@thmuse@iskv`), kept as written: a
/// note "Smith, 2001" is a note. Written as keys, a `_` in a label was a math error and "(label=ex:a_b)" its title (run
/// 336 first error `unexpected:_`, ~150 papers: 2002.09876, 2409.14881). thm-restate adds `restate=` (thm-restate.sty
/// :193-215, its "restate phase" families): the theorem stored for a later `\<name>`, its note none (2108.11527,
/// 2602.09462: "(restate=…)" was the title and `\<name>` undefined).
fn thmtools_use_keys(arg: &Tokens) -> Option<ThmUse> {
  let tokens = arg.unlist_ref();
  // the top-level `key=value` items, spaces trimmed, one brace level off a value (keyval's)
  let mut items: Vec<(String, Vec<Token>)> = Vec::new();
  let mut depth = 0i32;
  let mut start = 0;
  let mut push_item = |item: &[Token]| {
    let mut level = 0i32;
    let equals = item.iter().position(|t| {
      match t.get_catcode() {
        Catcode::BEGIN => level += 1,
        Catcode::END => level -= 1,
        _ => {},
      }
      level == 0 && t.get_catcode() == Catcode::OTHER && t.to_string() == "="
    });
    if let Some(equals) = equals {
      let key: String = item[..equals].iter().map(|t| t.to_string()).collect();
      let value = trim_spaces(&item[equals + 1..]);
      let value = match (value.first(), value.last()) {
        (Some(first), Some(last))
          if value.len() >= 2
            && first.get_catcode() == Catcode::BEGIN
            && last.get_catcode() == Catcode::END
            && balanced_group(&value) =>
        {
          value[1..value.len() - 1].to_vec()
        },
        _ => value,
      };
      items.push((key.trim().to_string(), value));
    }
  };
  for (i, token) in tokens.iter().enumerate() {
    match token.get_catcode() {
      Catcode::BEGIN => depth += 1,
      Catcode::END => depth -= 1,
      Catcode::OTHER if depth == 0 && token.to_string() == "," => {
        push_item(&tokens[start..i]);
        start = i + 1;
      },
      _ => {},
    }
  }
  push_item(&tokens[start..]);
  let mut iskv = false;
  // `\thmt@newoptarg`, `\@gobble` until a `name=` replaces it
  let mut gobbled = true;
  let mut note: Vec<Token> = Vec::new();
  let mut label = None;
  let mut continues = None;
  let mut restate = None;
  let mut restate_keys: Vec<Token> = Vec::new();
  let mut restate_extra: Vec<Token> = Vec::new();
  for (key, value) in items {
    if key != "restate" {
      if !restate_keys.is_empty() {
        restate_keys.push(T_OTHER!(","));
      }
      restate_keys.extend(Explode!(&key));
      restate_keys.extend([T_OTHER!("="), T_BEGIN!()]);
      restate_keys.extend(value.iter().copied());
      restate_keys.push(T_END!());
    }
    match key.as_str() {
      "label" => {
        iskv = true;
        label = Some(Tokens::new(value));
      },
      "name" => {
        iskv = true;
        // `name=[short]long`: the long form
        let long = match value.first() {
          Some(t) if t.get_catcode() == Catcode::OTHER && t.to_string() == "[" => {
            match value
              .iter()
              .position(|t| t.get_catcode() == Catcode::OTHER && t.to_string() == "]")
            {
              Some(close) => value[close + 1..].to_vec(),
              None => value,
            }
          },
          _ => value,
        };
        gobbled = false;
        note = long;
      },
      "continues" => {
        iskv = true;
        if !gobbled {
          note.extend([T_OTHER!(","), T_SPACE!()]);
        }
        gobbled = false;
        note.push(T_CS!("\\thmcontinues"));
        note.push(T_BEGIN!());
        note.extend(value.iter().copied());
        note.push(T_END!());
        continues = Some(Tokens::new(value));
      },
      "restate" => {
        iskv = true;
        // `\thmt@splitrestateargs`: `[<keys>]<name>`, the keys for the restated copy only; its `\@ifnextchar[` skips the
        // spaces before the bracket or the name (2602.09462 writes the value over several lines), a space after the
        // name is part of it
        let start = value
          .iter()
          .position(|t| t.get_catcode() != Catcode::SPACE)
          .unwrap_or(value.len());
        let value = value[start..].to_vec();
        let name = match value.first() {
          Some(t) if t.get_catcode() == Catcode::OTHER && t.to_string() == "[" => {
            // the `]` outside braces (`name={[x]}` keeps its own)
            let mut level = 0i32;
            let close = value.iter().position(|t| {
              match t.get_catcode() {
                Catcode::BEGIN => level += 1,
                Catcode::END => level -= 1,
                _ => {},
              }
              level == 0 && t.get_catcode() == Catcode::OTHER && t.to_string() == "]"
            });
            match close {
              Some(close) => {
                restate_extra = value[1..close].to_vec();
                value[close + 1..].to_vec()
              },
              None => value,
            }
          },
          _ => value,
        };
        if !name.is_empty() {
          restate = Some(Tokens::new(name));
        }
      },
      _ => {},
    }
  }
  if !restate_extra.is_empty() {
    if !restate_keys.is_empty() {
      restate_keys.push(T_OTHER!(","));
    }
    restate_keys.extend(restate_extra);
  }
  iskv.then_some(ThmUse {
    note,
    label,
    continues,
    restate,
    restate_keys,
  })
}

/// `tokens` without its leading and trailing spaces.
fn trim_spaces(tokens: &[Token]) -> Vec<Token> {
  let start = tokens
    .iter()
    .position(|t| t.get_catcode() != Catcode::SPACE)
    .unwrap_or(tokens.len());
  let end = tokens
    .iter()
    .rposition(|t| t.get_catcode() != Catcode::SPACE)
    .map_or(start, |i| i + 1);
  tokens[start..end].to_vec()
}

/// Whether `tokens`, which open with `{` and end with `}`, are one group (`{a}{b}` is two).
fn balanced_group(tokens: &[Token]) -> bool {
  let mut depth = 0i32;
  for (i, t) in tokens.iter().enumerate() {
    match t.get_catcode() {
      Catcode::BEGIN => depth += 1,
      Catcode::END => {
        depth -= 1;
        if depth == 0 && i + 1 < tokens.len() {
          return false;
        }
      },
      _ => {},
    }
  }
  true
}

/// A float's counter steps — at `\caption` (`\@@add@caption@counters`) or at a sub-float's
/// pre-increment. A continued float keeps its number instead: caption's `\continuedfloat` sets its
/// "continued" flag (`\caption@setcontinued`, caption.sty:192), under which the float's next step
/// is `\caption@@refcounter` (:557-568, chosen by `\caption@ifrefstepcounter`, :580) — here the
/// counter steps back and refsteps without resetting its sub-counters, once. The pending
/// continuation is one global value, the continued type (`lx@float@continued`), as caption's flags
/// are one global counter (:173-192). A real step of the current float's own counter
/// (`lx@float@main`, see `begin_float_continuation`), or of any float counter when no float type is
/// current, first runs caption's hook (as `prepare_float_step` does); the counter of a float caption never types
/// (`before_untyped_float`: a sub-float, a listing) steps plainly.
/// Witness 2605.17685; repro captions-floats/continuedfloat_keeps_the_number_and_the_letters.
pub fn step_float_counter(counter: &str) -> Result<SymHashMap<Stored>> {
  // Through `\refstepcounter`'s current meaning (`ref_step_counter_planned`): a measuring pass's
  // local rebinding (floatrow) leaves the counter, the continuation and caption's hook as they were;
  // a wrapper (hyperref, cleveref) reaches the kernel step, which carries them out (witnesses
  // kaytannollista-latexia, 2605.17685).
  if lookup_string("lx@float@untyped") == counter {
    return ref_step_counter_by_meaning(counter);
  }
  let continued = lookup_string("lx@float@continued") == counter;
  let main = lookup_string("lx@float@main");
  let hook = T_CS!("\\lx@float@stepped");
  let before = if !continued && (main.is_empty() || main == counter) && has_meaning(&hook) {
    let mut tokens = vec![hook, T_BEGIN!()];
    tokens.extend(Explode!(counter));
    tokens.push(T_END!());
    tokens
  } else {
    Vec::new()
  };
  ref_step_counter_planned(counter, before, continued)
}

/// caption's `\caption@prepare@stepcounter` (caption.sty:577-579): before a float's real step, the
/// `\lx@float@stepped{<counter>}` hook runs when a package defines it (caption's
/// `\caption@reset@continuedfloat`, :501-503: the type a `\ContinuedFloat` may continue, and its
/// count restarted). A sub-caption's step never prepares (caption's `\caption@subtypehook` points
/// its `\caption@refstepcounter` at plain `\refstepcounter`, caption.sty:662, :668).
pub fn prepare_float_step(counter: &str) -> Result<()> {
  let hook = T_CS!("\\lx@float@stepped");
  if has_meaning(&hook) {
    let mut tokens = vec![hook, T_BEGIN!()];
    tokens.extend(Explode!(counter));
    tokens.push(T_END!());
    digest(Tokens::new(tokens))?;
  }
  Ok(())
}

/// caption's `\caption@settype` (caption.sty:300-303): the float of this type begins, which
/// records it as the current float's type (`lx@float@main`, local — the counter whose real steps
/// `step_float_counter` prepares; an enclosing untyped float's mark, `lx@float@untyped`, is reset)
/// and clears any pending continuation (`\caption@clrflags`, :302, global). caption runs it at every float's begin built by `\@xfloat` (`\caption@xfloat`,
/// :271-275; `before_float_ex`), at `\captionof` (`\caption@of` → `\setcaptiontype*`, :389-391,
/// :288-297), at `\captionsetup{type=…}` (:283) and at a longtable's caption (`\caption@LT@settype`,
/// :1167); not for a float it never types (`before_untyped_float`). Guards
/// `perfect_kernel_batch56::{continuedfloat_counts_only_the_main_floats_steps,
/// continuedfloat_scope_opens_where_caption_sets_the_type}`.
pub fn begin_float_continuation(float_type: &str) {
  assign_value("lx@float@main", float_type.to_string(), Some(Scope::Local));
  assign_value("lx@float@untyped", String::new(), Some(Scope::Local));
  assign_value("lx@float@continued", String::new(), Some(Scope::Global));
}

/// `before_float` for a float caption never types: subfig's, subfigure.sty's and subfloat.sty's
/// sub-floats and containers (their own packages step them), and listings' floats — no
/// `\caption@settype`, so a pending continuation survives it, and its own counter's steps neither
/// prepare nor take the continuation (`step_float_counter`). Guard
/// `perfect_kernel_batch56::continuedfloat_scope_opens_where_caption_sets_the_type`.
pub fn before_untyped_float(float_type: &str) -> Result<()> {
  assign_value(
    "lx@float@untyped",
    float_type.to_string(),
    Some(Scope::Local),
  );
  begin_float(float_type, None, false, false)
}

/// latex.ltx `\@floatboxreset` (`\reset@font\normalsize`, run by `\@xfloat`): a floating
/// environment's text starts in the body font, size and leading whatever surrounds it — through
/// `\lx@floatbox@reset`, which setspace makes single-spaced (setspace.sty:418-422), so a document's
/// `\doublespacing` no longer stretches its tables (2605.18633 S6.T1). Called by the environments
/// that model `\@xfloat` (figure/table, `\@float`, float.sty, rotating, acmart, jhep, aas plates,
/// wrapstuff) — not by sub-floats, wrapfig/floatflt, an algorithm2e placed `[H]` (a minipage),
/// supertabular (`\lx@float@keepsize`) or listings, which reset nothing in LaTeX. A class with no
/// `\normalsize` (amsppt) has none to run. Guards `perfect_kernel_batch58::{line_spacing_commands_set_the_stretch,
/// only_floats_reset_the_size}`.
pub fn reset_float_box() -> Result<()> {
  // A binding that opens `{table}`/`{figure}` for an environment LaTeX does not float (supertabular)
  // asks for no reset with `\lx@float@keepsize`, consumed by the next float begin.
  if lookup_bool("lx@float@keepsize") {
    assign_value("lx@float@keepsize", false, Some(Scope::Global));
    return Ok(());
  }
  if !x_equals(&T_CS!("\\normalsize"), &T_CS!("\\lx@kernel@normalsize")) {
    digest(Tokens!(T_CS!("\\lx@floatbox@reset")))?;
  }
  Ok(())
}

/// The width registers a minipage of width `width` sets (latex.ltx:16312-16313 `\@iiiminipage`: `\hsize#4`,
/// `\textwidth\hsize \columnwidth\hsize`, then `\@parboxrestore`'s `\linewidth\hsize`), in the current group: the
/// `{minipage}` environment and the sub-float environments subcaption makes of one (subcaption.sty:70-92
/// `\subcaption@iiminipage`). Perl's `{minipage}` assigns the first three (latex_constructs.pool.ltxml:4785-4787;
/// `\linewidth` OXIDIZED_DESIGN tcolorbox note, witness 2605.02240) and its sub-floats only `beforeFloat`'s `\hsize`
/// (:3377), so a `width=\linewidth` image in a `{subfigure}{.28\textwidth}` took the page's width (KNOWN_PERL_ERRORS
/// #1028; witness 2509.19464 A1.F6). Guard `perfect_kernel_batch78d::subfigure_sets_the_line_width`.
pub fn set_minipage_width(width: Dimension) -> Result<()> {
  // The page outside the outermost such box (`PANEL_BOX_PAGE`), which a float held in it is a fraction of.
  if lookup_value(PANEL_BOX_PAGE).is_none() {
    let register = |name: &str| Stored::Dimension(lookup_dimension(name).unwrap_or_default());
    assign_value(
      PANEL_BOX_PAGE,
      Stored::VecDequeStored(VecDeque::from([
        register("\\textwidth"),
        register("\\columnwidth"),
      ])),
      Some(Scope::Local),
    );
  }
  let rv: RegisterValue = width.into();
  for register in ["\\hsize", "\\textwidth", "\\columnwidth", "\\linewidth"] {
    assign_register(register, rv.clone(), None, Vec::new())?;
  }
  Ok(())
}

/// Perl: beforeFloat (latex_constructs.pool.ltxml L3430-3438)
/// Sets \@captype, adjusts \hsize for single/double column floats.
/// `preincrement`: if Some("figure"), pre-increments the parent float counter
///   on first subfloat entry (before main caption), storing result for later use.
pub fn before_float(float_type: &str, preincrement: Option<&str>) -> Result<()> {
  before_float_ex(float_type, preincrement, false)
}
/// Extended version with `double` flag for `*` variants (span both columns).
pub fn before_float_ex(float_type: &str, preincrement: Option<&str>, double: bool) -> Result<()> {
  begin_float(float_type, preincrement, double, true)
}
fn begin_float(
  float_type: &str,
  preincrement: Option<&str>,
  double: bool,
  typed: bool,
) -> Result<()> {
  set_aside_pending_caption(float_type);
  // The boxes open in this float (`begin_caption_box`) end before it does; `after_float` unwinds any whose end an
  // error recovery skipped.
  assign_value(
    BOX_DEPTH_AT_FLOAT,
    Stored::Number(Number(caption_box_depth() as i64)),
    Some(Scope::Local),
  );
  def_macro(
    T_CS!("\\@captype"),
    None,
    Tokens::new(ExplodeText!(float_type)),
    None,
  )
  .ok();
  // Scoped to the float's group: a `\caption` digested with `\@captype` set but
  // without this marker is outside any float (sect09.rs `\lx@@caption`).
  let in_float = lookup_bool("lx@in@float");
  assign_value("lx@in@float", true, Some(Scope::Local));
  // The float's own frame: latex.ltx's `\@endfloatbox` closes a float box only when a raw
  // opener made one on top of it (sect09.rs).
  assign_value(
    "lx@float@frame",
    Stored::Number(Number(current_frame_id() as i64)),
    Some(Scope::Local),
  );
  // Every float and sub-float is its own caption's target: a capture an enclosing float set up
  // (floatrow's `\@floatcapt`) does not take it, as caption.sty resets `\caption@setfloatcapt`
  // for sub-types (:648, in `\caption@subtypehook`, :639-664). Guard `perfect_kernel_batch58::floatrow_subcaption_stays_in_its_panel`.
  Let!("\\lx@setfloatcapt", "\\lx@kernel@setfloatcapt");
  // Perl #2775: rebind \\ to \lx@newline in floats to prevent
  // alignment-token early-return when floats are inside tabulars.
  Let!("\\\\", "\\lx@newline");
  // Between a float's panels (the panel grid rule), while each has the kernel's meaning — an author's or a package's
  // own stands (review rev78x finding 11): a forced `\linebreak` ends their row as `\\` does (`\lx@panel@linebreak`,
  // sect09.rs); a `\vspace` does not (`\lx@panel@vspace`, sect12.rs), a `\vskip` does; the glue an `\hspace` states
  // counts towards their row (`\lx@panel@hspace`, sect12.rs).
  for (name, kernel, panel) in [
    (
      "\\linebreak",
      "\\lx@kernel@linebreak",
      "\\lx@panel@linebreak",
    ),
    ("\\vspace", "\\lx@kernel@vspace", "\\lx@panel@vspace"),
    ("\\hspace", "\\lx@kernel@hspace", "\\lx@panel@hspace"),
  ] {
    if x_equals(&T_CS!(name), &T_CS!(kernel)) {
      let_definition(&T_CS!(name), &T_CS!(panel), None);
    }
  }
  // Perl: AssignRegister('\hsize' => LookupDimension($options{double} ? '\textwidth' :
  // '\columnwidth'));
  let dim_name = if double {
    "\\textwidth"
  } else {
    "\\columnwidth"
  };
  let dim_val = lookup_dimension(dim_name).unwrap_or_default();
  assign_register("\\hsize", dim_val.into(), None, Vec::new()).ok();
  // The real line the float's panels are set in rows against (`install_panel_line`), in a float not inside another.
  if !in_float {
    install_panel_line(double);
  } else {
    note_panel_hbox();
  }
  // Perl: if (my $main = $options{preincrement}) {
  //   if (($type ne (LookupValue('LAST_FLOATTYPE') || ''))
  //     && !IfCondition('\iflx@donecaption')) {
  //     AssignValue('PREINCREMENTED_' . $main => { RefStepCounter($main) }, 'global'); } }
  // A typed float's begin is caption's `\caption@settype` (`begin_float_continuation`) — except the
  // float `\@captionof@` wraps a caption in (`lx@float@captionof`, one-shot: `\captionof` has set
  // the type, a typed `\caption` sets none) and a pre-incrementing sub-float environment, whose own
  // counter steps plainly (caption's sub-captions, caption.sty:662, :668).
  if lookup_bool("lx@float@captionof") {
    assign_value("lx@float@captionof", false, Some(Scope::Global));
  } else if preincrement.is_some() {
    assign_value(
      "lx@float@untyped",
      float_type.to_string(),
      Some(Scope::Local),
    );
  } else if typed {
    begin_float_continuation(float_type);
  }
  if let Some(main_counter) = preincrement {
    preincrement_float_counter(float_type, main_counter);
  }
  Ok(())
}
/// The state value (and the whatsit property `after_float` copies it to) of the real line a float's panels are set in
/// rows against, and the frame of the float that installed it.
const PANEL_LINE: &str = "panel_line_width";
const PANEL_LINE_FRAME: &str = "lx@panel@line@frame";
/// The whatsit property of the line a float with no real line (`PANEL_LINE`) is arranged at under geometry
/// (`after_float`): `\Gm@tw`, as before the panel grid rule.
const PANEL_FALLBACK_LINE: &str = "panel_fallback_line";
/// The state value (and the whatsit property `after_float` copies it to) of the real running-text column at the float
/// (`install_panel_line`: the class's real column in two columns, its text width in one) that the float's line is a
/// fraction of (`panelline`, `annotate_panel_grid`).
const PANEL_COLUMN: &str = "panel_column_width";
/// The state value of the float's `\hsize` as `install_panel_line` found it: one its binding sets afterwards (a
/// wrapfigure's width, wrapfig_sty.rs) is the float's own line, which the class's page does not replace
/// (`after_float`; 2609.34643 S3.SS3.fig1, a 0.27\textwidth wrapfigure).
const PANEL_LINE_HSIZE: &str = "lx@panel@line@hsize";
/// The state value of the geometry a float's stated widths are evaluated at (`install_panel_line`): the registers' own
/// values and the real ones, `\textwidth`, `\columnwidth`, the line (`\linewidth`, `\hsize`), in scaled points.
const PANEL_GEOMETRY: &str = "lx@panel@geometry";
/// The whatsit property of a float whose panels' widths are the real ones though it has no real line of its own
/// (`after_float`): the registers are the real geometry where it is set.
const PANEL_REGISTERS_REAL: &str = "panel_registers_real";
/// The whatsit property a panel's box carries once its stated width is evaluated (`note_stated_widths`): what the
/// real geometry adds to it (scaled points, a `Dimension`), for `arrange_panels` alone.
const PANEL_WIDTH_DELTA: &str = "panel_width_delta";
/// The whatsit property of an `\hspace` between a float's panels (`\lx@panel@hskip`): what the real geometry adds to
/// its natural width.
pub(crate) const PANEL_GLUE_DELTA: &str = "panel_glue_delta";

/// The kernel's `\textwidth` (latex_constructs.pool.ltxml:1038 `DefRegister('\textwidth' => Dimension('345pt'))`).
const KERNEL_TEXTWIDTH: i64 = 345 * 65536;

/// A class binding's declaration of the page its panels are set in rows against (`declare_panel_page`): the column
/// and text widths of the class's real page and the binding's own `\textwidth` (scaled points), and whether the page is
/// set in two columns now.
const PANEL_PAGE: &str = "lx@panel@page";
const PANEL_PAGE_TWOCOLUMN: &str = "lx@panel@page@twocolumn";
/// The state value of the page outside the outermost box that set the width registers (`set_minipage_width`: a
/// `{minipage}`), its `\textwidth` and `\columnwidth` (scaled points), for a float held in it (`install_panel_line`).
const PANEL_BOX_PAGE: &str = "lx@panel@box@page";

/// The panel grid rule's line (user ruling 2026-10-22, FIDELITY_CHARTER §2 "Float panels"): a float's panels are set in
/// rows against the class's real line (`panel_page`: a class binding's declared page, else the registers' and the
/// geometry's) — in two columns a `figure` against the column, a `figure*` (`double`) against the text width. The
/// registers keep Perl's values (latex_constructs.pool.ltxml:321 `\columnwidth` = `\textwidth`), so where the real
/// line differs the float keeps the widths its body states as tokens (`KEEP_STATED_LENGTHS`) and evaluates them once
/// more at the real geometry as its element opens (`note_stated_widths`), for `arrange_panels` alone. Witnesses 1707.00190
/// Fig 2, nucl-ex0511052 Fig 13, 2502.05562 Fig 6, astro-ph0205132 (`0.74\columnwidth` pairs in a two-column
/// `figure*`, one row in the PDF); 1508.01788 Figs 19-22, 1411.5428 Figs 6-7, 2111.05508 (the geometry's width).
/// OXIDIZED_DESIGN_DIVERGENCES #885.
fn install_panel_line(double: bool) {
  let register = |name: &str| lookup_dimension(name).map(|d| d.value_of());
  let (Some(textwidth), Some(columnwidth), Some(hsize)) = (
    register("\\textwidth"),
    register("\\columnwidth"),
    register("\\hsize"),
  ) else {
    return;
  };
  let linewidth = register("\\linewidth").unwrap_or(hsize);
  // A float held in a box that set the width registers (a `{minipage}`, latex.ltx `\@iiiminipage`: `\textwidth`,
  // `\columnwidth` and `\hsize` are its width) is set against that box's line, a fraction of the running-text column
  // outside it (`PANEL_BOX_PAGE`): a `[H]` figure in a `{minipage}{0.5\textwidth}` states `panelline` 0.5 as the PDF
  // gives it (review rev79c2 finding 2; the box's `\textwidth` read as the page stated 1). Only where the registers
  // outside the box are the real page are the box's registers real; elsewhere its real width is not known, and the
  // float is arranged at its `\hsize`, as before the rule, stating no line (never a guess).
  let box_page = match lookup_value(PANEL_BOX_PAGE) {
    Some(Stored::VecDequeStored(page)) => match (page.front(), page.get(1)) {
      (Some(Stored::Dimension(text)), Some(Stored::Dimension(column))) => {
        Some((text.value_of(), column.value_of()))
      },
      _ => None,
    },
    _ => None,
  };
  let (real_text, real_column, real_line, page_column) = match box_page {
    Some((outer_text, outer_column)) => {
      let (page_text, page_column) = panel_page(outer_text);
      if page_text != outer_text || page_column != outer_column {
        return;
      }
      (textwidth, columnwidth, hsize, page_column)
    },
    None => {
      let (real_text, real_column) = panel_page(textwidth);
      let real_line = if double { real_text } else { real_column };
      (real_text, real_column, real_line, real_column)
    },
  };
  assign_value(
    PANEL_COLUMN,
    Stored::Dimension(Dimension::new(page_column)),
    Some(Scope::Local),
  );
  assign_value(
    PANEL_LINE,
    Stored::Dimension(Dimension::new(real_line)),
    Some(Scope::Local),
  );
  assign_value(
    PANEL_LINE_FRAME,
    Stored::Number(Number(current_frame_id() as i64)),
    Some(Scope::Local),
  );
  assign_value(
    PANEL_LINE_HSIZE,
    Stored::Dimension(Dimension::new(hsize)),
    Some(Scope::Local),
  );
  // Where every register is already the real one, the stated widths add nothing.
  if [
    (textwidth, real_text),
    (columnwidth, real_column),
    (linewidth, real_line),
    (hsize, real_line),
  ]
  .iter()
  .all(|(ours, real)| ours == real)
  {
    return;
  }
  let geometry = [
    textwidth,
    columnwidth,
    linewidth,
    hsize,
    real_text,
    real_column,
    real_line,
  ];
  assign_value(
    PANEL_GEOMETRY,
    Stored::VecDequeStored(
      geometry
        .iter()
        .map(|v| Stored::Dimension(Dimension::new(*v)))
        .collect(),
    ),
    Some(Scope::Local),
  );
  assign_value(KEEP_STATED_LENGTHS, true, Some(Scope::Local));
}

/// The state values (and whatsit properties, `after_float`) of a sub-float set in a TeX `\hbox` (the panel grid rule):
/// its body is one (`PANEL_HBOX_BODY`: subfigure.sty's `\subfigure`, whose body `\@subfloat` sets in an `\hbox`,
/// subfigure.sty:232), or it is begun in one, with the group of that box it is begun in (`PANEL_HBOX_ROW`: the
/// sub-figures of an adjustbox, one `\hbox` in TeX, 2311.18531 A7.F5/F6). TeX breaks no line inside an `\hbox`, so
/// `arrange_panels` keeps such panels on one row however wide they add up, as a binding's fixed row (`_gridrow`);
/// overfull, they are TeX's overfull box (cond-mat0610021 Fig 11: two `width=200pt` graphics in one sub-figure).
const PANEL_HBOX_BODY: &str = "panel_hbox_body";
const PANEL_HBOX_ROW: &str = "panel_hbox_row";

/// A sub-float's `\hbox` (`PANEL_HBOX_BODY`, `PANEL_HBOX_ROW`), as it begins: its own mode (its environment's) and the
/// mode of the group it is begun in. The box is the group that set that mode, not the group the sub-float is begun
/// in: subcaption's `\subfloat` begins its sub-figure in a `\begingroup` of its own (subcaption_sty.rs
/// `subfloat_tokens`), so four `\subfloat`s in an `adjustbox` or a `\resizebox`, one `\hbox` in TeX, were four
/// groups and four rows (review rev78x2 finding 3; 2311.18531 A7.F5/F6 the environment form).
fn note_panel_hbox() {
  let modes = value_history("MODE", 2);
  let horizontal_box = |mode: Option<&String>| mode.is_some_and(|m| m == "restricted_horizontal");
  assign_value(
    PANEL_HBOX_BODY,
    horizontal_box(modes.first()),
    Some(Scope::Local),
  );
  if horizontal_box(modes.get(1))
    && let Some(frame) = frame_of_binding("MODE", 1)
    && let Some(group) = value_in_frame("lx@frame@id", frame)
  {
    assign_value(
      PANEL_HBOX_ROW,
      Stored::String(pin(group.to_string())),
      Some(Scope::Local),
    );
  } else {
    assign_value(PANEL_HBOX_ROW, Stored::None, Some(Scope::Local));
  }
}

/// The class's real text and column widths for panel rows (scaled points). The text width is `\textwidth`, or the
/// geometry's `\Gm@tw` where it computed one and `\textwidth` is still the class's (`\Gm@doctw`), or a class binding's
/// declared page (`declare_panel_page`) while `\textwidth` is still the binding's (an author's `\textwidth 7.2in` is
/// the page: 2211.03584). In two columns — the declared page's setting, else `\if@twocolumn` — the column is the
/// declared one on the declared page, else `(text − \columnsep)/2` as latex.ltx's `\twocolumn` sets `\columnwidth`, the
/// declared page's separation between its columns where it has one. A two-column binding that keeps the kernel's 345pt
/// line and declares no page states no page of its own: that line is the column, the text width two of them and the
/// column separation (1705.10798's 4cm pairs in a REVTeX column, one row in the PDF).
fn panel_page(textwidth: i64) -> (i64, i64) {
  let register = |name: &str| lookup_dimension(name).map(|d| d.value_of());
  let declared = match lookup_value(PANEL_PAGE) {
    Some(Stored::VecDequeStored(page)) => {
      match (page.front(), page.get(1), page.get(2), page.get(3)) {
        (
          Some(Stored::Dimension(column)),
          Some(Stored::Dimension(text)),
          Some(Stored::Dimension(own)),
          Some(Stored::Dimension(own_columnsep)),
        ) => Some((
          column.value_of(),
          text.value_of(),
          own.value_of(),
          own_columnsep.value_of(),
        )),
        _ => None,
      }
    },
    _ => None,
  };
  let mut real_text = textwidth;
  let geometry = match (register("\\Gm@tw"), register("\\Gm@doctw")) {
    (Some(gm), Some(doc)) if gm > 0 && gm != doc && textwidth == doc => Some(gm),
    _ => None,
  };
  if let Some(gm) = geometry {
    real_text = gm;
  } else if let Some((_, text, own, _)) = declared
    && textwidth == own
  {
    real_text = text;
  }
  let two_column = match declared {
    Some(_) => lookup_bool(PANEL_PAGE_TWOCOLUMN),
    None => if_condition(&T_CS!("\\if@twocolumn")).ok().flatten() == Some(true),
  };
  // (the declared page's separation while `\columnsep` is still the binding's: a style that sets its own page, as
  // 2008.06630's 3dv.sty over `[twocolumn]{article}`, `\textwidth 6.875in` 0.3125in apart, sets its separation)
  let columnsep_register = register("\\columnsep").unwrap_or(0);
  let columnsep = match declared {
    Some((column, text, _, own_columnsep)) if columnsep_register == own_columnsep => {
      text - 2 * column
    },
    _ => columnsep_register,
  };
  if !two_column {
    (real_text, real_text)
  } else if let Some((column, text, ..)) = declared
    && real_text == text
  {
    (text, column)
  } else if declared.is_none() && real_text == KERNEL_TEXTWIDTH {
    (2 * real_text + columnsep, real_text)
  } else {
    (real_text, (real_text - columnsep) / 2)
  }
}

/// A class binding declares its real page for panel rows (§3 of the panel grid rule, OXIDIZED_DESIGN_DIVERGENCES
/// #885): the column and text widths of the class's page in points (`column_pt`, `text_pt`, as pdflatex's
/// `\the\columnwidth`/`\the\textwidth` in a two-column float give them for the options given), and whether the page
/// is set in two columns now (`twocolumn`; `set_panel_page_twocolumn` follows a one-column region). The registers keep
/// the binding's values; only the rows are set against this page. Without a declaration the rows follow the registers
/// (`panel_page`).
pub fn declare_panel_page(column_pt: f64, text_pt: f64, twocolumn: bool) {
  let sp = |pt: f64| Stored::Dimension(Dimension::new((pt * 65536.0).round() as i64));
  // (and the binding's own `\textwidth`, while which the page is the declared one, and `\columnsep`, while which its
  // columns are the declared ones apart: `panel_page`)
  let own = lookup_dimension("\\textwidth").unwrap_or_default();
  let own_columnsep = lookup_dimension("\\columnsep").unwrap_or_default();
  assign_value(
    PANEL_PAGE,
    Stored::VecDequeStored(VecDeque::from([
      sp(column_pt),
      sp(text_pt),
      Stored::Dimension(own),
      Stored::Dimension(own_columnsep),
    ])),
    Some(Scope::Global),
  );
  assign_value(PANEL_PAGE_TWOCOLUMN_CLASS, twocolumn, Some(Scope::Global));
  set_panel_page_twocolumn(twocolumn);
}

/// A class binding that loaded another's declaration (`declare_panel_page`; AASTeX over REVTeX's) whose page it does
/// not know withdraws it: its rows follow the registers.
pub fn withdraw_panel_page() { assign_value(PANEL_PAGE, Stored::None, Some(Scope::Global)); }

/// The two-column page of LaTeX's standard classes (`declare_panel_page`), as `size10.clo`/`size11.clo`/`size12.clo`
/// (and `bk1x.clo`) set it for the document's class options: in two columns `\textwidth` is the paper's width less
/// 2in, at most twice the size's one-column 345/360/390pt, `\@settopoint`ed; `\columnwidth` is half of it less the
/// classes' 10pt `\columnsep` (article.cls:627, latex.ltx `\twocolumn`). The paper is the last declared of the
/// classes' paper options (article.cls:53-77, `landscape` swapping its sides), the size the last of `10pt`/`11pt`/
/// `12pt` (`\ProcessOptions` runs them in their declaration order); under 2.09 compatibility (`\documentstyle`) the
/// size files set 410pt (size10.clo `\if@compatibility\if@twocolumn\setlength\textwidth{410\p@}`). Returns (column,
/// text) in points; pdflatex gives 229.5/469pt on letter paper, 221/452pt on A4, 200/410pt for `\documentstyle`.
pub fn standard_class_twocolumn_page() -> (f64, f64) {
  if lookup_bool("2.09_COMPATIBILITY") {
    return (200.0, 410.0);
  }
  let (width, height) = match last_declared_class_option(&[
    "a4paper",
    "a5paper",
    "b5paper",
    "letterpaper",
    "legalpaper",
    "executivepaper",
  ]) {
    Some("a4paper") => (597.50787, 845.04684),
    Some("a5paper") => (421.10078, 597.50787),
    Some("b5paper") => (500.76849, 711.31892),
    Some("legalpaper") => (614.295, 1011.78),
    Some("executivepaper") => (523.9575, 758.835),
    _ => (614.295, 794.97),
  };
  let width = if given_class_options().iter().any(|o| o == "landscape") {
    height
  } else {
    width
  };
  let one_column = match last_declared_class_option(&["10pt", "11pt", "12pt"]) {
    Some("11pt") => 360.0,
    Some("12pt") => 390.0,
    _ => 345.0,
  };
  let text = (width - 144.54_f64).min(2.0 * one_column).floor();
  ((text - 10.0) / 2.0, text)
}

/// A standard class (article, report, book) that is the document's (`is_document_class`) declares its page in two
/// columns (`standard_class_twocolumn_page`) under `twocolumn` (the last declared of `onecolumn`/`twocolumn`): its
/// `figure` is set against the 229.5pt column, its `figure*` against the 469pt text on letter paper, where the
/// registers' 345pt read as the column and twice it as the text (`panel_page`; rev79c2: `figure*` `panelline` 2 for
/// the PDF's 2.044). A class loading it (OmniBus, elsarticle, REVTeX) is not it: its page is its own. One column is
/// the registers' page.
pub fn declare_standard_class_page(class: &str) {
  if is_document_class(class)
    && last_declared_class_option(&["onecolumn", "twocolumn"]) == Some("twocolumn")
  {
    let (column, text) = standard_class_twocolumn_page();
    declare_panel_page(column, text, true);
  }
}

/// The state value of a class whose floats are centred (`declare_panel_floats_centred`).
const PANEL_FLOATS_CENTRED: &str = "lx@panel@floats@centred";

/// A class binding declares that its floats are centred, every `\\` in them `\@centercr` (REVTeX's `\@xfloat@prep`,
/// revtex4-2.cls:6770-6772 `\appdef\@parboxrestore{\centering}`), where the registers' floats are not: an `\hspace`
/// after such a `\\` begins a paragraph, whose glue a float's row keeps (`arrange_panels`; review rev78x3 finding 3,
/// `p1.tex` F10). The markup is not changed.
pub fn declare_panel_floats_centred() {
  assign_value(PANEL_FLOATS_CENTRED, true, Some(Scope::Global));
}

/// A class binding that loaded REVTeX's (`declare_panel_floats_centred`) whose floats are not centred withdraws the
/// declaration: AASTeX 6.x and 7.x load revtex4-1 and then `\def\@xfloat@prep` again without `\centering`
/// (aastex63.cls:1450, aastex701.cls:8066, "without it, captions are centered"), as emulateapj.cls:949 does; AASTeX 5.x
/// is `article`'s (aastex.cls:129). Review rev78x4 finding 2 (n5: aastex701, `\\\hspace{0.15\columnwidth}` before
/// three 0.3\columnwidth panels, two lines in the PDF).
pub fn withdraw_panel_floats_centred() {
  assign_value(PANEL_FLOATS_CENTRED, false, Some(Scope::Global));
}

/// The document's class options, as given (`\@classoptionslist`), for a binding's `declare_panel_page`.
pub fn given_class_options() -> Vec<String> {
  let given = match lookup_definition(&T_CS!("\\@classoptionslist")) {
    Ok(Some(defn)) => match defn.get_expansion() {
      Some(ExpansionBody::Tokens(body)) => body.to_string(),
      _ => String::new(),
    },
    _ => String::new(),
  };
  given.split(',').map(|o| o.trim().to_string()).collect()
}

/// Which of `options` the document's class options give last (`\@classoptionslist`, a later option overriding an
/// earlier one), for a binding's `declare_panel_page`.
pub fn last_class_option<'a>(options: &[&'a str]) -> Option<&'a str> {
  given_class_options()
    .iter()
    .rev()
    .find_map(|g| options.iter().find(|o| **o == g).copied())
}

/// Which of `options` (in the order the class declares them) the document's class options give, the last declared:
/// the one that stands where `\ProcessOptions` runs the options in their declaration order (latex.ltx's
/// `\ProcessOptions`, as IEEEtran.cls's :376), for a binding's `declare_panel_page`.
pub fn last_declared_class_option<'a>(options: &[&'a str]) -> Option<&'a str> {
  let given = given_class_options();
  options
    .iter()
    .rev()
    .find(|o| given.iter().any(|g| g == **o))
    .copied()
}

/// The declared page (`declare_panel_page`) is set in two columns from here on, or in one (a class's `onecolumn`, a
/// one-column region: REVTeX's and AASTeX's `\onecolumngrid`/`\twocolumngrid`).
pub fn set_panel_page_twocolumn(twocolumn: bool) {
  assign_value(PANEL_PAGE_TWOCOLUMN, twocolumn, Some(Scope::Global));
}

/// The declared page's own column setting (`declare_panel_page`), which a region's end restores (REVTeX's
/// `\restorecolumngrid`).
const PANEL_PAGE_TWOCOLUMN_CLASS: &str = "lx@panel@page@twocolumn@class";

pub(crate) fn restore_panel_page_twocolumn() {
  set_panel_page_twocolumn(lookup_bool(PANEL_PAGE_TWOCOLUMN_CLASS));
}

/// The state value naming where a construct's stated width is (`declare_panel_width`), by the construct's control
/// sequence.
fn panel_width_key(cs: &str) -> String { s!("lx@panel@width@{cs}") }

/// A binding declares where a construct states its width (the panel grid rule, `note_stated_widths`): its argument
/// `arg` (1-based: `{minipage}{0.48\textwidth}`'s 4, `\resizebox`'s first), or, with `keys`, the last of those keys
/// given in its `KeyVals` argument `arg` (`\includegraphics[width=0.48\linewidth]`), and with `height_keys` the keys
/// whose value scales its natural width when it states none of `keys` (an aspect-keeping graphic's `height`). `cs` is
/// the construct's control sequence as its whatsit names it (`\begin{minipage}` for an environment).
pub fn declare_panel_width(cs: &str, arg: usize, keys: &[&str], height_keys: &[&str]) {
  let spec = if keys.is_empty() {
    arg.to_string()
  } else {
    s!("{arg}:{}|{}", keys.join(","), height_keys.join(","))
  };
  assign_value(
    &panel_width_key(cs),
    Stored::String(pin(&spec)),
    Some(Scope::Global),
  );
}

/// The state value marking a construct that sets its content in a parbox of the width it states
/// (`declare_panel_parbox_width`), by the construct's control sequence.
fn panel_parbox_key(cs: &str) -> String { s!("lx@panel@parbox@{cs}") }

/// A binding declares a construct that states its width as its argument `arg` (`declare_panel_width`) and sets its
/// content in a parbox of that width — a `{minipage}`, a `\parbox`, a sub-figure — whose `\linewidth`, `\hsize`,
/// `\textwidth` and `\columnwidth` are that width (latex.ltx:16312-16313 `\@iiiminipage`; for a `\parbox`, Perl's
/// latex_constructs.pool.ltxml:4746 `\hsize=#4\textwidth\hsize\columnwidth\hsize`, where latex.ltx:16249-16253
/// `\@iiiparbox` sets only `\hsize` and, by `\@parboxrestore`, `\linewidth`: the widths are evaluated as we digest
/// the box): its content's stated widths are evaluated against it (`note_stated_widths_in`). An `\hbox`'s
/// (`\makebox[w]`, `\resizebox{w}{!}`) content is set at the enclosing line.
pub fn declare_panel_parbox_width(cs: &str, arg: usize) {
  declare_panel_width(cs, arg, &[], &[]);
  assign_value(&panel_parbox_key(cs), true, Some(Scope::Global));
}

/// What a construct states its size with (`declare_panel_width`), where it was kept (`KEEP_STATED_LENGTHS`): the tokens
/// of its width and of its height, and whether it keeps its aspect ratio (a graphic), so that one of them alone scales
/// the other — an `\includegraphics[height=0.21\linewidth]`'s natural width (2209.01559 Fig 6) — and with graphicx's
/// `keepaspectratio` both bound it. A `{minipage}{0.48\textwidth}` or a `\resizebox{0.5\textwidth}{!}` states a
/// width only (a `\resizebox{!}{2cm}` none: its content's, scaled, `note_stated_widths_in`).
struct StatedSize {
  width:       Option<Tokens>,
  height:      Option<Tokens>,
  aspect:      bool,
  keep_within: bool,
  /// The width a positional argument gave the box (its digested value: the box is set at it), where the ratio of the
  /// real geometry to the registers' scales it (`note_stated_widths_in`).
  value:       Option<f64>,
  /// The width and the height the construct was given (a positional argument's digested value, a `width=`'s, a
  /// `height=`'s), which the stated tokens must give again at the registers' geometry (`note_stated_widths_in`).
  given:       (Option<f64>, Option<f64>),
  /// The construct sets its content in a parbox of its stated width (`declare_panel_parbox_width`).
  parbox:      bool,
}

/// What `whatsit` states its size with (`StatedSize`), if its binding declared where (`declare_panel_width`). A width or
/// height it was given whose tokens were not kept (a box built outside the float, `\sbox` … `\usebox`) is
/// `Unevaluable`: what it states is unknown.
fn stated_size_of(whatsit: &Whatsit) -> std::result::Result<Option<StatedSize>, Unevaluable> {
  let cs = whatsit.get_definition().get_cs_name().into_owned();
  let key = panel_width_key(&cs);
  let Some(Stored::String(spec)) = lookup_value(&key) else {
    return Ok(None);
  };
  let spec = to_string(spec);
  let (arg, keys) = match spec.split_once(':') {
    Some((arg, keys)) => (arg, Some(keys)),
    None => (spec.as_str(), None),
  };
  let Some(index) = arg.parse::<usize>().ok().and_then(|a| a.checked_sub(1)) else {
    return Ok(None);
  };
  let args = whatsit.get_args();
  let argument = args.get(index).and_then(Option::as_ref).map(Digested::data);
  match keys {
    None => {
      // a positional width (`!` for none)
      let value = match argument {
        Some(DigestedData::RegisterValue(v)) => Some(v.clone().value_of() as f64),
        _ => None,
      };
      let tokens = match whatsit.get_property(STATED_LENGTHS).as_deref() {
        Some(Stored::VecDequeStored(lengths)) => match lengths.get(index) {
          Some(Stored::Tokens(tokens)) => Some(tokens.clone()),
          _ => None,
        },
        _ => None,
      };
      match tokens {
        Some(tokens) if tokens.to_string().trim() != "!" => Ok(Some(StatedSize {
          width: Some(tokens),
          height: None,
          aspect: false,
          keep_within: false,
          value,
          given: (value, None),
          parbox: lookup_bool(&panel_parbox_key(&cs)),
        })),
        Some(_) => Ok(None),
        // (a length given whose tokens were not kept: a box built before the float; one a binding gave and withheld,
        // `WITHHOLD_STATED_LENGTHS`, states none — subcaption's `\subfloat`, review rev78x4 finding 3)
        None if value.is_some() && !whatsit.get_property_bool(STATED_LENGTHS_WITHHELD) => {
          Err(Unevaluable)
        },
        None => Ok(None),
      }
    },
    Some(keys) => {
      let (widths, heights) = keys.split_once('|').unwrap_or((keys, ""));
      let widths: Vec<&str> = widths.split(',').collect();
      let heights: Vec<&str> = heights.split(',').filter(|h| !h.is_empty()).collect();
      let Some(DigestedData::KeyVals(kv)) = argument else {
        return Ok(None);
      };
      // (a width or height given and not kept: what it states is unknown)
      let width = kv.stated_entry(&widths);
      let height = kv.stated_entry(&heights);
      if (width.is_none() && widths.iter().any(|w| kv.has_key(w)))
        || (height.is_none() && heights.iter().any(|h| kv.has_key(h)))
      {
        return Err(Unevaluable);
      }
      // (the value the key gave the box)
      let given = |entry: Option<(&str, &Tokens)>| {
        entry.and_then(
          |(key, _)| match kv.get_value_digested(key).map(Digested::data) {
            Some(DigestedData::RegisterValue(v)) => Some(v.clone().value_of() as f64),
            _ => None,
          },
        )
      };
      Ok((width.is_some() || height.is_some()).then(|| StatedSize {
        width:       width.map(|(_, tokens)| tokens.clone()),
        height:      height.map(|(_, tokens)| tokens.clone()),
        aspect:      true,
        keep_within: kv.has_key("keepaspectratio"),
        value:       None,
        given:       (given(width), given(height)),
        parbox:      false,
      }))
    },
  }
}

/// The registers a stated length is evaluated with: `\textwidth`, `\columnwidth`, `\linewidth`, `\hsize`.
const LINE_REGISTERS: [&str; 4] = ["\\textwidth", "\\columnwidth", "\\linewidth", "\\hsize"];

/// A format is being built (`--init`, a dump): what the panel grid rule records of a document's lengths
/// (`note_setlength`, `note_length_allocated`) is not recorded, so no dump holds it.
fn building_a_format() -> bool {
  static INI_MODE: std::sync::LazyLock<bool> =
    std::sync::LazyLock::new(|| std::env::var_os("LATEXML_INI_MODE").is_some());
  *INI_MODE || crate::dump_paths::building_format()
}

/// The state value in which a `\setlength` of a length register keeps what it stated (`note_setlength`), by the
/// register's name: the tokens of the length and the value they gave.
fn setlength_key(name: &str) -> String { s!("lx@panel@setlength@{name}") }

/// The state value marking a length register the document allocated (`note_length_allocated`), by its name.
fn allocated_length_key(name: &str) -> String { s!("lx@panel@allocated@{name}") }

/// A `\setlength` gave the length register `name` the `value` that `tokens` state (sect12.rs `assign_braced_length`,
/// calc's `\setlength`), recorded in the assignment's scope: a float's width read through the register
/// (`\setlength\figw{0.48\columnwidth}` … `\includegraphics[width=\figw]`) is evaluated at the class's real geometry as
/// that `\setlength` is there (`evaluate_stated_length`), while the register holds the value it gave. Review rev78x2
/// finding 1 (`q1.tex`/`q2.tex`: a register set in the preamble, in the body, in the float).
pub fn note_setlength(name: &str, tokens: &Tokens, value: i64) {
  if building_a_format() {
    return;
  }
  assign_value(
    &setlength_key(name),
    Stored::VecDequeStored(VecDeque::from([
      Stored::Tokens(tokens.clone()),
      Stored::Int(value),
    ])),
    None,
  );
}

/// The document allocated the length register `cs` (`\newlength`; `\newdimen`'s `\dimendef`, `\newskip`'s `\skipdef`,
/// tex_registers.rs): a value no `\setlength` recorded (`\figw=0.48\columnwidth`, `\settowidth`) is one of the
/// registers' geometry, which a float's stated width that reads it cannot evaluate at the real one
/// (`evaluate_stated_length`). The kernel's and the bindings' registers (`\tabcolsep`, `\columnsep`, `\p@`) are not
/// the document's.
pub fn note_length_allocated(cs: &Token) {
  if building_a_format() {
    return;
  }
  assign_value(
    &allocated_length_key(&cs.with_cs_name(ToString::to_string)),
    true,
    Some(Scope::Global),
  );
}

/// A stated length the real geometry cannot evaluate (`evaluate_stated_length`): it reads a length register the
/// document allocated whose value no `\setlength` recorded (or one changed since), a value of the registers' geometry;
/// or it cannot be read at the float's end, or does not give the box's value again there (`note_stated_widths_in`); or
/// a box was given a length whose tokens were not kept (`stated_size_of`). Its float's rows fall back to the line
/// before the rule, the geometry's included (`after_float`) — never worse than that (review rev78x2 finding 1,
/// rev78x3 findings 1-2).
struct Unevaluable;

/// The length registers the length `tokens` state reads besides the line's (`LINE_REGISTERS`), expanded as their read
/// expands them (`\newcommand\w{\figw}`).
fn stated_length_registers(tokens: &Tokens) -> Vec<Rc<Register>> {
  let _quiet = IgnoreDiagnosticsScope::new();
  begingroup();
  let found = read_braced(tokens.clone(), || {
    let mut found = Vec::new();
    while let Some(token) = read_x_token(None, false, None)? {
      if let Some(register) = lookup_register_definition(&token)
        && matches!(
          register.register_type(),
          Some(RegisterType::Dimension | RegisterType::Glue)
        )
        && !LINE_REGISTERS.contains(&register.get_cs_name().as_ref())
      {
        found.push(register);
      }
    }
    Ok(found)
  })
  .map(|(found, _)| found)
  .unwrap_or_default();
  endgroup().ok();
  found
}

/// Evaluate the length `tokens` state (a glue's natural width with `glue`) with the line registers at `registers`
/// (`LINE_REGISTERS`), in a group and a mouth of its own, as a braced length argument is read (calc's evaluator where
/// it is loaded, `Parameters::reparse_argument`). A length register it reads is what its `\setlength` stated, evaluated
/// at the same geometry (`note_setlength`); one the document allocated whose value no `\setlength` recorded makes it
/// `Unevaluable`. Its diagnostics were the source's when it was first read and are not reported again; one it cannot
/// read gives `None`.
fn evaluate_stated_length(
  tokens: &Tokens,
  registers: [i64; 4],
  glue: bool,
) -> std::result::Result<Option<i64>, Unevaluable> {
  evaluate_stated_length_at(tokens, registers, glue, 0)
}

fn evaluate_stated_length_at(
  tokens: &Tokens,
  registers: [i64; 4],
  glue: bool,
  depth: usize,
) -> std::result::Result<Option<i64>, Unevaluable> {
  let mut reads = Vec::new();
  for register in stated_length_registers(tokens) {
    let name = register.get_cs_name().into_owned();
    let current = register.value_of(Vec::new()).map(|v| v.value_of());
    match lookup_value(&setlength_key(&name)) {
      Some(Stored::VecDequeStored(record)) => match (record.front(), record.get(1)) {
        (Some(Stored::Tokens(stated)), Some(Stored::Int(value)))
          if current == Some(*value) && depth < 4 =>
        {
          let is_glue = register.register_type() == Some(RegisterType::Glue);
          let value =
            evaluate_stated_length_at(stated, registers, is_glue, depth + 1)?.ok_or(Unevaluable)?;
          reads.push((register, is_glue, value));
        },
        _ => return Err(Unevaluable),
      },
      _ if lookup_bool(&allocated_length_key(&name)) => return Err(Unevaluable),
      _ => {},
    }
  }
  let _quiet = IgnoreDiagnosticsScope::new();
  let kind = if glue {
    RegisterType::Glue
  } else {
    RegisterType::Dimension
  };
  begingroup();
  let value = (|| {
    for (name, value) in LINE_REGISTERS.iter().zip(registers) {
      assign_register(name, Dimension::new(value).into(), None, Vec::new())?;
    }
    for (register, is_glue, value) in &reads {
      let value: RegisterValue = if *is_glue {
        Glue::new(*value).into()
      } else {
        Dimension::new(*value).into()
      };
      register.set_value(value, None, Vec::new());
    }
    let (value, _tail) = read_braced(tokens.clone(), || match braced_length_evaluator() {
      Some(evaluate) => Ok(evaluate(kind)?.value_of()),
      None if glue => Ok(read_glue()?.value_of()),
      None => Ok(read_dimension()?.value_of()),
    })?;
    Ok::<i64, Error>(value)
  })();
  endgroup().ok();
  Ok(value.ok())
}

/// The line a stated width is evaluated against: the registers' values (`ours`: `\textwidth`, `\columnwidth`, the
/// line) and the real ones.
#[derive(Clone, Copy)]
struct PanelLineContext {
  ours: [i64; 4],
  real: [i64; 4],
}

impl PanelLineContext {
  /// Inside a box of `ours`/`real` width (a minipage sets `\textwidth`, `\columnwidth`, `\hsize` and `\linewidth` to
  /// its width, latex.ltx `\@iiiminipage`).
  fn inside(ours: i64, real: i64) -> Self {
    PanelLineContext {
      ours: [ours; 4],
      real: [real; 4],
    }
  }
}

/// What the real geometry adds to a box's width and height (scaled points), for `note_stated_widths`: a transformed
/// box's from its content's (a `\scalebox` scales it, a rotation turns its axes).
#[derive(Clone, Copy, Default)]
struct PanelDelta {
  width:  f64,
  height: f64,
}

impl PanelDelta {
  /// Two boxes side by side: their widths add, the taller's height stands (the one the geometry changes most).
  fn beside(self, other: PanelDelta) -> PanelDelta {
    PanelDelta {
      width:  self.width + other.width,
      height: if other.height.abs() > self.height.abs() {
        other.height
      } else {
        self.height
      },
    }
  }
}

/// A box's size property `key` (a `Dimension`, as its sizer or after-digest code recorded it), in scaled points.
fn box_dimension(whatsit: &Whatsit, key: &str) -> Option<f64> {
  whatsit
    .get_property(key)
    .and_then(|s| Option::<RegisterValue>::from(s.as_ref()))
    .map(|v| v.value_of() as f64)
}

/// A box's numeric property `key` (`xscale`, `angle`, as its after-digest code recorded it).
fn box_number(whatsit: &Whatsit, key: &str) -> Option<f64> {
  whatsit
    .get_property(key)
    .and_then(|s| s.to_string().trim().parse::<f64>().ok())
}

/// Inside the outermost float's group, at its end (`after_float`; its body is the box list being built, and every
/// macro and register the body defined is in force, review rev78x2 finding 1): every width a construct in it states
/// (`declare_panel_width`, kept as tokens under `KEEP_STATED_LENGTHS`) is evaluated at the registers' values and at
/// the real geometry (`install_panel_line`), and the box records what the real one adds (`PANEL_WIDTH_DELTA`) in the
/// ratio of the two — its width times `real/ours − 1`, so a width the body narrowed (`\setlength\linewidth{…}` before
/// `0.9\linewidth`) or a box a rotation turned scales as it does; a box's content is evaluated against its own width
/// (a child's `\linewidth` is its parent's evaluated width), an `\hspace`'s glue the same way (`PANEL_GLUE_DELTA`;
/// 2311.18531 Fig 1, `0.48 + 3×0.24\textwidth` with `−0.25\textwidth` of `\hspace` under geometry, one row of 4 in
/// the PDF), and a sub-float's line is its stated width at the real geometry (`{subfigure}{0.48\textwidth}`;
/// 2605.03152 S3.F2), or the float's real line where it kept the float's (subfigure.sty's in-line panels). A
/// `\scalebox` or `\resizebox` without a width of its own scales its content's (`xscale`), a rotation turns its axes
/// (`angle`; review rev78x2 finding 2, `q3.tex`). A box's own material is no width it states (1703.09474 Fig 13,
/// 2503.09651 Fig 1: `width=0.23\linewidth,height=0.23\linewidth`). Nothing is recorded, and `false` returned, when a
/// width cannot be evaluated (`Unevaluable`): the float's rows are set as before the rule (`after_float`). Review
/// rev78x findings 1, 3; rev78x3 findings 1-2.
fn note_stated_widths(boxes: &[Digested], geometry: &VecDeque<Stored>) -> bool {
  let geometry: Vec<i64> = geometry
    .iter()
    .map(|v| match v {
      Stored::Dimension(d) => d.value_of(),
      _ => 0,
    })
    .collect();
  let [
    textwidth,
    columnwidth,
    linewidth,
    hsize,
    real_text,
    real_column,
    real_line,
  ] = geometry[..]
  else {
    return true;
  };
  let context = PanelLineContext {
    ours: [textwidth, columnwidth, linewidth, hsize],
    real: [real_text, real_column, real_line, real_line],
  };
  let mut notes = Vec::new();
  for bx in boxes {
    if note_stated_widths_in(bx, context, &mut notes).is_err() {
      return false;
    }
  }
  for (bx, key, value) in notes {
    if let DigestedData::Whatsit(w) = bx.data() {
      w.borrow_mut().set_property(key, value);
    }
  }
  true
}

/// How far (scaled points) a stated length evaluated again at the registers' geometry may be from the value it gave
/// the box (`note_stated_widths_in`): the rounding of a value written out and read back (a graphic's `width=`).
const STATED_LENGTH_TOLERANCE: f64 = 4.0;

/// The properties `note_stated_widths` sets, once every width in the float is evaluated.
type PanelNotes = Vec<(Digested, &'static str, Stored)>;

fn note_stated_widths_in(
  bx: &Digested,
  context: PanelLineContext,
  notes: &mut PanelNotes,
) -> std::result::Result<PanelDelta, Unevaluable> {
  let note = |notes: &mut PanelNotes, key: &'static str, value: f64| {
    notes.push((
      bx.clone(),
      key,
      Stored::Dimension(Dimension::new(value.round() as i64)),
    ));
  };
  let ratio = |ours: i64, real: i64| {
    if ours != 0 {
      real as f64 / ours as f64
    } else {
      1.0
    }
  };
  let w = match bx.data() {
    DigestedData::List(_) => {
      let mut delta = PanelDelta::default();
      for item in bx.unlist() {
        let inner = note_stated_widths_in(&item, context, notes)?;
        delta = delta.beside(inner);
      }
      return Ok(delta);
    },
    DigestedData::Whatsit(w) => w,
    _ => return Ok(PanelDelta::default()),
  };
  // A caption is no panel and sets none (`arrange_panels` ends a row at it): the widths its text states stand as they
  // are, so a box it cannot read again leaves the float's panels evaluated (1307.6961 A2.F8: isotope.sty's
  // `\makebox[\@tempdimb]` of a `\settowidth` in a group that has ended, in the caption of three `\resizebox{11.3cm}`
  // panels on AASTeX 5's 469.755pt page).
  if matches!(
    w.borrow().get_definition().get_cs_name().as_ref(),
    "\\lx@@caption" | "\\@@toccaption"
  ) {
    return Ok(PanelDelta::default());
  }
  let (stated, glue) = {
    let w = w.borrow();
    (stated_size_of(&w)?, w.has_property(PANEL_GLUE_DELTA))
  };
  // A stated length evaluated at the registers' geometry and at the real one. Either one it cannot read (a macro
  // defined in a group that has ended, `{\def\w{0.48\columnwidth}…}`), or one that does not give again the value the
  // box was given (`given`: a macro the float redefined since, an outer `\def\w` an inner one shadowed), is
  // `Unevaluable` (review rev78x3 finding 1, `p1.tex` F1-F2, `p8.tex` F1).
  let evaluate = |tokens: &Tokens,
                  inside: PanelLineContext,
                  is_glue: bool,
                  given: Option<f64>|
   -> std::result::Result<(i64, i64), Unevaluable> {
    let ours = evaluate_stated_length(tokens, inside.ours, is_glue)?.ok_or(Unevaluable)?;
    let real = evaluate_stated_length(tokens, inside.real, is_glue)?.ok_or(Unevaluable)?;
    match given {
      Some(given) if (ours as f64 - given).abs() > STATED_LENGTH_TOLERANCE => Err(Unevaluable),
      _ => Ok((ours, real)),
    }
  };
  // An `\hspace` between panels (`\lx@panel@hskip`): its glue, at the real geometry.
  if glue {
    if let Some(StatedSize { width: Some(tokens), given, .. }) = stated {
      let width = box_dimension(&w.borrow(), "width");
      let (ours, real) = evaluate(&tokens, context, true, given.0.or(width))?;
      note(
        notes,
        PANEL_GLUE_DELTA,
        width.unwrap_or(ours as f64) * (ratio(ours, real) - 1.0),
      );
    }
    return Ok(PanelDelta::default());
  }
  let mut inner = context;
  let mut own: Option<PanelDelta> = None;
  let mut stated_width = false;
  if let Some(stated) = stated {
    let width = match &stated.width {
      Some(tokens) => Some(evaluate(tokens, context, false, stated.given.0)?),
      None => None,
    };
    let height = match &stated.height {
      Some(tokens) => Some(evaluate(tokens, context, false, stated.given.1)?),
      None => None,
    };
    // The box as it was sized at the registers' geometry: the width its argument gave it, or a graphic's box (its
    // rotation included).
    let box_width = stated.value.unwrap_or_else(|| {
      bx.get_width(None)
        .ok()
        .flatten()
        .map(|v| v.value_of() as f64)
        .unwrap_or(0.0)
    });
    // How much the real geometry scales the box across and down.
    let scale = match (width, height) {
      // graphicx's `keepaspectratio`: the larger box within both, bound by the height where the width did not
      // reach its stated value
      (Some((wo, wr)), Some((ho, hr))) if stated.keep_within => {
        let across = if box_width + 65536.0 < wo as f64 {
          ratio(ho, hr).min(wr as f64 / box_width.max(1.0))
        } else {
          ratio(wo, wr).min(ratio(ho, hr))
        };
        Some((across, across))
      },
      (Some((wo, wr)), Some((ho, hr))) => Some((ratio(wo, wr), ratio(ho, hr))),
      (Some((wo, wr)), None) if stated.aspect => Some((ratio(wo, wr), ratio(wo, wr))),
      (Some((wo, wr)), None) => Some((ratio(wo, wr), 1.0)),
      (None, Some((ho, hr))) if stated.aspect => Some((ratio(ho, hr), ratio(ho, hr))),
      _ => None,
    };
    if let Some((across, down)) = scale {
      // (its height and depth, for a rotation round it, where the geometry scales them: a graphic's)
      let box_height = if down == 1.0 {
        0.0
      } else {
        let explicit = {
          let w = w.borrow();
          box_dimension(&w, "height").zip(box_dimension(&w, "depth"))
        };
        match explicit {
          Some((height, depth)) => height + depth,
          None => bx
            .compute_size(SymHashMap::default())
            .map(|(_, height, depth)| (height.value_of() + depth.value_of()) as f64)
            .unwrap_or(0.0),
        }
      };
      let delta = PanelDelta {
        width:  box_width * (across - 1.0),
        height: box_height * (down - 1.0),
      };
      note(notes, PANEL_WIDTH_DELTA, delta.width);
      own = Some(delta);
    }
    // (and a parbox's panels are set against its own width at the real geometry: a `{minipage}{\textwidth}` of
    // sub-figures, 2504.07193 Fig 5; an `\hbox`'s content is set at the enclosing line, `declare_panel_parbox_width`)
    if let Some((ours, real)) = width {
      note(notes, PANEL_LINE, real as f64);
      if stated.parbox {
        inner = PanelLineContext::inside(ours, real);
      }
      stated_width = true;
    }
  }
  // A sub-float (its `after_float` recorded its `\hsize`): its line is its stated width at the real geometry, or the
  // float's real line where it kept the float's (subfigure.sty's in-line panels, whose content is evaluated as the
  // float's: 1902.04580 Figs 15-16, `0.45\textwidth` graphics in `\subfigure`s of an mnras column).
  let (args, body, floatwidth, xscale, yscale, angle) = {
    let w = w.borrow();
    (
      w.get_args().to_vec(),
      w.get_body().ok().flatten(),
      w.get_property("floatwidth")
        .and_then(|s| Option::<RegisterValue>::from(s.as_ref()))
        .map(|r| r.value_of()),
      box_number(&w, "xscale"),
      box_number(&w, "yscale"),
      box_number(&w, "angle"),
    )
  };
  if let Some(hsize) = floatwidth
    && !stated_width
    && hsize == context.ours[3]
  {
    note(notes, PANEL_LINE, context.real[3] as f64);
  }
  let mut content = PanelDelta::default();
  for arg in args.iter().flatten().chain(body.as_ref()) {
    let delta = note_stated_widths_in(arg, inner, notes)?;
    content = content.beside(delta);
  }
  if let Some(own) = own {
    return Ok(own);
  }
  // A transformed box of no stated width of its own: its content's, scaled (`\scalebox`, `\reflectbox`) or turned.
  let mut delta = content;
  if xscale.is_some() || yscale.is_some() {
    let xs = xscale.unwrap_or(1.0).abs();
    let ys = yscale.or(xscale).unwrap_or(1.0).abs();
    delta = PanelDelta {
      width:  content.width * xs,
      height: content.height * ys,
    };
  }
  if let Some(angle) = angle {
    let (s, c) = angle.to_radians().sin_cos();
    delta = PanelDelta {
      width:  c.abs() * delta.width + s.abs() * delta.height,
      height: s.abs() * delta.width + c.abs() * delta.height,
    };
  }
  if (xscale.is_some() || yscale.is_some() || angle.is_some()) && delta.width != 0.0 {
    note(notes, PANEL_WIDTH_DELTA, delta.width);
  }
  Ok(delta)
}

/// A caption's pending state, which `\@@add@caption@counters` (sect09) stores globally for its float's
/// `after_float` to take: `<captype>_tags`, `_id` and `_inlist` (Perl RescueCaptionCounters,
/// latex_constructs.pool.ltxml:3203-3214).
const PENDING_CAPTION_PARTS: [&str; 3] = ["tags", "id", "inlist"];
/// The float's own group keeps the enclosing float's pending caption state here while it is open.
const OUTER_PENDING_CAPTION: &str = "lx@float@outer@caption";

/// A float that begins inside a captioned float — a `[H]` table in a minipage in a table, a
/// tabularray table in a table, a `\captionof` inside a float — sets the enclosing float's pending
/// caption state aside, for the enclosing float's type (`\@captype`) and its own, so its own caption
/// neither overwrites it nor hands it to the wrong float; `after_float` puts it back. Perl keeps the
/// one global slot per type (latex_constructs.pool.ltxml:3193-3201, afterFloat :3384-3392): the inner
/// float takes the outer caption's number and the outer float is left untagged, its `\label` unbound
/// (KNOWN_PERL_ERRORS #395). Always assigned, in the float's group, so a float nested deeper never
/// restores an enclosing float's state. Witness 2605.26653 (a caption-less `[H]` table in a
/// minipage of a captioned table: its `\label` named no table). Guard
/// `perfect_kernel_batch57::nested_float_keeps_the_outer_caption`.
fn set_aside_pending_caption(float_type: &str) {
  let enclosing = has_meaning(&T_CS!("\\@captype"))
    .then(|| do_expand(T_CS!("\\@captype")).ok())
    .flatten()
    .map(|captype| captype.to_string());
  let mut outer = stored_map!();
  for captype in enclosing.iter().map(String::as_str).chain([float_type]) {
    for part in PENDING_CAPTION_PARTS {
      let key = pin(s!("{captype}_{part}"));
      if let Some(pending) = remove_value_sym(key) {
        outer.0.insert(key, pending);
      }
    }
  }
  assign_value(
    OUTER_PENDING_CAPTION,
    Stored::HashStored(outer),
    Some(Scope::Local),
  );
}

/// `after_float` of the float `set_aside_pending_caption` began: the enclosing float's pending
/// caption state is back, for that float's own `after_float`.
fn restore_pending_caption() {
  if let Some(Stored::HashStored(outer)) = remove_value(OUTER_PENDING_CAPTION) {
    for (key, pending) in outer {
      assign_value_sym(key, pending, Some(Scope::Global));
    }
  }
}

/// The pending caption states the boxes open in a float set aside (`begin_caption_box`), innermost last.
const BOX_OUTER_PENDING_CAPTIONS: &str = "lx@box@outer@captions";
/// How many boxes were open when the float began (`begin_float`).
const BOX_DEPTH_AT_FLOAT: &str = "lx@float@box@depth";

fn caption_box_depth() -> usize {
  match lookup_value(BOX_OUTER_PENDING_CAPTIONS) {
    Some(Stored::VecDequeStored(stack)) => stack.len(),
    _ => 0,
  }
}

/// `after_float`: a box begun in the float whose end an error recovery skipped (a minipage closed by `\end{figure}`)
/// gives back the state it set aside, where no later caption has set its own.
fn unwind_caption_boxes() {
  let depth = match lookup_value(BOX_DEPTH_AT_FLOAT) {
    Some(Stored::Number(Number(depth))) => depth.max(0) as usize,
    _ => return,
  };
  while caption_box_depth() > depth {
    let _ = pop_value(BOX_CAPTION_COUNTS);
    if let Ok(Some(Stored::HashStored(outer))) = pop_value(BOX_OUTER_PENDING_CAPTIONS) {
      for (key, pending) in outer {
        if with(key, lookup_value).is_none() {
          assign_value_sym(key, pending, Some(Scope::Global));
        }
      }
    } else {
      break;
    }
  }
}

/// The float type a box (`{minipage}`, `\parbox`) begun in a float captions for: `\@captype`, inside a float.
fn caption_box_type() -> Option<String> {
  (lookup_bool("lx@in@float") && has_meaning(&T_CS!("\\@captype")))
    .then(|| do_expand(T_CS!("\\@captype")).ok())
    .flatten()
    .map(|captype| captype.to_string().trim().to_string())
    .filter(|captype| !captype.is_empty())
}

/// A box (`{minipage}`, `\parbox`) begun in a float sets the float's pending caption state aside, so that a caption
/// in the box is the box's own: the box is a group (latex.ltx `\@iiiminipage`, `\@iiiparbox`), and the
/// `\@currentlabel` that `\caption`'s `\refstepcounter` sets in it — what a `\label` after it names — ends with it.
/// Two captioned minipages side by side in a figure are two numbered figures, each `\ref` its own number; the float
/// took the last caption's counters and every `\label` in it (Perl's `insertBlock` captures the box into an id-less
/// block that `floatToLabel` climbs past, TeX_Box.pool.ltxml:449-519, Document.pm:1098-1127), so each `\ref` read
/// the last number. `end_caption_box` hands the box what its captions left, and gives the float its state back.
pub fn begin_caption_box() -> Result<()> {
  let mut outer = stored_map!();
  let mut counts = stored_map!();
  if let Some(captype) = caption_box_type() {
    for captype in caption_box_types(&captype) {
      for part in PENDING_CAPTION_PARTS {
        let key = pin(s!("{captype}_{part}"));
        if let Some(pending) = remove_value_sym(key) {
          outer.0.insert(key, pending);
        }
      }
      let count = s!("{CAPTION_COUNT_PREFIX}{captype}");
      counts
        .0
        .insert(pin(&count), Stored::Number(Number(lookup_int(&count))));
    }
  }
  push_value(BOX_CAPTION_COUNTS, Stored::HashStored(counts))?;
  push_value(BOX_OUTER_PENDING_CAPTIONS, Stored::HashStored(outer))
}

/// The prefix of the state key counting the captions made of a type (`\@@add@caption@counters`, sect09.rs).
pub(crate) const CAPTION_COUNT_PREFIX: &str = "lx@caption@count@";
/// The caption counts as each open box (`begin_caption_box`) began, innermost last.
const BOX_CAPTION_COUNTS: &str = "lx@box@caption@counts";

/// The caption types a box begun in a float of `captype` may hold its own caption of: the float's, and its sub-type,
/// which subcaption's `\subcaption` captions in (`\caption` under `\@captype` `sub<type>`, subcaption.sty:251): the
/// minipage holding it is the sub-float, as a `{subfigure}` is (subcaption.sty:97-108 makes `{subfigure}` that
/// minipage). Inside a sub-float the type is the sub-type already.
fn caption_box_types(captype: &str) -> Vec<String> {
  if captype.starts_with("sub") {
    vec![captype.to_string()]
  } else {
    vec![captype.to_string(), s!("sub{captype}")]
  }
}

/// The end of a box `begin_caption_box` began: the counters a caption in it stepped become the box's properties
/// `caption_tags`, `caption_id`, `caption_inlist` and `caption_type` (`insert_block_in_paragraph` makes the box that
/// caption's float panel), and the float's pending caption state is back.
pub fn end_caption_box(whatsit: &mut Whatsit) -> Result<()> {
  // A box in math (a minipage in an `array` cell) leaves its caption to the float, as before: the ids made in math
  // are the math's own, and the float's must exist when they are made.
  let in_math = lookup_bool("IN_MATH");
  let counts = match pop_value(BOX_CAPTION_COUNTS)? {
    Some(Stored::HashStored(counts)) => counts,
    _ => stored_map!(),
  };
  // A box with two sub-captions (2008.05644: two `\includegraphics` + `\subcaption` pairs in one minipage) is not one
  // sub-figure: its captions stay its float's, as before.
  let made_one = |captype: &str| {
    let count = s!("{CAPTION_COUNT_PREFIX}{captype}");
    let before = match counts.0.get(&pin(&count)) {
      Some(Stored::Number(Number(n))) => *n,
      _ => 0,
    };
    lookup_int(&count) - before == 1
  };
  if let Some(captype) = caption_box_type().and_then(|captype| {
    let types = caption_box_types(&captype);
    let main = types.first().cloned();
    types.into_iter().find(|captype| {
      lookup_value(&s!("{captype}_id")).is_some()
        && (Some(captype) == main.as_ref() || made_one(captype))
    })
  }) && !in_math
  {
    for part in PENDING_CAPTION_PARTS {
      if let Some(pending) = remove_value(&s!("{captype}_{part}")) {
        whatsit.set_property(&s!("caption_{part}"), pending);
      }
    }
    whatsit.set_property("caption_type", Stored::String(pin(&captype)));
  }
  if let Some(Stored::HashStored(outer)) = pop_value(BOX_OUTER_PENDING_CAPTIONS)? {
    for (key, pending) in outer {
      if !(in_math && with(key, lookup_value).is_some()) {
        assign_value_sym(key, pending, Some(Scope::Global));
      }
    }
  }
  Ok(())
}

/// A sub-float's first opener in its float steps the float's own counter ahead of its caption
/// (Perl beforeFloat `preincrement`): once per float — not when the last float to close was of this
/// sub-type (`LAST_FLOATTYPE`, set by `after_float`) or the float's caption is done.
pub fn preincrement_float_counter(float_type: &str, main_counter: &str) {
  let last_type = lookup_value("LAST_FLOATTYPE")
    .map(|s| s.to_string())
    .unwrap_or_default();
  let done_caption = if_condition(&T_CS!("\\iflx@donecaption"))
    .unwrap_or(None)
    .unwrap_or(false);
  if float_type != last_type
    && !done_caption
    && let Ok(props) = step_float_counter(main_counter)
  {
    let prekey = s!("PREINCREMENTED_{main_counter}");
    assign_value(&prekey, props, Some(Scope::Global));
  }
}
/// Perl: afterFloat (latex_constructs.pool.ltxml L3440-3448)
/// Rescues caption counters into the whatsit properties.
/// A sectioning unit in a float's body (an `ltx:inline-sectional-block`, OXIDIZED_DESIGN_DIVERGENCES #189) steps the
/// section counters the float's id is made within (`idwithin => "section"`), and the float's caption, which comes
/// after it, would make the id in the unit's context: a `\section` in the first figure of section 1 gave the figure
/// `S2.F1`. The float's id is fixed first, in its own context, once per float, and its caption takes it
/// (`\@@add@caption@counters`); the caption still numbers the float, so a figure numbered within sections
/// reads its section as pdflatex prints it. Repro sectioning-frontmatter/sections_inside_floats_keep_number_id_and_ref.
pub fn pin_float_id_before_its_sectioning() -> Result<()> {
  if !lookup_bool("lx@in@float") {
    return Ok(());
  }
  let captype = do_expand(T_CS!("\\@captype"))?.to_string();
  let key = s!("lx@float@pinned@id@{captype}");
  // Nothing to fix when the float already has its id: its caption ran (`\iflx@donecaption`), or a sub-float stepped
  // its counter first (`PREINCREMENTED_<captype>`, which the caption takes, made in the float's own context).
  if captype.is_empty()
    || has_value(&key)
    || has_value(&s!("PREINCREMENTED_{captype}"))
    || if_condition(&T_CS!("\\iflx@donecaption"))
      .unwrap_or(None)
      .unwrap_or(false)
  {
    return Ok(());
  }
  // The id the caption's step would make now: `\the<counter>@ID` with `\@<counter>@ID` at the counter's next value
  // (`step_counter`), in a group, so neither the counter nor the definition moves.
  let counter = lookup_mapping("counter_for_type", &captype)
    .map(|c| c.to_string())
    .unwrap_or_else(|| captype.clone());
  // A float type with no counter has no numbered id to fix.
  if lookup_register(&s!("\\c@{counter}"), Vec::new())
    .ok()
    .flatten()
    .is_none()
  {
    return Ok(());
  }
  let next = counter_value(&counter)?.value_of() + 1;
  bgroup();
  def_macro(
    T_CS!(s!("\\@{counter}@ID")),
    None,
    Tokens::new(Explode!(next)),
    None,
  )?;
  let id = digest_literal(T_CS!(s!("\\the{counter}@ID")));
  egroup()?;
  assign_value(
    &key,
    Stored::String(pin(clean_id(&id?.to_string()))),
    Some(Scope::Global),
  );
  Ok(())
}

pub fn after_float(whatsit: &mut Whatsit) {
  unwind_caption_boxes();
  let captype = digest(T_CS!("\\@captype"))
    .map(|d| d.to_string())
    .unwrap_or_default();
  remove_value(&s!("lx@float@pinned@id@{captype}"));
  // Perl: AssignValue('PREINCREMENTED_' . $type => undef, 'global');
  let prekey = s!("PREINCREMENTED_{captype}");
  remove_value(&prekey);
  // Perl L3389: $whatsit->setProperty(floatwidth => LookupRegister('\hsize'));
  // Capture \hsize as it stands at the END of the float's digestion (its
  // column/textwidth). `arrange_panels` reads this back from the box in the
  // afterClose hook to size the per-row layout — NOT the ambient \hsize at
  // construction time, which may have been restored to an unrelated value.
  let hsize = lookup_register("\\hsize", Vec::new()).ok().flatten();
  if let Some(hsize) = hsize.clone() {
    whatsit.set_property("floatwidth", Stored::from(hsize));
  }
  // The real line its panels are set in rows against (`install_panel_line`), the float's own when it is not inside
  // another, once the widths its body states are evaluated at the real geometry — here, in the float's group, its body
  // the box list being built (`note_stated_widths`; a sub-float's line with them). A width that cannot be evaluated
  // there, or an `\hsize` its binding set (a wrapfigure's), leaves the float the registers' line.
  if let Some(Stored::Dimension(line)) = lookup_value(PANEL_LINE)
    && lookup_value(PANEL_LINE_FRAME) == lookup_value("lx@float@frame")
    && matches!(lookup_value(PANEL_LINE_HSIZE), Some(Stored::Dimension(begun))
      if hsize.as_ref().is_some_and(|h| h.clone().value_of() == begun.value_of()))
  {
    let evaluated = match lookup_value(PANEL_GEOMETRY) {
      Some(Stored::VecDequeStored(geometry)) => {
        let body = with_box_list(<[Digested]>::to_vec);
        note_stated_widths(&body, &geometry)
      },
      _ => true,
    };
    if evaluated {
      whatsit.set_property(PANEL_LINE, Stored::Dimension(line));
    }
  }
  // A float whose panels' widths are the real ones without an evaluation: in a float that set its line
  // (`install_panel_line`, this one or the one it is in) where every register is the real geometry already
  // (`PANEL_GEOMETRY` unset) — a sub-float's own `\hsize`, a wrapfigure's — for the grid's markup
  // (`annotate_panel_grid`).
  if !whatsit.has_property(PANEL_LINE)
    && lookup_value(PANEL_LINE).is_some()
    && lookup_value(PANEL_GEOMETRY).is_none()
  {
    whatsit.set_property(PANEL_REGISTERS_REAL, true);
  }
  // The running-text column its line is a fraction of, where its widths are the real ones (`annotate_panel_grid`).
  if (whatsit.has_property(PANEL_LINE) || whatsit.get_property_bool(PANEL_REGISTERS_REAL))
    && let Some(Stored::Dimension(column)) = lookup_value(PANEL_COLUMN)
  {
    whatsit.set_property(PANEL_COLUMN, Stored::Dimension(column));
  }
  // A float whose rows are not set against the real line (a width it states cannot be evaluated, a float inside
  // another, a binding's `\hsize`) is arranged as before the rule (review rev78x3 finding 2, `p2.tex` F2-F4: four
  // `\figw=3.8cm` panels under `[a4paper,margin=1in]{geometry}`, one row in the PDF). Under geometry (geometry_sty /
  // OXIDIZED_DESIGN #99), a full-column float (its `\hsize` the class-default `\Gm@doctw`, not the live `\textwidth`,
  // which a minipage sets) whose page the geometry widened (`\Gm@tw`) is arranged at that width: its panels are
  // geometry-sized SVG pictures (their `\linewidth` raised inside the picture), so two `0.495\linewidth` boxes that
  // sit side by side in the PDF stay one row. Only the arrangement changes — the float's HTML content keeps the
  // class-default width.
  if !whatsit.has_property(PANEL_LINE)
    && let Some(hsize) = hsize
    && let (Ok(Some(gm_tw)), Ok(Some(doctw))) = (
      lookup_register("\\Gm@tw", Vec::new()),
      lookup_register("\\Gm@doctw", Vec::new()),
    )
  {
    let (h, g, d) = (hsize.value_of(), gm_tw.clone().value_of(), doctw.value_of());
    if h == d && g > h {
      whatsit.set_property(PANEL_FALLBACK_LINE, Stored::from(gm_tw));
    }
  }
  // A sub-float set in an `\hbox` (`note_panel_hbox`).
  if lookup_bool(PANEL_HBOX_BODY) {
    whatsit.set_property(PANEL_HBOX_BODY, true);
  }
  if let Some(Stored::String(group)) = lookup_value(PANEL_HBOX_ROW) {
    whatsit.set_property(PANEL_HBOX_ROW, Stored::String(group));
  }
  // A sub-float's tags — `sub<type>` of a float type (it has a list, `\ext@<type>`) — are its panel number (a
  // sub-caption, `\phantomcaption`, a bare `\subfloat`):
  // `collapse_float` keeps such a panel apart from its float.
  let is_sub_float = captype
    .strip_prefix("sub")
    .is_some_and(|main| has_meaning(&T_CS!(s!("\\ext@{main}"))));
  if is_sub_float && lookup_value(&s!("{captype}_tags")).is_some() {
    whatsit.set_property("panel_number", Stored::Bool(true));
  }
  rescue_caption_counters(&captype, whatsit);
  restore_pending_caption();
  assign_value(
    "LAST_FLOATTYPE",
    Stored::String(pin(&captype)),
    Some(Scope::Global),
  );
}
/// A sub-float panel's stated width is its body's, never its caption's: subfigure.sty's `\@subfloat` measures the body
/// (subfigure.sty:232-233 `\setbox\@tempboxa\hbox{#4}\@tempdima=\wd\@tempboxa`) and sets the sub-caption in a
/// `\parbox` of that width (`\@makesubfigurecaption`), where our caption is a paragraph of the column's `\hsize`
/// (`\lx@@caption PBoxContents`). Measured with its caption, a captioned panel took a row of its own in
/// `arrange_panels`: `\subfigure[]{…}` panels stacked one per row (2006.08949 Figure 2, one row in the PDF;
/// cond-mat9707225, astro-ph0205132, 2012.09475, 1110.0084, hep-ex0004031, 1703.00699). The panel, captioned or not
/// (1007.2355, OXIDIZED_DESIGN_DIVERGENCES #885), is the width of the body's material less its
/// `\lx@@caption`/`\@@toccaption`, its `\lx@label`s (a
/// `\label` makes no box, a `\write`, where the `\lx@label` whatsit was sized by its key's text, as Perl's,
/// Whatsit.pm:253-265 (0 wide since 79c, KNOWN_PERL_ERRORS #1144; left out here all the same): 0907.1044's
/// `\subfigure[…]{\label{figure:mfexampleofc2} \includegraphics…}` was 96pt wider
/// than its graphic) and its spaces, with the natural width of the glue it states (`\hskip`, `\hspace`, `\quad`; an
/// `\hfill`'s is 0; kerns are kept): the material and glue the source states, not how TeX sets the spaces round it
/// (FIDELITY_CHARTER §2 "Float panels"; the panel grid of SYNC_STATUS's grid-rule row counts the stated glue between
/// panels the same way; 1307.3388 Fig 3, guard `perfect_kernel_batch78x::a_sub_float_bodys_stated_glue_counts`). A material of no width is
/// measured again with its glue (1408.2843 Fig 1, a graphic whose file we did not find); only one 0 wide with its glue
/// too leaves the panel its box's width (`panel_width`). Perl measured the whole box
/// (latex_constructs.pool.ltxml:3286; OXIDIZED_DESIGN_DIVERGENCES #876). Witnesses 2511.21020 Figs 7-8, 1607.02761,
/// 2211.12798, 1601.02116 (a label between spaces). Guard `perfect_kernel_batch78t::subfigure_panel_is_as_wide_as_its_body`.
pub fn note_sub_float_body_width(whatsit: &mut Whatsit) {
  let Ok(Some(body)) = whatsit.get_body() else {
    return;
  };
  // The body's boxes, its lists (a group's) opened, less its captions, labels and (unless `with_glue`) glue; the glue it
  // states (`\hspace`, `\hskip`, `\quad`; an `\hfill` is 0 wide), not its spaces, adds its natural width (`stated`), as
  // between panels (1307.3388 Fig 3's `\subfigure[]{⏎\hspace{-0.4cm}⏎\includegraphics…}` pairs, 2+2 in the PDF;
  // 2008.06630 Figs 1, 7).
  fn material(boxes: Vec<Digested>, with_glue: bool, kept: &mut Vec<Digested>, stated: &mut i64) {
    for bx in boxes {
      match bx.data() {
        DigestedData::List(_) => material(bx.unlist(), with_glue, kept, stated),
        _ if !with_glue && is_glue(&bx) => *stated += stated_glue_width(&bx),
        DigestedData::Whatsit(w) => match w.borrow().get_definition().get_cs_name().as_ref() {
          "\\lx@@caption" | "\\@@toccaption" | "\\lx@label" => {},
          _ => kept.push(bx.clone()),
        },
        _ => kept.push(bx),
      }
    }
  }
  let (mut boxes, mut stated) = (Vec::new(), 0);
  material(body.unlist(), false, &mut boxes, &mut stated);
  // Nothing but the caption: the panel hosts a `\captionof` whose material is outside it (caption_sty.rs
  // `captionof_environment@subfigure`), and keeps the box's. An uncaptioned body is its material too: its spaces and
  // `\label`s count nothing (the panel grid rule; 1007.2355 Figs 1, 3: `\subfigure{⏎\includegraphics[width=0.45
  // \textwidth]{…}⏎\label{…}}` pairs, one row in the PDF, measured with the spaces and the label's key text).
  if boxes.is_empty() {
    return;
  }
  let font = match whatsit.get_property("font").as_deref() {
    Some(Stored::Font(font)) => Some(font.clone()),
    _ => lookup_font(),
  };
  let Some(font) = font else {
    return;
  };
  let measure = |boxes: &[Digested]| {
    let mut options = SymHashMap::default();
    options.insert("mode", Stored::String(pin!("restricted_horizontal")));
    font
      .compute_boxes_size(boxes, options)
      .ok()
      .map(|(width, ..)| width)
  };
  let mut width = measure(&boxes);
  // A material of no width — a graphic whose file we did not find (1408.2843 Fig 1: `\subfigure[]{⏎\includegraphics
  // {recomb.eps}}` shipping only recomb.pdf) — is measured again with its glue, the measure before the glue was left
  // out (int61a), so such a panel still shares its row; only a body that is 0 wide with its glue falls back to the box
  // (`panel_width`; 1610.06413, nlin0606018). Review rev78t5 finding 1; guard
  // `perfect_kernel_batch78t::subfigure_panels_of_no_known_width_share_a_row`.
  if width.as_ref().is_none_or(|w| w.value_of() <= 0) {
    let mut with_glue = Vec::new();
    material(body.unlist(), true, &mut with_glue, &mut 0);
    width = measure(&with_glue);
  } else if let Some(material) = width.as_mut()
    && material.value_of() + stated > 0
  {
    // (with the glue it states, where that leaves it a width)
    *material = Dimension::new(material.value_of() + stated);
  }
  if let Some(width) = width {
    whatsit.set_property("panel_body_width", Stored::Dimension(width));
  }
}

/// The natural width (scaled points) of the glue `bx` the source states — `\hskip`, `\hspace`, `\quad` (`isSkip`, its
/// `width`; an `\hfill`'s is 0) — and 0 for a space.
fn stated_glue_width(bx: &Digested) -> i64 {
  fn width(b: &impl BoxOps) -> i64 {
    if b.get_property_bool("isSkip") {
      b.get_property("width")
        .and_then(|w| Option::<RegisterValue>::from(w.as_ref()))
        .map_or(0, |w| w.value_of())
    } else {
      0
    }
  }
  match bx.data() {
    DigestedData::TBox(b) => width(&*b.borrow()),
    DigestedData::Whatsit(w) => width(&*w.borrow()),
    _ => 0,
  }
}

/// Is `bx` glue — a space, `\hskip`, `\hspace`, `\quad`, `\hfill` (`isSkip`, or `isSpace` and not a kern)?
fn is_glue(bx: &Digested) -> bool {
  fn glue(b: &impl BoxOps) -> bool {
    b.get_property_bool("isSkip")
      || (b.get_property_bool("isSpace") && !b.get_property_bool("isKern"))
  }
  match bx.data() {
    DigestedData::TBox(b) => glue(&*b.borrow()),
    DigestedData::Whatsit(w) => glue(&*w.borrow()),
    _ => false,
  }
}

/// Is `qname` a "panel break" element (Perl `figure_panel_break_names`,
/// L3245-3251) — a break/caption/metadata child that flushes the current row
/// rather than being a panel itself?
pub fn is_panel_break_name(qname: SymStr) -> bool {
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

/// A sub-float panel's measured body width (scaled points; `note_sub_float_body_width`), where it has one above 0.
fn panel_body_width_of(document: &Document, node: &Node) -> Option<f64> {
  match document.get_node_box(node).as_ref().map(Digested::data) {
    Some(DigestedData::Whatsit(w)) => {
      match w.borrow().get_property("panel_body_width").as_deref() {
        Some(Stored::Dimension(width)) if width.value_of() > 0 => Some(width.value_of() as f64),
        _ => None,
      }
    },
    _ => None,
  }
}

/// Width (in scaled points) of the box that produced `node`, or 0 if unknown.
/// Perl: `$document->getNodeBox($child)->getWidth->valueOf()`. When the node has
/// no tracked box width (e.g. a minipage/parbox, whose set width rides on the
/// element's `width` ATTRIBUTE rather than a box property in our tree), fall back
/// to that attribute — otherwise the panel reads as zero-width and gets spuriously
/// merged into its neighbour (figure_grids minipage grids).
fn panel_width(document: &Document, node: &Node) -> f64 {
  if let Some(width) = box_panel_width(node) {
    return width;
  }
  // A sub-float panel's body, its caption left out (`note_sub_float_body_width`). A body of no known width (0) falls
  // back to the box, as before the body measure: as 0 it joined every neighbour's row and, at a row's start, left the
  // panel list (1610.06413 Figs 2, 4, hep-ex0703011, nlin0606018).
  if let Some(width) = panel_body_width_of(document, node) {
    return width;
  }
  let node_box = document.get_node_box(node);
  let box_width = node_box
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

/// Width (scaled points) a `\parbox`/minipage is set at: any node carrying its class and `width` —
/// the box itself, or the one child it was folded into (`insert_block`, Perl TeX_Box.pool.ltxml:489-493:
/// `<p class="ltx_minipage" width=…>`).
/// TeX sets such a box as a `\vbox` whose `\hsize` is the declared width (latex.ltx:16249-16253,
/// 16305-16312), so a folded `ltx:p` is a panel of that width, not a paragraph on its own row:
/// Perl's `%standalone_panel_names` and its box-width sizing (latex_constructs.pool.ltxml:3225-3227,
/// 3277, 3286, 3335) stacked text minipages one per row, each at its text's natural width (KPE #310;
/// witnesses 2605.27134 S5.F8, 2605.17146 S4.F4). A source `\par` between rows is a row separator of its own
/// (`note_panel_separator`; `captions-floats/par_breaks_a_panel_row`).
fn box_panel_width(node: &Node) -> Option<f64> {
  let class = node.get_attribute("class")?;
  if !class
    .split_whitespace()
    .any(|c| c == "ltx_minipage" || c == "ltx_parbox")
  {
    return None;
  }
  node
    .get_attribute("width")
    .and_then(|w| Dimension::spec_to_f64(&w).ok())
    .filter(|w| *w > 0.0)
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
/// The float's captured `\hsize` (see `after_float`), read from the box the
/// afterClose hook receives; 345pt when absent. The rows' line is the real one
/// where `after_float` recorded it (`panel_line_of`).
fn float_width_of(whatsit: Option<&Digested>) -> f64 {
  whatsit
    .and_then(|w| w.get_property("floatwidth"))
    .and_then(|s| Option::<RegisterValue>::from(s.as_ref()))
    .map(|r| r.value_of() as f64)
    .filter(|w| *w > 0.0)
    .unwrap_or(345.0 * 65536.0)
}

/// The real line a float's panels are set in rows against (`install_panel_line`), when it recorded one.
fn panel_line_of(whatsit: Option<&Digested>) -> Option<f64> {
  panel_dimension_of(whatsit, PANEL_LINE)
}

/// A float's positive dimension property `key` (scaled points).
fn panel_dimension_of(whatsit: Option<&Digested>, key: &str) -> Option<f64> {
  whatsit
    .and_then(|w| w.get_property(key))
    .and_then(|s| Option::<RegisterValue>::from(s.as_ref()))
    .map(|r| r.value_of() as f64)
    .filter(|w| *w > 0.0)
}

/// What the real geometry adds to a panel's stated width (scaled points, `note_stated_widths`): its own construct's
/// (`{minipage}{0.48\columnwidth}`, `\includegraphics[width=0.48\columnwidth]`), else the sum over its contents'
/// outermost ones (a sub-figure's graphic). 0 where no width it states names a register whose real value differs.
fn panel_width_delta(document: &Document, node: &Node) -> f64 {
  if let Some(delta) = node
    .get_attribute(PANEL_DELTA_ATTRIBUTE)
    .and_then(|d| d.parse::<f64>().ok())
  {
    return delta;
  }
  if let Some(bx) = document.get_node_box(node)
    && let Some(delta) = bx.get_property(PANEL_WIDTH_DELTA)
    && let Some(delta) = Option::<RegisterValue>::from(delta.as_ref())
  {
    return delta.value_of() as f64;
  }
  node
    .get_child_elements()
    .iter()
    .map(|child| panel_width_delta(document, child))
    .sum()
}

/// The internal attribute a minipage or `\parbox` carries what the real geometry adds to its stated width on
/// (`PANEL_WIDTH_DELTA`), which survives its fold into its lone child (`insert_block_as`: a `{minipage}{0.83\linewidth}`
/// holding one `\scalebox`, 2007.15568 Tables 2-3).
const PANEL_DELTA_ATTRIBUTE: &str = "_panelwidthdelta";

/// Put a box's `PANEL_WIDTH_DELTA` (`props`) on the attributes of the element it makes (`PANEL_DELTA_ATTRIBUTE`).
pub(crate) fn note_panel_width_delta(
  props: &SymHashMap<Stored>,
  attr: &mut HashMap<String, String>,
) {
  if let Some(Stored::Dimension(delta)) = props.get(PANEL_WIDTH_DELTA) {
    attr.insert(
      PANEL_DELTA_ATTRIBUTE.to_string(),
      delta.value_of().to_string(),
    );
  }
}

/// The `\hbox` group a sub-float panel was begun in (`PANEL_HBOX_ROW`, `note_panel_hbox`).
fn hbox_row(document: &Document, node: &Node) -> Option<SymStr> {
  match document
    .get_node_box(node)?
    .get_property(PANEL_HBOX_ROW)
    .as_deref()
  {
    Some(Stored::String(group)) => Some(*group),
    _ => None,
  }
}

/// The state value counting the `{center}`-like environments whose bodies are being built (`aligning_environment`): a
/// row separator in their capture is the float's (`note_panel_separator`). In the document's state, so it ends with it.
const ALIGNING_CAPTURES: &str = "lx@aligning@captures";

/// Build an aligning environment's body (`build`), its capture's row separators the float's (`note_panel_separator`).
pub fn in_aligning_capture<T>(build: impl FnOnce() -> T) -> T {
  let depth = aligning_captures();
  assign_value(
    ALIGNING_CAPTURES,
    Stored::Int(depth + 1),
    Some(Scope::Global),
  );
  let result = build();
  assign_value(ALIGNING_CAPTURES, Stored::Int(depth), Some(Scope::Global));
  result
}

fn aligning_captures() -> i64 {
  match lookup_value(ALIGNING_CAPTURES) {
    Some(Stored::Int(n)) => n,
    _ => 0,
  }
}

/// The internal attribute of a row separator the source states between a float's panels (`note_panel_separator`):
/// `par` (a blank line or `\par`), `env` (a `{center}`/`{flushleft}`/`{flushright}` boundary), `vskip`, `cr` (a `\\`
/// in such an environment: `\@centercr`, which ends the paragraph), `break` (a `\linebreak`, which ends a line of it),
/// `glue` (`\hskip`, `\hspace`, `\quad`; its natural width in `PANEL_GLUE`; `glue*` an `\hspace*`'s, which a line break
/// keeps).
/// `arrange_panels` turns a break between two panels into the row's `ltx:break` and drops the rest.
pub(crate) const PANEL_SEPARATOR: &str = "_panelsep";
const PANEL_GLUE: &str = "_panelglue";

/// A row separator `note_panel_separator` recorded: a mark for the float's `arrange_panels`, not content.
pub(crate) fn is_panel_separator(node: &Node) -> bool {
  node.get_attribute(PANEL_SEPARATOR).is_some()
}

/// The internal attribute of a `\newline`'s break (sect01.rs): `\@normalcr`, the justified break in every context
/// (latex.ltx:9256 `\DeclareRobustCommand\newline{\@normalcr\relax}`), where a `\\` under `\centering` or in a
/// `{center}` is `\@centercr`. `arrange_panels` reads it as a line's end, not a paragraph's.
const PANEL_NORMALCR: &str = "_normalcr";

/// Record a row separator the source states at the current insertion point, when that is a float's own panel list
/// (the float, or the capture of a `{center}`-like environment directly in it) — the panel grid rule (user ruling
/// 2026-10-22, FIDELITY_CHARTER §2 "Float panels"): an explicit break always breaks, and the glue the source states
/// between two panels counts towards their row's width (plain spaces and `\hfill` count 0). Our panels are digested
/// in vertical mode, where TeX's are in a paragraph (subfigure.sty's `\@subfloat`, graphicx's `\leavevmode`), so a
/// `\par` or the glue between them leaves nothing in the tree to set the rows by. A `\par` that leaving horizontal
/// mode inserts is not the source's (2605.00347: one per inter-panel `\hfill` broke a 4-panel row into four).
/// Witnesses 1110.0084 Figs 12-16, 27, 29 (a blank line between sub-figures), 0706.4410 Figs 3, 5 (`\hspace{1in}`
/// between two 0.45\textwidth panels: two rows in the PDF). OXIDIZED_DESIGN_DIVERGENCES #885.
pub fn note_panel_separator(document: &mut Document, kind: &str, glue: Option<i64>) -> Result<()> {
  let Some(context) = document.get_element() else {
    return Ok(());
  };
  // The float's own list, or the capture of a `{center}`-like environment directly in it (its lines are the float's
  // rows; `aligning_environment`), never a box's: a separator in a minipage's capture changed how the capture settles.
  let qname = document::get_node_qname(&context);
  let at_float = with(qname, |q| PANEL_FLOATS.contains(&q))
    || (aligning_captures() > 0
      && document::is_capture_block(&context)
      && context.get_parent().is_some_and(|parent| {
        with(document::get_node_qname(&parent), |q| {
          PANEL_FLOATS.contains(&q)
        })
      }));
  if !at_float {
    return Ok(());
  }
  let mut attributes = map!(PANEL_SEPARATOR => kind.to_string());
  if let Some(width) = glue {
    attributes.insert(PANEL_GLUE.to_string(), width.to_string());
  }
  document.insert_element("ltx:break", Vec::new(), Some(attributes))?;
  Ok(())
}

/// A paragraph of nothing but float-boxes (57g; user rulings 2026-09-27): boxes that became floats
/// (`absorb_figure_material`, base_utilities.rs) sit in the paragraph they start as inline blocks;
/// a paragraph holding only them — besides `\\` breaks and whitespace — is a line of nothing but
/// those boxes, so it gives way to them ([`settle_float_paragraphs`], once the document is built).
/// One box becomes its float, the box's class and sizes folded onto it as Perl's single-node fold
/// does (`<figure
/// class="ltx_minipage">`); several become a row: an uncaptioned `ltx:figure` holding them as
/// panels, with the author's breaks, laid out by `arrange_panels` (the row's `after_close`) — the
/// markup LaTeXML gives a figure float holding captioned minipages. A paragraph holding anything
/// else (text, a box that is not a float, arXiv 2605.18920's algorithm box beside a table box)
/// keeps its boxes inline, side by side. Only a paragraph of one `ltx:p`, in a context that holds
/// floats and is not a float's own panel list.
pub(crate) fn settle_float_paragraph(document: &mut Document, para: &Node) -> Result<()> {
  let Some(parent) = para.get_parent() else {
    return Ok(());
  };
  if parent.get_type() != Some(NodeType::ElementNode) {
    return Ok(());
  }
  // A streaming segment's parse wrapper stands for the segment's real parent.
  let parent_qname = match document::get_node_qname(&parent) {
    wrapper if with(wrapper, |q| q == "ltx:_lxfragment") => match document.fragment_parent_qname {
      Some(real) => real,
      None => return Ok(()),
    },
    qname => qname,
  };
  if with(parent_qname, |q| PANEL_FLOATS.contains(&q)) {
    return Ok(());
  }
  let whitespace = |n: &Node| n.get_content().trim().is_empty();
  let mut paragraph = None;
  for child in para.get_child_nodes() {
    match child.get_type() {
      Some(NodeType::TextNode) if whitespace(&child) => {},
      Some(NodeType::ElementNode)
        if paragraph.is_none() && with(document::get_node_qname(&child), |q| q == "ltx:p") =>
      {
        paragraph = Some(child)
      },
      Some(NodeType::ElementNode) | Some(NodeType::TextNode) => return Ok(()),
      _ => {},
    }
  }
  let Some(paragraph) = paragraph else {
    return Ok(());
  };
  // The line's items: each box with its one float, and the breaks between them.
  let mut items: Vec<(Node, Option<Node>)> = Vec::new();
  for child in paragraph.get_child_nodes() {
    match child.get_type() {
      Some(NodeType::TextNode) if whitespace(&child) => {},
      Some(NodeType::ElementNode) => {
        let qname = document::get_node_qname(&child);
        if with(qname, |q| q == "ltx:break") {
          items.push((child, None));
          continue;
        }
        if !with(qname, |q| {
          q == "ltx:inline-logical-block" || q == "ltx:inline-block"
        }) {
          return Ok(());
        }
        let mut float = None;
        for inner in child.get_child_nodes() {
          match inner.get_type() {
            Some(NodeType::TextNode) if whitespace(&inner) => {},
            Some(NodeType::ElementNode)
              if float.is_none()
                && with(document::get_node_qname(&inner), |q| {
                  PANEL_FLOATS.contains(&q)
                }) =>
            {
              float = Some(inner)
            },
            Some(NodeType::ElementNode) | Some(NodeType::TextNode) => return Ok(()),
            _ => {},
          }
        }
        let Some(float) = float else {
          return Ok(());
        };
        if !document::can_contain_qsym(parent_qname, document::get_node_qname(&float)) {
          return Ok(());
        }
        items.push((child, Some(float)));
      },
      Some(NodeType::TextNode) => return Ok(()),
      _ => {},
    }
  }
  let boxes = items.iter().filter(|(_, float)| float.is_some()).count();
  if boxes == 0 || (boxes == 1 && items.len() > 1) {
    return Ok(());
  }
  let align = paragraph.get_attribute("align");
  let class = paragraph.get_attribute("class");
  let mut placed = Vec::new();
  let mut anchor = para.clone();
  for (wrapper, float) in items {
    let mut node = match float {
      Some(mut float) => {
        // The box's own annotation onto its float: `class` merges, the rest where absent.
        for (name, value) in wrapper.get_attributes() {
          if name == "class" {
            document.add_class(&mut float, &value)?;
          } else if name != "xml:id"
            && name != "id"
            && float.get_attribute(&name).is_none()
            && document::can_node_have_attribute(&float, &name)
          {
            document.set_attribute(&mut float, &name, &value)?;
          }
        }
        float
      },
      None => wrapper,
    };
    node.unlink();
    anchor.add_next_sibling(&mut node)?;
    anchor = node.clone();
    placed.push(node);
  }
  // A row takes over the paragraph's identity — its id and id counters, minted during
  // construction — so it is numbered as in the whole document when it settles in a streaming
  // segment, and no paragraph number is skipped (57g review).
  let mut identity: Vec<(String, String)> = para
    .get_attributes()
    .into_iter()
    .filter(|(name, _)| name.starts_with("_ID_counter"))
    .collect();
  identity.sort();
  if let Some(id) = para.get_attribute_ns("id", XML_NS) {
    identity.insert(0, ("xml:id".to_string(), id));
  }
  // The paragraph's content lives on in the floats: keep its box (`remove_node_keeping_box`).
  document.remove_node_keeping_box(para.clone());
  let mut settled = if boxes == 1 {
    placed.into_iter().next()
  } else {
    document.wrap_nodes_as("ltx:figure", placed, &identity)?
  };
  // The line's alignment (`\\centering`, `{center}`, `\\raggedleft`) goes with it.
  if let Some(settled) = settled.as_mut() {
    if let Some(align) = align
      && settled.get_attribute("align").is_none()
    {
      document.set_attribute(settled, "align", &align)?;
    }
    if let Some(class) = class {
      document.add_class(settled, &class)?;
    }
  }
  Ok(())
}

/// Settle every paragraph of nothing but float-boxes (`settle_float_paragraph`), innermost
/// first, once construction is over: every alignment has been stamped and no construction step
/// holds a handle into a paragraph any more (settling at a paragraph's close freed nodes a
/// `{center}` still held, dropping its alignment — 57g review).
pub fn settle_float_paragraphs(document: &mut Document, root: &Node) -> Result<()> {
  // (and the row separators no float's `arrange_panels` took: those of a box's own content, `note_panel_separator`)
  fn collect(node: &Node, paras: &mut Vec<Node>, separators: &mut Vec<Node>) {
    for child in node.get_child_elements() {
      collect(&child, paras, separators);
      let qname = document::get_node_qname(&child);
      if with(qname, |q| q == "ltx:para") {
        paras.push(child);
      } else if with(qname, |q| q == "ltx:break") && child.get_attribute(PANEL_SEPARATOR).is_some()
      {
        separators.push(child);
      }
    }
  }
  let (mut paras, mut separators) = (Vec::new(), Vec::new());
  collect(root, &mut paras, &mut separators);
  for separator in separators {
    document.remove_node(separator);
  }
  for para in paras {
    settle_float_paragraph(document, &para)?;
  }
  Ok(())
}

/// Port of Perl's `arrange_panels_and_breaks`
/// (latex_constructs.pool.ltxml L3229-3349): partition a figure/table/float's
/// children into rows, inserting `<ltx:break>` where the accumulated panel WIDTH
/// would overflow the float width. Author-deposited breaks and captions/metadata
/// flush the current row. Panels are marked with `ltx_figure_panel` only when the
/// figure has more than one. Beyond Perl, the panel grid rule (user rulings
/// 2026-10-22, OXIDIZED_DESIGN_DIVERGENCES #885): the row separators the source
/// states (`note_panel_separator`: a `\par`, a `{center}` boundary, `\vskip`,
/// glue) break the row or count towards it, and the widths and the line are the
/// class's real geometry's (`install_panel_line`, `panel_width_delta`).
fn arrange_panels(
  document: &mut Document,
  node: &mut Node,
  whatsit: Option<&Digested>,
) -> Result<PanelLayout> {
  // The rows are set against the real line (`install_panel_line`): a panel's width is its stated width at the real
  // geometry, the width its box has at the registers' values plus what the real values add (`panel_width_delta`).
  // The float's `\hsize` (Perl's `floatwidth`) is what a panel with no width of its own fills.
  // A row with no real line of its own (a paragraph of floats settled into a row, `settle_float_paragraph`) is set at
  // the registers' widths, as before the rule (2202.03884 S4.SS1.fig1).
  // A float with no real line is arranged as before the rule: at its `\hsize`, or the geometry's text width
  // (`PANEL_FALLBACK_LINE`, `after_float`), which a panel with no width of its own then fills.
  let real_line = panel_line_of(whatsit);
  let float_width = real_line
    .or_else(|| panel_dimension_of(whatsit, PANEL_FALLBACK_LINE))
    .unwrap_or_else(|| float_width_of(whatsit));
  let hsize = if real_line.is_some() {
    float_width_of(whatsit)
  } else {
    float_width
  };
  // Perl L3233: 0.03125x min width => at most 32 panels per row.
  let min_panel_width = 0.03125 * float_width;
  let hbox_body = whatsit.is_some_and(|w| w.get_property_bool(PANEL_HBOX_BODY));

  let note_qname = pin!("ltx:note");
  let caption_qname = pin!("ltx:caption");
  let block_qname = pin!("ltx:block");

  let mut current_width: f64 = 0.0;
  let mut all_panels: Vec<Node> = Vec::new();
  // Bookkeeping triples for the current row: (node, qname, width).
  let mut row: Vec<(Node, SymStr, f64)> = Vec::new();
  // The source's row separators (`note_panel_separator`): the glue stated since the row's last panel, and a row break
  // the source stated after it, kept only when another panel follows before the next break or caption.
  let mut pending_glue: f64 = 0.0;
  let mut pending_break: Option<Node> = None;
  // A row that begins a paragraph (the float's first, one after a `\par`, a `{center}` boundary, a `\vskip`, a
  // caption, or a centred `\\`) keeps the glue stated before its first panel, as TeX keeps glue at a paragraph's start;
  // a row after a justified `\\` or a `\linebreak` keeps only an `\hspace*`'s (`glue*`: latex.ltx's `\vrule
  // width\z@\nobreak\hskip`, which a line break does not discard) and a computed break none (2212.03198 Figs 6, 10-12,
  // 15: `\hspace*{-0.5cm}` before three sub-figures, one row in the PDF; `\\\hspace*{-1.1cm}` before a second row of
  // four). Under `\centering`, `\raggedright`, `\raggedleft` and in a `{center}`, `{flushleft}` or `{flushright}`, `\\`
  // is `\@centercr` (latex.ltx `\let\\\@centercr`), whose `\par` ends the paragraph, so an `\hspace` after it starts
  // the next one and is kept (review rev78x3 finding 3, `p7.tex` F1: `\\\hspace{-0.05\columnwidth}` before three
  // 0.34\columnwidth panels, one line in the PDF); so is every `\\` in a class's centred floats (`PANEL_FLOATS_CENTRED`).
  // A `\newline` is `\@normalcr` in every context (latex.ltx:9256), a justified break, whatever alignment class its
  // break took (`PANEL_NORMALCR`; review rev78x4 finding 1: n3 F4 `\centering`, n7 F1 `{center}`,
  // `\newline\hspace{0.15\columnwidth}` before three 0.3\columnwidth panels, two lines in the PDF).
  let floats_centred = whatsit.is_some() && lookup_bool(PANEL_FLOATS_CENTRED);
  let centred_break = |node: &Node| {
    node.get_attribute(PANEL_NORMALCR).is_none()
      && (floats_centred
        || node.get_attribute("class").is_some_and(|class| {
          class
            .split_whitespace()
            .any(|c| matches!(c, "ltx_centering" | "ltx_align_left" | "ltx_align_right"))
        }))
  };
  let mut paragraph_start = true;
  let mut leading_glue: f64 = 0.0;
  // The width each panel was set at, and whether the source states it (`panel_width_is_stated`), for the grid's
  // markup (`annotate_panel_grid`); a merged block's, its parts'.
  let mut widths: Vec<(Node, f64, bool)> = Vec::new();
  let stated_of = |widths: &[(Node, f64, bool)], node: &Node| {
    widths
      .iter()
      .find(|(n, ..)| n == node)
      .is_some_and(|(.., stated)| *stated)
  };
  let record =
    |widths: &mut Vec<(Node, f64, bool)>, node: &Node, width: f64, stated: bool| match widths
      .iter_mut()
      .find(|(n, ..)| n == node)
    {
      Some(entry) => {
        entry.1 = width;
        entry.2 = stated;
      },
      None => widths.push((node.clone(), width, stated)),
    };

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

    // A separator the source states between panels: glue counts towards the row's width, a break ends the row
    // before the next panel; one before the row's first panel or after its last is dropped.
    if let Some(kind) = child.get_attribute(PANEL_SEPARATOR) {
      if kind == "glue" || kind == "glue*" {
        let width = child
          .get_attribute(PANEL_GLUE)
          .and_then(|w| w.parse::<f64>().ok())
          .unwrap_or(0.0);
        if !row.is_empty() && pending_break.is_none() {
          pending_glue += width;
        } else if paragraph_start || kind == "glue*" {
          leading_glue += width;
        }
        document.remove_node(child);
      } else if row.is_empty() || pending_break.is_some() {
        if kind != "break" && pending_break.is_none() {
          paragraph_start = true;
          leading_glue = 0.0;
        }
        document.remove_node(child);
      } else {
        // The row's break, as a computed one: an alignment's class stamped on the separator (`\centering`) is not its.
        // (a `\linebreak` begins no paragraph, a centred `\\`, `cr`, does)
        paragraph_start = kind != "break";
        child.remove_attribute(PANEL_SEPARATOR).ok();
        child.set_attribute("class", "ltx_break").ok();
        pending_break = Some(child);
      }
      continue;
    }

    if is_panel_break_name(child_name) {
      // Perl L3269-3275: a break/caption/meta flushes the current row.
      if let Some(stated) = pending_break.take() {
        document.remove_node(stated);
      }
      pending_glue = 0.0;
      leading_glue = 0.0;
      // (a caption is a paragraph of its own; a justified `\\` ends a line of the paragraph, a centred one the
      // paragraph)
      paragraph_start = child_name != pin!("ltx:break") || centred_break(&child);
      current_width = 0.0;
      row.clear();
      continue;
    }
    // A sub-float holding nothing but its number — `\subfloat{\label{…}}`, a label's anchor (2208.08007's
    // `\labelphantom`) — makes no box: it is no panel and takes no place in a row (a `\label` is a `\write`).
    if holds_only_its_number(&child) {
      continue;
    }
    // A row break the source stated before this panel: the row ends there.
    if pending_break.take().is_some() {
      pending_glue = 0.0;
      current_width = 0.0;
      row.clear();
    }

    // Perl L3277-3284: a standalone block on its own row — break first.
    if is_standalone_panel_name(child_name) && box_panel_width(&child).is_none() && !row.is_empty()
    {
      insert_break_before(document, &mut child)?;
      current_width = 0.0;
      row.clear();
    }

    let mut child_width = panel_width(document, &child);
    // #6903: a subcaptionbox / subfig \subfloat panel is sized to the full float
    // \hsize (it has no explicit `{width}`), so it would take its own row. When
    // it wraps a single narrower graphic, size it to that graphic so sibling
    // panels share a row — an explicit-width `{subfigure}{W}` already does.
    let mut sole_graphic = false;
    if let Some(inner) = sole_graphic_width(document, &child) {
      if child_width >= hsize && inner < child_width {
        child_width = inner;
        sole_graphic = true;
      } else {
        // (a sub-float measured by its body, `panel_body_width` — subfig's and subfigure.sty's — that holds one graphic
        // states its width as the one the full-`\hsize` box did above, its body's in the row: 1903.10543 S4.F7,
        // `\subfloat[…]{\begin{tabular}{c}\includegraphics[width=5.6cm]{…}\end{tabular}}`, 79f)
        sole_graphic = panel_body_width_of(document, &child).is_some();
      }
    }
    let child_stated = sole_graphic || panel_width_is_stated(document, &child, child_width, hsize);
    if real_line.is_some() {
      child_width = (child_width + panel_width_delta(document, &child)).max(0.0);
    }
    // The glue the source states between this panel and the row's previous one.
    let glue = std::mem::take(&mut pending_glue);

    // A row a binding fixed (AASTeX's `\gridline`, a `\hbox to\hsize` of panels: `_gridrow`) stays one row however
    // wide its panels add up (2609.09897's two 0.53\textwidth panels and a third, beside negative `\hspace`s).
    // So does a row TeX sets in one `\hbox` (`note_panel_hbox`): a sub-float's body, the sub-floats of one box.
    let same_fixed_row = hbox_body
      || row.last().is_some_and(|(prev, ..)| {
        prev
          .get_attribute("_gridrow")
          .is_some_and(|r| child.get_attribute("_gridrow").as_ref() == Some(&r))
          || hbox_row(document, prev).is_some_and(|r| hbox_row(document, &child) == Some(r))
      });
    if !row.is_empty() && !same_fixed_row && (current_width + glue + child_width > float_width) {
      // Perl L3287-3295: row overflow — break before this child, start a new row.
      insert_break_before(document, &mut child)?;
      document.add_class(&mut child, "ltx_figure_panel")?;
      all_panels.push(child.clone());
      record(&mut widths, &child, child_width, child_stated);
      row = vec![(child, child_name, child_width)];
      current_width = child_width;
      paragraph_start = false;
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
      // Perl reuses any `ltx:block` as the container, a minipage's or `\parbox`'s own
      // included: the neighbouring panel went inside that box, which TeX set at its width
      // with only its own material — a label minipage beside a picture minipage came out
      // under the picture (witness 2605.00042 S5.F4; KNOWN_PERL_ERRORS #331). Such a box
      // stays closed and the merge wraps the two.
      let reuse_prev = prev_name == block_qname && box_panel_width(&prev_node).is_none();
      let reuse_child = child_name == block_qname && box_panel_width(&child).is_none();
      let merge_is_valid = if reuse_prev {
        model::can_contain_sym(block_qname, child_name)
      } else if reuse_child {
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
        if reuse_prev {
          child.unlink_node();
          prev_node.add_child(&mut child).ok();
          let stated = stated_of(&widths, &prev_node) && child_stated;
          record(&mut widths, &prev_node, merged_width, stated);
          row.push((prev_node, prev_name, merged_width));
        } else if reuse_child {
          // The previous panel goes FIRST in the block, keeping source order.
          // Perl `$child->appendChild($prev_node)` (L3318) puts it last: an
          // `\includegraphics` before a `{minipage}` came out after the
          // minipage's text (KNOWN_PERL_ERRORS #274; guard
          // `node_box_append::a_panel_merged_into_a_block_keeps_its_place`).
          prev_node.unlink_node();
          match child.get_first_child() {
            Some(mut first) => {
              first.add_prev_sibling(&mut prev_node).ok();
            },
            None => {
              child.add_child(&mut prev_node).ok();
            },
          }
          all_panels.push(child.clone());
          let stated = stated_of(&widths, &prev_node) && child_stated;
          record(&mut widths, &child, merged_width, stated);
          row.push((child, child_name, merged_width));
        } else {
          // The wrapper is the panel now: a row-start box already marked one (overflow, Perl
          // L3292) keeps its own classes only, and leaves the panel list (Perl L3324 pops the last
          // entry, which a zero-width row start never was).
          if all_panels.last() == Some(&prev_node) {
            all_panels.pop();
          }
          document.remove_class(&mut prev_node, "ltx_figure_panel");
          let stated = stated_of(&widths, &prev_node) && child_stated;
          if let Some(block) = document.wrap_nodes("ltx:block", vec![prev_node, child])? {
            all_panels.push(block.clone());
            record(&mut widths, &block, merged_width, stated);
            row.push((block, block_qname, merged_width));
          }
        }
      } else {
        // Perl L3327-3330: keep the previous panel, append this one as a sibling.
        let prev_stated = stated_of(&widths, &prev_node);
        record(&mut widths, &prev_node, prev_width, prev_stated);
        record(&mut widths, &child, child_width, child_stated);
        row.push((prev_node, prev_name, prev_width));
        all_panels.push(child.clone());
        row.push((child, child_name, child_width));
      }
      current_width += glue + child_width;
    } else {
      // Perl L3331-3333: no previous panel in the row — just add this child.
      paragraph_start = false;
      record(&mut widths, &child, child_width, child_stated);
      if child_width > 0.0 {
        all_panels.push(child.clone());
      }
      if is_standalone_panel_name(child_name) && box_panel_width(&child).is_none() {
        // Perl L3334-3342: a standalone panel as the sole row content flushes the
        // row and forces a break before the next sibling (unless that sibling is
        // itself a break/caption/meta), so subsequent content starts a new row.
        if let Some(mut trailer) = child.get_next_sibling() {
          // (a row separator the source states is passed over: what follows it decides)
          let mut next = Some(trailer.clone());
          while let Some(node) = next
            .as_ref()
            .filter(|n| n.get_attribute(PANEL_SEPARATOR).is_some())
          {
            next = node.get_next_sibling();
          }
          let trailer_name = next
            .as_ref()
            .map_or(pin!("ltx:caption"), document::get_node_qname);
          if !is_panel_break_name(trailer_name) {
            insert_break_before(document, &mut trailer)?;
          }
        }
        // The standalone panel is a row of its own, so the next row starts
        // empty. Perl resets `$current_width` here and then adds the panel's
        // width to it (L3341-3343), so the next row starts that full: with
        // W12's node boxes the `{\small\makebox[0.49\textwidth]{…}\hfill
        // \makebox[0.49\textwidth]{…}}` caption line of a subfigure grid is
        // its whole width, and the four `0.24\textwidth` panels after it split
        // 1 | 3, where pdflatex sets them in one row (KNOWN_PERL_ERRORS #274;
        // witness 2605.02317; guard `node_box_append::a_standalone_panel_row_starts_the_next_row_empty`).
        current_width = 0.0;
      } else {
        row.push((child, child_name, child_width));
        // Perl L3343: the row was empty (current_width is 0), so this seeds
        // the accumulator with the child's width (and the glue its paragraph's start keeps).
        current_width += child_width + std::mem::take(&mut leading_glue);
      }
    }
  }

  if let Some(stated) = pending_break {
    document.remove_node(stated);
  }
  // Perl L3346-3348: only mark panels when the figure is complex (>1 panel).
  if all_panels.len() > 1 {
    for panel in &mut all_panels {
      document.add_class(panel, "ltx_figure_panel")?;
    }
  }
  // The widths are the real ones where the float's line is (`install_panel_line`, `note_stated_widths`), or where the
  // registers are the real geometry already (`PANEL_REGISTERS_REAL`); a float arranged at the registers' line or the
  // geometry's fallback (a width it could not evaluate) states none for its markup.
  let evaluated =
    real_line.is_some() || whatsit.is_some_and(|w| w.get_property_bool(PANEL_REGISTERS_REAL));
  Ok(PanelLayout {
    widths,
    line: float_width,
    column: panel_dimension_of(whatsit, PANEL_COLUMN).filter(|_| evaluated),
    evaluated,
  })
}
/// What `arrange_panels` decided a float's panels are set at, for its grid's markup (`annotate_panel_grid`): each
/// panel's width (scaled points; a merged block's its parts'), the line its rows were set against, the real
/// running-text column at the float (`PANEL_COLUMN`, where the widths are the real geometry's), and whether those are
/// the real geometry's (`evaluated`: the float's line is the class's real one, or the registers are it).
pub(crate) struct PanelLayout {
  widths:    Vec<(Node, f64, bool)>,
  line:      f64,
  column:    Option<f64>,
  evaluated: bool,
}

/// A sub-float holding nothing but its number (`ltx:tags`): `\subfloat{\label{…}}` (2208.08007's
/// `\newcommand{\labelphantom}[1]{\subfloat{\label{#1}}}`, five or six before the one graphic that draws every
/// panel). TeX sets it as an empty box (a `\label` is a `\write`), so it is no panel of its float.
fn holds_only_its_number(node: &Node) -> bool {
  with(document::get_node_qname(node), |q| {
    PANEL_FLOATS.contains(&q)
  }) && node
    .get_child_nodes()
    .iter()
    .all(|child| match child.get_type() {
      Some(NodeType::ElementNode) => document::get_node_qname(child) == pin!("ltx:tags"),
      Some(NodeType::TextNode) => child.get_content().trim().is_empty(),
      _ => true,
    })
}

/// The panel grid's attributes on a float (`annotate_panel_grid`).
const PANEL_GRID_ATTRIBUTES: [&str; 4] = ["panelrows", "panelcols", "panelline", "panelfill"];

/// How far apart (a fraction of the line) two panel widths may be and still be one column's (`annotate_panel_grid`).
const PANEL_WIDTH_TOLERANCE: f64 = 0.01;

/// The panel grid's markup (user rulings 2026-10-22, FIDELITY_CHARTER §2 "Float panels"; grid design study option
/// (a)): the float `arrange_panels` set in rows says them — `panelrows`, `panelcols` (the most columns a row takes),
/// and `ltx_panels_matrix` or `ltx_panels_gallery` — and each panel its width as a fraction of the row's line
/// (`panelwidth`, 3 decimals) where that width is one the source states (`states_its_width`: a graphic's, a rule's,
/// a box's of a declared width, or theirs side by side; not a measure of our own setting of text, a tabular, a list)
/// taken at the real geometry (none where the float could not take its widths there, `PanelLayout`: never a guess),
/// and, in a matrix, the columns it spans when 2 or more (`panelspan`); where every panel states its width, the float
/// its line as a fraction of the PDF's running-text column at the float (`panelline`: the class's real column in two
/// columns, its text width in one; 1 for a `figure`, about 2.07 for a two-column class's `figure*`, under 1 for a
/// wrapfigure) and its widest row's sum of `panelwidth` (`panelfill`, fractions of its own line) — user ruling
/// 2026-10-23: a panel renders at `panelwidth` × `panelline` of the theme's column, divided by max(1, `panelfill` ×
/// `panelline`), so a `figure*` fills the column and a narrow float stays narrow; the converter knows only the PDF's
/// column and the float's line, never a theme constant. A matrix only on strong signals: an
/// author-built `tabular` of panels (`tabular_panel_grid`, its cells' `colspan` and `rowspan` kept), or 2 or more rows
/// of the same columns — as many panels, each as wide as the other row's (within `PANEL_WIDTH_TOLERANCE`), another
/// row's panel as wide as the columns it spans — whose panels' own captions are none or exactly one whole row's, its
/// columns' labels (captioned panels elsewhere are a gallery's: independent, their order not their place;
/// `matrix_spans`). A float of one panel states none of it. A
/// sub-float group states its own (its panels' widths a fraction of its line), as it is a flex figure of its own in
/// HTML; as a panel of its float it states its width there too. Perl emits none of this
/// (latex_constructs.pool.ltxml:3229-3349; subcaption.sty.ltxml:60-66 stores `width`, never emits it).
/// OXIDIZED_DESIGN_DIVERGENCES #887.
pub(crate) fn annotate_panel_grid(
  document: &mut Document,
  node: &mut Node,
  layout: &PanelLayout,
) -> Result<()> {
  // A lone inner float's grid, the float's since `collapse_float` made the inner's panels its own.
  if node.get_attribute("panelrows").is_some() {
    return Ok(());
  }
  // The rows, as the HTML sets them: runs of panels between breaks and captions.
  let mut rows: Vec<Vec<Node>> = vec![Vec::new()];
  for child in node.get_child_elements() {
    if is_panel_break_name(document::get_node_qname(&child)) {
      rows.push(Vec::new());
    } else if child
      .get_attribute("class")
      .is_some_and(|class| class.split_whitespace().any(|c| c == "ltx_figure_panel"))
      && let Some(row) = rows.last_mut()
    {
      row.push(child);
    }
  }
  rows.retain(|row| !row.is_empty());
  if rows.iter().map(Vec::len).sum::<usize>() < 2 {
    if let Some((rows, cols)) = tabular_panel_grid(document, node) {
      document.add_class(node, "ltx_panels_matrix")?;
      document.set_attribute(node, "panelrows", &rows.to_string())?;
      document.set_attribute(node, "panelcols", &cols.to_string())?;
    }
    return Ok(());
  }
  let line = layout.line;
  let width_of = |panel: &Node| {
    layout
      .widths
      .iter()
      .find(|(n, ..)| n == panel)
      .filter(|(_, w, stated)| *stated && layout.evaluated && line > 0.0 && *w > 0.0)
      .map(|(_, w, _)| *w * graphic_rounding(document, panel) / line)
  };
  let widths: Vec<Vec<Option<f64>>> = rows
    .iter()
    .map(|row| row.iter().map(width_of).collect())
    .collect();
  let spans = matrix_spans(&rows, &widths);
  let cols = match &spans {
    Some(spans) => spans
      .iter()
      .map(|row| row.iter().sum::<usize>())
      .max()
      .unwrap_or(0),
    None => rows.iter().map(Vec::len).max().unwrap_or(0),
  };
  for (r, row) in rows.iter_mut().enumerate() {
    for (c, panel) in row.iter_mut().enumerate() {
      if let Some(width) = widths[r][c] {
        document.set_attribute(panel, "panelwidth", &panel_fraction(width))?;
      }
      if let Some(span) = spans.as_ref().map(|s| s[r][c]).filter(|span| *span >= 2) {
        document.set_attribute(panel, "panelspan", &span.to_string())?;
      }
    }
  }
  let class = if spans.is_some() {
    "ltx_panels_matrix"
  } else {
    "ltx_panels_gallery"
  };
  document.add_class(node, class)?;
  document.set_attribute(node, "panelrows", &rows.len().to_string())?;
  document.set_attribute(node, "panelcols", &cols.to_string())?;
  // Where every panel states its width: the line as a fraction of the running-text column (`panelline`), and the
  // widest row's sum of the widths its panels state (`panelfill`, fractions of the float's own line).
  if let Some(column) = layout.column.filter(|c| *c > 0.0)
    && widths.iter().flatten().all(Option::is_some)
  {
    let stated = |width: f64| (width * 1000.0).round() / 1000.0;
    let fill = widths
      .iter()
      .map(|row| row.iter().flatten().map(|w| stated(*w)).sum::<f64>())
      .fold(0.0, f64::max);
    document.set_attribute(node, "panelline", &panel_fraction(line / column))?;
    document.set_attribute(node, "panelfill", &panel_fraction(fill))?;
  }
  Ok(())
}

/// What a graphic panel's box lost to its image's pixel rounding, as a factor of its width (`annotate_panel_grid`; the
/// row rule scales the box to the real geometry, `note_stated_widths_in`, so the factor holds there too): the width its
/// `\includegraphics` states (`width=`, its only size) over its box's, where the two are within 1% and a point
/// (KNOWN_PERL_ERRORS #1142: a raster image's box is 72/72.27 of the stated width rounded up to a whole pixel, so
/// 2208.08007 S0.F1.g1's `width=\linewidth` stated 0.997; review rev79c finding 4); 1 for any other panel, or a graphic
/// its options size otherwise (a height, a scale, an angle).
fn graphic_rounding(document: &Document, panel: &Node) -> f64 {
  if document::get_node_qname(panel) != pin!("ltx:graphics") {
    return 1.0;
  }
  let Some(options) = panel.get_attribute("options") else {
    return 1.0;
  };
  let mut stated = None;
  for option in options.split(',') {
    let (key, value) = option.split_once('=').unwrap_or((option, ""));
    match key.trim() {
      "width" => stated = Dimension::spec_to_f64(value.trim()).ok(),
      "height" | "totalheight" | "scale" | "angle" | "viewport" | "bb" | "natwidth"
      | "natheight" => return 1.0,
      _ => {},
    }
  }
  let boxed = document
    .get_node_box(panel)
    .and_then(|b| b.get_width(None).ok().flatten())
    .map_or(0.0, |w| w.value_of() as f64);
  match stated {
    Some(stated)
      if stated > 0.0 && boxed > 0.0 && (stated - boxed).abs() <= 0.01 * stated + 65536.0 =>
    {
      stated / boxed
    },
    _ => 1.0,
  }
}

/// A fraction of a line, as `panelwidth` states it: 3 decimals, no trailing zeros.
fn panel_fraction(width: f64) -> String {
  let fixed = format!("{width:.3}");
  fixed
    .trim_end_matches('0')
    .trim_end_matches('.')
    .to_string()
}

/// The columns each panel spans when the rows are a matrix (`annotate_panel_grid`), else `None`. The columns are those
/// of 2 or more rows that agree panel for panel (the most panels a row takes, 2 or more; each width within
/// `PANEL_WIDTH_TOLERANCE` of the other row's), and every other row takes them all, each panel as wide as the columns
/// it spans; every width is known; and the panels' own captions (`holds_a_caption`) are none, or exactly one whole
/// row's — they label its columns (main-session decision 3, 2026-10-23, review rev79c finding 2, checked on the PDFs:
/// 2209.01559 Figs 3, 5; 1404.0431 Fig 1; 2212.03198 Figs 7, 10, 13, 14; 1811.02385 S4.F3-F6). Captions on every
/// panel, on more than one row, or on some panels of a row make the panels a gallery's — independent, their order
/// not their place (2503.15158 S4.F7, 2404.12123 S3.F6, 2106.04682 A2.F4 (a)-(e) and a legend, 1808.10020
/// chapter5.figure11 and a blank panel, 2602.17978 Ch4.F2/F3).
fn matrix_spans(rows: &[Vec<Node>], widths: &[Vec<Option<f64>>]) -> Option<Vec<Vec<usize>>> {
  if rows.len() < 2 {
    return None;
  }
  // (each row's captioned panels, and its panels)
  let captioned: Vec<(usize, usize)> = rows
    .iter()
    .map(|row| {
      (
        row.iter().filter(|panel| holds_a_caption(panel)).count(),
        row.len(),
      )
    })
    .collect();
  let some_of_a_row = captioned.iter().any(|(c, n)| *c > 0 && c < n);
  let whole_rows = captioned.iter().filter(|(c, n)| c == n).count();
  let any = captioned.iter().any(|(c, _)| *c > 0);
  if any && (some_of_a_row || whole_rows != 1) {
    return None;
  }
  let known: Vec<Vec<f64>> = widths
    .iter()
    .map(|row| row.iter().copied().collect())
    .collect::<Option<_>>()?;
  let agree = |a: &[f64], b: &[f64]| {
    a.len() == b.len()
      && a
        .iter()
        .zip(b)
        .all(|(x, y)| (x - y).abs() <= PANEL_WIDTH_TOLERANCE)
  };
  let most = known.iter().map(Vec::len).max().unwrap_or(0);
  if most < 2 {
    return None;
  }
  let columns = known
    .iter()
    .filter(|row| row.len() == most)
    .find(|row| known.iter().filter(|other| agree(row, other)).count() >= 2)?;
  // Each row's panels over the columns, in order: a panel spans the fewest columns whose widths add up to its own.
  known
    .iter()
    .map(|row| {
      let mut at = 0;
      let mut spans = Vec::with_capacity(row.len());
      for width in row {
        let mut sum = 0.0;
        let span = (at..columns.len()).position(|column| {
          sum += columns[column];
          (sum - width).abs() <= PANEL_WIDTH_TOLERANCE
        })?
          + 1;
        spans.push(span);
        at += span;
      }
      (at == columns.len()).then_some(spans)
    })
    .collect()
}

/// Is the width `arrange_panels` set a panel at (`width`, against the float's `hsize`) one the source states
/// (`annotate_panel_grid`): a box of a declared width (`box_panel_width`), or a panel that states its width
/// (`states_its_width`) — except a sub-float set at the float's whole `\hsize` with no body width of its own
/// (`panel_body_width`): it states none, its box takes the line (subfig's `\subfloat`, Perl's measure; the PDF's box is
/// its content's).
fn panel_width_is_stated(document: &Document, panel: &Node, width: f64, hsize: f64) -> bool {
  if box_panel_width(panel).is_some() {
    return true;
  }
  let sub_float = with(document::get_node_qname(panel), |q| {
    PANEL_FLOATS.contains(&q)
  });
  let body_width = panel_body_width_of(document, panel).is_some();
  if sub_float && !body_width && width >= hsize {
    return false;
  }
  states_its_width(panel)
}

/// Is a panel's width one the source states (`annotate_panel_grid`), not a measure of our own setting of its material:
/// a graphic's (its stated size, or its image's), a rule's, a picture's, a box's of a declared width (a minipage, a
/// `\parbox`: `box_panel_width`), or a container's whose every part is one of those, side by side (a sub-float's body,
/// a merged block; its tags, captions and breaks aside). A paragraph of text, a tabular, a list, a listing, math: our
/// measure of their natural width is no width TeX sets them at (a paragraph is the line; 1807.05122 S2.F2's text
/// panel measured 9.7 lines wide).
fn states_its_width(panel: &Node) -> bool {
  let qname = document::get_node_qname(panel);
  if with(qname, |q| {
    matches!(q, "ltx:graphics" | "ltx:rule" | "ltx:picture")
  }) || box_panel_width(panel).is_some()
  {
    return true;
  }
  if !with(qname, |q| {
    matches!(
      q,
      "ltx:figure"
        | "ltx:table"
        | "ltx:float"
        | "ltx:block"
        | "ltx:inline-block"
        | "ltx:p"
        | "ltx:text"
    )
  }) {
    return false;
  }
  let mut parts = 0;
  for child in panel.get_child_nodes() {
    match child.get_type() {
      Some(NodeType::TextNode) if !child.get_content().trim().is_empty() => return false,
      Some(NodeType::ElementNode) => {
        let name = document::get_node_qname(&child);
        if is_panel_break_name(name) || with(name, |q| matches!(q, "ltx:note" | "ltx:indexmark")) {
          continue;
        }
        if !states_its_width(&child) {
          return false;
        }
        parts += 1;
      },
      _ => {},
    }
  }
  parts > 0
}

/// A panel with a caption of its own (a sub-float's, a captioned minipage's), anywhere in it.
fn holds_a_caption(panel: &Node) -> bool {
  panel
    .get_child_elements()
    .iter()
    .any(|child| document::get_node_qname(child) == pin!("ltx:caption") || holds_a_caption(child))
}

/// The least size (scaled points) a graphic or picture in a `tabular` cell has, both wide and tall (its width and its
/// height plus depth), to be a panel of a matrix (`tabular_panel_grid`): 24pt, two lines of text. A data table's icons,
/// bars and sparklines are small or thin in one of them (review rev79c finding 1: 2608.05898 S4.T2/T5's 102pt x
/// 6.69pt bars, 2506.23665 Ch3.T4/T5, Ch5.T4/T5's 18pt-high sparklines, 2411.16612 S7.T1/T2's 8pt icons, 2501.18280
/// S4.T3/A4.T6, 1509.07152 S10.T23, 1805.10096 Sx4.T1, 2504.02271 S2.T1); a matrix's panels are not (1403.6637 S4.F6,
/// S4.F8's 22-30pt x 38pt, 2008.05524 T11-16 72pt, 0908.0534 Figs 1-2, 1104.5469; 2502.08151 S7.T3's 491pt x 29pt
/// image strips). Of the 253 `tabular` matrices r1 found in the 1,907-paper population this keeps 240: the larger
/// dimension alone kept 246, the bars and sparklines among them.
const PANEL_GRAPHIC_LEAST: f64 = 24.0 * 65536.0;

/// Is `node` a graphic or a picture of a panel's size (`PANEL_GRAPHIC_LEAST`), each dimension its box's or its
/// element's stated `width`/`height` (a picture's, in points), the larger.
fn is_panel_graphic(document: &Document, node: &Node) -> bool {
  if !with(document::get_node_qname(node), |q| {
    matches!(q, "ltx:graphics" | "ltx:picture")
  }) {
    return false;
  }
  let size = |value: Option<RegisterValue>| value.map_or(0.0, |v| v.value_of() as f64);
  let (boxed_width, boxed_height) = document.get_node_box(node).map_or((0.0, 0.0), |b| {
    (
      size(b.get_width(None).ok().flatten()),
      size(b.get_height()) + size(b.get_depth()),
    )
  });
  let stated = |name: &str| {
    node.get_attribute(name).map_or(0.0, |value| {
      let value = value.trim();
      match value.strip_suffix("pt").unwrap_or(value).parse::<f64>() {
        Ok(points) => points * 65536.0,
        Err(_) => Dimension::spec_to_f64(value).unwrap_or(0.0),
      }
    })
  };
  boxed_width
    .max(stated("width"))
    .min(boxed_height.max(stated("height")))
    >= PANEL_GRAPHIC_LEAST
}

/// An author-built `tabular` of panels (`annotate_panel_grid`): the float's one content is a `tabular` (alone, or in
/// a box of nothing else: `\resizebox`, `adjustbox`) 2 or more of whose cells hold a graphic of a panel's size
/// (`is_panel_graphic`; a data table's icons, bars and sparklines make no matrix). Its rows and the most columns a row
/// takes (cells' `colspan` counted).
fn tabular_panel_grid(document: &Document, node: &Node) -> Option<(usize, usize)> {
  let content = |node: &Node| -> Vec<Node> {
    node
      .get_child_elements()
      .into_iter()
      .filter(|child| {
        let qname = document::get_node_qname(child);
        !is_panel_break_name(qname) && !with(qname, |q| matches!(q, "ltx:note" | "ltx:indexmark"))
      })
      .collect()
  };
  let mut children = content(node);
  let tabular = loop {
    let [only] = children.as_slice() else {
      return None;
    };
    let qname = document::get_node_qname(only);
    if qname == pin!("ltx:tabular") {
      break only.clone();
    }
    if qname != pin!("ltx:inline-block") {
      return None;
    }
    children = content(only);
  };
  // (its own rows and cells, not a nested tabular's)
  fn own(node: &Node, qname: &str, out: &mut Vec<Node>) {
    for child in node.get_child_elements() {
      let name = document::get_node_qname(&child);
      if with(name, |n| n == qname) {
        out.push(child);
      } else if !with(name, |n| n == "ltx:tabular") {
        own(&child, qname, out);
      }
    }
  }
  fn holds_graphic(document: &Document, node: &Node) -> bool {
    node
      .get_child_elements()
      .iter()
      .any(|child| is_panel_graphic(document, child) || holds_graphic(document, child))
  }
  let mut rows = Vec::new();
  own(&tabular, "ltx:tr", &mut rows);
  let mut graphic_cells = 0;
  let mut cols = 0;
  for row in &rows {
    let mut cells = Vec::new();
    own(row, "ltx:td", &mut cells);
    graphic_cells += cells
      .iter()
      .filter(|cell| holds_graphic(document, cell))
      .count();
    cols = cols.max(
      cells
        .iter()
        .map(|cell| {
          cell
            .get_attribute("colspan")
            .and_then(|c| c.parse::<usize>().ok())
            .unwrap_or(1)
        })
        .sum::<usize>(),
    );
  }
  (graphic_cells >= 2).then_some((rows.len(), cols))
}

/// Perl: collapseFloat (latex_constructs.pool.ltxml:3437-3464)
/// If a figure/table/float contains exactly one inner float child,
/// and they don't BOTH have captions, collapse the inner into the outer.
fn collapse_float(document: &mut Document, float: &mut Node) -> Result<()> {
  let caption_qname = pin!("ltx:caption");
  // Find inner float/figure/table children
  let mut inners: Vec<Node> = Vec::new();
  for child in float.get_child_elements() {
    if with(document::get_node_qname(&child), |q| {
      PANEL_FLOATS.contains(&q)
    }) {
      inners.push(child);
    }
  }
  if inners.len() != 1 {
    return Ok(());
  }
  let mut inner = inners.into_iter().next().unwrap();
  // Check captions: collapse only if they don't BOTH have captions (Perl collapseFloat,
  // latex_constructs.pool.ltxml:3444). A float numbered by a phantom caption — its own `ltx:tags`, no
  // `ltx:caption` — is captioned in caption.sty's terms (`\caption@refstepcounter`, caption.sty:392-395):
  // the outer float's tags when the inner has tags of its own, an inner float's only when they came from a sub-float
  // counter (`after_float`'s `panel_number`: `\phantomcaption`, a bare `\subfloat{…}`,
  // `\phantomsubcaption`), and such a panel keeps its number and its `\label` (2605.28276
  // `\cref{fig:corridor}` dangled when the collapse copied the panel's `labels` over the figure's;
  // 2605.04869, 2605.17547, 2605.20770, 2605.27546 read the panel's graphic after the figure caption).
  // An inner float's other tags — a longtable's own step (2605.18937, ltablex), the outer caption's
  // counters taken by an inner float that closed first (2605.26653) — leave it collapsible, as Perl.
  let tags_qname = pin!("ltx:tags");
  let has_child = |node: &Node, want: SymStr| {
    node
      .get_child_elements()
      .iter()
      .any(|c| document::get_node_qname(c) == want)
  };
  // The outer's tags count only beside an inner numbered by its own tags: a tagless inner holds the
  // very caption whose counters the outer rescued (`\caption` in a minipage or `\parbox` of a
  // figure, which `insert_block` makes a figure; 57bu.2 A/B: 40 papers, 2605.01437 Fig. 1).
  let outer_has_caption = has_child(float, caption_qname)
    || (has_child(float, tags_qname) && has_child(&inner, tags_qname));
  let inner_has_caption = has_child(&inner, caption_qname)
    || (has_child(&inner, tags_qname)
      && document
        .get_node_box(&inner)
        .is_some_and(|b| b.get_property_bool("panel_number")));
  if outer_has_caption && inner_has_caption {
    return Ok(());
  }
  // The collapsed float keeps the id of the element that carries its number. An inner float numbered
  // by its caption or by its tags alone — a caption-less longtable's own step, a tabularray tall table
  // whose theme hides the caption — in a float with neither gives the float its id (`S1.T1`, not the
  // outer's `S1.tab1`); a float holding the tags a caption in its minipage stepped keeps its own
  // (`S1.F1`, not the minipage's `S1.F1.fig1`: 2605.01437's four figures, 2605.03502, 2605.15932's
  // sub-figures). Perl moves the id exactly when the inner has a caption (collapseFloat,
  // latex_constructs.pool.ltxml:3449-3452). Divergence #372; guards
  // `perfect_kernel_batch57::captionless_float_takes_the_inner_number`,
  // `perfect_kernel_batch57::tabularray_tall_table_in_a_captionless_float` (57cp review 5: kcoll, fl).
  let inner_is_the_number = !has_child(float, tags_qname)
    && !has_child(float, caption_qname)
    && (has_child(&inner, caption_qname) || has_child(&inner, tags_qname));
  // An inner float beside other content — a sibling panel, not only what `arrange_panels` counts as no
  // panel (tags, captions, breaks, notes) — is one panel of the outer: its box geometry and its box and
  // panel classes describe that panel, not the float. Perl copies every attribute (collapseFloat,
  // latex_constructs.pool.ltxml:3447-3449, KNOWN_PERL_ERRORS #388), so a side caption's `0.3\linewidth`
  // minipage gave its figure that width beside the `0.65\linewidth` panel (2605.03502), and a subfigure
  // of a `0.05\linewidth` caption minipage and a `0.86\linewidth` panel claimed 17.3pt (2605.15932,
  // whose two `\linewidth` subfigures then shared one row); 57by, divergence #375. A lone inner float is
  // the float, geometry and all (an `algorithm` of one minipage). Classes merge in both cases, as
  // `insert_block` merges them (divergence #125): an `algorithm` keeps `ltx_float_algorithm`
  // (2605.17037).
  let beside_content = float.get_child_elements().iter().any(|child| {
    let qname = document::get_node_qname(child);
    child != &inner
      && !is_panel_break_name(qname)
      && !with(qname, |q| matches!(q, "ltx:note" | "ltx:indexmark"))
  });
  // Copy inner's attributes to outer (except xml:id); the labels and classes of both are kept.
  let attrs = inner.get_attributes();
  for (name, value) in &attrs {
    // get_attributes() may return the key as "id" (local name) or "xml:id" (prefixed)
    if name == "xml:id" || name == "id" {
      continue;
    }
    // (nor its panel grid's markup, `annotate_panel_grid`: the float's own is made of its panels, the inner's among
    // them; a lone inner's grid is the float's)
    if beside_content && matches!(name.as_str(), "width" | "height" | "depth" | "vattach") {
      continue;
    }
    if (beside_content && PANEL_GRID_ATTRIBUTES.contains(&name.as_str()))
      || matches!(name.as_str(), "panelwidth" | "panelspan")
    {
      continue;
    }
    if name == "class" {
      for class in value.split_whitespace() {
        if !(beside_content
          && matches!(
            class,
            "ltx_minipage"
              | "ltx_parbox"
              | "ltx_figure_panel"
              | "ltx_panels_gallery"
              | "ltx_panels_matrix"
          ))
        {
          document.add_class(float, class)?;
        }
      }
    } else if name == "labels"
      && let Some(outer) = float.get_attribute("labels")
    {
      let mut labels: Vec<&str> = outer.split_whitespace().collect();
      for label in value.split_whitespace() {
        if !labels.contains(&label) {
          labels.push(label);
        }
      }
      document.set_attribute(float, name, &labels.join(" "))?;
    } else {
      document.set_attribute(float, name, value)?;
    }
  }
  // If inner has caption (or is the float's only number), promote inner's xml:id to outer
  if inner_is_the_number {
    let inner_id = inner
      .get_attribute("xml:id")
      .or_else(|| inner.get_attribute_ns("id", "http://www.w3.org/XML/1998/namespace"));
    if let Some(id) = inner_id {
      // Unrecord the outer's old ID and remove the attribute before setting the new one
      if let Some(old_id) = float.get_attribute_ns("id", "http://www.w3.org/XML/1998/namespace") {
        document.unrecord_id(&old_id);
      }
      remove_xml_id(float);
      document.unrecord_id(&id);
      document.set_attribute(float, "xml:id", &id)?;
    }
  }
  // An inner float that was one panel of the outer (`arrange_panels` ran first) leaves its content
  // standing as that panel: its elements other than breaks, captions and notes take the panel class
  // its own box carried, or a tall table beside a tabular in a caption-less `table` stacked the two
  // (57cp review 5, side6; 2605.31204 A1.F2, a tabular beside its colorbar; guard
  // `perfect_kernel_batch57::tabularray_tall_table_in_a_captionless_float`). The inner's caption stays
  // where it stood: after a first captioned panel it splits the row (the XSLT groups a figure's panels
  // between captions), as before the class (2605.01542 A3.F8; divergence #372).
  let inner_was_a_panel = beside_content
    && inner
      .get_attribute("class")
      .is_some_and(|classes| classes.split_whitespace().any(|c| c == "ltx_figure_panel"));
  // Replace inner element with its children, where it stood (Perl saves and re-appends the
  // following siblings, latex_constructs.pool.ltxml:3454-3462).
  let float_has_tags = has_child(float, tags_qname);
  let children: Vec<Node> = inner.get_child_nodes();
  for mut child in children {
    // decided before the move: libxml2 merges a moved text node into an adjacent one and frees it
    let panel = inner_was_a_panel && child.is_element_node() && {
      let qname = document::get_node_qname(&child);
      !is_panel_break_name(qname) && !with(qname, |q| matches!(q, "ltx:note" | "ltx:indexmark"))
    };
    child.unlink_node();
    // (a panel's width there was a fraction of the inner's line, not the float's: dropped beside other content, the
    // inner a panel of the float or not — beside content of no width it was the float's only panel, its children then
    // the float's, review rev79c finding 3; a lone inner's grid, its widths with it, is the float's)
    if beside_content && child.is_element_node() {
      for attribute in ["panelwidth", "panelspan"] {
        child.remove_attribute(attribute).ok();
      }
    }
    // The number an inner float brings to a float without one (a captioned minipage beside other content,
    // `end_caption_box`) heads it, where a float's own template puts its tags.
    if !float_has_tags
      && document::get_node_qname(&child) == tags_qname
      && let Some(mut first) = float.get_first_element_child()
    {
      first.add_prev_sibling(&mut child).ok();
      continue;
    }
    inner.add_prev_sibling(&mut child).ok();
    if panel {
      document.add_class(&mut child, "ltx_figure_panel")?;
    }
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

  // latex.ltx's tabbing is built from boxes, never `\halign` (tex_tables.rs `ALIGNMENT_WITHOUT_HALIGN`).
  let mut tabbing_properties = SymHashMap::default();
  tabbing_properties.insert(
    crate::tex_tables::ALIGNMENT_WITHOUT_HALIGN,
    Stored::Bool(true),
  );
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
    properties:      tabbing_properties,
    xml_attributes:  xml_attrs,
  });
  assign_alignment(alignment, None);
  clear_alignment_row_hooks();

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

/// A backmatter element (`thebibliography`'s `ltx:bibliography`) begun inside a `{quote}`, `{quotation}` or `{verse}`
/// (the AAAI template's `\begin{quote}\begin{small}\bibliography{…}`): neither the unit it stands as nor the element
/// itself may go in an `ltx:quote` (LaTeXML-block.rnc `quote_model`), and the quote does not auto-close, so both erred
/// and the bibliography lost its place in the back matter. LaTeX's `thebibliography` is `\section*{\refname}` and a
/// `\list` (article.cls:565-584; book.cls, report.cls alike), typeset inside the quote's `\list` margins — the heading
/// and the list are the document's bibliography, the quote only indents it: the quotes in the way are closed when that
/// alone makes the bibliography's place reachable (`Document::close_quotes_blocking`, shared with a sectioning unit in a
/// quote). The quote's font changes (`\small`) are digested into the entries and stay. User ruling 2026-10-16
/// (OXIDIZED_DESIGN_DIVERGENCES #677); witnesses 2203.09193 1904.12066 1610.09996 2109.09861 2403.09260 1609.06434
/// (AAAI), astro-ph0410583 (quote), 1504.06728 (verse). Guard `perfect_kernel_batch69a::bibliography_in_a_quote_closes_the_quote`.
fn close_list_containers_before_backmatter(
  document: &mut Document,
  asif: &str,
  element: &str,
) -> Result<()> {
  if !document.is_openable(asif) && !document.is_openable(element) {
    document.close_quotes_blocking(&[pin(asif), pin(element)])?;
  }
  Ok(())
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
    close_list_containers_before_backmatter(document, &asif, &element)?;
    let mut target = backmatter_insertion_target(document, &asif, &element);
    // Where the unit it stands as would go cannot hold it — an inline sectional block, whose model is body content only
    // (a list item's or a float's sections, OXIDIZED_DESIGN_DIVERGENCES #189, then a `thebibliography` in it) — it goes
    // where it can go from here, inside the last unit. Decided before any node is closed.
    let mut ancestor = Some(document.get_node().clone());
    while let Some(node) = ancestor {
      if document::can_contain(&node, &target) {
        if document::get_node_qname(&node) == pin!("ltx:inline-sectional-block")
          && !document::can_contain(&node, &element)
        {
          target = element.clone();
        }
        break;
      }
      ancestor = node.get_parent();
    }
    let point = document.find_insertion_point(&target, None)?;
    document.set_node(&point);
  }
  Ok(())
}

// Do this before digesting the body of a bibliography
// Perl: beforeDigestBibliography in latex_constructs.pool.ltxml L3900
pub fn before_digest_bibliography() -> Result<()> {
  // The document body has begun: a sectionless letter's bibliography seals its frontmatter as a heading would
  // (`seal_digested_frontmatter`; a PRL body before its supplement, 2609.07332).
  seal_digested_frontmatter();
  AssignValue!("inPreamble" => false);
  Digest!("\\@lx@inbibliographytrue")?;
  // an explicit bibliography: not the lonely one a natbib `\bibitem` opened (sect11.rs `\lx@bibliography@lonely`)
  assign_value("lx_bibliography_lonely", false, Some(Scope::Global));
  // The list a heading inside the author's `{thebibliography}` interrupted (sect11.rs `\lx@bibliography@sectioned`)
  // goes on: its entries keep counting, as pdflatex numbers them through the one list.
  let continued = lookup_bool("lx_bibliography_sectioned");
  // whether this is the author's `{thebibliography}` (not one a `\bibitem` opened in another environment), for the
  // environment's group
  assign_value(
    "lx_bibliography_author_env",
    lookup_string("current_environment") == "thebibliography",
    None,
  );
  assign_value("lx_bibliography_sectioned", false, Some(Scope::Global));
  assign_value(
    "lx_bibliography_is_continued",
    continued,
    Some(Scope::Global),
  );
  // …whose number is that of the one it continues when that held no entry and is pruned (`begin_bibliography_clean`)
  let reuse_number = continued && lookup_bool("lx_bibliography_sectioned_empty");
  assign_value(
    "lx_bibliography_sectioned_empty",
    false,
    Some(Scope::Global),
  );
  assign_value(
    "lx_bibliography_reuse_number",
    reuse_number,
    Some(Scope::Global),
  );
  // …nor does natbib's pending entry stop of an earlier one (`\@bibstop`, natbib.sty:880-905) carry into its first entry
  Let!("\\@bibstop", "\\@undefined");
  // …and it has a list end of its own (sect11.rs `\lx@bibliography@nolistend`)
  assign_value("lx_bibliography_no_list_end", false, Some(Scope::Global));
  def_macro_noop("\\bibliographystyle{}")?;
  def_macro_noop("\\bibliography {}")?;
  // avoid \let-based redefinitions of the ending.
  Let!("\\endthebibliography", "\\saved@endthebibliography");
  if !continued {
    ResetCounter!("@bibitem");
  }
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
  // Rust's `.bbl` rebuilder used to reach it: a biber `.bbl` carries one
  // `\datalist` per sorting scheme (apa emits `nyt/apasortcite//…` *and*
  // `nyt/global//…`), and every `\enddatalist` expanded to a whole bare
  // `\thebibliography…\endthebibliography`, so the first arming was still in
  // force when the second opened. Witness: arXiv 2605.17646
  // (`Fatal:Timeout:TokenLimit`, 1e9 tokens). Since batch 56jc the `.bbl` is
  // read into bibentries instead; the hand-written bare pair stays guarded by
  // `bare_thebibliography_twice_arms_once`.
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
/// What a package typesets between a bibliography's heading and its list (`\lx@bibliography@preamble`, sect11.rs),
/// digested after the heading into the bibliography's `#preamble` — before the list's `\bibitem` redirection is armed.
pub fn digest_bibliography_preamble(whatsit: &mut Whatsit) -> Result<()> {
  let preamble = Digest!(T_CS!("\\lx@bibliography@preamble"))?;
  if !preamble.is_empty()? {
    whatsit.set_property("preamble", preamble);
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
  // (the continuation of a bibliography a heading interrupted before its first entry takes that one's number, and the
  // id `prune_interrupted_bibliography` frees with it, sect11.rs `\lx@bibliography@sectioned`; Rust-only, as the
  // continuation is, OXIDIZED_DESIGN_DIVERGENCES #653)
  let reuse_number = lookup_bool("lx_bibliography_reuse_number");
  if reuse_number {
    assign_value("lx_bibliography_reuse_number", false, Some(Scope::Global));
  }
  let bibnumber = match lookup_int("n_bibliographies") {
    n if reuse_number && n > 0 => n,
    n => 1 + n,
  };
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
  // A package's title for this bibliography (`\lx@bibliography@title`, sect11.rs: biblatex's `title=`) outranks
  // the sectioning default; it names the title, so it may itself be `\refname`.
  let hook_title = match DigestIf!("\\lx@bibliography@title")? {
    Some(title) if !title.is_empty()? => Some(title),
    _ => None,
  };
  let title_opt = if hook_title.is_some() {
    hook_title
  } else if let Some(bt) = bibtitle {
    Some(Digest!(bt)?)
  } else {
    match digest_in_group_if(T_CS!("\\refname"))? {
      Some(v) => Some(v),
      None => digest_in_group_if(T_CS!("\\bibname"))?,
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
    // jurabib's styles (each `.bst`'s SORT): author-year, sorted but for jurunsrt.
    "jurabib" | "jox" | "jureco" => Some(("authoryear", "true")),
    "jurunsrt" => Some(("authoryear", "false")),
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
    // abnTeX2's two styles (the abntex2cite binding records its default at load):
    // abntex2-num.bst's `presort` drops the sort key "such that SORT has no effect"
    // (:2021-2022), so it numbers by first citation; abntex2-alf.bst sorts by author and
    // cites author-year ("(FARIA, 1994)").
    "abntex2-num" => Some(("numbers", "false")),
    "abntex2-alf" => Some(("authoryear", "true")),
    // figbib.bst (READ, ITERATE {call.type$}) has no SORT: its figure list is in citation order.
    "figbib" => Some(("numbers", "false")),
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
/// The kernel's `env/<name>/<point>` hook call, in its own token shapes:
/// bare `\UseHook{env/#1/<point>}` for before/begin/after
/// (latex.ltx:15347/15362/15391) and, for `end`,
/// `\romannumeral\IfHookEmptyTF{env/#1/end}{\expandafter\z@}{\z@\UseHook
/// {env/#1/end}}` (:15386-15388): an empty hook vanishes at EXPANSION time,
/// so nothing sits between an alignment's last cell and `\endtabular`'s
/// implicit `\crcr` (a trailing `\multicolumn` row leaked its group when an
/// unexpandable token stood there). Emitted only when the L3 hook system
/// exists and reports code for the hook (its own `\IfHookEmptyTF`,
/// dialect.rs): with no hook there is nothing to fire, and a line-reading
/// environment (comment.sty) must not see extra tokens at its `\end`.
pub(crate) fn env_hook_tokens(name: &str, point: &str) -> Vec<Token> {
  if !env_hook_has_code(name, point) {
    return Vec::new();
  }
  let hook = |v: &mut Vec<Token>| {
    v.push(T_CS!("\\UseHook"));
    v.push(T_BEGIN!());
    v.extend(ExplodeText!(s!("env/{name}/{point}")));
    v.push(T_END!());
  };
  let mut v = Vec::new();
  if point == "end" {
    v.push(T_CS!("\\romannumeral"));
    v.push(T_CS!("\\IfHookEmptyTF"));
    v.push(T_BEGIN!());
    v.extend(ExplodeText!(s!("env/{name}/end")));
    v.push(T_END!());
    v.extend([T_BEGIN!(), T_CS!("\\expandafter"), T_CS!("\\z@"), T_END!()]);
    v.push(T_BEGIN!());
    v.push(T_CS!("\\z@"));
    hook(&mut v);
    v.push(T_END!());
  } else {
    hook(&mut v);
  }
  v
}

/// LaTeX's `\begin{name}` for an environment defined by macros
/// (latex.ltx:15346-15364): the `before` hook outside the group, then inside
/// it `\@currenvir`, the `begin` hook and `\name` — `\begin`'s path whenever
/// no `\begin{name}` constructor is defined. `env` is the name as written.
pub(crate) fn begin_environment_by_macro(name: &str, env: Tokens) -> Result<Vec<Token>> {
  let mut out = lookup_tokens(&format!("@environment@{name}@beforebegin"))
    .map(Tokens::unlist)
    .unwrap_or_default();
  out.extend(env_hook_tokens(name, "before"));
  out.push(T_CS!("\\begingroup"));
  if let Some(after) = lookup_tokens(&format!("@environment@{name}@atbegin")) {
    out.extend(after.unlist());
  }
  out.extend(Invocation!(T_CS!("\\lx@setcurrenvir"), vec![env]).unlist());
  out.extend(env_hook_tokens(name, "begin"));
  out.push(T_CS!(format!("\\{name}")));
  Ok(out)
}

/// LaTeX's `\end{name}` for an environment defined by macros
/// (latex.ltx:15386-15391): the `end` hook inside the group before
/// `\endname`, the `after` hook after the `\endgroup`. The epilogue `\end`
/// shares with every path (`\if@ignore…\fi`) is the caller's.
pub(crate) fn end_environment_by_macro(name: &str) -> Vec<Token> {
  let mut out = lookup_tokens(&format!("@environment@{name}@atend"))
    .map(Tokens::unlist)
    .unwrap_or_default();
  out.extend(env_hook_tokens(name, "end"));
  let end = T_CS!(format!("\\end{name}"));
  if is_defined_token(&end) {
    out.push(end);
  }
  // latex.ltx:15389 `\@checkend{#1}` before the `\endgroup`. Not for a
  // nameless `\end` (AMS-TeX's document end read as LaTeX's `\end`, 0902.2535,
  // alg-geom9410004): there `\end` + "" is `\end` itself, which would take
  // `\lx@checkend` as its argument, as TeX's `\end` would take `\@checkend`,
  // and leave `{}` behind. A nameless `\end` names no environment to check;
  // skipping it is this binding's choice (no latex.ltx or tex.web rule), kept
  // because the doubled diagnostic carries nothing (cond-mat9211014 shows the
  // same capture through a `\let\endX\end`, which is left as TeX does it).
  if !name.is_empty() {
    out.push(T_CS!("\\lx@checkend"));
    out.push(T_BEGIN!());
    out.extend(Explode!(name));
    out.push(T_END!());
  }
  out.push(T_CS!("\\endgroup"));
  out.extend(env_hook_tokens(name, "after"));
  if let Some(afterend) = lookup_tokens(&format!("@environment@{name}@afterend")) {
    out.extend(afterend.unlist());
  }
  out
}

/// The makeindex characters an entry is written with: makeindex(1)'s style
/// keys `level`, `actual`, `encap` and `quote` (defaults `!`, `@`, `|`, `"`).
/// A document that writes its entries for a style of its own declares them:
/// doc.sty:521-524 defines `\levelchar` `>`, `\actualchar` `=`, `\encapchar`
/// `|` and `\quotechar` `!` to match gind.ist/gglo.ist and assembles every
/// `\index`/`\glossary` entry from those macros (doc.sty:626-650, 1054-1093);
/// amsldoc.cls:74-77 and tvz-user.sty:99-102 spell out makeindex's defaults
/// the same way. Split with the defaults, each doc.sty entry stayed ONE
/// phrase (`foo=\verb!*+\foo+`, `v1.0>!!=General:>…`); split with the
/// characters in force it parses as the pdflatex + makeindex run does.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct IndexChars {
  level:  String,
  actual: String,
  encap:  String,
  quote:  String,
}

/// The macros a writer spells its makeindex characters with (doc.sty:521-524).
const INDEX_CHAR_MACROS: [&str; 4] = ["\\levelchar", "\\actualchar", "\\encapchar", "\\quotechar"];

impl IndexChars {
  /// makeindex's own characters: `!`, `@`, `|`, `"`.
  pub(crate) fn makeindex_default() -> Self {
    IndexChars {
      level:  "!".to_string(),
      actual: "@".to_string(),
      encap:  "|".to_string(),
      quote:  "\"".to_string(),
    }
  }

  /// The characters an `\index` entry was written with. The style is chosen
  /// per index FILE, outside the document (`makeindex -s gind.ist`), and one
  /// document can hold entries of both kinds: keytheorems-doc and
  /// csvsimple-l3 load doc.sty (through ltxdoc) but index with tcolorbox's
  /// documentation library, which writes makeindex's defaults
  /// (tcbdocumentation.code.tex:214 `index default settings`: `@`) for a
  /// default `makeindex` run; read in doc's characters their sort keys were
  /// never split off and raised `Script _`. So an entry is read in the
  /// declared characters only when it shows them — spelled with
  /// `\levelchar` & co., as every doc.sty writer's entry is (doc.sty:541,
  /// 1054-1093; hypdoc.sty:351; amsldoc.cls:88), or holding the declared
  /// actual character itself at its top level, as nlctdoc.cls:496-570 writes
  /// `\index{#1=\appfmt{#1}…}` for its own style (nlctdoc.cls:140-150) — and
  /// in makeindex's defaults otherwise.
  pub(crate) fn for_index_entry(entry: &Tokens) -> Self {
    let declared = Self::in_force();
    let default = Self::makeindex_default();
    if declared == default {
      return default;
    }
    let mut depth = 0i32;
    let mut in_math = false;
    for t in entry.unlist_ref() {
      match t.get_catcode() {
        Catcode::BEGIN => depth += 1,
        Catcode::END => depth -= 1,
        Catcode::MATH if depth == 0 => in_math = !in_math,
        Catcode::CS if t.with_str(|s| INDEX_CHAR_MACROS.contains(&s)) => return declared,
        _ => {},
      }
      if depth == 0
        && !in_math
        && declared.actual != default.actual
        && t.get_catcode() != Catcode::CS
        && t.with_str(|s| s == declared.actual)
      {
        return declared;
      }
    }
    default
  }

  /// The characters in force. Each of `\levelchar`, `\actualchar`,
  /// `\encapchar` and `\quotechar` that is a parameterless macro whose body
  /// is one character overrides makeindex's default for its key. (hypdoc
  /// re-`\def`s `\encapchar` WITH parameters inside its own writers,
  /// hypdoc.sty:344-353 — no declaration; the default stands.) A `\glossary`
  /// entry is always read in them: the `.glo` file needs a style that names
  /// its `\glossaryentry` keyword (gglo.ist `keyword`), and doc.sty's
  /// `\changes` expands the characters into the entry before writing it
  /// (doc.sty:627 `\protected@edef`), so they leave no spelling to see.
  pub(crate) fn in_force() -> Self {
    let default = Self::makeindex_default();
    let pick = |cs: &str, default: String| declared_index_char(cs).unwrap_or(default);
    IndexChars {
      level:  pick("\\levelchar", default.level),
      actual: pick("\\actualchar", default.actual),
      encap:  pick("\\encapchar", default.encap),
      quote:  pick("\\quotechar", default.quote),
    }
  }
}

/// The one character a `\def\levelchar{>}`-style declaration stands for.
fn declared_index_char(cs: &str) -> Option<String> {
  let defn = lookup_definition(&T_CS!(cs)).ok()??;
  if !defn.is_expandable()
    || defn
      .get_parameters()
      .is_some_and(|p| !p.get_parameters().is_empty())
  {
    return None;
  }
  match defn.get_expansion() {
    Some(ExpansionBody::Tokens(body)) => match body.unlist_ref().as_slice() {
      [t] if !t.get_catcode().is_active_or_cs() => Some(t.with_str(|s| s.to_string())),
      _ => None,
    },
    _ => None,
  }
}

/// The next token of a `\verb` run as the `.ind` re-read receives it. A
/// control sequence is EXPANDED, as `\protected@write` expands a
/// macro-assembled entry (doc.sty's `\verb\quotechar*\verbatimchar…`;
/// l3doc.cls:2151 `\verbatimchar` = `&`, an unexpanded `&` is a stray
/// alignment tab — ltx-talk, postnotes, pythonimmediate). Returns the token,
/// the unconsumed rest, and whether a macro supplied it; None when nothing is
/// left. A failed expansion is the caller's error.
fn written_index_token(toks: Vec<Token>) -> Result<Option<(Token, Vec<Token>, bool)>> {
  let Some(first) = toks.first().copied() else {
    return Ok(None);
  };
  if first.get_catcode() != Catcode::CS {
    return Ok(Some((first, toks[1..].to_vec(), false)));
  }
  let tail = Tokens::new(toks);
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
  let (first, remaining) = expanded?;
  Ok(first.map(|t| (t, remaining, true)))
}

/// [`written_index_token`] after makeindex has read it: the QUOTE character
/// takes the next character literally and is dropped (makeindex(1) `quote`),
/// so doc.sty's `\verb!*+\foo+` is `\verb*+\foo+` and lshort's
/// `\index{^@\verb"|^"|}` is `\verb|^|` (lshort-*/math.tex; real makeindex
/// output checked).
fn unquoted_index_token(
  toks: Vec<Token>,
  chars: &IndexChars,
) -> Result<Option<(Token, Vec<Token>, bool)>> {
  let Some((t, rest, from_macro)) = written_index_token(toks)? else {
    return Ok(None);
  };
  if t.get_catcode() != Catcode::CS && t.with_str(|s| s == chars.quote) && !rest.is_empty() {
    return Ok(
      written_index_token(rest)?
        .map(|(quoted, rest, quoted_from_macro)| (quoted, rest, from_macro || quoted_from_macro)),
    );
  }
  Ok(Some((t, rest, from_macro)))
}

/// A `\verb` body as makeindex hands it on: the quote character is dropped
/// and makes the next character literal — a quoted delimiter still closes
/// the run, since makeindex writes it out bare — and a control SYMBOL named
/// by the quote character is makeindex's ESCAPE rule (`\"` stays `\"`, the
/// umlaut) unless the backslash is itself quoted: amsldoc.tex's
/// `\index{"|@\verb"*+"\"|+}` is `\verb*+\|+` (makeindex output checked).
/// The re-tokenization of the entry welded that `\` and `"` into one token.
struct IndexVerbBody<'q> {
  delim:  String,
  quote:  &'q str,
  quoted: bool,
  body:   Vec<Token>,
}

impl IndexVerbBody<'_> {
  /// Take one written token; true when it closes the run.
  fn feed(&mut self, t: Token) -> bool {
    let is_cs = t.get_catcode() == Catcode::CS;
    let s = t.with_str(|s| s.to_string());
    if !self.quoted {
      if !is_cs && s == self.quote {
        self.quoted = true;
        return false;
      }
    } else if is_cs && s.strip_prefix('\\') == Some(self.quote) {
      self.body.push(T_OTHER!("\\"));
      return false;
    }
    self.quoted = false;
    if s == self.delim {
      return true;
    }
    self.body.push(t);
    false
  }
}

/// The script characters of an entry's display as the `.ind`/`.gls` re-read
/// gives them. The entry was re-tokenized with the internal catcodes (`_`
/// subscript, `^` superscript); makeindex's output is read back in the
/// document, where underscore.sty makes `_` active (underscore.sty:29-31,
/// `\catcode`_\active` at begin document; l3doc.cls:432 loads it): active in
/// the document, the character is active here too. Outside math, where the
/// entry was read under a `\@sanitize` (doc.sty's `\changes`, :626-650) or the
/// document made the character text, it is the text character — an other `_`
/// would print through OT1's slot as `˙`. lt3graph's `\changes{…}{…}{\texttt{
/// \textbackslash graph_(g)put_vertex}}` prints `\graph_(g)put_vertex` (golden
/// lt3graph.pdf) where the subscript raised `Script _ can only appear in math
/// mode`. A subscript `_` of the document stays one, in math and out (pdflatex
/// reports the latter too).
fn reread_script_chars(phrase: Vec<Token>) -> Vec<Token> {
  let mut in_math = false;
  phrase
    .into_iter()
    .map(|t| {
      let cc = t.get_catcode();
      if cc == Catcode::MATH {
        in_math = !in_math;
      }
      if cc != Catcode::SUB && cc != Catcode::SUPER {
        return t;
      }
      let Some(c) = t.with_str(|s| {
        let mut chars = s.chars();
        chars.next().filter(|_| chars.next().is_none())
      }) else {
        return t;
      };
      match lookup_catcode(c) {
        Some(Catcode::ACTIVE) => T_ACTIVE!(c),
        Some(Catcode::SUB | Catcode::SUPER) | None => t,
        Some(_) if in_math => t,
        Some(_) if cc == Catcode::SUB => T_CS!("\\textunderscore"),
        Some(_) => T_CS!("\\textasciicircum"),
      }
    })
    .collect()
}

/// Perl: process_index_phrases — expand \index{a!b@c|see{d}} into
/// \@index{\@indexphrase{a}\@indexphrase[c]{b}} etc.
///
/// Port of latex_constructs.pool.ltxml L4528-4591
/// #354 surpass (OXIDIZED_DESIGN #119): a `\verb`/`\verb*` inside `\index`.
/// `\index` reads its argument `SanitizedVerbatim` and re-reads it (`reread_sanitized_entry`) —
/// collapsing `\verb`'s raw body back into control sequences (`\delta`, not
/// `\`,`d`,…) and leaving `\verb` with no mouth to scan a delimiter from. In
/// both engines this yielded an empty `<verbatim/>` with the body leaking out
/// mis-tokenized, and a `|` delimiter additionally collided with the makeindex
/// encap separator. Consume each whole `\verb<D>body<D>` run HERE — before
/// expansion and before the `!`/`@`/`|` split can see the delimiter — and emit
/// `\@internal@text@verb{star}{D}{body}` (a non-expandable constructor, so the
/// run also survives the `\protected@write` expansion below) so the body
/// renders as typewriter.
///
/// The run is read as makeindex reads the written entry
/// ([`unquoted_index_token`], [`IndexVerbBody`]). When a MACRO supplied the
/// head (doc.sty:1054-1093 `\verb\quotechar*\verbatimchar\bslash\@gtempa
/// \verbatimchar`), the body was assembled by macros too and is expanded
/// before it is scanned — `\protected@write` expands it before makeindex ever
/// sees it — so it is `\foo`, closed by the expanded `\verbatimchar`, not the
/// literal `\bslash\@gtempa\verbatimchar…` running to the end of the entry
/// (every doc.sty usage entry: source2e, sunpath). A literal head keeps its
/// body literal (`\index{\verb+\delta+}` is `\delta`, not δ). The caller
/// runs this inside its `\protected@write` frame (`\protect`, inert symbols).
fn absorb_index_verb_runs(toks: &[Token], chars: &IndexChars) -> Result<Vec<Token>> {
  let mut out: Vec<Token> = Vec::with_capacity(toks.len());
  let mut i = 0;
  while i < toks.len() {
    let tok = toks[i];
    i += 1;
    // `\string\verb` WRITES the characters `\verb`, and the `.ind` re-read
    // makes them the command again: amsldoc.cls:87-92 `\@indexcs`
    // (`\string\verb\quotechar*\verbatimchar…`), doc.sty's `\changes`
    // (:641). Left in place, the `\string` stringified the run's opening
    // brace (`–` through OT1, amsldoc-it, amsldoc-vi).
    if tok == T_CS!("\\string") && toks.get(i) == Some(&T_CS!("\\verb")) {
      continue;
    }
    if tok != T_CS!("\\verb") {
      out.push(tok);
      continue;
    }
    if i >= toks.len() {
      out.push(tok);
      continue;
    }
    // The head: an optional `*`, then the delimiter.
    let head = match unquoted_index_token(toks[i..].to_vec(), chars)? {
      Some((first, rest, m1)) if first == T_OTHER!("*") => {
        unquoted_index_token(rest, chars)?.map(|(delim, rest, m2)| (true, delim, rest, m1 || m2))
      },
      Some((first, rest, m1)) => Some((false, first, rest, m1)),
      None => None,
    };
    let Some((starred, delim, rest, from_macro)) = head else {
      // Not a verbatim invocation at all: keep `\verb` as index text.
      out.extend(Explode!("\\verb"));
      continue;
    };
    let mut body = IndexVerbBody {
      delim:  delim.with_str(|d| d.to_string()),
      quote:  &chars.quote,
      quoted: false,
      body:   Vec::new(),
    };
    // A macro-assembled run is `\protected@write`-expanded before it is read
    // (robust and `\protected` commands stay, as in the `\write`): source2e's
    // `\DescribeMacro{\g__hook_\meta{hook}_code_prop}` (lthooks.dtx) carries
    // doc.sty's `\meta` into the body; read with `read_x_token`, which
    // expands robust commands, it ran through `\meta`'s internals (65
    // errors on lthooks.dtx). The remainder is handed on expanded; the
    // entry's own expansion then finds nothing more to expand in it (a
    // `\protect`ed command re-protects itself).
    //
    // The run may close inside a group of the expanded entry: doc.sty:544-548
    // `\LeftBraceIndex` writes `…\verbatimchar\quotechar\bslash{\verbatimchar
    // \string\iffalse}\string\fi`, whose re-read makes `\iffalse…\fi` hide the
    // `}` again; a remainder that does not balance is read literally instead.
    let literal_rest = rest.clone();
    let mut rest = if from_macro {
      do_expand_partially(Tokens::new(rest))?.unlist()
    } else {
      rest
    };
    let scan = |rest: &[Token], body: &mut IndexVerbBody| {
      let mut j = 0;
      while j < rest.len() && !body.feed(rest[j]) {
        j += 1;
      }
      // consume the closing delimiter
      rest[(j + 1).min(rest.len())..].to_vec()
    };
    let mut after: Vec<Token> = scan(&rest, &mut body);
    if from_macro && !Tokens::new(after.clone()).is_balanced() {
      body.body.clear();
      body.quoted = false;
      rest = literal_rest;
      after = scan(&rest, &mut body);
    }
    // The re-tokenized body collapsed `\verb`'s raw chars back into control
    // sequences; `untex` + `Explode!` restores them to catcode-OTHER literals
    // so the digested `#3` renders as typewriter text instead of re-expanding
    // (which is exactly the `\delta`→math-δ leak this fixes).
    let mut body_toks = body.body;
    if from_macro {
      // A robust command is written by its name: the `.idx` line of
      // `\DescribeMacro{\foo\meta{name}}` reads `\verb!*+\foo\meta  {name}+`
      // (pdflatex), without the `\protect` the expansion stops at.
      let protect = T_CS!("\\protect");
      let mut kept = Vec::with_capacity(body_toks.len());
      for (k, t) in body_toks.iter().enumerate() {
        let next_is_cs = body_toks
          .get(k + 1)
          .is_some_and(|n| n.get_catcode() == Catcode::CS);
        if !(*t == protect && next_is_cs) {
          kept.push(*t);
        }
      }
      body_toks = kept;
    }
    let body_str = Tokens::new(body_toks).untex();
    // In a group, as `\verb` itself is: the constructor's typewriter font
    // must not run on into the rest of the phrase (doc.sty's change entry
    // `\verb*+\foo+:` → the `:` and the next level were typewriter).
    out.push(T_BEGIN!());
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
    out.push(T_END!());
    // The remainder may hold further runs; it is a fresh slice now.
    out.extend(absorb_index_verb_runs(&after, chars)?);
    return Ok(out);
  }
  Ok(out)
}

/// The tokens that typeset `tokens`, an entry TeX writes for the list `list` (`idx`, a named index, `glo`,
/// `nomenclature`) and prints only from `\printindex` or its like: the entry itself once the list is printed, else the
/// anchor `\lx@deferred@print{id}` of an entry kept raw until it is (latexml_core `defer_print`;
/// OXIDIZED_DESIGN_DIVERGENCES #738).
pub fn deferred_print_tokens(list: &str, tokens: Tokens) -> Tokens {
  if tokens.is_empty() || deferred_list_is_printed(list) {
    return tokens;
  }
  let id = defer_print(list, tokens);
  let mut out = vec![T_CS!("\\lx@deferred@print"), T_BEGIN!()];
  out.extend(Explode!(id.to_string()));
  out.push(T_END!());
  Tokens::new(out)
}

/// `chars` are the makeindex characters the entry is split with
/// ([`IndexChars::in_force`]); `inlist` names the list the mark belongs to
/// (Perl's third argument: `\@index[style][inlist]`) — `\glossary` marks
/// list `glo`, collected by a `.gls` stand-in's `<ltx:index lists="glo">`;
/// `formed` are the control words the `.idx` re-read formed from typed
/// characters ([`reread_sanitized_entry`]), written verbatim, never expanded.
pub(crate) fn process_index_phrases(
  tokens: Tokens,
  chars: &IndexChars,
  inlist: Option<&str>,
  formed: &[Token],
) -> Result<Tokens> {
  if tokens.is_empty() {
    return Ok(Tokens::new(vec![]));
  }
  // Perl pool:4332-4336: an entry whose groups do not balance is discarded, as
  // makeindex rejects it. doc.sty's `\changes{v1.2i}{…}{Use \cs{protected} for
  // \cs{\bslash} variant}` (ltmath.dtx, source2e) writes `\cs{\}`, whose
  // re-read `\}` leaves the `{` open (the golden change history has no such
  // entry); read as a phrase it ran the entry's reader off its end.
  if !tokens.is_balanced() {
    Warn!(
      "malformed",
      "indexentry",
      s!(
        "index entry has unbalanced groups, discarding: \"{}\"",
        tokens
      )
    );
    return Ok(Tokens::new(vec![]));
  }
  let token_list = tokens.unlist();
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
  // A control SYMBOL in the entry is never expanded here. After `\@sanitize`
  // (latex.ltx:1778) real `\@wrindex` writes a `\string`ed control symbol as
  // two characters, makeindex drops the sort key entirely and `\printindex`
  // re-reads only the display — so amsldoc.cls:84-89's sort key `\*` for
  // `\cn{\\*}` is never executed. The `.idx` re-read (`reread_sanitized_entry`)
  // welds those two characters back into the live `\*` (amsldoc.cls:213
  // `\def\*#1`), which ate the rest of the entry (itamsldoc, amsldoc-vi;
  // Perl shares it, PLANS P73). Freezing every control symbol with
  // `\noexpand` keeps `\*` inert (the key is a string anyway) while accents
  // in the display (`M\"uller`) are constructors and unaffected; separators
  // still come from control WORDS (`\idx@actual`). Guard:
  // `perfect_kernel_batch54::index_sanitized_backslash_symbol_stays_literal`.
  // An UNDEFINED control word is frozen too: the entry is a `.idx` STRING
  // that `\printindex` re-reads and digests in order, so manyind.sty:100/119's
  // `\protect\def \nwletre {\"O}…\nwletre` defines the name before its use;
  // force-expanding the bare `\nwletre` here stubbed it as `<ltx:ERROR/>`
  // (mindsample; Perl, which never expands, is clean). Guard:
  // `perfect_kernel_batch56::index_entry_passes_undefined_words_inert`.
  // The freeze is a STATE-level `\let` to `\relax` inside the expansion
  // frame, never an injected `\noexpand` TOKEN: `\string` reads the next
  // token literally, so `\string\noexpand\cmd` stringified "\noexpand" and
  // re-exposed `\cmd` to expansion — beamerug-macros.tex:44's
  // `\gdef\stripcommand#1{\expandafter\@gobble\string#1}` (which hides the
  // `\string` from any token-level neighbour check) made every documented
  // command undefined and every `\if…` name a runaway conditional
  // (beameruserguide 167 errors; Perl 0). Let to `\relax`, the name is
  // unexpandable for the pass, stringifies by NAME, and is restored to its
  // real (or undefined) meaning when the frame pops. Guard:
  // `cluster_package_guards::index_string_of_undefined_command_stringifies_its_name`.
  let inert: Vec<Token> = token_list
    .iter()
    .filter(|t| {
      t.get_catcode() == Catcode::CS
        && (t.with_str(|s| {
          let mut chars = s.chars();
          chars.next() == Some('\\')
            && chars.next().is_some_and(|c| !c.is_alphabetic())
            && chars.next().is_none()
        }) || lookup_meaning(t).is_none()
          || formed.contains(t))
    })
    .cloned()
    .collect();
  push_frame();
  let_i(
    &T_CS!("\\protect"),
    &T_CS!("\\@unexpandable@protect"),
    Some(Scope::Local),
  );
  for t in &inert {
    let_i(t, &T_CS!("\\relax"), Some(Scope::Local));
  }
  // The `\verb` runs are absorbed inside the same frame: a macro-assembled
  // run's body is expanded as it is scanned, and amsldoc's `\cn{\\*}`
  // entry (`\@indexcs`, amsldoc.cls:87-92) carries the control symbol `\*`
  // into it, whose amsldoc.cls:213 `\def\*#1` ate the rest of the entry and
  // spilled keyval errors (amsldoc-it, amsldoc-vi).
  // A Fatal inside either stays Fatal; the frame pops first.
  let expanded = absorb_index_verb_runs(&token_list, chars)
    .and_then(|toks| do_expand_partially(Tokens::new(toks)));
  pop_frame()?;
  // The `.ind` re-read: `\printindex` `\input`s the written entry in the
  // document body, where `@` is OTHER, so a control word that still carries
  // an `@` after the `\protected@write` expansion splits there —
  // mindsample.tex:208 `\index{\AB@\relax…}` is the key `\AB` and the
  // separator (Perl's `TokenizeInternal` re-read keeps one undefined `\AB@`;
  // OXIDIZED_DESIGN_DIVERGENCES #222). Names the expansion consumed
  // (`\kvtcb@doc@sortindex`, `\idx@actual`) never reach this point; a
  // DEFINED survivor is a robust command or one of our own markers
  // (`\@internal@text@verb` from the `\verb` absorption) and stays whole —
  // only an undefined `@`-name, which could otherwise just error, is re-read.
  // No `\protect` reaches the file: the `\write` is expanded again at
  // shipout under `\let\protect\noexpand` (latex.ltx:20913 `\@outputpage`),
  // which writes each frozen `\protect X` as `X` — tikz-ext's
  // `\index{…\protect\string\protect\pgftext…}` is written `\string\pgftext`
  // and prints "\pgftext"; kept, its `\pgftext` ran as the entry was read
  // (501 `\egroup` errors). Guard:
  // `perfect_kernel_batch61::index_entry_protect_is_consumed_at_shipout`.
  let token_list: Vec<Token> = expanded?
    .unlist()
    .into_iter()
    .flat_map(|t| {
      if t.get_catcode() == Catcode::CS && t.with_str(|s| s == "\\protect") {
        vec![]
      } else if t.get_catcode() == Catcode::CS
        && t.with_str(|s| s[1..].contains('@'))
        && lookup_meaning(&t).is_none()
      {
        t.with_str(|s| mouth::tokenize(TeXString::assembled(s.to_string())).unlist())
      } else {
        vec![t]
      }
    })
    .collect();
  // Add a terminal level character if not present
  let mut toks = token_list;
  if toks
    .last()
    .map(|t| t.with_str(|s| s != chars.level))
    .unwrap_or(true)
  {
    toks.push(T_OTHER!(chars.level.as_str()));
  }
  let mut expansion: Vec<Token> = Vec::new();
  let mut phrase: Vec<Token> = Vec::new();
  let mut sortas: Vec<Token> = Vec::new();
  let mut style: Option<String> = None;
  let mut i = 0;
  // Separator chars (makeindex's quote/actual/level/encap, by default
  // `"`/`@`/`!`/`|`) act ONLY at brace depth 0. A flat
  // scan (Perl latex_constructs.pool.ltxml L4326-4350 — Perl shares this
  // byte-identically) cuts through nested groups: packdoc.sty L328/L331
  // writes `\index{#2@\PDElement{#1}{#2}\csuse{packdoc@#1@IndexRemark}}`,
  // whose in-group `@`s shredded the phrase and emitted UNBALANCED braces
  // into the live stream — one mode error + one orphaned indexphrase per
  // `\OptionInd` (algxpar-doc 162+149 errs, numerica; pdflatex clean —
  // real makeindex splits the out-of-band .idx STRING where imbalance
  // cannot corrupt the document). KNOWN_PERL_ERRORS #83.
  // Nor inside math: `\index{arroba@$@$}` (latex-via-exemplos.tex:1042
  // `\arrobasymbforindex` = `$@$`, used at :4238/:4691/:5363) is the key
  // `arroba` and the display `$@$`; splitting at the `@` inside the math
  // emitted `\@indexphrase[$]{$}` with a lone `$` that opened inline math
  // the bounded `\@index` box could never close (three errors per entry and
  // a leaked `<ltx:XMath>` swallowing the paragraph; Perl digests the lone
  // `$` silently, pdflatex is clean). A `$` at brace depth 0 toggles the
  // math state (`$$` twice: display math toggles net zero as it should).
  let mut depth: i32 = 0;
  let mut in_math = false;
  while i < toks.len() {
    let tok = toks[i];
    match tok.get_catcode() {
      Catcode::BEGIN => depth += 1,
      Catcode::END => depth -= 1,
      Catcode::MATH if depth == 0 => in_math = !in_math,
      _ => {},
    }
    if depth > 0
      || tok.get_catcode() == Catcode::END && depth == 0
      || in_math
      || tok.get_catcode() == Catcode::MATH
    {
      // Inside a group (or the closing brace returning to depth 0) or
      // inside math (or its closing `$`): plain phrase material, never a
      // separator.
      phrase.push(tok);
      i += 1;
      continue;
    }
    let s = tok.with_str(|s| s.to_string());
    i += 1;
    if s == chars.quote && i < toks.len() {
      // Escaped character: take next token literally
      phrase.push(toks[i]);
      i += 1;
    } else if s == chars.actual {
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
      // An undefined control word in the key is literal text as well:
      // makeindex sorts on the characters `\A`/`\AB` (mindsample.tex:200-208
      // `\index{\A>@…}`, `\index{\AB@…}` — manyind's sort-key idiom), and
      // digesting it can only raise `undefined` (Perl shares that error).
      // Guard: `perfect_kernel_batch56::index_sort_key_undefined_word_is_text`.
      // So is any control word typed in the source (`formed`): written
      // verbatim, the key is its characters — `\index{\key @\show}` sorts on
      // `\key`, whatever `\key` means.
      // So is an implicit brace (`\bgroup`/`\egroup`, a `\let` to `{`/`}`):
      // doc.sty:544-555 `\LeftBraceIndex`/`\RightBraceIndex` sort their entries
      // under `\bgroup`/`\egroup`, and digested they opened or closed a group
      // of the document.
      let implicit_brace = |t: &Token| {
        with_meaning(t, |m| {
          matches!(m, Some(Stored::Token(l))
            if matches!(l.get_catcode(), Catcode::BEGIN | Catcode::END))
        })
      };
      sortas = phrase
        .drain(..)
        .flat_map(|t| match t.get_catcode() {
          Catcode::MATH
          | Catcode::ALIGN
          | Catcode::PARAM
          | Catcode::SUPER
          | Catcode::SUB
          | Catcode::ACTIVE => vec![T_OTHER!(t.with_str(|s| s.to_string()))],
          Catcode::CS
            if lookup_meaning(&t).is_none() || implicit_brace(&t) || formed.contains(&t) =>
          {
            // `\textbackslash` + the name: an OTHER `\` would typeset
            // through the OT1 slot (“).
            let mut lit = vec![T_CS!("\\textbackslash")];
            t.with_str(|s| {
              lit.extend(s.chars().skip(1).map(|c| T_OTHER!(c.to_string())));
            });
            lit
          },
          _ => vec![t],
        })
        .collect();
    } else if s == chars.level || s == chars.encap {
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
        expansion.extend(reread_script_chars(std::mem::take(&mut phrase)));
        expansion.push(T_END!());
      }
      sortas.clear();
      if s == chars.encap {
        // Collect remaining tokens as style/see/seealso
        if i < toks.len()
          && toks
            .last()
            .map(|t| t.with_str(|s| s == chars.level))
            .unwrap_or(false)
        {
          // Remove the terminal level-character stopbit
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
            // An encap with arguments is `\cmd{args}{page}`; the style is
            // the command — hypdoc's `hdpindex{main}` / `hdclindex{2}{usage}`
            // (hypdoc.sty:344-364, every doc.sty index entry under
            // hyperref) are `hdpindex` / `hdclindex`. Perl keeps the whole
            // encap, a `font` no stylesheet names (`ltx_font_hdpindex{main}`).
            other => match other.split_once('{') {
              Some((cmd, _)) if !cmd.trim().is_empty() => cmd.trim().to_string(),
              _ => other.to_string(),
            },
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
  // Wrap in \@index[style][inlist]{...} (Perl: the `[style]` slot is
  // written, possibly empty, whenever a list follows it). No group around it:
  // `\@index` neutralizes the font itself, as Perl's (latex_constructs.pool.ltxml:4410);
  // a `{\normalfont …}` wrapper left `{}` in the tex= of a formula holding an `\index`.
  let mut result = vec![T_CS!("\\@index")];
  let style = style.filter(|sty| !sty.is_empty());
  if style.is_some() || inlist.is_some() {
    result.push(T_OTHER!("["));
    if let Some(sty) = style {
      // ONE token, as Perl's `T_OTHER($style)`: exploded, the braces of an
      // encap with arguments (hypdoc's `hdpindex{main}`, hypdoc.sty:344-353) were
      // digested through the OT1 slots into `hdpindex–main˝`.
      result.push(T_OTHER!(sty.as_str()));
    }
    result.push(T_OTHER!("]"));
  }
  if let Some(list) = inlist {
    result.push(T_OTHER!("["));
    result.extend(Explode!(list));
    result.push(T_OTHER!("]"));
  }
  result.push(T_BEGIN!());
  result.extend(expansion);
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
/// Maps single chars (0-9, a-z, A-Z) and numbers 10-20 to their circled Unicode equivalents; beyond Perl's table,
/// the circled numbers 21-35 (U+3251-325F) and 36-50 (U+32B1-32BF) of Enclosed CJK Letters and Months, which Perl
/// prints as "21⃝" (KNOWN_PERL_ERRORS #361).
fn unicode_enclosed_alphanumeric(text: &str) -> Option<String> {
  if let Ok(n) = text.parse::<u32>()
    && text.bytes().all(|b| b.is_ascii_digit())
    && !text.starts_with('0')
  {
    let ch = match n {
      21..=35 => char::from_u32(0x3251 + (n - 21)),
      36..=50 => char::from_u32(0x32B1 + (n - 36)),
      _ => None,
    };
    if let Some(ch) = ch {
      return Some(ch.to_string());
    }
  }
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
/// `\makebox[w][align]` horizontal alignment (Perl `%makebox_alignment` #2829:
/// l/r/c/s → left/right/center). The `s` (stretch-to-fill) case is emitted as
/// `justified` — the schema `align` enum is `left|center|right|justified`
/// (`LaTeXML-common.rnc:216`), with no `stretched`; `[s]` justifies the box's
/// inter-word glue to fill the width, which `text-align:justify` renders
/// faithfully. Perl emits the schema-invalid `stretched` here; this is an
/// intentional, schema-validity divergence (OXIDIZED_DESIGN #239).
fn makebox_alignment(key: &str) -> &'static str {
  match key {
    "l" => "left",
    "r" => "right",
    "c" => "center",
    "s" => "justified",
    _ => "",
  }
}

// The `LoadDefinitions!` body is split into per-section modules (Lamport
// Appendix-C order), called below in the original sequence: the dump is a
// definition-ORDER snapshot, so the order is load-bearing (verified by a
// bitwise dump diff at the split, 2026-09-03).
mod sect01;
pub(crate) use sect01::make_missing_float_counters;
pub use sect01::{
  CLASS_DECLARED, note_document_class, settle_chapter_after_class_load,
  store_chapter_after_latex_ix_class_load,
};
mod sect02;
mod sect03;
mod sect04;
pub use sect04::kernel_chapter_with_warning;
mod sect05;
pub use sect05::note_title_page_part;
pub(crate) use sect05::{
  SpeculativeRun, replay_class_begin_code, speculative_title_page_run, title_page_deposit_tokens,
  title_page_print_tokens, title_page_replay_tokens,
};
mod reference_math_slots;
mod sect06;
mod sect07;
mod sect08;
mod sect09;
mod sect10;
mod sect11;
mod sect12;
mod sect13;
use reference_math_slots::{ReferenceSymbol, reference_math_slot};
pub use sect04::toc_select;
pub use sect07::prepare_latex_equation_counter;
pub use sect08::{declare_bound_text_symbols, declare_capital_greek, declare_capital_greek_named};
pub use sect10::math_array_bindings;
pub use sect11::{
  brace_groups, external_label_field, external_label_tex, external_ref_properties,
  external_ref_properties_with, label_type, read_back_aux_text, record_external_cref,
  record_external_label, record_newlabel, settle_empty_held_tags, typed_label,
};
pub use sect13::nfss_selected_font;

/// amsmath.sty:155-175 declares the spacing commands again — `\,` `\!` `\:` `\;`, `\negmedspace`, `\negthickspace`
/// (`\let\!\@undefined` …, then `\DeclareRobustCommand\!{\tmspace…}`) — over whatever a package loaded before
/// made of them: tipa.sty:429-458's T3 text accents `\!` `\;` `\:`, which would read the next token as their
/// argument (`\!\! &`). The amsmath binding calls this; the commands are the kernel's (plain_constructs.rs,
/// sect07.rs). Witnesses 2603.03017, 2605.00529, 2608.02769; guards `perfect_kernel_batch77x::t3_tipa_then_amsmath_*`.
pub fn define_amsmath_tmspace_commands() -> Result<()> {
  crate::plain_constructs::define_tmspace_commands()?;
  sect07::define_negative_tmspaces()
}

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
    let delim = T_ACTIVE!(init_ch);
    // latex.ltx:15504-15510 `\verb@eol@error`: `\obeylines` makes the end of
    // the line active, and that `^^M` closes the verb group with ONE
    // recoverable "\verb ended by end of line" error. Perl's `readUntil`
    // (latex_constructs.pool.ltxml:1797) has no such bound, so an unterminated
    // `\verb v1.1 )` scanned across lines and swallowed a later `{verbatim}`
    // (bigints manual, 10-error cascade; SHARED, pdflatex recovers with one
    // error). Guard: `perfect_kernel_batch56::verb_ended_by_end_of_line_recovers`.
    assign_catcode('\r', Catcode::ACTIVE, None);
    let eol = T_ACTIVE!('\r');
    let par = T_CS!("\\par");
    // tabularx's `\TX@verb` (tabularx.sty:91, :205-209) reads up to the delimiter token as it stands, so within a
    // tabularx a `\verb|…|` in an argument read before it (`\makecell{\verb|x|}`, the `|` frozen "other") still ends
    // there (2404.06128, 2309.04295); the kernel `\verb` needs it active.
    let token_delimited = lookup_bool("lx@tabularx@verb");
    let mut body = Vec::new();
    let mut terminated = false;
    while let Some(token) = read_token()? {
      if token == delim
        || (token_delimited
          && token.get_catcode() == Catcode::OTHER
          && token.with_str(|s| s.starts_with(init_ch) && s.chars().count() == 1))
      {
        terminated = true;
        break;
      }
      if token == eol || token == par {
        Error!("unexpected", "\\verb", "\\verb ended by end of line", "");
        terminated = true;
        break;
      }
      body.push(token);
    }
    if !terminated {
      // Perl `Gullet::readUntil` (Core/Gullet.pm:683-685): ran out of tokens
      // before the delimiter — unread everything scanned and return an EMPTY
      // body. That is where a `\verb` inside a pre-tokenized argument
      // (`\footnote{… \verb+x+ …}`) lands: the closing `+` was frozen as OTHER
      // when the argument was read, so the ACTIVE delimiter never matches and
      // the scan hits the end of the argument. Keeping the scanned tokens as
      // the body swallowed the rest of the note (an `<item>` inside
      // `<verbatim>`; platex-tools/plarray.tex:20, RUST-ONLY). Guard:
      // `verb_inside_pretokenized_argument_unreads_instead_of_running_away`.
      unread(Tokens::new(std::mem::take(&mut body)));
    }
    let body = Tokens::new(body);
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
      // Exactly the two tag constructors (`\lx@tag`, `\lx@tags`), by
      // identity — not any internal name containing `lx@tag`.
      let is_tag = {
        let whatsit = w.borrow();
        let name = whatsit.get_definition().get_cs_name().into_owned();
        name == "\\lx@tag" || name == "\\lx@tags"
      };
      if is_tag {
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
