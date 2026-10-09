//! Stub for interact.cls (Taylor & Francis interact class).
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
  );
});
