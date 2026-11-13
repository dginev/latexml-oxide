//! autobreak.sty — line breaking of a long formula inside `align`. The package reads the environment's body with its
//! line ends active (autobreak.sty:97-118, its own `\collect@body`), takes the first non-empty line as the left-hand
//! side (`#1{}&`, :283-299 `\@autobreak@processline`), and joins the other lines after it, breaking with `\notag\\&`
//! where the measured width overflows `\linewidth`. It reaches the body through amsmath's internals — the `\start@align`
//! hook that allows it (:120-125) and `\@envbody`/`\collect@@body` — which the native `align` binding never runs, so
//! the raw package rejected every use ("autobreak is not allowed here", then `\@envbody` undefined; witness 2609.08470,
//! 17 environments, Fatal TooManyErrors). Perl LaTeXML ships no binding (the environment is undefined there).
//!
//! The binding keeps the package's reading of the source: a line is what `\@autobreak@scanline#1^^M` (:163) takes, so
//! a line end inside braces stays in the line as a space (`\def^^M{ }`, :98); a line that starts with `.`, `,`, `;` or
//! `:` lends that character to the line before (`\@autobreak@ifnextpunct`, :166-197); the first non-empty line is the
//! left-hand side and the rest follow the alignment tab; the environment closes before them (as `\@autobreak` does,
//! :144-150, so the tab is outside its group). The width-driven breaks are page layout and are not made, and a first
//! line of zero width (the package measures it, :285) is still the left-hand side. Outside `align` the package's
//! "autobreak is not allowed here" (:131-137) is not raised; the stray alignment tab is reported instead.
use latexml_package::prelude::*;

LoadDefinitions!({
  RequirePackage!("amsmath");
  // autobreak.sty:4-5
  RawTeX!(r"\newtoks\everybeforeautobreak\newtoks\everyafterautobreak");
  // autobreak.sty:131-141: the body is read with `^^M` active, so each source line ends in one.
  DefMacro!(
    "\\autobreak",
    "\\catcode13=\\active\\relax\\lx@autobreak@body"
  );
  def_macro_noop("\\endautobreak")?;
  DefMacro!("\\lx@autobreak@body", {
    let end: Vec<Token> = Tokenize!("\\end{autobreak}").unlist();
    let mut body: Vec<Token> = Vec::new();
    loop {
      match read_token()? {
        Some(token) => {
          body.push(token);
          if body.len() >= end.len() && body[body.len() - end.len()..] == end[..] {
            body.truncate(body.len() - end.len());
            break;
          }
        },
        None => {
          Error!(
            "expected",
            "\\end{autobreak}",
            "The autobreak environment is not closed"
          );
          break;
        },
      }
    }
    // The lines, split at the line ends outside braces; one inside braces is a space.
    let newline = T_ACTIVE!('\r');
    let mut lines: Vec<Vec<Token>> = vec![Vec::new()];
    let mut depth = 0usize;
    for token in body {
      match token.get_catcode() {
        Catcode::BEGIN => depth += 1,
        Catcode::END => depth = depth.saturating_sub(1),
        _ => {},
      }
      if token == newline {
        if depth == 0 {
          lines.push(Vec::new());
        } else if let Some(line) = lines.last_mut() {
          line.push(T_SPACE!());
        }
      } else if let Some(line) = lines.last_mut() {
        line.push(token);
      }
    }
    let mut joined: Vec<Vec<Token>> = Vec::new();
    for line in lines {
      let start = line
        .iter()
        .position(|t| t.get_catcode() != Catcode::SPACE)
        .unwrap_or(line.len());
      let stop = line
        .iter()
        .rposition(|t| t.get_catcode() != Catcode::SPACE)
        .map_or(start, |i| i + 1);
      let mut line = line[start..stop].to_vec();
      if line.is_empty() {
        continue;
      }
      // `\@autobreak@scanline@gobble` (:189-195) takes every leading punctuation character.
      let punctuation = |t: &Token| {
        t.get_catcode() == Catcode::OTHER && t.with_str(|s| matches!(s, "." | "," | ";" | ":"))
      };
      if let Some(previous) = joined.last_mut()
        && line.first().is_some_and(punctuation)
      {
        while line.first().is_some_and(punctuation) {
          previous.push(line.remove(0));
        }
        if line.iter().all(|t| t.get_catcode() == Catcode::SPACE) {
          continue;
        }
      }
      joined.push(line);
    }
    let mut out = end.clone();
    let mut lines = joined.into_iter();
    if let Some(lhs) = lines.next() {
      out.extend(lhs);
      out.extend([T_BEGIN!(), T_END!(), T_ALIGN!()]);
    }
    for line in lines {
      out.extend(line);
    }
    Ok(Tokens::new(out))
  });
});
