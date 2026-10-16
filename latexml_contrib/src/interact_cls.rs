//! Stub for interact.cls (Taylor & Francis interact class).
use latexml_core::definition::origin::{DefinitionOrigin, OriginGuard};
use latexml_package::prelude::*;

LoadDefinitions!({
  LoadClass!("OmniBus");
  RequirePackage!("amsmath");
  RequirePackage!("amsthm");
  RequirePackage!("amssymb");
  // Eager xcolor preload removed for Perl parity: it makes a later document
  // xcolor[table] load a no-op, so colortbl/array never load and array m{}/b{}
  // columns break (Unrecognized tabular template -> Extra alignment tab). The
  // document loads xcolor itself; color/definecolor stay via hyperref->color.
  // See ifacconf_cls.rs and SYNC_STATUS (eager-xcolor cluster).
  RequirePackage!("hyperref");
  RequirePackage!("booktabs");
  RequirePackage!("graphicx");
  // interact.cls:134-138 (v1.01-v1.05): `\def\bm#1{\mathchoice{\mbox{\boldmath$\displaystyle#1$}}…}`, bold math the
  // class defines itself, its argument in a box — 2310.17051, 1803.04839, 2212.00021. A `\left` without its `\right`
  // in the argument is TeX's error at the box's end; it is set at the outer level with a Warning, as bm.sty would set
  // it (bm_sty.rs `define_grouped_bm`, OXIDIZED_DESIGN #544).
  bm_sty::define_grouped_bm()?;

  // interact.cls:266-274: `\author{\name{…}\affil{…}}` holds the whole block, and `\name{X}` prints `X\\`, `\affil{Y}`
  // and `\email{Z}` lines of their own: the names line and the affiliation lines the kernel author parse reads, so a
  // name list `\name{A\textsuperscript{a}, B\textsuperscript{b}}` is its authors, linked by their marks (Perl maps
  // `\name` to its argument, one creator; 2112.10522, 2312.11500, 2403.20182). Each later `\name` opens a group of its
  // own (`\name{A}\affil{X}\name{B}\affil{Y}`), and the class's `\and` prints "and ".
  DefMacro!("\\author{}", sub[(body)] {
    let toks = body.unlist();
    let mut lines: Vec<Token> = Vec::new();
    let mut named = false;
    let mut i = 0;
    while i < toks.len() {
      let t = toks[i];
      i += 1;
      if t == T_CS!("\\and") {
        // (before a later `\name`, which opens a group of its own, it separates nothing more)
        if toks[i..].iter().find(|u| **u != T_SPACE!()) != Some(&T_CS!("\\name")) {
          lines.extend(Tokenize!("and").unlist());
          lines.push(T_SPACE!());
        }
        continue;
      }
      if t != T_CS!("\\name") && t != T_CS!("\\affil") && t != T_CS!("\\email") {
        lines.push(t);
        continue;
      }
      if t == T_CS!("\\name") {
        if named {
          lines.push(T_CS!("\\and"));
        }
        named = true;
      }
      while toks.get(i) == Some(&T_SPACE!()) {
        i += 1;
      }
      if toks.get(i).is_none_or(|u| u.get_catcode() != Catcode::BEGIN) {
        lines.push(t);
        continue;
      }
      let mut depth = 0usize;
      let start = i;
      while i < toks.len() {
        match toks[i].get_catcode() {
          Catcode::BEGIN => depth += 1,
          Catcode::END => depth -= 1,
          _ => {},
        }
        i += 1;
        if depth == 0 {
          break;
        }
      }
      for u in &toks[start + 1..i - 1] {
        if *u == T_CS!("\\and") {
          lines.extend(Tokenize!("and").unlist());
          lines.push(T_SPACE!());
        } else {
          lines.push(*u);
        }
      }
      lines.push(T_CS!("\\\\"));
    }
    let mut out = vec![T_CS!("\\gdef"), T_CS!("\\@author"), T_BEGIN!()];
    out.extend(toks.iter().copied());
    out.push(T_END!());
    out.extend(Invocation!(T_CS!("\\lx@add@authors"), vec![Some(Tokens::new(lines))]).unlist());
    Ok(Tokens::new(out))
  }, locked => true);
  // Author-block macros — preserve author content.
  DefMacro!("\\name{}", "#1");
  DefMacro!(
    "\\affil{}",
    "\\@add@frontmatter{ltx:note}[role=affiliation]{#1}"
  );
  def_macro_noop("\\affilskip")?;

  // {amscode} env — interact L507.
  DefEnvironment!(
    "{amscode}",
    "<ltx:classification scheme='AMS'>#body</ltx:classification>"
  );
  // \amscodename — the "AMS CLASSIFICATION" label (interact.cls L718:
  // `\newcommand\amscodename{AMS CLASSIFICATION}`). Used inside {amscode}'s
  // body, but papers also call it standalone, e.g.
  // `\amscodename{: Primary 60H15; 37H05.}`. Define verbatim as the label so
  // the classification text survives inline. Witness 2008.01335 (1 err → 0;
  // Perl, with no interact binding → OmniBus, errors on both \amscodename and
  // \name). Mirror sibling label macros \keyname/\jelname if seen later.
  DefMacro!("\\amscodename", "AMS CLASSIFICATION");

  // Frontmatter metadata — preserve author content.
  DefMacro!(
    "\\articletype{}",
    "\\@add@frontmatter{ltx:note}[role=articletype]{#1}"
  );
  DefMacro!("\\authormark{}", "\\textsuperscript{#1}");
  // interact.cls:493-499 `\tbl{caption}{body}`: the table's caption, then its body (as ws_journal_cls.rs's);
  // 2312.11500, 2501.02233.
  DefMacro!("\\tbl{}{}", "\\caption{#1}#2");
  DefMacro!(
    "\\corres{}",
    "\\@add@frontmatter{ltx:note}[role=corresponding]{#1}"
  );
  DefMacro!(
    "\\thanks{}",
    "\\@add@frontmatter{ltx:note}[role=thanks]{#1}"
  );
  DefMacro!(
    "\\journalname{}",
    "\\@add@frontmatter{ltx:note}[role=journal]{#1}"
  ); // interact.cls v1.05 (2017/07/31; 90 of the 124 copies papers ship, 1711.05866's; v1.01-v1.03 the same), what
  // papers call (run 347: 46 interact papers), a copy without one leaving it to the paper (`DefinitionOrigin::Fallback`):
  // - :493-527 the keyword-like blocks under the abstract: `{abbreviations}` "Abbreviations: …" (10 papers: 1901.07015,
  //   2102.04929), and the classifications `{jelcode}` "JEL CLASSIFICATION" and `{pacscode}` "PACS CLASSIFICATION",
  //   as `{amscode}` (2306.12924, 2307.12161; 1610.08232, 1611.05230), and v4.0's (2015/06/03, 1709.02637's copy :342)
  //   `{classcode}` "AMS Subject Classification";
  // - :167-226 the upright Greek letters `\ualpha`…`\uvarphi`, `\upartial` and their bold forms `\ubalpha`…
  //   (`\mathup`, the upright math version, and `\mathbup` its bold, here `\boldsymbol` of the upright letter;
  //   `\umu` 5 papers: 1709.03559, 2207.06123; `\upartial` 4, `\upi` 3,
  //   `\uomega`, `\udelta`);
  // - :635-640 `{tabnote}[heading]`, a note under a table, its heading italic (1711.08876, 1712.07848, 2304.00568);
  // - :237-243 `\jvol`, `\jnum`, `\jmonth`, `\jyear`, `\jname`, the running head's journal issue (2208.03936,
  //   2211.01356): stored, as the class stores them; :268 `\history`, the article history `\received` sets
  //   (2609.06135); :591 `\figwidth`/`\figheight`; :642 `\x`, the column spacing of a full-width table;
  // - v4.0 :572-573 `\colrule`, `\botrule`, the tables' inner and closing rules (1709.02637).
  {
    let _class = OriginGuard::new(DefinitionOrigin::Fallback);
    RawTeX!(
      r"\newcommand\abbreviationsname{Abbreviations}
\newenvironment{abbreviations}{\par\noindent{\bfseries \abbreviationsname: }\ignorespaces}{\par}
\newcommand\jelcodename{JEL CLASSIFICATION}
\newcommand\pacscodename{PACS CLASSIFICATION}
\def\tabnotefont{\footnotesize}
\newenvironment{tabnote}{\par\tabnotefont\@ifnextchar[{\@tabnote}{\@tabnote[]}}{\par}
\def\@tabnote[#1]{\def\@Tempa{#1}\ifx\@Tempa\@empty\else{\itshape #1:}\ \fi\ignorespaces}
\def\jname#1{\gdef\@jname{#1}}\gdef\@jname{}
\def\jvol#1{\gdef\@jvol{#1}}\gdef\@jvol{00}
\def\jnum#1{\gdef\@jnum{#1}}\gdef\@jnum{00}
\def\jmonth#1{\gdef\@jmonth{#1}}\gdef\@jmonth{Month}
\def\jyear#1{\gdef\@jyear{#1}}\gdef\@jyear{20XX}
\def\history#1{\received{#1}}
\newdimen\figheight
\newdimen\figwidth
\def\x{@{\extracolsep{\fill}}}
\def\Hline{\noalign{\ifnum0=`}\fi\hrule \@height 2\arrayrulewidth \futurelet\@tempa\@xhline}
\def\colrule{\\[-7.5pt]\hline\\[-5pt]}
\def\botrule{\\[-7.7pt]\Hline}"
    );
    DefEnvironment!(
      "{jelcode}",
      "<ltx:classification scheme='JEL'>#body</ltx:classification>"
    );
    DefEnvironment!(
      "{pacscode}",
      "<ltx:classification scheme='PACS'>#body</ltx:classification>"
    );
    DefEnvironment!(
      "{classcode}",
      "<ltx:classification scheme='AMS'>#body</ltx:classification>"
    );
    DefMath!("\\ualpha", "α", font => { shape => "upright", forceshape => true });
    DefMath!("\\ubeta", "β", font => { shape => "upright", forceshape => true });
    DefMath!("\\ugamma", "γ", font => { shape => "upright", forceshape => true });
    DefMath!("\\udelta", "δ", font => { shape => "upright", forceshape => true });
    DefMath!("\\uepsilon", "ϵ", font => { shape => "upright", forceshape => true });
    DefMath!("\\uzeta", "ζ", font => { shape => "upright", forceshape => true });
    DefMath!("\\ueta", "η", font => { shape => "upright", forceshape => true });
    DefMath!("\\utheta", "θ", font => { shape => "upright", forceshape => true });
    DefMath!("\\uiota", "ι", font => { shape => "upright", forceshape => true });
    DefMath!("\\ukappa", "κ", font => { shape => "upright", forceshape => true });
    DefMath!("\\ulambda", "λ", font => { shape => "upright", forceshape => true });
    DefMath!("\\umu", "μ", font => { shape => "upright", forceshape => true });
    DefMath!("\\unu", "ν", font => { shape => "upright", forceshape => true });
    DefMath!("\\uxi", "ξ", font => { shape => "upright", forceshape => true });
    DefMath!("\\upi", "π", font => { shape => "upright", forceshape => true });
    DefMath!("\\urho", "ρ", font => { shape => "upright", forceshape => true });
    DefMath!("\\usigma", "σ", font => { shape => "upright", forceshape => true });
    DefMath!("\\utau", "τ", font => { shape => "upright", forceshape => true });
    DefMath!("\\uupsilon", "υ", font => { shape => "upright", forceshape => true });
    DefMath!("\\uphi", "ϕ", font => { shape => "upright", forceshape => true });
    DefMath!("\\uchi", "χ", font => { shape => "upright", forceshape => true });
    DefMath!("\\upsi", "ψ", font => { shape => "upright", forceshape => true });
    DefMath!("\\uomega", "ω", font => { shape => "upright", forceshape => true });
    DefMath!("\\uvarepsilon", "ε", font => { shape => "upright", forceshape => true });
    DefMath!("\\uvartheta", "ϑ", font => { shape => "upright", forceshape => true });
    DefMath!("\\uvarpi", "ϖ", font => { shape => "upright", forceshape => true });
    DefMath!("\\uvarphi", "φ", font => { shape => "upright", forceshape => true });
    DefMath!("\\upartial", "∂", font => { shape => "upright", forceshape => true });
    DefMacro!("\\ubalpha", "\\boldsymbol{\\ualpha}");
    DefMacro!("\\ubbeta", "\\boldsymbol{\\ubeta}");
    DefMacro!("\\ubgamma", "\\boldsymbol{\\ugamma}");
    DefMacro!("\\ubdelta", "\\boldsymbol{\\udelta}");
    DefMacro!("\\ubepsilon", "\\boldsymbol{\\uepsilon}");
    DefMacro!("\\ubzeta", "\\boldsymbol{\\uzeta}");
    DefMacro!("\\ubtheta", "\\boldsymbol{\\utheta}");
    DefMacro!("\\ubiota", "\\boldsymbol{\\uiota}");
    DefMacro!("\\ubkappa", "\\boldsymbol{\\ukappa}");
    DefMacro!("\\ublambda", "\\boldsymbol{\\ulambda}");
    DefMacro!("\\ubmu", "\\boldsymbol{\\umu}");
    DefMacro!("\\ubnu", "\\boldsymbol{\\unu}");
    DefMacro!("\\ubxi", "\\boldsymbol{\\uxi}");
    DefMacro!("\\ubpi", "\\boldsymbol{\\upi}");
    DefMacro!("\\ubrho", "\\boldsymbol{\\urho}");
    DefMacro!("\\ubsigma", "\\boldsymbol{\\usigma}");
    DefMacro!("\\ubtau", "\\boldsymbol{\\utau}");
    DefMacro!("\\ubupsilon", "\\boldsymbol{\\uupsilon}");
    DefMacro!("\\ubphi", "\\boldsymbol{\\uphi}");
    DefMacro!("\\ubchi", "\\boldsymbol{\\uchi}");
    DefMacro!("\\ubpsi", "\\boldsymbol{\\upsi}");
    DefMacro!("\\ubomega", "\\boldsymbol{\\uomega}");
    DefMacro!("\\ubvarepsilon", "\\boldsymbol{\\uvarepsilon}");
    DefMacro!("\\ubvartheta", "\\boldsymbol{\\uvartheta}");
    DefMacro!("\\ubvarpi", "\\boldsymbol{\\uvarpi}");
    DefMacro!("\\ubvarphi", "\\boldsymbol{\\uvarphi}");
    DefMacro!("\\ubpartial", "\\boldsymbol{\\upartial}");
    // (:203 `\uboldeta`, not `\ubeta`, which is the upright beta; :193-194 `\let\uvarrho\varrho`, `\let\uvarsigma
    // \varsigma`)
    DefMacro!("\\uboldeta", "\\boldsymbol{\\ueta}");
    Let!("\\uvarrho", "\\varrho");
    Let!("\\uvarsigma", "\\varsigma");
  }
});
