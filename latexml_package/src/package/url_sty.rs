use crate::prelude::*;

pub static LEADING_BACKSLASH_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^\\").unwrap());


LoadDefinitions!({
  AssignValue!("BASE_URL", "");

  // Ignorable stuff, since we're not doing linebreaks.
  def_macro_noop("\\UrlBreaks")?;
  def_macro_noop("\\UrlBigBreaks")?;
  def_macro_noop("\\UrlNoBreaks")?;
  def_macro_noop("\\UrlOrds")?;
  def_macro_noop("\\UrlSpecials")?;

  // Font style definitions.
  DefMacro!(
    "\\urlstyle{}",
    r#"\expandafter\protect\csname url@#1style\endcsname"#
  );
  DefMacro!("\\url@ttstyle", "\\def\\UrlFont{\\ttfamily}");
  DefMacro!("\\url@rmstyle", "\\def\\UrlFont{\\rmfamily}");
  DefMacro!("\\url@sfstyle", "\\def\\UrlFont{\\sffamily}");
  def_macro_noop("\\url@samestyle")?;
  DefMacro!("\\UrlFont", "\\ttfamily");

  // Bracketting.
  Let!("\\UrlLeft", "\\@empty");
  Let!("\\UrlRight", "\\@empty");

  // \DeclareUrlCommand\cmd{settings}
  // Have this expand into \lx@url@url w/ the declared cmd as arg, so it gets reflected in XML.
  DefMacro!(
    "\\DeclareUrlCommand{}{}",
    r#"\def#1{\begingroup #2\lx@url@url#1}"#
  );

  // This is an extended version of \Url that takes an extra token as 1st arg.
  // That token is the cs that invoked it, so that it can be reflected in the generated XML,
  // as well as used to generate the reversion.
  // In any case, we read the verbatim arg, and build a Whatsit for @@Url
  DefMacro!("\\lx@url@url Token", sub[(cmd)] {
    let perc = vec!['%'];
   begin_semiverbatim(Some(&perc));
    let mut open = gullet::read_token()?.unwrap();
    let close;
    let url = if open.get_catcode() == Catcode::BEGIN {
      open = T_OTHER!("{");
      close = T_OTHER!("}");
      gullet::read_balanced(ExpansionLevel::Off,false,false)?.unwrap_or_default()
    } else {
      open = open.as_other();
      close = open;
      // Once we've determined the delimiter is non-brace (e.g. `|`),
      // demote `{` and `}` to OTHER so read_until_token doesn't engage
      // its balanced-read branch on `{`. Real users write
      // `\\path|{partial,| ... |partial}|` across multiple
      // \\urldef calls, where the `{` and `}` are LITERAL chars in
      // separate `|...|` paths, not balanced TeX groups. Driver:
      // 1906.08946 — without this demotion our read_until('|') on
      // line 25's `\\path|{...,|` reads PAST the closing `|`,
      // through line 26 and into line 27 looking for the matching
      // `}` — which it finds in `lncs}@springer.com|`, swallowing
      // the intervening `\\urldef{\\mailsb}` and `\\urldef{\\mailsc}`
      // declarations entirely.
      latexml_core::state::assign_catcode('{', Catcode::OTHER, Some(Scope::Local));
      latexml_core::state::assign_catcode('}', Catcode::OTHER, Some(Scope::Local));
      gullet::read_until_token(close)?
    };
    end_semiverbatim()?;
    let toks : Vec<Token> = url.unlist().into_iter().filter(|t| t.get_catcode() != Catcode::SPACE)
      .map(|t| t.as_other()).collect();

    let mut url_wrapped = vec![T_CS!("\\UrlFont"), T_CS!("\\UrlLeft")];
    url_wrapped.extend(toks.clone());
    url_wrapped.push(T_CS!("\\UrlRight"));
    let mut invocation_tokens = Invocation!(T_CS!("\\lx@url@url@nolink"),vec![
        Tokens!(cmd.as_other()),
        Tokens!(open),
        Tokens!(close),
        Tokens::new(toks),
        Tokens::new(url_wrapped)]).unlist();
    invocation_tokens.push(T_CS!("\\endgroup"));
    Ok(Tokens::new(invocation_tokens))
  });

  // Define \Url, in case its used; won't be represented as nicely
  DefMacro!("\\Url", {
    unread_one(T_OTHER!("\\Url"));
    Ok(Tokens!(T_CS!("\\lx@url@url")))
  });

  DefConstructor!("\\lx@url@url@nolink Undigested {}{} Semiverbatim {}",// Allow this to work in Math!
    "?#isMath(<ltx:XMWrap class='ltx_nolink #class' href='#href'>#5</ltx:XMWrap>)(<ltx:ref href='#href' class='ltx_nolink #class'>#5</ltx:ref>)",
    properties => sub[args] {
      unref!(args => cmd, _open, _close, url, _formattedurl);
      let ltx_cmd = s!("ltx_{}", LEADING_BACKSLASH_RE.replace(&cmd.to_string(),""));
      Ok(stored_map!(
        "href" => compose_url(&state::lookup_string("BASE_URL"), &url.to_string(), None),
        // TODO: why was class realized in Perl as "sub" closure here?
        "class"=> ltx_cmd
      ))
    },
    sizer     => "#5",
    enter_horizontal => true,
    reversion => "#1#2#4#3");

  // These are the expansions of \DeclareUrlCommand
  DefMacro!("\\path", r"\begingroup\urlstyle{tt}\lx@url@url\path");
  DefMacro!("\\url", r"\begingroup\lx@url@url\url", locked => true);

  // \urldef{newcmd}\cmd{arg}
  // Kinda tricky, since we need to get the expansion of \cmd as the value of \newcmd
  // Along with the annoying \endgroup that must balance the one always preceding \Url!
  DefRegister!("\\Urlmuskip", MuGlue::default());

  // Perl L91-99: \urldef{newcmd}{arg} — takes TWO brace args. The second
  // `{}` lets TeX parse a balanced brace group without expansion, then
  // unreads it back so digest_next_body can digest up through \endgroup.
  // Prior Rust had only one arg `{}` and no unread, so `\urldef\foo{...}`
  // digested whatever followed in the input stream instead of the braced arg.
  DefPrimitive!("\\urldef{}{}", sub[(cmd, start)] {
    let cmd = cmd.to_string();
    gullet::unread(start);
    let expansion : Vec<Digested> = stomach::digest_next_body(Some(T_CS!("\\endgroup")))?;
    DefPrimitive!(&cmd, { Ok(expansion.clone()) });
    Ok(vec![])
  });
});
