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
  RequirePackage!("aas_support");
  // One binding for every AASTeX version, the union of their commands (user directives 2026-10-09, 2026-10-10: the 5.x
  // `\subsubsubsection`/`\supportfrom` stay for a 6+ class too, where a paper's own `\newcommand` of them is skipped and
  // its text kept) — dispatched by version only where a version's command would take the paper's own away:
  if !aastex6_plus {
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
