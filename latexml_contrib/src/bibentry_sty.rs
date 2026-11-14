//! bibentry.sty — bibliography entries in the text (Patrick W. Daly, 2007/10/30 v1.5).
//!
//! `\nobibliography{<db>}` (or `\nobibliography*` before a later `\bibliography`) reads the `.bbl` in a store mode:
//! each `\bibitem` keeps its entry's text as `\BR@r@<key>` and the list prints nothing (bibentry.sty:44-62,
//! `\BR@nobib`'s `{thebibliography}` and `\BR@c@bibitem#1 #2 \par`); `\bibentry{<key>}` cites the key and prints that
//! text without its final period (:63-66, `\BR@nodot`). Raw, the store mode cannot run: its `\renewenvironment
//! {thebibliography}` meets the kernel's locked bibliography (sect11.rs), so `\nobibliography*` printed the whole list
//! and each `\bibentry` printed nothing ("Paper: ." for pdflatex's "Paper: A. Able. First paper. 2001."). Perl has no
//! binding and leaves both commands undefined (KNOWN_PERL_ERRORS #1059).
//!
//! The binding reads the `<jobname>.bbl` itself, as `\bibliography`'s `\@input@{\jobname.bbl}` would, and keeps each
//! entry's tokens up to the blank line that ends it. `\nobibliography{<db>}` without the star still builds the list
//! where it stands: in TeX its `\bibitem`s write the labels that resolve every `\cite`, and here the list is what the
//! citations resolve to (OXIDIZED_DESIGN_DIVERGENCES #831). Witnesses 2210.16098 (AAAI, 48 `\bibentry` lines after
//! `\nobibliography{aaai23}`), 2505.07606 (`\nobibliography*` then `\bibliography`, 29 entries). Guards
//! `perfect_kernel_batch78k::bibentry_*`.
use latexml_package::prelude::*;

/// The entries of a `.bbl`'s token list, as `(key, text)`: each `\bibitem[<label>]{<key>}` and the tokens after it up
/// to the paragraph end (`\par`, a blank line), the next `\bibitem` or the list's `\end`, at brace depth 0; the text
/// trimmed of spaces and of one final period (bibentry.sty:63 `\BR@nodot`).
fn bbl_entries(tokens: &[Token]) -> Vec<(String, Vec<Token>)> {
  let bibitem = T_CS!("\\bibitem");
  let ends = [
    T_CS!("\\par"),
    bibitem,
    T_CS!("\\end"),
    T_CS!("\\endthebibliography"),
  ];
  let is_space = |t: &Token| t.get_catcode() == Catcode::SPACE;
  let mut entries = Vec::new();
  let mut i = 0;
  while i < tokens.len() {
    if tokens[i] != bibitem {
      i += 1;
      continue;
    }
    i += 1;
    while i < tokens.len() && is_space(&tokens[i]) {
      i += 1;
    }
    // the optional label, `[…]` at brace depth 0
    if i < tokens.len() && tokens[i] == T_OTHER!("[") {
      let mut depth = 0i32;
      while i < tokens.len() {
        match tokens[i].get_catcode() {
          Catcode::BEGIN => depth += 1,
          Catcode::END => depth -= 1,
          _ if depth == 0 && tokens[i] == T_OTHER!("]") => {
            i += 1;
            break;
          },
          _ => {},
        }
        i += 1;
      }
    }
    while i < tokens.len() && is_space(&tokens[i]) {
      i += 1;
    }
    // the key, a group or a single token
    let mut key = Vec::new();
    if i < tokens.len() && tokens[i].get_catcode() == Catcode::BEGIN {
      let mut depth = 1i32;
      i += 1;
      while i < tokens.len() {
        match tokens[i].get_catcode() {
          Catcode::BEGIN => depth += 1,
          Catcode::END => depth -= 1,
          _ => {},
        }
        if depth == 0 {
          i += 1;
          break;
        }
        key.push(tokens[i]);
        i += 1;
      }
    } else if i < tokens.len() {
      key.push(tokens[i]);
      i += 1;
    }
    let key = Tokens::new(key).to_string().trim().to_string();
    // the entry's text
    let mut text = Vec::new();
    let mut depth = 0i32;
    while i < tokens.len() {
      let t = tokens[i];
      if depth == 0 && ends.contains(&t) {
        break;
      }
      match t.get_catcode() {
        Catcode::BEGIN => depth += 1,
        Catcode::END => depth -= 1,
        _ => {},
      }
      text.push(t);
      i += 1;
    }
    while text.first().is_some_and(is_space) {
      text.remove(0);
    }
    while text.last().is_some_and(is_space) {
      text.pop();
    }
    if text.last() == Some(&T_OTHER!(".")) {
      text.pop();
    }
    if !key.is_empty() {
      entries.push((key, text));
    }
  }
  entries
}

LoadDefinitions!({
  // bibentry.sty:42-82: the commands, the store mode a Rust primitive (`\lx@BR@store`, below). The store mode's own
  // `\BR@b@bibitem`, `\BR@c@bibitem` and `\BR@nodot` (:58-63) are defined as there, for code that redefines them:
  // tufte-common.def:876-882 `\renewcommand\BR@b@bibitem` before its own `\nobibliography*`. The star records that a
  // `\bibliography` follows; bibentry's `\def\bibliography` wrapper that resets it (:74-75) is refused by the kernel's
  // lock, so the `\AtEndDocument` warning (:76-79) asks the kernel's count of `\bibliography` calls instead
  // (`lx_bibliography_calls`, sect11.rs `\lx@ifusebbl`) whether one came after the last `\nobibliography`.
  RawTeX!(
    r"\newif\ifBR@star
\newcommand\nobibliography{\@ifstar{\BR@nobib{}}{\BR@nobib}}
\newcommand\BR@nobib[1]{\ifx\relax#1\relax\global\BR@startrue\else\global\BR@starfalse\fi
\lx@BR@store\ifBR@star\else\bibliography{#1}\fi\lx@BR@mark}
\newcommand\bibentry[1]{\nocite{#1}{\frenchspacing
\hyper@natanchorstart{#1\@extra@b@citeb}\lx@BR@entry{#1\@extra@b@citeb}\hyper@natanchorend}}
\newcommand\BR@b@bibitem[2][]{\ifx\relax#1\relax\BR@bibitem{#2}\else \BR@bibitem[#1]{#2}\fi \BR@c@bibitem{#2}}
\def\BR@c@bibitem#1 #2 \par{{\let\protect\@unexpandable@protect
\expandafter \gdef\csname BR@r@#1\@extra@b@citeb\endcsname{\BR@nodot#2\relax.\relax\relax}}}
\def\BR@nodot#1.\relax#2\relax{#1}
\AtEndDocument{\ifBR@star\lx@BR@ifnobibliography{\PackageWarningNoLine{bibentry}
{You have used \string\nobibliography* \MessageBreak
without a following \string\bibliography.\MessageBreak
You may not be able to run BibTeX}}\fi}
\providecommand\@extra@b@citeb{}
\providecommand\hyper@natanchorstart[1]{}
\providecommand\hyper@natanchorend{}
\AtBeginDocument{\providecommand*{\urlprefix}{URL }\providecommand*{\url}[1]{\texttt{#1}}}
\AtBeginDocument{\@ifpackageloaded{url}{\providecommand{\doi}{doi: \begingroup \urlstyle{rm}\Url}}
{\providecommand{\doi}[1]{doi: #1}}}"
  );

  // The store mode: the `<jobname>.bbl` `\bibliography` would read (`\lx@ifusebbl`'s bbl route), each entry kept
  // globally as `\BR@r@<key>` (bibentry.sty:60-62 `\gdef\csname BR@r@#1\endcsname{…}`). Without a `.bbl` (a paper
  // shipping only its `.bib`) nothing is stored, and `\bibentry` reports each key it cannot print.
  DefPrimitive!("\\lx@BR@store", sub[_args] {
    let jobname = Expand!(T_CS!("\\jobname")).to_string();
    let Some(path) = find_file(
      &jobname,
      Some(FindFileOptions {
        ext_type: Some(Cow::Borrowed("bbl")),
        ..Default::default()
      }),
    ) else {
      Info!("expected", "bbl", s!("bibentry: no {jobname}.bbl, so no entry is stored for \\bibentry"));
      return Ok(Vec::new());
    };
    let text = vfs_read(&path).or_else(|| {
      std::fs::read(&path)
        .ok()
        .map(|b| String::from_utf8_lossy(&b).into_owned())
    }).unwrap_or_default();
    let tokens = mouth::tokenize(TeXString::assembled(text));
    for (key, entry) in bbl_entries(tokens.unlist_ref()) {
      def_macro(T_CS!(s!("\\BR@r@{key}")), None, Tokens::new(entry), Some(ExpandableOptions { scope: Some(Scope::Global), ..Default::default() }))?;
    }
  });

  // The count of `\bibliography` calls when `\nobibliography` ran, and whether none came since.
  DefPrimitive!("\\lx@BR@mark", sub[_args] {
    assign_value("BR_bibliography_calls", Stored::Int(lookup_int("lx_bibliography_calls")), Some(Scope::Global));
  });
  DefMacro!("\\lx@BR@ifnobibliography{}", sub[(warning)] {
    Ok(if lookup_int("lx_bibliography_calls") == lookup_int("BR_bibliography_calls") { warning } else { Tokens!() })
  });

  // bibentry.sty:66 `\@nameuse{BR@r@#1\@extra@b@citeb}`: the stored text; a key with none prints nothing, as there,
  // and is reported (pdflatex reports the undefined citation of its `\nocite`).
  DefMacro!("\\lx@BR@entry{}", sub[(key)] {
    let key = Expand!(key).to_string();
    let cs = T_CS!(s!("\\BR@r@{key}"));
    if has_meaning(&cs) {
      Ok(Tokens!(cs))
    } else {
      Warn!("missing", "bibentry", s!("bibentry: no stored entry for key '{key}' (no \\nobibliography or no .bbl entry)"));
      Ok(Tokens!())
    }
  });
});
