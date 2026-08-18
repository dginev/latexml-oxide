use crate::prelude::*;

/// The prefix of a raw body line up to its first *unescaped* `%` — comment.sty
/// reads its body with `%` active as a TeX comment, so an `\end{name}` sitting
/// behind a `%` (e.g. `% […] \end{comment}`) does NOT close the environment.
/// `%` and `\` are ASCII, so byte scanning is UTF-8-safe (the cut index always
/// lands on a char boundary).
fn strip_tex_comment(line: &str) -> &str {
  let bytes = line.as_bytes();
  for i in 0..bytes.len() {
    if bytes[i] == b'%' {
      let mut bs = 0;
      while i > bs && bytes[i - 1 - bs] == b'\\' {
        bs += 1;
      }
      if bs % 2 == 0 {
        return &line[..i];
      }
    }
  }
  line
}

LoadDefinitions!({
  //**********************************************************************
  // Define \name and \begin{name} to start an ignored section
  // until \endname or \end{name}, respectively
  let define_excluded: PrimitiveClosure = Rc::new(|mut args: Vec<ArgWrap>| {
    let name = args.remove(0).owned_tokens().unwrap();
    let begin_mark = s!("\\begin{{{name}}}");
    DefConstructor!(T_CS!(begin_mark), None, None,
    after_digest => {
      // Detect `\end{name}` MID-LINE (allowing spaces, `\end {name}`), but only
      // in the code part of the line — an `\end` hidden behind a `%` comment
      // does NOT close the environment (comment.sty reads its body with `%`
      // active). Perl comment.sty.ltxml L30 matched only a whole line,
      // `/^\s*\Q$endmark\E\s*$/`, and our prior port did the same
      // (`line.trim() == end_mark`), so a comment ending `…text.\end{name}`
      // overran to EOF and silently swallowed everything after it — a document's
      // bibliography included. Surpass-Perl divergence (OXIDIZED_DESIGN #133):
      // both LaTeXML engines lose it, and pdflatex keeps it for the witness
      // arXiv:2606.11493 (`\begin{comment}…\(G(h_1)=0\).\end{comment}` swallowed
      // a 31-`\bibitem` thebibliography). The `%`-guard keeps the tokenize
      // `comment` fixture's `% […] \end{Excluded}` from closing early. Guard
      // `comment_midline_end_keeps_bibliography`.
      let end_re = Regex::new(&format!("\\\\end\\s*\\{{{name}\\}}")).unwrap();
      let mut nlines = 0;
      read_raw_line();    // IGNORE 1st line (after the \begin{$name} !!!
      while let Some(line) = read_raw_line() {
        let code = strip_tex_comment(&line);
        if let Some(m) = end_re.find(code) {
          // comment.sty expects `\end{name}` at the line's end; trailing content
          // on that line is dropped, as verbatim.sty's `\verbatim@` does too.
          if !code[m.end()..].trim().is_empty() {
            Info!("unexpected", "stuff",
              s!("Characters dropped after '\\end{{{name}}}'"));
          }
          break;
        }
        nlines += 1;
      }
      note_progress(&s!("[Skipped {name} ({nlines} lines)]"));
      Ok(Vec::new())
    });
    Ok(Vec::new())
  });

  // I don't understand Rust closures enough to figure out how to clone one, so instantiating it
  // twice instead, via a macro
  let define_included: PrimitiveBody = PrimitiveBody::Closure(Rc::new(|mut args: Vec<ArgWrap>| {
    args.reverse(); // we'll be using .pop() from the front
    let name = args
      .pop()
      .unwrap()
      .owned_tokens()
      .expect("expecting a Tokens argument")
      .to_string();
    let mut before_tokens = match args.pop() {
      Some(arg) => arg.unlist(),
      None => Vec::new(),
    };
    before_tokens.push(T_CS!("\\ignorespaces"));
    let mut after_tokens = match args.pop() {
      Some(arg) => arg.unlist(),
      None => Vec::new(),
    };
    after_tokens.push(T_CS!("\\ignorespaces"));
    // Note that we define the `magic' environment control sequences,
    // but DO NOT do any of the normal environ things, like \begingroup \endgroup!
    DefMacro!(T_CS!(s!("\\begin{{{name}}}")), None, {
      read_raw_line(); // IGNORE 1st line (after the \begin{$name} !!!
      before_tokens.clone()
    });
    DefMacro!(
      T_CS!(s!("\\end{{{name}}}")),
      None,
      Tokens::new(after_tokens)
    );
    Ok(Vec::new())
  }));

  define_excluded(vec![ArgWrap::Tokens(Tokenize!("comment"))])?;

  DefPrimitive!("\\includecomment{}", Some(define_included.clone()));
  DefPrimitive!(
    "\\excludecomment{}",
    Some(PrimitiveBody::Closure(define_excluded))
  );
  DefPrimitive!("\\specialcomment{}{}{}", Some(define_included));
  DefPrimitive!("\\processcomment{}{}{}{}", None);
  DefMacro!("\\csarg{}{}", r"\expandafter#1\csname#2\endcsname");
});
