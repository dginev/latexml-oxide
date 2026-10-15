use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: aastex.cls.ltxml — AAS TeX document class

  // Ignorable options
  //
  // Note on `revtex4`: Perl #2698 (2026) makes it an explicit no-op
  // because the class now loads revtex4 unconditionally anyway. Adding
  // it here prevents the option from falling through to the article
  // fallback below and getting spuriously flagged.
  for option in [
    "10pt", "11pt", "12pt",
    "manuscript", "preprint", "preprint2", "longabstract",
    "tighten", "landscape",
    "aasms4", "aaspp4", "aas2pp4", "aj_pt4", "apjpt4", "astro",
    "flushrt", "anonymous",
    "revtex4",
  ].iter() {
    DeclareOption!(*option, None);
  }

  // Number equations within sections
  DeclareOption!("eqsecnum", "\\AtEndOfClass{\\eqsecnum}");

  // Anything else is for article
  DeclareOption!(None, {
    Digest!("\\PassOptionsToClass{\\CurrentOption}{article}")?;
  });

  ProcessOptions!();

  load_class("revtex4", Vec::new(), Tokens!())?;
  // Every AASTeX numbers its sections in arabic (aastex.cls 5.x:1076-1093, aastex63.cls:1117, aastex701.cls:7629-7636),
  // as does emulateapj.cls:676, where revtex4's are Roman (revtex4_support_sty.rs `\thesection`): every AAS paper's
  // section numbers and `\ref`s read "I", "I.1" (2609.21324, astro-ph/0003209; Perl the same: aastex.cls.ltxml:44 +
  // revtex4_support.sty.ltxml:111; KNOWN_PERL_ERRORS #557).
  RawTeX!(r"\def\thesection{\arabic{section}}\def\thesubsection{\thesection.\arabic{subsection}}\def\thesubsubsection{\thesubsection.\arabic{subsubsection}}\def\theparagraph{\thesubsubsection.\arabic{paragraph}}");
  // The AASTeX version the document asked for: a versioned fallback's request (`aastex701`, `aastex61` answered by this
  // binding), else the name this class loads under (`aastex`, `aastex63`, `aastex7`; emulateapj_cls.rs loads it as
  // `aastex`, then undefines both versions' own commands).
  let request = lookup_string("fallback_request");
  let requested = if request.is_empty() {
    Expand!(Tokens!(T_CS!("\\@currname"))).to_string()
  } else {
    request.rsplit('/').next().unwrap_or_default().trim_end_matches(".cls").to_string()
  };
  // AASTeX 6 and later carry their version in the class name (aastex6, aastex61 … aastex701); plain `aastex` is 5.x,
  // unless the paper ships its own `aastex.cls` that is a 6+ class under the old name (aastex6.cls:85
  // `\ProvidesClass{aastex}`; a 6+ class has `\gridline`; 1810.04684, 2010.06977, 2103.01741).
  let aastex6_plus = requested
    .strip_prefix("aastex")
    .is_some_and(|version| version.starts_with(|c: char| c.is_ascii_digit()))
    || bundled_aastex_is_6_plus();
  // aastex7.cls:11484 / aastex701.cls:11416 `\usepackage[figuresright]{rotating}`, ahead of their own `\rotate`
  // (:12264 / :12196), which a deluxetable's `\rotate` (deluxetable_sty.rs) must stay; aastex631 and earlier do not load
  // it. Loaded here for every AASTeX 7 name, direct (`aastex7`, `aastex70`) or through the versioned fallback
  // (`aastex701`): read from the class file after this binding, rotating's `{rotate}` environment replaced the class's
  // `\rotate`, and `\rotate` in a `deluxetable*` opened a box that never closed (2609.09266).
  if requested.starts_with("aastex7") {
    RequirePackage!("rotating", options => vec![s!("figuresright")]);
  }
  // AASTeX 6.x is REVTeX 4.1 underneath (aastex62.cls:278-292 `\LoadClass[…]{revtex4-1}`), and aastex701.cls:3689-3718
  // carries the same float repair: `\newfloat@ltx`, which a preamble lets `\newfloat` to (2002.07183; KNOWN_PERL_ERRORS
  // #683). 5.x is REVTeX 4.0, which has none.
  if aastex6_plus {
    revtex4_support_sty::define_float_repair()?;
  }
  RequirePackage!("aas_support");
  // AASTeX 5.x's `{plate}` (aastex.cls 5.x:1685-1704) is gone from 6 on (aastex6.cls … aastex701.cls define no plate):
  // there, the name is the paper's own — tikz's bayesnet library `\newcommand{\plate}[4][]`
  // (tikzlibrarybayesnet.code.tex:85), which the binding's environment refused, and `\plate` in a tikzpicture opened a
  // float (witnesses 1909.00812, 2006.16258, 2305.06275).
  if aastex6_plus {
    undefine_plate();
  }
  // AASTeX 7 prints an `\email` only with `[show]` (in the title footnote "Email: …", aastex701.cls:2455-2456) or after
  // `\correspondingauthor`/`\cofirstauthor` (a footnote, :13341-13353); any other address it discards, keeping only that
  // the author has one (:13350, for the check at :12441) (2608.05283, 2608.21320). The address is kept as the author's
  // `schema:email` metadata (OXIDIZED_DESIGN #479). A corresponding author's address is that author's, wherever it
  // stands (2608.02190 ahead of the authors, 2608.00173 after them). AASTeX 6.x prints every `\email`
  // (aastex631.cls:6924-6931, the copy 2505.20669 ships), as aas_support_sty.rs's `\email` does (Perl
  // aas_support.sty.ltxml:122 for every version; KNOWN_PERL_ERRORS #559).
  // AASTeX 6+ sets `\homepage` with `\href{#2}{#2}` in the current font — 6.x through REVTeX 4.1
  // (aastex631.cls:284-308, the copy 2101.06145 ships; revtex4-1.cls:2387-2390), 7 itself
  // (aastex701.cls:2213-2216) — not in REVTeX 4.0's typewriter type, which revtex4_cls.rs installs (this binding loads
  // under that name)
  if aastex6_plus {
    DefMacro!("\\homepage[] Semiverbatim", "\\lx@add@url[name={#1}]{\\lx@address@textfont{#2}}");
  }
  if requested.starts_with("aastex7") {
    // (one `\lx@add@` call, so an author line holding `\email{…}` reads it as the class's frontmatter command:
    // `adds_to_frontmatter`)
    DefMacro!("\\email[]{}", "\\lx@add@aas@email{#1}{#2}");
    // `\cofirstauthor` (aastex701.cls:13330-13337) sets the flag and prints nothing; AASTeX 7's only, so a 6.x paper's own
    // `\newcommand\cofirstauthor` stands.
    DefMacro!("\\cofirstauthor", "\\global\\lx@aas@correspauthortrue\\global\\let\\lx@aas@correspondent\\empty");
    DefMacro!("\\lx@add@aas@email{}{}",
      "\\iflx@aas@correspauthor\\expandafter\\lx@aas@email@corresp\\else\\expandafter\\lx@aas@email@author\\fi{#1}{#2}\\global\\lx@aas@correspauthorfalse");
    DefMacro!("\\lx@aas@email@corresp{}{}",
      "\\ifx\\lx@aas@correspondent\\empty\\lx@add@email{\\lx@address@textfont{#2}}\\else\\lx@add@email[label={fuzzy:\\lx@aas@correspondent}]{\\lx@address@textfont{#2}}\\fi");
    DefMacro!("\\lx@aas@email@author{}{}", sub[(option, address)] {
      // (a printed address in the current font, aastex701.cls:2455-2456 and :13348; the metadata is its text;
      // 2501.12613's `j_serna@…`)
      Ok(if option.to_string().trim() == "show" {
        Invocation!(T_CS!("\\lx@add@email"), vec![None, Some(Invocation!(T_CS!("\\lx@address@textfont"), vec![Some(address)]))])
      } else {
        Invocation!(T_CS!("\\lx@add@email@metadata"), vec![None, Some(address)])
      })
    });
  }
  // One binding for every AASTeX version, the union of their commands (user directives 2026-10-09, 2026-10-10: the 5.x
  // `\subsubsubsection`/`\supportfrom` stay for a 6+ class too, where a paper's own `\newcommand` of them is skipped and
  // its text kept) — dispatched by version only where a version's command would take the paper's own away:
  if !aastex6_plus {
    // AASTeX 5.x is an article class (aastex.cls:136) whose `\email` sets the address in `\tt` (:841, :903-905 in the
    // 5.2 copy 1001.0026 ships): typewriter type, not the current font of the 6+ classes (aas_support_sty.rs).
    // emulateapj, which loads this binding, sets it in roman (emulateapj_cls.rs).
    DefMacro!("\\email[]{}", "\\lx@add@email{\\lx@address@urlfont{#2}}");
    // The `\fig` family and `\gridline`: AASTeX 6+ only (aastex6.cls:4909). A 5.x paper's `\fig` is its own:
    // astro-ph/0003209 `\newcommand{\fig}…` now prints "Fig. <ref>", where the binding's `\fig` read the argument as a
    // `\ref` of the wrong label (64e review r2).
    Let!("\\fig", "\\@undefined");
    Let!("\\leftfig", "\\@undefined");
    Let!("\\rightfig", "\\@undefined");
    Let!("\\boxedfig", "\\@undefined");
    Let!("\\rotatefig", "\\@undefined");
    Let!("\\gridline", "\\@undefined");
  }
  // aastex701.cls:13637-13638 — `\digitalasset` flags a digital-asset paper
  // (aastex701-sample; the Perl reimplementation lacks it too).
  RawTeX!(r"\newif\ifdigitasset\def\digitalasset{\digitassettrue}");
  // aastex701.cls:13494-13497 — `\centerwidetable` centers a wide deluxetable instead of setting it sideways, a page
  // layout flag (2609.06985).
  RawTeX!(r"\newif\ifcenterwidetable\def\centerwidetable{\global\centerwidetabletrue}");
  // aastex701.cls:11756, 13690 — the offsets a table is moved by on its page (`\movetabledown=2cm`, 2609.00308).
  RawTeX!(r"\newdimen\movetabledown\newdimen\movetableright");
});

/// Whether the paper ships an `aastex.cls` that is an AASTeX 6+ class under the old name — one that defines `\gridline`
/// (aastex6.cls:85 `\ProvidesClass{aastex}`); TeX Live has none, so the file found is the paper's own.
fn bundled_aastex_is_6_plus() -> bool {
  find_file(
    "aastex",
    Some(FindFileOptions {
      ext_type: Some("cls".into()),
      ..FindFileOptions::default()
    }),
  )
  .and_then(|path| std::fs::read(path).ok())
  .is_some_and(|bytes| String::from_utf8_lossy(&bytes).contains("\\gridline"))
}

/// Leave `{plate}` and `{plate*}` (`\begin{plate}`, `\end{plate}`, `\plate`, `\endplate`; aas_support_sty.rs) undefined,
/// for an AAS class that has no plate: AASTeX 6 and later, emulateapj.
pub fn undefine_plate() {
  for command in [
    "\\plate",
    "\\endplate",
    "\\begin{plate}",
    "\\end{plate}",
    "\\plate*",
    "\\endplate*",
    "\\begin{plate*}",
    "\\end{plate*}",
  ] {
    Let!(T_CS!(command), "\\@undefined");
  }
}
