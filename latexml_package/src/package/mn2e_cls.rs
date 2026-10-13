use latexml_core::definition::origin::{DefinitionOrigin, OriginGuard};

use crate::{prelude::*, shipped_class::shipped_class_text};

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: mn2e.cls.ltxml

  // Generally ignorable options
  for option in [
    "draft", "twocolumn", "onecolumn", "letters", "landscape", "galley",
    "referee", "doublespacing",
  ].iter() {
    DeclareOption!(*option, None);
  }

  DeclareOption!("usenatbib", {
    AssignValue!("@usenatbib" => 1i64);
  });
  // `usedcolumn`, which mn2e_support reads as `@usedcolumn`; Perl mn2e.cls.ltxml:24 declares `usedcolum` (KNOWN_PERL_ERRORS
  // #500). The misspelling is kept as an alias.
  for option in ["usedcolumn", "usedcolum"] {
    DeclareOption!(option, {
      AssignValue!("@usedcolumn" => 1i64);
    });
  }
  DeclareOption!("usegraphicx", {
    AssignValue!("@usegraphicx" => 1i64);
  });
  DeclareOption!("useAMS", {
    AssignValue!("@useAMS" => 1i64);
  });

  // Anything else is for article.
  DeclareOption!(None, {
    Digest!("\\PassOptionsToClass{\\CurrentOption}{article}")?;
  });

  ProcessOptions!();
  load_class("article", Vec::new(), Tokens!())?;
  RequirePackage!("mn2e_support");
  // mn2e.cls copies with the one "2001/02/06 v2.2" stamp differ in the journal abbreviations `.bbl`s from the
  // astronomy `.bst`s write (`\apj`, `\mnras`): the common 1574-line copy defines none, a 1695-line one
  // `\def\apj{ApJ}%` and the rest of the AAS set (0804.2668; 92 of the run-336 papers whose first error was one ship
  // it), the 1696-line one the same set on `\ref@jnl` (0711.3034, mn2e.cls:1575-1686). So they follow the copy the
  // paper ships: its own definition lines of those names, none when it has none (a document's own
  // `\newcommand{\apjl}{ApJL}` then stands, 0901.1150); with no (readable) copy shipped, the 1696-line set as a fallback a
  // document's own definition replaces.
  match shipped_class_text("mn2e")? {
    Some(text) => {
      let lines = shipped_journal_abbreviations(&text);
      if !lines.is_empty() {
        RawTeX!(&lines);
      }
    },
    None => {
      let _fallback = OriginGuard::new(DefinitionOrigin::Fallback);
      RawTeX!(MN2E_JOURNAL_ABBREVIATIONS);
    },
  }
});

/// The journal abbreviations of the 1696-line mn2e.cls v2.2 (mn2e.cls:1575-1686), as written there.
const MN2E_JOURNAL_ABBREVIATIONS: &str = r"\let\jnl@style=\rmfamily
\def\ref@jnl#1{{\jnl@style#1}}
\newcommand\aj{\ref@jnl{AJ}}
\newcommand\actaa{\ref@jnl{Acta Astron.}}
\newcommand\araa{\ref@jnl{ARA\&A}}
\newcommand\apj{\ref@jnl{ApJ}}
\newcommand\apjl{\ref@jnl{ApJ}}
\newcommand\apjs{\ref@jnl{ApJS}}
\newcommand\ao{\ref@jnl{Appl.~Opt.}}
\newcommand\apss{\ref@jnl{Ap\&SS}}
\newcommand\aap{\ref@jnl{A\&A}}
\newcommand\aapr{\ref@jnl{A\&A~Rev.}}
\newcommand\aaps{\ref@jnl{A\&AS}}
\newcommand\azh{\ref@jnl{AZh}}
\newcommand\baas{\ref@jnl{BAAS}}
\newcommand\caa{\ref@jnl{Chinese Astron. Astrophys.}}
\newcommand\cjaa{\ref@jnl{Chinese J. Astron. Astrophys.}}
\newcommand\icarus{\ref@jnl{Icarus}}
\newcommand\jcap{\ref@jnl{J. Cosmology Astropart. Phys.}}
\newcommand\jrasc{\ref@jnl{JRASC}}
\newcommand\memras{\ref@jnl{MmRAS}}
\newcommand\mnras{\ref@jnl{MNRAS}}
\newcommand\na{\ref@jnl{New A}}
\newcommand\nar{\ref@jnl{New A Rev.}}
\newcommand\pra{\ref@jnl{Phys.~Rev.~A}}
\newcommand\prb{\ref@jnl{Phys.~Rev.~B}}
\newcommand\prc{\ref@jnl{Phys.~Rev.~C}}
\newcommand\prd{\ref@jnl{Phys.~Rev.~D}}
\newcommand\pre{\ref@jnl{Phys.~Rev.~E}}
\newcommand\prl{\ref@jnl{Phys.~Rev.~Lett.}}
\newcommand\pasa{\ref@jnl{PASA}}
\newcommand\pasp{\ref@jnl{PASP}}
\newcommand\pasj{\ref@jnl{PASJ}}
\newcommand\qjras{\ref@jnl{QJRAS}}
\newcommand\rmxaa{\ref@jnl{Rev. Mexicana Astron. Astrofis.}}
\newcommand\skytel{\ref@jnl{S\&T}}
\newcommand\solphys{\ref@jnl{Sol.~Phys.}}
\newcommand\sovast{\ref@jnl{Soviet~Ast.}}
\newcommand\ssr{\ref@jnl{Space~Sci.~Rev.}}
\newcommand\zap{\ref@jnl{ZAp}}
\newcommand\nat{\ref@jnl{Nature}}
\newcommand\iaucirc{\ref@jnl{IAU~Circ.}}
\newcommand\aplett{\ref@jnl{Astrophys.~Lett.}}
\newcommand\apspr{\ref@jnl{Astrophys.~Space~Phys.~Res.}}
\newcommand\bain{\ref@jnl{Bull.~Astron.~Inst.~Netherlands}}
\newcommand\fcp{\ref@jnl{Fund.~Cosmic~Phys.}}
\newcommand\gca{\ref@jnl{Geochim.~Cosmochim.~Acta}}
\newcommand\grl{\ref@jnl{Geophys.~Res.~Lett.}}
\newcommand\jcp{\ref@jnl{J.~Chem.~Phys.}}
\newcommand\jgr{\ref@jnl{J.~Geophys.~Res.}}
\newcommand\jqsrt{\ref@jnl{J.~Quant.~Spec.~Radiat.~Transf.}}
\newcommand\memsai{\ref@jnl{Mem.~Soc.~Astron.~Italiana}}
\newcommand\nphysa{\ref@jnl{Nucl.~Phys.~A}}
\newcommand\physrep{\ref@jnl{Phys.~Rep.}}
\newcommand\physscr{\ref@jnl{Phys.~Scr}}
\newcommand\planss{\ref@jnl{Planet.~Space~Sci.}}
\newcommand\procspie{\ref@jnl{Proc.~SPIE}}
\let\astap=\aap
\let\apjlett=\apjl
\let\apjsupp=\apjs
\let\applopt=\ao";

/// The lines of a shipped mn2e.cls that define the journal abbreviations (the names of
/// [`MN2E_JOURNAL_ABBREVIATIONS`]), each as the copy writes it — one-line definitions, braces balanced — preceded by the
/// one-line definitions of the internal (`@`) macros they are built on, which copies name differently (`\ref@jnl` and
/// its `\jnl@style`, 0711.3034; `\reff@jnl`, 1209.3072). A line filter, not TeX: the bytes are read as Latin-1; lines
/// between a line-leading `\iffalse` and `\fi` are skipped; a definition is taken only whole on its line (a multi-line
/// one is left out), and a line is taken whole (a second definition on it comes along); a `%`-commented line is not a
/// definition.
fn shipped_journal_abbreviations(text: &str) -> String {
  let name_rx = Regex::new(r"\\(?:newcommand|def|let)\\([A-Za-z@]+)").unwrap();
  let wanted: Vec<String> = name_rx
    .captures_iter(MN2E_JOURNAL_ABBREVIATIONS)
    .filter_map(|c| c.get(1).map(|m| m.as_str().to_string()))
    .filter(|name| !name.contains('@'))
    .collect();
  let line_rx = Regex::new(r"^\\(?:newcommand\*?|def|let)\s*\\([A-Za-z@]+)").unwrap();
  let internal_rx = Regex::new(r"\\([A-Za-z]*@[A-Za-z@]*)").unwrap();
  // Each one-line definition in the copy, by the name it defines.
  let mut in_iffalse = false;
  let definitions: Vec<(String, &str)> = text
    .lines()
    .map(str::trim)
    .filter(|line| {
      if line.starts_with("\\iffalse") {
        in_iffalse = true;
      } else if in_iffalse && line.starts_with("\\fi") {
        in_iffalse = false;
        return false;
      }
      !in_iffalse
    })
    .filter_map(|line| {
      let name = line_rx.captures(line)?.get(1)?.as_str().to_string();
      let bare = line.replace("\\{", "").replace("\\}", "");
      (bare.matches('{').count() == bare.matches('}').count()).then_some((name, line))
    })
    .collect();
  let mut lines: Vec<&str> = Vec::new();
  let mut helpers: Vec<&str> = Vec::new();
  for (name, line) in &definitions {
    if wanted.contains(name) {
      lines.push(line);
    }
  }
  // The internal macros those lines use, and the ones theirs use.
  let mut used: Vec<String> = Vec::new();
  let mut pending: Vec<String> = lines
    .iter()
    .flat_map(|line| {
      internal_rx
        .captures_iter(line)
        .filter_map(|c| c.get(1).map(|m| m.as_str().to_string()))
    })
    .collect();
  while let Some(name) = pending.pop() {
    if used.contains(&name) {
      continue;
    }
    if let Some((_, line)) = definitions.iter().find(|(defined, _)| *defined == name) {
      helpers.insert(0, line);
      pending.extend(
        internal_rx
          .captures_iter(line)
          .filter_map(|c| c.get(1).map(|m| m.as_str().to_string()))
          .filter(|used_name| *used_name != name),
      );
    }
    used.push(name);
  }
  let mut out = String::new();
  for line in helpers.into_iter().chain(lines) {
    out.push_str(line);
    out.push('\n');
  }
  out
}
