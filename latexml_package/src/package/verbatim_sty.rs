use crate::prelude::*;

LoadDefinitions!({
  //======================================================================
  // Note that we CAN process the verbatim.sty file and that works,
  // although the xml it generates is pretty pointless
  ////// InputDefinitions('verbatim', type => 'sty', noltxml => 1);
  //======================================================================
  // Thus, we set out to define the essentials, but keep as close
  // to verbatim's internals as we can

  // Since LaTeX.pool has already defined {verbatim} as an environment,
  // (so that \begin{verbatim} takes precedence over \verbatim!)
  // we have to be more forceful so that \verbatim & \endverbatim
  // are even seen!
  AssignMeaning!(&T_CS!("\\begin{verbatim}"), Stored::None);
  AssignMeaning!(&T_CS!("\\begin{verbatim*}"), Stored::None);
  AssignMeaning!(&T_CS!("\\end{verbatim}"), Stored::None);
  AssignMeaning!(&T_CS!("\\end{verbatim*}"), Stored::None);

  DefRegister!("\\every@verbatim", Tokens!());
  DefRegister!("\\verbatim@line", Tokens!());

  //======================================================================
  // Mostly simplified versions of what"s in verbatim....
  DefMacro!(r"\verbatim@startline", r"\verbatim@line{}");
  DefMacro!(
    r"\verbatim@addtoline{}",
    r"\verbatim@line\expandafter{\the\verbatim@line#1}"
  );
  DefMacro!(r"\verbatim@processline", r"\the\verbatim@line\par");

  DefMacro!(
    r"\verbatim@font",
    r"\normalfont\ttfamily\hyphenchar\font\m@ne\@noligs"
  );
  DefMacro!(
    r"\@verbatim",
    r"\the\every@verbatim
     \obeylines
     \let\do\@makeother \dospecials
     \verbatim@font"
  );

  DefConstructor!("\\lx@verbatim@", "<ltx:verbatim font='#font'>",
    before_digest => { Let!(T_CS!("\\par"), T_CR!()); },
    before_construct => sub[document, _whatsit] {
      document.maybe_close_element("ltx:p")?; }
  );

  // We HAVE to get this guy in, to close the <ltx:verbatim>"
  DefConstructor!("\\lx@end@verbatim@{}", "</ltx:verbatim>");

  // Note: We need the internal T_CS!("\\foo*") to attach the star to the CS, however,
  //       the current DefMacroI can not accept a string expansion, hence TokenizeInternal!() the
  // RHS
  //
  DefMacro!(
    "\\verbatim",
    "\\begingroup\\@verbatim\\frenchspacing\\@vobeyspaces\\lx@verbatim@\\verbatim@start"
  );
  DefMacro!(
    T_CS!("\\verbatim*"),
    None,
    TokenizeInternal!("\\begingroup\\@verbatim\\lx@verbatim@\\verbatim@start")
  );
  DefMacro!("\\endverbatim", "\\lx@end@verbatim@\\endgroup");
  DefMacro!(
    T_CS!("\\endverbatim*"),
    None,
    TokenizeInternal!("\\lx@end@verbatim@\\endgroup")
  );

  DefMacro!(
    "\\comment",
    r"\let\do\@makeother\dospecials\catcode`\^^M\active
\let\verbatim@startline\relax
\let\verbatim@addtoline\@gobble
\let\verbatim@processline\relax
\verbatim@"
  );
  def_macro_noop("\\endcomment")?;

  // verbatim.sty:107-112 `\verbatim@start#1` consumes the token that
  // follows it when `\if\noexpand#1\noexpand~` holds — the active `^^M`
  // ending the `\begin{…}` line in the normal case, but ANY control
  // sequence too (`\noexpand` renders it `\relax`-like, char code 256 on
  // both sides). That is the documented idiom `\verbatim@start\relax` for
  // starting a capture from inside a macro body (curve2e-manual.tex:95
  // `{Esempio}`, memoir, digiconfigs): the `\relax` stands in for the
  // line end. Perl's `\verbatim@start` = `\lx@verbatim@\verbatim@` (:76)
  // never looks, so `read_raw_line` serialised the pending `\relax` (gullet
  // pushback + line remainder) as the FIRST captured line. Only the PUSHBACK
  // is inspected: a `\relax` from a macro body lives there, while the mouth's
  // line remainder after `\begin{verbatim}` must stay unread — Perl's first
  // raw line is that (empty) remainder, which is the leading newline every
  // `<ltx:verbatim>` golden carries (`tests/tokenize/verbata.xml`). Guard
  // `perfect_kernel_batch50::verbatim_start_relax_idiom`.
  DefMacro!("\\verbatim@start", {
    if pushback_holds_nonspace()
      && let Some(t) = read_token()?
      && t.get_catcode() != Catcode::CS
    {
      unread_one(t);
    }
    Ok(Tokens!(T_CS!("\\verbatim@")))
  });

  //======================================================================
  // Here's the interesting bit.
  // Why do things the hard way, when we can pull lines out of the Mouth
  // and match them as text ?
  // Well, we have to dance a bit...
  //
  // NOTE: the part AFTER the \end{whatever}, should be lost (and message about it!)
  DefMacro!("\\verbatim@", {
    let env = lookup_string_from_sym(pin!("current_environment"));
    // Note: This should allow a regexp, since there can be spaces between \end and { !!!
    let mut lines = Vec::new();
    // TODO: UGH!!! Isn't there a better way to approximate
    // the Perl simplicity of writing an inline regex?
    // the escaping is very easy to get wrong!
    let env_re = Regex::new(&format!("^(.*)\\\\end\\s*\\{{{env}\\}}(.*)$")).unwrap();
    while let Some(line) = read_raw_line() {
      if let Some(caps) = env_re.captures(&line) {
        let pre = caps.get(1).map_or("", |m| m.as_str()).to_string();
        let post = caps.get(2).map_or("", |m| m.as_str()).to_string();
        lines.push(pre);
        if !post.is_empty() {
          let message = s!("Characters dropped after '\\end{{{}}}'", env);
          Info!("unexpected", "stuff", message);
        }
        break;
      } else {
        lines.push(line);
      }
    }
    if lines.last() == Some(&String::new()) {
      lines.pop();
    }
    let mut tokens = Vec::new();
    for line in &lines {
      tokens.push(T_CS!("\\verbatim@startline"));
      tokens.extend(
        Invocation!(T_CS!("\\verbatim@addtoline"), vec![Tokens::new(
          ExplodeText!(line)
        )])
        .unlist(),
      );
      tokens.push(T_CS!("\\verbatim@processline"));
    }
    tokens.extend(Invocation!(T_CS!("\\end"), vec![T_OTHER!(env)]).unlist());
    Ok(Tokens::new(tokens))
  });

  // //======================================================================
  // // Read verbatim material from file.
  DefMacro!("\\verbatiminput {}", sub[(file)] {
    // Expand the argument (`\verbatiminput{\jobname.tmp}`) and serve
    // in-memory filecontents/VerbatimOut captures before touching disk.
    let name = do_expand(file)?.to_string();
    if let Some(content) = vfs_read(&name) {
      let mut tokens = Vec::new();
      for line in content.lines() {
        tokens.push(T_CS!("\\verbatim@startline"));
        tokens.extend(Invocation!(T_CS!("\\verbatim@addtoline"),
          vec![Tokens::new(ExplodeText!(line.to_string()))]).unlist());
        tokens.push(T_CS!("\\verbatim@processline"));
      }
      return Ok(Tokens!(
        T_CS!("\\begingroup"), T_CS!("\\@verbatim"),
        T_CS!("\\frenchspacing"), T_CS!("\\@vobeyspaces"),
        T_CS!("\\lx@verbatim@"), Tokens::new(tokens), T_CS!("\\lx@end@verbatim@"), T_CS!("\\endgroup")));
    }
    let file = Tokens!(ExplodeText!(name));
    if let Some(path) = find_file(&file.to_string(), None) {
      reading_from_mouth(Mouth::create(&path, MouthOptions::default())?,
            || -> Result<Tokens> {
          let mut lines = Vec::new();
          with_mouth_mut(|mouth_opt| if let Some(mouth) = mouth_opt {
            while let Some(line) = mouth.read_raw_line(false) {
              lines.push(line);
            }
          });
          let mut tokens = Vec::new();
          for line in lines.into_iter() {
            tokens.push(T_CS!("\\verbatim@startline"));
            tokens.extend(Invocation!(T_CS!("\\verbatim@addtoline"),
              vec![Tokens::new(ExplodeText!(line))]).unlist());
            tokens.push(T_CS!("\\verbatim@processline"));
          }
          Ok(Tokens!(
            T_CS!("\\begingroup"), T_CS!("\\@verbatim"),
            T_CS!("\\frenchspacing"), T_CS!("\\@vobeyspaces"),
            T_CS!("\\lx@verbatim@"), tokens, T_CS!("\\lx@end@verbatim@"), T_CS!("\\endgroup"))
          )
        },
      )
    } else {
      let message = s!("\\verbatiminput found no file for {:?}, output may be incomplete", file);
      Error!("binding", "missing_file", message);
      Ok(Tokens!())
    }
  });

  // //======================================================================
  // // Getting verbatim text into arguments
  // DefPrimitive!("\\newverbtext DefToken", sub[args] {
  //     unpack!(args => cs);
  //     let mouth = gullet_mut!().get_mouth_mut();
  //     my ($init, $body);
  //     StartSemiverbatim();
  //     AssignCatcode('\\', CC_OTHER);
  //     AssignCatcode('{',  CC_OTHER);
  //     AssignCatcode('}',  CC_OTHER);
  //     $init = $mouth->readToken;
  //     $init = $mouth->readToken if ToString($init) == "*";    // Should I bother handling \verb*
  // ?

  //     if (!$init) {    // typically read too far, got \verb and the content is somewhere else..?
  //       Error("expected", "delimiter", $stomach,
  //         "Verbatim argument lost", "Bindings for preceding code is probably broken");
  //       EndSemiverbatim();
  //       return (); }
  //     $body = $mouth->readTokens($init);

  //     EndSemiverbatim();
  //     DefMacroI($cs, None, $body);
  //     return; });

  //**********************************************************************
  // verbatim.sty `\newread\verbatim@in@stream` — the read stream a document
  // may address directly (ltug notes-for-authors).
  RawTeX!(r"\newread\verbatim@in@stream");
});
