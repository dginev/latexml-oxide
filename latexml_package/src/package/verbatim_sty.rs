use crate::prelude::*;

/// A file of the session's virtual store (`{filecontents}`, `\openout`,
/// `{VerbatimOut}`), found as `\input` finds it: the name as given, else the
/// name `find_file` resolves it to — `<name>.tex` first, the name an
/// extension-less `\openout` writes (`output_file_name`, tex.web §537/§1374).
/// `\verbatiminput{democode}` after `{filecontents*}{democode}` read
/// `democode.tex` from disk and raised `Error:I/O` (batch 56jt review).
fn read_virtual_file(name: &str) -> Option<String> {
  vfs_read(name).or_else(|| find_file(name, None).and_then(|path| vfs_read(&path)))
}

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
    r"\verbatim@finish",
    r"\ifcat$\the\verbatim@line$\else\verbatim@processline\fi"
  );

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

  // verbatim.sty:107-112 `\verbatim@start#1` tests `\if\noexpand#1\noexpand~`,
  // `~` being the active end-of-line: only that line end is dropped. Any
  // control sequence compares as code 256 against its 13 (tex.web §506) and is
  // PREPENDED to line 1 (`\def\next{\verbatim@#1}`), where `\verbatim@processline`
  // runs it with the line's text after it. So `\verbatim@start\relax`, the idiom
  // for starting a capture from a macro body (curve2e-manual.tex:95 `{Esempio}`,
  // memoir, digiconfigs), makes an empty first line (dropped here: a no-op, and
  // our default `\verbatim@processline` would print it; only a `\relax` meaning,
  // OXIDIZED_DESIGN #274), and verbatimbox's
  // `\verbatim\verbbox@inner` (verbatimbox.sty:90/107) consumes the environment's
  // `[\footnotesize]` from line 1 (readarray.tex:440). The swallowed command lost
  // both: the bracket was captured as text and the size never set. Perl's
  // `\verbatim@start` = `\lx@verbatim@\verbatim@` (:76) never looks. A pending
  // character is a body character and goes back. Only the PUSHBACK is inspected:
  // the mouth's line remainder after `\begin{verbatim}` is line 1, the leading
  // newline every `<ltx:verbatim>` golden carries (`tests/tokenize/verbata.xml`).
  // Guards `perfect_kernel_batch50::verbatim_start_relax_idiom`,
  // `perfect_kernel_batch56::verbatim_start_runs_a_prepended_command`.
  DefMacro!("\\verbatim@start", {
    if pushback_holds_nonspace()
      && let Some(t) = read_token()?
    {
      if t.get_catcode() != Catcode::CS {
        unread_one(t);
      } else if !matches!(lookup_meaning(&t), Some(Stored::Primitive(ref p)) if *p.get_cs() == T_CS!("\\relax"))
      {
        return read_verbatim_lines(vec![t]);
      }
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
  DefMacro!("\\verbatim@", { read_verbatim_lines(Vec::new()) });

  // //======================================================================
  // // Read verbatim material from file.
  DefMacro!("\\verbatim@readfile {}", sub[(file)] {
    let name = do_expand(file)?.to_string();
    let trimmed = name.trim().trim_matches('"');
    if let Some(content) = read_virtual_file(trimmed) {
      let mut tokens = Vec::new();
      tokens.push(T_CS!("\\verbatim@startline"));
      for line in content.lines() {
        tokens.extend(
          Invocation!(
            T_CS!("\\verbatim@addtoline"),
            vec![Tokens::new(ExplodeText!(line.to_string()))]
          )
          .unlist(),
        );
        tokens.push(T_CS!("\\verbatim@processline"));
        tokens.push(T_CS!("\\verbatim@startline"));
      }
      tokens.push(T_CS!("\\verbatim@finish"));
      return Ok(Tokens::new(tokens));
    }
    if let Some(path) = find_file(trimmed, None) {
      reading_from_mouth(
        Mouth::create(&path, MouthOptions::default())?,
        || -> Result<Tokens> {
          let mut lines = Vec::new();
          with_mouth_mut(|mouth_opt| {
            if let Some(mouth) = mouth_opt {
              while let Some(line) = mouth.read_raw_line(false) {
                lines.push(line);
              }
            }
          });
          let mut tokens = Vec::new();
          tokens.push(T_CS!("\\verbatim@startline"));
          for line in lines.into_iter() {
            tokens.extend(
              Invocation!(
                T_CS!("\\verbatim@addtoline"),
                vec![Tokens::new(ExplodeText!(line))]
              )
              .unlist(),
            );
            tokens.push(T_CS!("\\verbatim@processline"));
            tokens.push(T_CS!("\\verbatim@startline"));
          }
          tokens.push(T_CS!("\\verbatim@finish"));
          Ok(Tokens::new(tokens))
        },
      )
    } else {
      let message = s!("No file {}. (\\verbatim@readfile)", trimmed);
      Warn!("binding", "missing_file", message);
      Ok(Tokens!(T_CS!("\\verbatim@finish")))
    }
  });

  DefMacro!("\\verbatiminput {}", sub[(file)] {
    // Expand the argument (`\verbatiminput{\jobname.tmp}`) and serve
    // in-memory filecontents/VerbatimOut captures before touching disk.
    let name = do_expand(file)?.to_string();
    let trimmed = name.trim().trim_matches('"');
    if let Some(content) = read_virtual_file(trimmed) {
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
    if let Some(path) = find_file(trimmed, None) {
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
      // verbatim.sty:210-217 `\verbatim@input` = `\IfFileExists{#2}{…}{\typeout
      // {No file #2.}}`: a missing file is a message, not an error (msc.tex:287
      // `\verbatiminput{COPYRIGHT}` beside COPYRIGHT.txt; lnosuppl.tex:89).
      // Perl verbatim.sty.ltxml:108 opens an empty-path Mouth and errors —
      // SHARED, pdflatex clean. Guard: `perfect_kernel_batch56::verbatiminput_missing_file_is_not_an_error`.
      let message = s!("No file {}. (\\verbatiminput)", trimmed);
      Warn!("binding", "missing_file", message);
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

/// Read the raw lines up to `\end{<current environment>}` as verbatim.sty's
/// line loop: `\verbatim@startline`, `\verbatim@addtoline{<line>}`,
/// `\verbatim@processline` per line, then `\end{<env>}`. `first` is prepended to
/// line 1 (the control sequence `\verbatim@start` found pending).
fn read_verbatim_lines(first: Vec<Token>) -> Result<Tokens> {
  let env = lookup_string_from_sym(pin!("current_environment"));
  // Note: This should allow a regexp, since there can be spaces between \end and { !!!
  let mut lines = Vec::new();
  // TODO: UGH!!! Isn't there a better way to approximate
  // the Perl simplicity of writing an inline regex?
  // the escaping is very easy to get wrong!
  let env_re = Regex::new(&format!("^(.*)\\\\end\\s*\\{{{env}\\}}(.*)$")).unwrap();
  // Decoded through the 8-bit input encoding, as `{verbatim}` is (latex_constructs).
  while let Some(line) = read_raw_line_decoded() {
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
  if !first.is_empty() && lines.is_empty() {
    lines.push(String::new());
  }
  for (n, line) in lines.iter().enumerate() {
    let mut content = if n == 0 { first.clone() } else { Vec::new() };
    content.extend(ExplodeText!(line));
    tokens.push(T_CS!("\\verbatim@startline"));
    tokens.extend(Invocation!(T_CS!("\\verbatim@addtoline"), vec![Tokens::new(content)]).unlist());
    tokens.push(T_CS!("\\verbatim@processline"));
  }
  tokens.extend(Invocation!(T_CS!("\\end"), vec![T_OTHER!(env)]).unlist());
  Ok(Tokens::new(tokens))
}
