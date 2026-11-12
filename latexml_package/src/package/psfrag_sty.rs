//! psfrag.sty — PostScript fragment overlays on EPS images
//! Perl: psfrag.sty.ltxml — 166 lines
//! Saves the `\psfrag` fragments for the EPS graphics in their scope; they are not rendered yet (PSFR), so each
//! graphic counts its fragments by a Warning (`psfrag_graphic`).
use latexml_core::binding::virtual_files::vfs_read;

use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // psfrag.sty:35 `\RequirePackage{graphics}`, before its options: a paper that loads psfrag has `\includegraphics`
  // and `\scalebox` without loading graphics itself (Perl's binding lacks the load, KNOWN_PERL_ERRORS #692; witnesses
  // 0801.1195, 0802.1229, 0704.2560).
  RequirePackage!("graphics");
  // Perl L24-27: the fragments saved for the next EPS graphic (local, as psfrag's `\pfg@pcount`), and the scan
  // switches (psfrag_scan_all from the 2.09_COMPATIBILITY of a `\documentstyle`).
  AssignValue!("saved_psfragments" => Tokens::new(Vec::new()));
  AssignValue!("psfrag_scan_all" => lookup_bool("2.09_COMPATIBILITY"));
  AssignValue!("psfrag_scan"     => 0i32);

  // Options — Perl L28-32. Each declared option toggles psfrag_scan_all:
  //   209mode, scanall → true
  //   2emode           → false
  DeclareOption!("209mode", { AssignValue!("psfrag_scan_all" => true); });
  DeclareOption!("2emode",  { AssignValue!("psfrag_scan_all" => false); });
  DeclareOption!("scanall", { AssignValue!("psfrag_scan_all" => true); });
  ProcessOptions!();

  // Perl L46-53: `\psfrag` saves its fragment for the graphic's hook (`psfrag_graphic`); an empty `$$` replacement
  // saves none. It leaves the empty box a `None` primitive does, so `tex=` keeps the `\psfrag` (Perl's sub drops it).
  DefPrimitive!("\\psfrag OptionalMatch:* Semiverbatim [][][][]{}", sub[(star, tag, a, b, c, d, text)] {
    let args = vec![star, Some(tag), a, b, c, d, Some(text)];
    let reversion = Invocation!(T_CS!("\\psfrag"), args.clone());
    let mut fragment = args;
    let math = |t: &Token| t.get_catcode() == Catcode::MATH;
    let empty_math = |t: &Tokens| matches!(&t.unlist_ref()[..], [x, y] if math(x) && math(y));
    if fragment[6].as_ref().is_some_and(empty_math) {
      fragment[6] = Some(Tokens::new(Vec::new()));
    }
    save_psfrag(Invocation!(T_CS!("\\lx@delayed@psfrag"), fragment).unlist());
    Ok(vec![Digested::from(Tbox::new(pin!(""), None, None, reversion, stored_map!("isEmpty" => true)))])
  });
  DefConstructor!("\\lx@delayed@psfrag OptionalMatch:* Semiverbatim [][][][]{}", "",
    alias => "\\psfrag");

  // Perl L55-62 (constructors with an empty template there: they typeset nothing)
  DefPrimitive!("\\psfragscanon", {
    save_psfrag(vec![T_CS!("\\psfragscanon")]);
    AssignValue!("psfrag_scan" => 1i32);
  });
  DefPrimitive!("\\psfragscanoff", {
    save_psfrag(vec![T_CS!("\\psfragscanoff")]);
    AssignValue!("psfrag_scan" => 0i32);
  });

  // Rescan/debug macros — Perl psfrag.sty.ltxml.
  // NOTE: do NOT define `\tex`. Real psfrag.sty does NOT define `\tex` as a
  // LaTeX macro (its only `\tex` is `\string\\tex…` written verbatim into the
  // .ps file for PostScript-side processing, psfrag.sty L201), and neither
  // does Perl LaTeXML's psfrag binding. A prior Rust-only
  // `DefMacro!("\\tex Semiverbatim", "#1")` intercepted papers that load
  // psfrag but `\newcommand{\tex}{…}` for their own purposes — e.g.
  // 1505.01267 does `\newcommand{\tex}{\textstyle}` and writes
  // `\tex{…c_0…\mbox{as $\hat s\to\infty$}…}` inside display math. With the
  // Rust `\tex` winning (newcommand ignored) and reading its arg Semiverbatim,
  // `_` was neutralized to OTHER (literal `\_`) and the inline-math `\to`
  // built an `ltx:XMApp` that leaked into the surrounding `ltx:text` →
  // "ltx:XMApp isn't allowed in <ltx:text>". Leaving `\tex` undefined (Perl-
  // faithful) lets the author's `\newcommand` win, so `\tex{…}`=`\textstyle{…}`
  // digests as math.
  def_macro_noop("\\psfragrescan")?;
  def_macro_noop("\\psfragrescanoff")?;
  def_macro_noop("\\psfragrescanon")?;
  def_macro_noop("\\psfragdebugon")?;
  def_macro_noop("\\psfragdebugoff")?;

  // Perl psfrag.sty.ltxml L149: DefEnvironment('{psfrags}', '#body').
  // Pure grouping, no content transform. Previously unported.
  DefEnvironment!("{psfrags}", "#body");
  // psfrag.sty:151 `\newwrite\pfg@temp`: the write-stream allocation the
  // binding skipped shifted every later `\newwrite` down by one, so psfragx's
  // `\newread\pfx@in` and `\newwrite\pfx@out` (psfragx.sty:62-63) shared the
  // number 3 and its `\ifeof\pfx@out` guard read the open input stream
  // ("psfragx: output file already exists!", psfragx_example; SHARED with
  // Perl's binding, pdflatex clean).
  RawTeX!(r"\ifdefined\pfg@temp\else\newwrite\pfg@temp\fi");
});

/// Perl `save_psfrag` (L34-39): appends to the saved fragments, NOT globally, the leading control sequence deferred.
fn save_psfrag(stuff: Vec<Token>) {
  let mut saved = lookup_tokens("saved_psfragments")
    .map(Tokens::unlist)
    .unwrap_or_default();
  saved.push(T_CS!("\\noexpand"));
  saved.extend(stuff);
  assign_value("saved_psfragments", Tokens::new(saved), None);
}

/// The hooks' `afterDigest` (Perl L82-87, L105-110, L127-132) for a graphic `whatsit` of any including command
/// (`\includegraphics`, `\epsfbox` and its `\let` copies, `\psfig`). An EPS graphic (`psfrag_requirepicture`) is
/// where psfrag places the saved fragments (psfrag.sty:46-62 `\Ginclude@eps`/`\epsfbox`) and, under a scan, the
/// `(\\tex` strings of the file (psfrag.sty:152-209 `\pfg@scan`); they are not rendered (PSFR), so each counts as a
/// Warning. The scan stays on until `\psfragscanoff` (psfrag.sty:24, latex + dvips), where Perl resets it per graphic.
pub fn psfrag_graphic(whatsit: &Whatsit) {
  let Some(saved) = lookup_tokens("saved_psfragments") else {
    return;
  };
  let mut graphic = whatsit.get_property_string("graphic");
  if graphic.is_empty() {
    graphic = whatsit.get_property_string("path");
  }
  let Some(eps) = eps_file(&whatsit.get_property_string("candidates"), graphic.trim()) else {
    return;
  };
  let scan = lookup_bool("psfrag_scan") || lookup_bool("psfrag_scan_all");
  let tags = fragment_tags(&saved);
  for tag in &tags {
    Warn!(
      "missing_feature",
      "psfrag",
      s!("The psfrag replacement of '{tag}' in the graphic '{graphic}' is not rendered")
    );
  }
  if scan {
    let content = eps.unwrap_or_else(|file| std::fs::read(file).unwrap_or_default());
    let embedded = content.windows(6).filter(|w| w == br"(\\tex").count();
    for _ in 0..embedded {
      Warn!(
        "missing_feature",
        "psfrag",
        s!(
          "A psfrag replacement embedded in the graphic '{graphic}' (\\psfragscanon) is not rendered"
        )
      );
    }
  }
}

/// The EPS file the graphic includes: a `.eps`/`.ps` candidate (`Err`, its path), else a `{filecontents}` file of the
/// graphic's name or that name with dvips.def's extensions `.eps`, `.ps` (`Ok`, its content); `None` for any other.
fn eps_file(
  candidates: &str,
  graphic: &str,
) -> Option<std::result::Result<Vec<u8>, std::path::PathBuf>> {
  let is_eps = |name: &str| {
    let ext = std::path::Path::new(name)
      .extension()
      .map(|e| e.to_string_lossy().to_lowercase());
    matches!(ext.as_deref(), Some("eps" | "ps"))
  };
  if let Some(found) = candidates.split(',').map(str::trim).find(|c| is_eps(c)) {
    return Some(Err(
      std::path::Path::new(&lookup_string("SOURCEDIRECTORY")).join(found),
    ));
  }
  let names = [graphic.to_string(), s!("{graphic}.eps"), s!("{graphic}.ps")];
  names
    .iter()
    .filter(|name| is_eps(name))
    .find_map(|name| vfs_read(name))
    .map(|text| Ok(text.into_bytes()))
}

/// The tag of each saved `\lx@delayed@psfrag` invocation: its first braced argument, after an optional `*`.
fn fragment_tags(saved: &Tokens) -> Vec<String> {
  let tokens = saved.unlist_ref();
  let mut tags = Vec::new();
  for (i, t) in tokens.iter().enumerate() {
    if t.get_catcode() != Catcode::CS || t.with_str(|s| s != "\\lx@delayed@psfrag") {
      continue;
    }
    let mut rest = tokens[i + 1..]
      .iter()
      .skip_while(|t| t.get_catcode() == Catcode::OTHER && t.with_str(|s| s == "*"));
    if rest
      .next()
      .is_none_or(|t| t.get_catcode() != Catcode::BEGIN)
    {
      continue;
    }
    let mut depth = 0usize;
    let tag: Vec<Token> = rest
      .take_while(|t| match t.get_catcode() {
        Catcode::BEGIN => {
          depth += 1;
          true
        },
        Catcode::END if depth == 0 => false,
        Catcode::END => {
          depth -= 1;
          true
        },
        _ => true,
      })
      .cloned()
      .collect();
    tags.push(Tokens::new(tag).untex());
  }
  tags
}
