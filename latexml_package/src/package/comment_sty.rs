use crate::prelude::*;

LoadDefinitions!({
  //**********************************************************************
  // Define \name and \begin{name} to start an ignored section
  // until \endname or \end{name}, respectively
  let define_excluded: PrimitiveClosure = Rc::new(|mut args: Vec<ArgWrap>| {
    let name = args.remove(0).owned_tokens().unwrap();
    let name_str = name.to_string();
    let begin_mark = format!("\\begin{{{name_str}}}");
    let name_clone = name_str.clone();
    DefConstructor!(T_CS!(begin_mark), None, None,
    after_digest => {
      // comment.sty:186-193 reads the body LINE by line with every special
      // made innocent and compares each whole line against `\end{name}`
      // (`\ProcessCommentLine#1^^M{\def\test{#1}\csarg\ifx{End…Test}\test`):
      // only a line that IS `\end{name}` ends the comment — TeX's line
      // reader strips trailing spaces (tex.web §362), so `\end{name}   ` does,
      // while leading spaces, a trailing `%` (innocent, so literal), text
      // before it, or text after it do not, and the comment then runs to the
      // end of the file where pdflatex stops with "File ended while scanning
      // use of \next" (verified on each shape, 2026-09-05). The earlier
      // mid-line detection (OXIDIZED_DESIGN #133, since retracted) showed
      // MORE than pdflatex; we now report TeX's error at EOF instead.
      // The terminator is handed to the CURRENT `\end` macro (K3, #199).
      // Guards: `00_tokenize::comment_test` (golden), `06_cluster_bibliography::
      // comment_midline_end_runs_to_eof_like_pdflatex`,
      // `perfect_kernel_gemini::comment_self_terminating_hands_to_end`.
      let end_line = format!("\\end{{{name_clone}}}");
      let mut nlines = 0;
      let mut ended = false;
      read_raw_line();    // IGNORE 1st line (after the \begin{$name} !!!
      while let Some(line) = read_raw_line() {
        if line.trim_end_matches(' ') == end_line {
          let mut end_tokens = vec![T_CS!("\\end"), T_BEGIN!()];
          end_tokens.extend(ExplodeText!(&name_clone));
          end_tokens.push(T_END!());
          unread_expansion(Tokens::new(end_tokens));
          ended = true;
          break;
        }
        nlines += 1;
      }
      if !ended {
        Error!("unexpected", "EOF",
          s!("File ended while scanning use of \\next (no whole-line \\end{{{name_clone}}} closes this {name_clone})"));
      }
      note_progress(&s!("[Skipped {name_clone} ({nlines} lines)]"));
      Ok(Vec::new())
    });
    DefMacro!(T_CS!(format!("\\end{{{name_str}}}")), None, Tokens!());
    DefMacro!(T_CS!(format!("\\end{name_str}")), None, Tokens!());
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
