use crate::prelude::*;

LoadDefinitions!({
  //**********************************************************************
  // Define \name and \begin{name} to start an ignored section
  // until \endname or \end{name}, respectively
  let define_excluded: PrimitiveClosure = Rc::new(|mut args: Vec<ArgWrap>| {
    let name = args.remove(0).owned_tokens().unwrap();
    let begin_mark = s!("\\begin{{{name}}}");
    let end_mark = s!("\\end{{{name}}}");
    DefConstructor!(T_CS!(begin_mark), None, None,
    after_digest => {
      let mut nlines = 0;
      read_raw_line();    // IGNORE 1st line (after the \begin{$name} !!!
      // Perl comment.sty.ltxml L30 matches `/^\s*\Q$endmark\E\s*$/` —
      // the end line may carry leading/trailing whitespace. Strict
      // equality missed indented `  \end{comment}  `, stranding the
      // excluded block consumption.
      while let Some(line) = read_raw_line() {
        if line.trim() == end_mark {
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
