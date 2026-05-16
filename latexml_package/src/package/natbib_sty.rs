use crate::prelude::*;

// Helper: set citation style values.
// Perl natbib.sty.ltxml L379-405: sub setCitationStyle { while (@pairs) { ($key,$value)=... }
// Unified dispatch: callers may pass flag-only pairs (value=None) from DeclareOption
// handlers or key=value pairs (value=Some(tokens)) from `\setcitestyle`'s getPairs.
fn set_citation_style(pairs: &[(&str, Option<Tokens>)]) {
  for (key, value) in pairs {
    match *key {
      // Perl L384: AssignValue(CITE_STYLE => 'authoryear');
      "authoryear" => {
        assign_value("CITE_STYLE", arena::pin("authoryear"), None);
      },
      // Perl L385:
      "numbers" => {
        assign_value("CITE_STYLE", arena::pin("numbers"), None);
      },
      // Perl L386:
      "super" => {
        assign_value("CITE_STYLE", arena::pin("super"), None);
      },
      // Perl L387-388: AssignValue(CITE_OPEN=>T_OTHER('(')); AssignValue(CITE_CLOSE=>T_OTHER(')'));
      "round" => {
        assign_value("CITE_OPEN", Stored::Token(T_OTHER!("(")), None);
        assign_value("CITE_CLOSE", Stored::Token(T_OTHER!(")")), None);
      },
      // Perl L389-390:
      "square" => {
        assign_value("CITE_OPEN", Stored::Token(T_OTHER!("[")), None);
        assign_value("CITE_CLOSE", Stored::Token(T_OTHER!("]")), None);
      },
      // Perl L391-392:
      "curly" => {
        assign_value("CITE_OPEN", Stored::Token(T_OTHER!("{")), None);
        assign_value("CITE_CLOSE", Stored::Token(T_OTHER!("}")), None);
      },
      // Perl L393-394:
      "angle" => {
        assign_value("CITE_OPEN", Stored::Token(T_OTHER!("<")), None);
        assign_value("CITE_CLOSE", Stored::Token(T_OTHER!(">")), None);
      },
      // Perl L395: AssignValue(CITE_OPEN => $value);
      "open" => {
        if let Some(v) = value {
          assign_value("CITE_OPEN", Stored::Tokens(v.clone()), None);
        }
      },
      // Perl L396:
      "close" => {
        if let Some(v) = value {
          assign_value("CITE_CLOSE", Stored::Tokens(v.clone()), None);
        }
      },
      // Perl L397:
      "semicolon" => {
        assign_value("CITE_SEPARATOR", Stored::Token(T_OTHER!(";")), None);
      },
      // Perl L398:
      "comma" => {
        assign_value("CITE_SEPARATOR", Stored::Token(T_OTHER!(",")), None);
      },
      // Perl L399: AssignValue(CITE_AY_SEPARATOR => $value);
      "aysep" => {
        if let Some(v) = value {
          assign_value("CITE_AY_SEPARATOR", Stored::Tokens(v.clone()), None);
        }
      },
      // Perl L400:
      "yysep" => {
        if let Some(v) = value {
          assign_value("CITE_YY_SEPARATOR", Stored::Tokens(v.clone()), None);
        }
      },
      // Perl L401:
      "notesep" => {
        if let Some(v) = value {
          assign_value("CITE_NOTE_SEPARATOR", Stored::Tokens(v.clone()), None);
        }
      },
      // Perl L402-404: fall-through — warn & default to authoryear.
      _ => {
        assign_value("CITE_STYLE", arena::pin("authoryear"), None);
        Info!(
          "unexpected",
          key,
          s!("Unexpected Citation Style keyword '{key}' using authoryear")
        );
      },
    }
  }
}

fn is_some_and_nonempty(opt: &Option<Tokens>) -> bool {
  match opt {
    Some(t) => !t.is_empty(),
    None => false,
  }
}

// Helper: get CITE_STYLE as string
fn cite_style() -> String { state::lookup_string("CITE_STYLE") }

fn cite_open() -> Tokens { state::lookup_tokens("CITE_OPEN").unwrap_or(NO_TOKENS) }
fn cite_close() -> Tokens { state::lookup_tokens("CITE_CLOSE").unwrap_or(NO_TOKENS) }
fn cite_ns() -> Tokens { state::lookup_tokens("CITE_NOTE_SEPARATOR").unwrap_or(NO_TOKENS) }
fn cite_ay() -> Tokens { state::lookup_tokens("CITE_AY_SEPARATOR").unwrap_or(NO_TOKENS) }

// Helper: handle the [pre][post] optional arg swap (if !post { pre,post = undef,pre })
fn swap_pre_post(pre: Option<Tokens>, post: Option<Tokens>) -> (Option<Tokens>, Option<Tokens>) {
  if !is_some_and_nonempty(&post) {
    (None, pre.filter(|t| !t.is_empty()))
  } else {
    (
      pre.filter(|t| !t.is_empty()),
      post.filter(|t| !t.is_empty()),
    )
  }
}

LoadDefinitions!({
  //======================================================================
  // 5. Package Options
  DeclareOption!("numbers", {
    set_citation_style(&[("numbers", None)]);
    Digest!("\\ExecuteOptions{square,comma,nobibstyle}")?;
  });
  DeclareOption!("super", {
    set_citation_style(&[("super", None)]);
    assign_value("CITE_OPEN", Stored::Tokens(Tokens!()), None);
    assign_value("CITE_CLOSE", Stored::Tokens(Tokens!()), None);
    Digest!("\\ExecuteOptions{nobibstyle}")?;
  });
  DeclareOption!("authoryear", {
    set_citation_style(&[("authoryear", None)]);
    Digest!("\\ExecuteOptions{round,semicolon,bibstyle}")?;
  });
  DeclareOption!("round", {
    set_citation_style(&[("round", None)]);
    Digest!("\\ExecuteOptions{nobibstyle}")?;
  });
  DeclareOption!("curly", {
    set_citation_style(&[("curly", None)]);
    Digest!("\\ExecuteOptions{nobibstyle}")?;
  });
  DeclareOption!("square", {
    set_citation_style(&[("square", None)]);
    Digest!("\\ExecuteOptions{nobibstyle}")?;
  });
  DeclareOption!("angle", {
    set_citation_style(&[("angle", None)]);
    Digest!("\\ExecuteOptions{nobibstyle}")?;
  });
  DeclareOption!("comma", {
    set_citation_style(&[("comma", None)]);
    Digest!("\\ExecuteOptions{nobibstyle}")?;
  });
  DeclareOption!("semicolon", {
    set_citation_style(&[("semicolon", None)]);
    Digest!("\\ExecuteOptions{nobibstyle}")?;
  });
  DeclareOption!("colon", {
    Digest!("\\ExecuteOptions{semicolon}")?;
  });
  DeclareOption!("nobibstyle", {
    Let!("\\bibstyle", "\\@gobble");
  });
  DeclareOption!("bibstyle", {
    Let!("\\bibstyle", "\\@citestyle");
  });
  DeclareOption!("sort", "");
  DeclareOption!("sort&compress", "");
  DeclareOption!("compress", "");
  DeclareOption!("longnamesfirst", "");
  DeclareOption!("openbib", "");
  DeclareOption!("sectionbib", {
    AssignMapping!("BACKMATTER_ELEMENT", "ltx:bibliography" => "ltx:section");
  });
  DeclareOption!("nonamebreak", "");

  // Perl: setCitationStyle(round => 1, semicolon => 1);
  set_citation_style(&[("round", None), ("semicolon", None)]);

  // \bibstyle dispatches to \bibstyle@NAME if defined
  DefMacro!("\\bibstyle{}", sub[(style_arg)] {
    let style = style_arg.to_string();
    let cs_name = s!("\\bibstyle@{style}");
    if has_meaning(&T_CS!(&cs_name)) {
      Ok(Tokens::new(vec![T_CS!(cs_name)]))
    } else {
      Ok(Tokens::new(vec![T_CS!("\\relax")]))
    }
  });
  Let!("\\@citestyle", "\\bibstyle");

  // Default: authoryear
  Digest!("\\ExecuteOptions{round,semicolon,authoryear}")?;
  AssignValue!("CITE_STYLE", "authoryear");

  ProcessOptions!();

  // CITE_AY_SEPARATOR default
  AssignValue!("CITE_AY_SEPARATOR", T_OTHER!(","));

  //======================================================================
  // 2.3 Basic Citation Commands
  // Override \cite from LaTeX.pool with natbib version
  DefMacro!("\\cite OptionalMatch:* [][] Semiverbatim", sub[args] {
    let mut it = args.into_iter();
    let star: Option<Tokens> = it.next().unwrap().into();
    let pre_raw: Option<Tokens> = it.next().unwrap().into();
    let post_raw: Option<Tokens> = it.next().unwrap().into();
    let keys: Tokens = it.next().unwrap().into();

    let style = cite_style();
    let open = cite_open();
    let close = cite_close();
    let ns = cite_ns();
    let ay = cite_ay();

    let (pre, post) = swap_pre_post(pre_raw, post_raw);
    let is_star = is_some_and_nonempty(&star);
    let author = if is_star { "FullAuthors" } else { "Authors" };

    if style == "numbers" {
      // Invocation(\@@cite, Tokens(Explode('cite')),
      //   Tokens(open, (pre? (pre, T_SPACE)),
      //     Invocation(\@@bibref, Tokens(Explode("Number")), keys, undef, undef),
      //     (post? (ns, T_SPACE, post)), close))
      let bibref = Invocation!(T_CS!("\\@@bibref"),
        vec![Tokens::new(Explode!("Number")), keys, Tokens!(), Tokens!()]);
      let mut body = open.unlist();
      if let Some(p) = pre.clone() { body.extend(p.unlist()); body.push(T_SPACE!()); }
      body.extend(bibref.unlist());
      if let Some(p) = post.clone() {
        body.extend(ns.unlist()); body.push(T_SPACE!()); body.extend(p.unlist());
      }
      body.extend(close.unlist());
      Ok(Invocation!(T_CS!("\\@@cite"),
        vec![Tokens::new(Explode!("cite")), Tokens::new(body)]))
    } else if style == "super" {
      // superscript style
      let bibref = Invocation!(T_CS!("\\@@bibref"),
        vec![Tokens::new(Explode!("Number")), keys, Tokens!(), Tokens!()]);
      let sup_arg = Invocation!(T_CS!("\\textsuperscript"), vec![bibref]);
      let mut body = Vec::new();
      if let Some(p) = pre.clone() { body.extend(p.unlist()); body.push(T_SPACE!()); }
      body.extend(sup_arg.unlist());
      if let Some(p) = post.clone() { body.push(T_SPACE!()); body.extend(p.unlist()); }
      Ok(Invocation!(T_CS!("\\@@cite"),
        vec![Tokens::new(Explode!("cite")), Tokens::new(body)]))
    } else {
      // authoryear
      if pre.is_some() || post.is_some() {
        // parenthetical with pre/post
        let show = s!("{author}Phrase1Year");
        let mut ay_space = ay.unlist();
        ay_space.push(T_SPACE!());
        let phrase1 = Invocation!(T_CS!("\\@@citephrase"),
          vec![Tokens::new(ay_space)]);
        let bibref = Invocation!(T_CS!("\\@@bibref"),
          vec![Tokens::new(Explode!(show)), keys, phrase1, Tokens!()]);
        let mut body = open.unlist();
        if let Some(p) = pre.clone() { body.extend(p.unlist()); body.push(T_SPACE!()); }
        body.extend(bibref.unlist());
        if let Some(p) = post.clone() {
          body.extend(ns.unlist()); body.push(T_SPACE!()); body.extend(p.unlist());
        }
        body.extend(close.unlist());
        Ok(Invocation!(T_CS!("\\@@cite"),
          vec![Tokens::new(Explode!("cite")), Tokens::new(body)]))
      } else {
        // textual (default authoryear with no pre/post)
        let show = s!("{author} Phrase1YearPhrase2");
        let phrase1 = Invocation!(T_CS!("\\@@citephrase"), vec![open]);
        let phrase2 = Invocation!(T_CS!("\\@@citephrase"), vec![close]);
        let bibref = Invocation!(T_CS!("\\@@bibref"),
          vec![Tokens::new(Explode!(show)), keys, phrase1, phrase2]);
        Ok(Invocation!(T_CS!("\\@@cite"),
          vec![Tokens::new(Explode!("cite")), bibref]))
      }
    }
  }, locked => true);

  //======================================================================
  // \citet
  DefMacro!("\\citet OptionalMatch:* [][] Semiverbatim", sub[args] {
    let mut it = args.into_iter();
    let star: Option<Tokens> = it.next().unwrap().into();
    let pre_raw: Option<Tokens> = it.next().unwrap().into();
    let post_raw: Option<Tokens> = it.next().unwrap().into();
    let keys: Tokens = it.next().unwrap().into();

    let style = cite_style();
    let open = cite_open();
    let close = cite_close();
    let ns = cite_ns();

    let (pre, post) = swap_pre_post(pre_raw, post_raw);
    let is_star = is_some_and_nonempty(&star);
    let author = if is_star { "FullAuthors" } else { "Authors" };

    if style == "numbers" {
      let show = s!("{author} Phrase1NumberPhrase2");
      let mut p1_toks = open.unlist();
      if let Some(p) = pre.clone() { p1_toks.extend(p.unlist()); p1_toks.push(T_SPACE!()); }
      let phrase1 = Invocation!(T_CS!("\\@@citephrase"), vec![Tokens::new(p1_toks)]);
      let mut p2_toks = Vec::new();
      if let Some(p) = post.clone() {
        p2_toks.extend(ns.unlist()); p2_toks.push(T_SPACE!()); p2_toks.extend(p.unlist());
      }
      p2_toks.extend(close.unlist());
      let phrase2 = Invocation!(T_CS!("\\@@citephrase"), vec![Tokens::new(p2_toks)]);
      let bibref = Invocation!(T_CS!("\\@@bibref"),
        vec![Tokens::new(Explode!(show)), keys, phrase1, phrase2]);
      Ok(Invocation!(T_CS!("\\@@cite"),
        vec![Tokens::new(Explode!("citet")), bibref]))
    } else if style == "super" {
      let show = s!("{author} Phrase1SuperPhrase2");
      let bibref = Invocation!(T_CS!("\\@@bibref"),
        vec![Tokens::new(Explode!(show)), keys, Tokens!(), Tokens!()]);
      let mut body = Vec::new();
      if let Some(p) = pre.clone() { body.extend(p.unlist()); body.push(T_SPACE!()); }
      body.extend(bibref.unlist());
      if let Some(p) = post.clone() {
        body.extend(ns.unlist()); body.push(T_SPACE!()); body.extend(p.unlist());
      }
      Ok(Invocation!(T_CS!("\\@@cite"),
        vec![Tokens::new(Explode!("citet")), Tokens::new(body)]))
    } else {
      // authoryear
      let show = s!("{author} Phrase1YearPhrase2");
      let mut p1_toks = open.unlist();
      if let Some(p) = pre.clone() { p1_toks.extend(p.unlist()); p1_toks.push(T_SPACE!()); }
      let phrase1 = Invocation!(T_CS!("\\@@citephrase"), vec![Tokens::new(p1_toks)]);
      let mut p2_toks = Vec::new();
      if let Some(p) = post.clone() {
        p2_toks.extend(ns.unlist()); p2_toks.push(T_SPACE!()); p2_toks.extend(p.unlist());
      }
      p2_toks.extend(close.unlist());
      let phrase2 = Invocation!(T_CS!("\\@@citephrase"), vec![Tokens::new(p2_toks)]);
      let bibref = Invocation!(T_CS!("\\@@bibref"),
        vec![Tokens::new(Explode!(show)), keys, phrase1, phrase2]);
      Ok(Invocation!(T_CS!("\\@@cite"),
        vec![Tokens::new(Explode!("citet")), bibref]))
    }
  }, locked => true);

  //======================================================================
  // \citep
  DefMacro!("\\citep OptionalMatch:* [][] Semiverbatim", sub[args] {
    let mut it = args.into_iter();
    let star: Option<Tokens> = it.next().unwrap().into();
    let pre_raw: Option<Tokens> = it.next().unwrap().into();
    let post_raw: Option<Tokens> = it.next().unwrap().into();
    let keys: Tokens = it.next().unwrap().into();

    let style = cite_style();
    let open = cite_open();
    let close = cite_close();
    let ns = cite_ns();
    let ay = cite_ay();

    let (pre, post) = swap_pre_post(pre_raw, post_raw);
    let is_star = is_some_and_nonempty(&star);
    let author = if is_star { "FullAuthors" } else { "Authors" };

    if style == "numbers" {
      let bibref = Invocation!(T_CS!("\\@@bibref"),
        vec![Tokens::new(Explode!("Number")), keys, Tokens!(), Tokens!()]);
      let mut body = open.unlist();
      if let Some(p) = pre.clone() { body.extend(p.unlist()); body.push(T_SPACE!()); }
      body.extend(bibref.unlist());
      if let Some(p) = post.clone() {
        body.extend(ns.unlist()); body.push(T_SPACE!()); body.extend(p.unlist());
      }
      body.extend(close.unlist());
      Ok(Invocation!(T_CS!("\\@@cite"),
        vec![Tokens::new(Explode!("citep")), Tokens::new(body)]))
    } else if style == "super" {
      let bibref = Invocation!(T_CS!("\\@@bibref"),
        vec![Tokens::new(Explode!("Super")), keys, Tokens!(), Tokens!()]);
      let mut body = Vec::new();
      if let Some(p) = pre.clone() { body.extend(p.unlist()); body.push(T_SPACE!()); }
      body.extend(bibref.unlist());
      if let Some(p) = post.clone() { body.push(T_SPACE!()); body.extend(p.unlist()); }
      Ok(Invocation!(T_CS!("\\@@cite"),
        vec![Tokens::new(Explode!("citep")), Tokens::new(body)]))
    } else {
      // authoryear
      let show = s!("{author}Phrase1Year");
      let mut ay_space = ay.unlist();
      ay_space.push(T_SPACE!());
      let phrase1 = Invocation!(T_CS!("\\@@citephrase"),
        vec![Tokens::new(ay_space)]);
      let bibref = Invocation!(T_CS!("\\@@bibref"),
        vec![Tokens::new(Explode!(show)), keys, phrase1, Tokens!()]);
      let mut body = open.unlist();
      if let Some(p) = pre.clone() { body.extend(p.unlist()); body.push(T_SPACE!()); }
      body.extend(bibref.unlist());
      if let Some(p) = post.clone() {
        body.extend(ns.unlist()); body.push(T_SPACE!()); body.extend(p.unlist());
      }
      body.extend(close.unlist());
      Ok(Invocation!(T_CS!("\\@@cite"),
        vec![Tokens::new(Explode!("citep")), Tokens::new(body)]))
    }
  }, locked => true);

  //======================================================================
  // 2.4 Extended Citation Commands

  // \@@cite@noparens — temporarily set open/close to empty
  DefMacro!("\\@@cite@noparens", {
    assign_value("CITE_OPEN", Stored::Tokens(Tokens!()), None);
    assign_value("CITE_CLOSE", Stored::Tokens(Tokens!()), None);
    Tokens!()
  });

  // \citealt = \bgroup \@@cite@noparens \citet ... \egroup
  DefMacro!("\\citealt OptionalMatch:* [][] Semiverbatim", sub[args] {
    let mut it = args.into_iter();
    let star: Option<Tokens> = it.next().unwrap().into();
    let pre: Option<Tokens> = it.next().unwrap().into();
    let post: Option<Tokens> = it.next().unwrap().into();
    let keys: Option<Tokens> = it.next().unwrap().into();
    let mut result = vec![T_CS!("\\bgroup"), T_CS!("\\@@cite@noparens")];
    let citet_inv = Invocation!(T_CS!("\\citet"),
      vec![star, pre, post, keys]);
    result.extend(citet_inv.unlist());
    result.push(T_CS!("\\egroup"));
    Ok(Tokens::new(result))
  });

  // \citealp = \bgroup \@@cite@noparens \citep ... \egroup
  DefMacro!("\\citealp OptionalMatch:* [][] Semiverbatim", sub[args] {
    let mut it = args.into_iter();
    let star: Option<Tokens> = it.next().unwrap().into();
    let pre: Option<Tokens> = it.next().unwrap().into();
    let post: Option<Tokens> = it.next().unwrap().into();
    let keys: Option<Tokens> = it.next().unwrap().into();
    let mut result = vec![T_CS!("\\bgroup"), T_CS!("\\@@cite@noparens")];
    let citep_inv = Invocation!(T_CS!("\\citep"),
      vec![star, pre, post, keys]);
    result.extend(citep_inv.unlist());
    result.push(T_CS!("\\egroup"));
    Ok(Tokens::new(result))
  });

  // \citenum
  DefMacro!("\\citenum Semiverbatim", sub[(keys)] {
    let bibref = Invocation!(T_CS!("\\@@bibref"),
      vec![Tokens::new(Explode!("Number")), keys, Tokens!(), Tokens!()]);
    Ok(Invocation!(T_CS!("\\@@cite"),
      vec![Tokens::new(Explode!("citenum")), bibref]))
  });

  // \citetext = \@@cite
  DefMacro!("\\citetext", "\\@@cite");

  // \citeauthor
  DefMacro!("\\citeauthor OptionalMatch:* [][] Semiverbatim", sub[args] {
    let mut it = args.into_iter();
    let star: Option<Tokens> = it.next().unwrap().into();
    let pre_raw: Option<Tokens> = it.next().unwrap().into();
    let post_raw: Option<Tokens> = it.next().unwrap().into();
    let keys: Tokens = it.next().unwrap().into();

    let is_star = is_some_and_nonempty(&star);
    let author = if is_star { "FullAuthors" } else { "Authors" };
    let (_, post) = swap_pre_post(pre_raw, post_raw);
    let ns = cite_ns();

    let bibref = Invocation!(T_CS!("\\@@bibref"),
      vec![Tokens::new(Explode!(author)), keys, Tokens!(), Tokens!()]);
    let mut body = bibref.unlist();
    if let Some(p) = post.clone() {
      body.extend(ns.unlist()); body.push(T_SPACE!()); body.extend(p.unlist());
    }
    Ok(Invocation!(T_CS!("\\@@cite"),
      vec![Tokens::new(Explode!("citeauthor")), Tokens::new(body)]))
  });

  // \citefullauthor
  DefMacro!("\\citefullauthor [][] Semiverbatim", sub[args] {
    let mut it = args.into_iter();
    let pre_raw: Option<Tokens> = it.next().unwrap().into();
    let post_raw: Option<Tokens> = it.next().unwrap().into();
    let keys: Tokens = it.next().unwrap().into();

    let (_, post) = swap_pre_post(pre_raw, post_raw);
    let ns = cite_ns();

    let bibref = Invocation!(T_CS!("\\@@bibref"),
      vec![Tokens::new(Explode!("FullAuthors")), keys, Tokens!(), Tokens!()]);
    let mut body = bibref.unlist();
    if let Some(p) = post.clone() {
      body.extend(ns.unlist()); body.push(T_SPACE!()); body.extend(p.unlist());
    }
    Ok(Invocation!(T_CS!("\\@@cite"),
      vec![Tokens::new(Explode!("citefullauthor")), Tokens::new(body)]))
  });

  // \citeyear
  DefMacro!("\\citeyear [][] Semiverbatim", sub[args] {
    let mut it = args.into_iter();
    let pre_raw: Option<Tokens> = it.next().unwrap().into();
    let post_raw: Option<Tokens> = it.next().unwrap().into();
    let keys: Tokens = it.next().unwrap().into();

    let (_, post) = swap_pre_post(pre_raw, post_raw);
    let ns = cite_ns();

    let bibref = Invocation!(T_CS!("\\@@bibref"),
      vec![Tokens::new(Explode!("Year")), keys, Tokens!(), Tokens!()]);
    let mut body = bibref.unlist();
    if let Some(p) = post.clone() {
      body.extend(ns.unlist()); body.push(T_SPACE!()); body.extend(p.unlist());
    }
    Ok(Invocation!(T_CS!("\\@@cite"),
      vec![Tokens::new(Explode!("citeyear")), Tokens::new(body)]))
  });

  // \citeyearpar
  DefMacro!("\\citeyearpar [][] Semiverbatim", sub[args] {
    let mut it = args.into_iter();
    let pre_raw: Option<Tokens> = it.next().unwrap().into();
    let post_raw: Option<Tokens> = it.next().unwrap().into();
    let keys: Tokens = it.next().unwrap().into();

    let open = cite_open();
    let close = cite_close();
    let (pre, post) = swap_pre_post(pre_raw, post_raw);
    let ns = cite_ns();

    let bibref = Invocation!(T_CS!("\\@@bibref"),
      vec![Tokens::new(Explode!("Year")), keys, Tokens!(), Tokens!()]);
    let mut body = open.unlist();
    if let Some(p) = pre.clone() { body.extend(p.unlist()); body.push(T_SPACE!()); }
    body.extend(bibref.unlist());
    if let Some(p) = post.clone() {
      body.extend(ns.unlist()); body.push(T_SPACE!()); body.extend(p.unlist());
    }
    body.extend(close.unlist());
    Ok(Invocation!(T_CS!("\\@@cite"),
      vec![Tokens::new(Explode!("citeyearpar")), Tokens::new(body)]))
  });

  //======================================================================
  // 2.5 Forcing Upper Cased Name (SUPPOSED to capitalize first letter)
  DefMacro!("\\Citet", "\\citet");
  DefMacro!("\\Citep", "\\citep");
  DefMacro!("\\Citealt", "\\citealt");
  DefMacro!("\\Citealp", "\\citealp");
  DefMacro!("\\Citeauthor", "\\citeauthor");

  //======================================================================
  // 2.6 Citation Aliasing
  DefPrimitive!("\\defcitealias Semiverbatim {}", sub[args] {
    let mut it = args.into_iter();
    let key: Tokens = it.next().unwrap().into();
    let text: Tokens = it.next().unwrap().into();
    let key_str = key.to_string();
    def_macro(T_CS!(s!("\\al@{key_str}")), None, text, None)?;
  });

  DefMacro!("\\citetalias [][] Semiverbatim", sub[args] {
    let mut it = args.into_iter();
    let pre_raw: Option<Tokens> = it.next().unwrap().into();
    let post_raw: Option<Tokens> = it.next().unwrap().into();
    let key: Tokens = it.next().unwrap().into();

    let (pre, post) = swap_pre_post(pre_raw, post_raw);
    let key_str = key.to_string();
    let alias_cs = T_CS!(s!("\\al@{key_str}"));

    let phrase1 = Invocation!(T_CS!("\\@@citephrase"),
      vec![Tokens::new(vec![alias_cs])]);
    let bibref = Invocation!(T_CS!("\\@@bibref"),
      vec![Tokens::new(Explode!("Phrase1")), key, phrase1, Tokens!()]);
    let mut body = Vec::new();
    if let Some(p) = pre.clone() { body.extend(p.unlist()); body.push(T_SPACE!()); }
    body.extend(bibref.unlist());
    if let Some(p) = post.clone() { body.push(T_SPACE!()); body.extend(p.unlist()); }
    Ok(Invocation!(T_CS!("\\@@cite"),
      vec![Tokens::new(Explode!("citealias")), Tokens::new(body)]))
  });

  DefMacro!("\\citepalias [][] Semiverbatim", sub[args] {
    let mut it = args.into_iter();
    let pre_raw: Option<Tokens> = it.next().unwrap().into();
    let post_raw: Option<Tokens> = it.next().unwrap().into();
    let key: Tokens = it.next().unwrap().into();

    let open = cite_open();
    let close = cite_close();
    let ns = cite_ns();
    let (pre, post) = swap_pre_post(pre_raw, post_raw);
    let key_str = key.to_string();
    let alias_cs = T_CS!(s!("\\al@{key_str}"));

    let phrase1 = Invocation!(T_CS!("\\@@citephrase"),
      vec![Tokens::new(vec![alias_cs])]);
    let bibref = Invocation!(T_CS!("\\@@bibref"),
      vec![Tokens::new(Explode!("Phrase1")), key, phrase1, Tokens!()]);
    let mut body = open.unlist();
    if let Some(p) = pre.clone() { body.extend(p.unlist()); body.push(T_SPACE!()); }
    body.extend(bibref.unlist());
    if let Some(p) = post.clone() {
      body.extend(ns.unlist()); body.push(T_SPACE!()); body.extend(p.unlist());
    }
    body.extend(close.unlist());
    Ok(Invocation!(T_CS!("\\@@cite"),
      vec![Tokens::new(Explode!("citepalias")), Tokens::new(body)]))
  });

  //======================================================================
  // 2.9 Selecting Citation Punctuation

  // Perl natbib.sty.ltxml L363-375 — \setcitestyle keys.
  // Note: Perl declares these exact keys — no `curly`, no `angle`. The internal
  // `setCitationStyle` helper handles those flags via DeclareOption dispatch only.
  DefKeyVal!("natbib", "authoryear", "");
  DefKeyVal!("natbib", "numbers", "");
  DefKeyVal!("natbib", "super", "");
  DefKeyVal!("natbib", "round", "");
  DefKeyVal!("natbib", "square", "");
  DefKeyVal!("natbib", "open", "");
  DefKeyVal!("natbib", "close", "");
  DefKeyVal!("natbib", "semicolon", "");
  DefKeyVal!("natbib", "comma", "");
  DefKeyVal!("natbib", "citesep", "");
  DefKeyVal!("natbib", "aysep", "");
  DefKeyVal!("natbib", "yysep", "");
  DefKeyVal!("natbib", "notesep", "");

  // \setcitestyle — Perl L407-408: DefPrimitive('\setcitestyle RequiredKeyVals:natbib',
  //   sub { setCitationStyle($_[1]->getPairs); });
  DefPrimitive!("\\setcitestyle RequiredKeyVals:natbib", sub[(kv)] {
    // Own each pair so the helper can borrow &str slices safely.
    let owned: Vec<(String, Option<Tokens>)> = kv
      .get_pairs()
      .map(|(k, v)| {
        let toks = v.as_tokens().ok().flatten().map(|c| c.into_owned());
        (k.clone(), toks)
      })
      .collect();
    let pairs: Vec<(&str, Option<Tokens>)> =
      owned.iter().map(|(k, v)| (k.as_str(), v.clone())).collect();
    set_citation_style(&pairs);
  });

  // \bibpunct
  DefPrimitive!("\\bibpunct[]{}{}{}{}{}{}", sub[args] {
    let mut it = args.into_iter();
    let notesep: Option<Tokens> = it.next().unwrap().into();
    let open_arg: Option<Tokens> = it.next().unwrap().into();
    let close_arg: Option<Tokens> = it.next().unwrap().into();
    let sep_arg: Option<Tokens> = it.next().unwrap().into();
    let style_arg: Option<Tokens> = it.next().unwrap().into();
    let aysep_arg: Option<Tokens> = it.next().unwrap().into();
    let yysep_arg: Option<Tokens> = it.next().unwrap().into();

    if let Some(o) = open_arg {
      assign_value("CITE_OPEN", Stored::Tokens(o), None);
    }
    if let Some(c) = close_arg {
      assign_value("CITE_CLOSE", Stored::Tokens(c), None);
    }
    if let Some(s) = sep_arg {
      assign_value("CITE_SEPARATOR", Stored::Tokens(s), None);
    }
    if let Some(st) = style_arg {
      let style_str = Digest!(st)?.to_string();
      let cite_style = if style_str == "n" { "numbers" }
        else if style_str == "s" { "super" }
        else { "authoryear" };
      assign_value("CITE_STYLE", arena::pin(cite_style), None);
    }
    if let Some(ay) = aysep_arg {
      assign_value("CITE_AY_SEPARATOR", Stored::Tokens(ay), None);
    }
    if let Some(yy) = yysep_arg {
      assign_value("CITE_YY_SEPARATOR", Stored::Tokens(yy), None);
    }
    if is_some_and_nonempty(&notesep) {
      assign_value("CITE_NOTE_SEPARATOR", Stored::Tokens(notesep.unwrap()), None);
    }
  });

  DefMacro!("\\citestyle{}", "\\@citestyle{#1}\\let\\bibstyle\\@gobble");

  // bibstyle@* macros for known styles
  DefMacro!("\\bibstyle@chicago", "\\bibpunct{(}{)}{;}{a}{,}{,}");
  DefMacro!("\\bibstyle@named", "\\bibpunct{[}{]}{;}{a}{,}{,}");
  DefMacro!("\\bibstyle@agu", "\\bibpunct{[}{]}{;}{a}{,}{,~}");
  DefMacro!("\\bibstyle@copernicus", "\\bibpunct{(}{)}{;}{a}{,}{,}");
  Let!("\\bibstyle@egu", "\\bibstyle@copernicus");
  Let!("\\bibstyle@egs", "\\bibstyle@copernicus");
  DefMacro!(
    "\\bibstyle@agsm",
    "\\bibpunct{(}{)}{,}{a}{}{,}\\gdef\\harvardand{\\&}"
  );
  DefMacro!(
    "\\bibstyle@kluwer",
    "\\bibpunct{(}{)}{,}{a}{}{,}\\gdef\\harvardand{\\&}"
  );
  DefMacro!(
    "\\bibstyle@dcu",
    "\\bibpunct{(}{)}{;}{a}{;}{,}\\gdef\\harvardand{and}"
  );
  DefMacro!("\\bibstyle@aa", "\\bibpunct{(}{)}{;}{a}{}{,}");
  DefMacro!("\\bibstyle@pass", "\\bibpunct{(}{)}{;}{a}{,}{,}");
  DefMacro!("\\bibstyle@anngeo", "\\bibpunct{(}{)}{;}{a}{,}{,}");
  DefMacro!("\\bibstyle@nlinproc", "\\bibpunct{(}{)}{;}{a}{,}{,}");
  DefMacro!("\\bibstyle@cospar", "\\bibpunct{/}{/}{,}{n}{}{}");
  DefMacro!("\\bibstyle@esa", "\\bibpunct{(Ref.~}{)}{,}{n}{}{}");
  DefMacro!(
    "\\bibstyle@nature",
    "\\bibpunct{}{}{,}{s}{}{\\textsuperscript{,}}"
  );
  DefMacro!("\\bibstyle@plain", "\\bibpunct{[}{]}{,}{n}{}{,}");
  Let!("\\bibstyle@alpha", "\\bibstyle@plain");
  Let!("\\bibstyle@abbrv", "\\bibstyle@plain");
  Let!("\\bibstyle@unsrt", "\\bibstyle@plain");
  DefMacro!("\\bibstyle@plainnat", "\\bibpunct{[}{]}{,}{a}{,}{,}");
  Let!("\\bibstyle@abbrvnat", "\\bibstyle@plainnat");
  Let!("\\bibstyle@unsrtnat", "\\bibstyle@plainnat");
  //======================================================================
  // 2.12 Other Formatting Options
  DefMacro!("\\bibname", "Bibliography");
  DefMacro!("\\refname", "References");
  DefMacro!("\\bibsection", "");
  DefMacro!("\\bibpreamble", "");
  DefMacro!("\\bibfont", "");
  DefMacro!("\\citenumfont", "");
  DefMacro!("\\bibnumfmt{}", "#1");
  DefRegister!("\\bibhang", Dimension::new(0));
  DefRegister!("\\bibsep", Glue::new(0));

  //======================================================================
  // 2.13 Automatic Indexing of Citations
  RawTeX!("\\newif\\ifciteindex");
  DefMacro!("\\citeindextrue", "");
  DefMacro!("\\citeindexfalse", "");
  DefMacro!("\\citeindextype", "");

  // natbib boolean flags consulted by raw-loaded sibling packages
  // and some user macros. Predefine the standard `\if<name>` triple
  // via `\newif` (creates `\NAT@superfalse`/`\NAT@supertrue` too).
  // Witness cluster: arXiv:2506.21088 / .21438 (papers calling
  // `\bibpunct` or similar that consult `\ifNAT@super`).
  RawTeX!("\\newif\\ifNAT@super");
  RawTeX!("\\newif\\ifNAT@numbers");
  RawTeX!("\\newif\\ifNAT@longnamesfirst");
  RawTeX!("\\newif\\ifNAT@swa");

  // 2.17 Long Author List on First Citation
  DefMacro!("\\shortcites Semiverbatim", "");

  //======================================================================
  // Bibliography item handling

  // NAT@wrout: format the refnum based on citation style
  DefMacro!("\\NAT@wrout{}{}{}{} Semiverbatim", sub[args] {
    let mut it = args.into_iter();
    let number: Option<Tokens> = it.next().unwrap().into();
    let year: Option<Tokens> = it.next().unwrap().into();
    let authors: Option<Tokens> = it.next().unwrap().into();
    let fullauthors: Option<Tokens> = it.next().unwrap().into();
    let key: Option<Tokens> = it.next().unwrap().into();

    let style = cite_style();
    let open = cite_open();
    let close = cite_close();

    // If authors or year empty, fall back to number style
    let use_number = !is_some_and_nonempty(&authors) || !is_some_and_nonempty(&year);
    let style = if use_number { "number" } else { &style };

    let refnum = if style == "number" {
      let mut toks = open.unlist();
      if let Some(n) = number.clone() { toks.extend(n.unlist()); }
      toks.extend(close.unlist());
      Some(Tokens::new(toks))
    } else {
      let mut toks = Vec::new();
      if let Some(a) = authors.clone() { toks.extend(a.unlist()); }
      toks.push(T_SPACE!());
      toks.extend(open.unlist());
      if let Some(y) = year.clone() { toks.extend(y.unlist()); }
      toks.extend(close.unlist());
      Some(Tokens::new(toks))
    };

    Ok(Invocation!(T_CS!("\\NAT@@wrout"),
      vec![number, year, authors, fullauthors, refnum, key]))
  });

  // NAT@@wrout — constructor to produce the ltx:tags
  //
  // Perl uses `bounded => 1` here (`natbib.sty.ltxml:632`); the `bounded`
  // flag triggers a `bgroup`/`egroup` pair around the constructor body.
  // In Rust the equivalent `bounded => true` interacts badly with
  // `\@@cite`-in-author-tag digestion: the inner `\@@cite` (mode='text',
  // enterHorizontal) leaks a `BOUND_MODE`/`MODE` binding into our outer
  // bgroup frame, and the closing `egroup` then errors with "Attempt
  // to close a group that switched to mode internal_vertical".
  // Driver: 2404.06289 — `\bibitem [{...\cite{a}...}]{key}` cascade
  // (\NAT@wrout passes the expanded label tokens, which still contain
  // `\@@cite[opt]{body}` Constructor invocations, to `#3`/authors).
  // Skipping `bounded` — and thus the egroup's mode-frame check —
  // matches what Perl gets in practice (Perl runs the same input
  // cleanly), without the parselabel-rewrite churn. The Let on T_ALIGN
  // moves into a manual bgroup/egroup so the assignment is still
  // scope-isolated to argument digestion.
  DefConstructor!("\\NAT@@wrout{}{}{}{}{} Semiverbatim",
    "<ltx:tags>\
      ?#1(<ltx:tag role='number'>#1</ltx:tag>)\
      ?#2(<ltx:tag role='year'>#2</ltx:tag>)\
      ?#3(<ltx:tag role='authors'>#3</ltx:tag>)\
      ?#4(<ltx:tag role='fullauthors'>#4</ltx:tag>)\
      ?#5(<ltx:tag role='refnum'>#5</ltx:tag>)\
      ?#6(<ltx:tag role='key'>#6</ltx:tag>)\
    </ltx:tags>",
    before_digest => {
      bgroup();
      Let!(T_ALIGN!(), T_CS!("\\&"));
    },
    after_digest => sub[_whatsit] {
      // Soft egroup: pop_stack_frame directly, bypassing the standard
      // `egroup`'s `is_value_bound("BOUND_MODE", Some(0))` mode-switch
      // check. Inner `\@@cite`-style mode pushes can leak a BOUND_MODE
      // binding into our frame, but we want to recover (matching Perl
      // behavior on the same input).
      latexml_core::stomach::pop_stack_frame(false)?;
      Ok(Vec::new())
    }
  );

  //======================================================================
  // \lx@NAT@parselabel — parse the bibitem label
  // This is complex in Perl; simplified version that handles the main patterns
  DefMacro!("\\lx@NAT@parselabel{}{}", sub[args] {
    let mut it = args.into_iter();
    let label: Tokens = it.next().unwrap().into();
    let key: Tokens = it.next().unwrap().into();

    let mut tokens = label.clone().unlist();
    let number: Option<Tokens> = None;
    let mut year: Option<Tokens> = None;
    let mut authors: Option<Tokens> = None;
    let mut fullauthors: Option<Tokens> = None;
    let mut bare = true;

    // Skip \protect if present
    if !tokens.is_empty() && tokens[0] == T_CS!("\\protect") {
      tokens.remove(0);
    }

    // Check if first token is a CS
    if !tokens.is_empty() && tokens[0].get_catcode() == Catcode::CS {
      let cs = tokens[0];
      if cs == T_CS!("\\citeauthoryear") {
        tokens.remove(0);
        let (a1, rest) = nat_peel_arg(tokens);
        let (a2, rest) = nat_peel_arg(rest);
        if !rest.is_empty() {
          fullauthors = a1;
          authors = a2;
          year = Some(Tokens::new(rest));
        } else {
          authors = a1;
          year = a2;
        }
        bare = false;
      } else if cs == T_CS!("\\astroncite") {
        tokens.remove(0);
        let (a1, rest) = nat_peel_arg(tokens);
        let (a2, _rest) = nat_peel_arg(rest);
        authors = a1;
        year = a2;
        bare = false;
      } else if cs == T_CS!("\\citename") {
        tokens.remove(0);
        let (a1, rest) = nat_peel_arg(tokens);
        authors = a1;
        year = Some(Tokens::new(rest));
        bare = false;
      }
    }

    if bare {
      // If the label contains "complex" CSes (e.g. `\cite`, `\href`) that
      // expand into Constructor invocations whose parameter readers
      // would drain the wrapping `do_expand` gullet hunting for absent
      // arguments, the resulting `readBalanced ran out of input`
      // diagnostic is spurious — extracting an author/year out of such
      // a cite-bearing label is meaningless anyway. Skip the expansion
      // and just walk the raw label tokens for the `(year)` pattern.
      // Perl on the same input is silent (`natbib.sty.ltxml:564`'s
      // `Expand` runs the macros differently for these cases). Driver:
      // 2404.06289 `\bibitem [{...\cite{a}...}]{key}`.
      let has_complex_cs = label.unlist_ref().iter().any(|t| {
        if t.get_catcode() != Catcode::CS { return false; }
        let n = t.to_string();
        matches!(n.as_str(),
          "\\cite" | "\\citet" | "\\citep" | "\\citeauthor" | "\\citeyear"
          | "\\href" | "\\hyperref" | "\\url" | "\\nolinkurl"
          | "\\BibitemOpen" | "\\BibitemShut" | "\\bibinfo" | "\\bibfield"
        )
      });
      let expanded = if has_complex_cs {
        label.clone()
      } else {
        Expand!(label.clone())
      };
      let exp_tokens = expanded.unlist();
      let mut author_toks = Vec::new();
      let mut year_toks = Vec::new();
      let mut rest_idx = 0;
      let mut found_paren = false;

      // Find the opening paren
      for (i, t) in exp_tokens.iter().enumerate() {
        if *t == T_OTHER!("(") {
          rest_idx = i + 1;
          found_paren = true;
          break;
        }
        author_toks.push(*t);
      }

      if found_paren {
        // Read until closing paren
        for (i, t) in exp_tokens[rest_idx..].iter().enumerate() {
          if *t == T_OTHER!(")") {
            rest_idx = rest_idx + i + 1;
            break;
          }
          year_toks.push(*t);
        }
      } else {
        // No paren — try splitting digits from end
        while !author_toks.is_empty() {
          let last = author_toks.last().unwrap();
          if last.get_catcode() == Catcode::OTHER {
            let ch = last.to_string();
            if ch.chars().all(|c| c.is_ascii_digit()) && !ch.is_empty() {
              year_toks.insert(0, author_toks.pop().unwrap());
              continue;
            }
          }
          break;
        }
      }

      authors = if author_toks.is_empty() { None }
        else { Some(Tokens::new(author_toks)) };
      year = if year_toks.is_empty() { None }
        else { Some(Tokens::new(year_toks)) };
      // Remaining tokens after ) are fullauthors
      if rest_idx < exp_tokens.len() {
        let fa: Vec<_> = exp_tokens[rest_idx..].to_vec();
        if !fa.is_empty() { fullauthors = Some(Tokens::new(fa)); }
      }
    }

    let number_arg: Option<Tokens> = match number {
      Some(n) => Some(n),
      None => Some(Tokens::new(vec![T_CS!("\\the@bibitem")]))
    };
    // All args for required {} params must be Some (even if empty) to ensure
    // revert produces {} braces. None would skip the arg entirely, causing arg-shifting.
    let empty = || Some(Tokens!());
    Ok(Invocation!(T_CS!("\\NAT@wrout"),
      vec![number_arg, year.or_else(empty), authors.or_else(empty),
           fullauthors.or_else(empty), Some(key)]))
  });

  // \lx@nat@bibitem — natbib's version of \bibitem
  DefMacro!("\\lx@nat@bibitem",
    "\\reset@natbib@cites\\refstepcounter{@bibitem}\\@ifnextchar[{\\@lbibitem}{\\@lbibitem[]}",
    locked => true);
  Let!("\\bibitem", "\\lx@nat@bibitem");

  // RawTeX for citeauthoryear, astroncite, citename, harvarditem, NAT@ifcmd etc.
  RawTeX!(
    "\\def\\citeauthoryear#1#2#3(@)(@)\\@nil#4{%
\\if\\relax#3\\relax%
\\NAT@wrout{\\the@bibitem}{#2}{#1}{}{#4}\\else%
\\NAT@wrout{\\the@bibitem}{#3}{#2}{#1}{#4}\\fi}"
  );
  RawTeX!("\\let\\natbib@citeauthoryear\\citeauthoryear");
  RawTeX!(
    "\\def\\astroncite#1#2(@)(@)\\@nil#3{%
\\NAT@wrout{\\the@bibitem}{#2}{#1}{}{#3}}"
  );
  RawTeX!("\\let\\natbib@astroncite\\astroncite");
  RawTeX!(
    "\\def\\citename#1#2(@)(@)\\@nil#3{%
\\expandafter\\NAT@apalk#1#2, \\@nil{#3}}"
  );
  RawTeX!("\\let\\natbib@citename\\citename");
  RawTeX!(
    "\\newcommand\\harvarditem[4][]{%
\\if\\relax#1\\relax\\bibitem[#2(#3)]{#4}\\else\\bibitem[#1(#3)#2]{#4}\\fi}"
  );
  RawTeX!(
    "\\def\\NAT@apalk#1, #2, #3\\@nil#4{%
\\NAT@wrout{\\the@bibitem}{#2}{#1}{}{#4}}"
  );

  // \reset@natbib@cites
  DefPrimitive!("\\reset@natbib@cites", None,
  after_digest => {
    Let!("\\citeauthoryear", "\\natbib@citeauthoryear");
    Let!("\\astroncite", "\\natbib@astroncite");
    Let!("\\citename", "\\natbib@citename");
  });

  // \@lbibitem — use lx@NAT@parselabel instead of raw NAT@ifcmd.
  //
  // Read the {key} arg as `Semiverbatim` (catcodes neutralized to OTHER)
  // not plain `{}`. Bibitem keys can contain `_`, `^`, `&` — DBLP-style
  // keys like `DBLP:conf/nips/incontext_AgarwalSZBRCZAA24` are common in
  // arXiv `.bbl` files (witnesses: 2509.20805 / 2510.00068 / 2510.00632).
  // Without semiverbatim, the `_` retains catcode SUB, and after macro
  // substitution the SUB token leaks into the post-`\@@lbibitem`
  // `\newblock` text stream — Stomach then errors with
  // `Error:unexpected:_ Script _ can only appear in math mode`.
  // Perl's natbib.sty.ltxml:638 uses `[]{}` and gets away with it because
  // its parameter reader's brace-group capture neutralizes catcodes by
  // default; Rust's `{}` arg type preserves the source catcodes faithfully.
  // 54 occurrences of `Error:unexpected:_` in Stage-16 v5 cluster here.
  DefMacro!("\\@lbibitem[] Semiverbatim",
    "\\@@lbibitem{#2}\\lx@NAT@parselabel{#1}{#2}\\newblock",
    locked => true);

  // \@@lbibitem — constructor opening the bibitem element
  DefConstructor!("\\@@lbibitem Semiverbatim",
  "<ltx:bibitem key='#key' xml:id='#id'>",
  after_digest => sub[whatsit] {
    let key_str = whatsit.get_arg(1).map(|a| {
      clean_bib_key(&a.to_string())
    }).unwrap_or_default();
    let id = Digest!(T_CS!("\\the@bibitem@ID"))?.to_string();
    whatsit.set_property("key", key_str);
    whatsit.set_property("id", id);
  });

  //======================================================================
  // Misc macros
  DefMacro!("\\citestarts", { cite_open() });
  DefMacro!("\\citeends", { cite_close() });
  DefMacro!("\\betweenauthors", "and");

  DefMacro!("\\harvardleft", { cite_open() });
  DefMacro!("\\harvardright", { cite_close() });
  DefMacro!("\\harvardyearleft", { cite_open() });
  DefMacro!("\\harvardyearright", { cite_close() });
  DefMacro!("\\harvardand", "and");

  // Perl L663-666: DefConstructor('\harvardurl Semiverbatim', ..., enterHorizontal => 1, ...).
  // WISDOM #45 (wisdom_mode_text_auto_enter_horizontal): Rust's
  // DefConstructor with `mode => "text"` already auto-triggers
  // `needs_enter_horizontal`, so `mode => "text"` alone covers Perl's
  // enterHorizontal => 1. Keeping `mode => "text"` and NOT adding a
  // redundant `enter_horizontal => true`.
  DefConstructor!("\\harvardurl Semiverbatim",
    "<ltx:ref href='#href'>#1</ltx:ref>",
    mode => "text",
    properties => sub[args] {
      unpack_opt_ref!(args => url_opt);
      let href = url_opt.as_ref().map_or(String::new(), |u| u.to_string());
      Ok(stored_map!("href" => href))
    }
  );

  Let!("\\citeN", "\\cite");
  Let!("\\shortcite", "\\cite");
  Let!("\\citeasnoun", "\\cite");

  DefMacro!("\\natexlab{}", "#1");
});

// Helper: peel a brace-delimited argument from a token list
fn nat_peel_arg(mut tokens: Vec<Token>) -> (Option<Tokens>, Vec<Token>) {
  if tokens.is_empty() {
    return (None, tokens);
  }
  if tokens[0].get_catcode() != Catcode::BEGIN {
    // Not braced — single token
    let t = tokens.remove(0);
    return (Some(Tokens::new(vec![t])), tokens);
  }
  // Braced group
  tokens.remove(0); // remove opening brace
  let mut arg = Vec::new();
  let mut level = 1i32;
  while let Some(t) = tokens.first().cloned() {
    tokens.remove(0);
    let cc = t.get_catcode();
    if cc == Catcode::BEGIN {
      level += 1;
    }
    if cc == Catcode::END {
      level -= 1;
      if level == 0 {
        break;
      }
    }
    arg.push(t);
  }
  let result = if arg.is_empty() {
    None
  } else {
    Some(Tokens::new(arg))
  };
  (result, tokens)
}
