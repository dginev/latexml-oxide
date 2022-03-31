use crate::package::*;

lazy_static! {
  static ref LEADING_BACKSLASH_RE: Regex = Regex::new(r"^\\").unwrap();
}

LoadDefinitions!(state, {
  AssignValue!("BASE_URL", "");

  // Ignorable stuff, since we're not doing linebreaks.
  DefMacro!("\\UrlBreaks", "");
  DefMacro!("\\UrlBigBreaks", "");
  DefMacro!("\\UrlNoBreaks", "");
  DefMacro!("\\UrlOrds", "");
  DefMacro!("\\UrlSpecials", "");

  // Font style definitions.
  DefMacro!("\\urlstyle{}", "\\expandafter\\protect\\csname url@#1style\\endcsname");
  DefMacro!("\\url@ttstyle", "\\def\\UrlFont{\\ttfamily}");
  DefMacro!("\\url@rmstyle", "\\def\\UrlFont{\\rmfamily}");
  DefMacro!("\\url@sfstyle", "\\def\\UrlFont{\\sffamily}");
  DefMacro!("\\url@samestyle", "");
  DefMacro!("\\UrlFont", "\\ttfamily");

  // Bracketting.
  Let!("\\UrlLeft", "\\@empty");
  Let!("\\UrlRight", "\\@empty");

  // \DeclareUrlCommand\cmd{settings}
  // Have this expand into \@Url w/ the declared cmd as arg, so it gets reflected in XML.
  DefMacro!("\\DeclareUrlCommand{}{}", "\\def#1{\\begingroup #2\\@Url#1}");

  // This is an extended version of \Url that takes an extra token as 1st arg.
  // That token is the cs that invoked it, so that it can be reflected in the generated XML,
  // as well as used to generate the reversion.
  // In any case, we read the verbatim arg, and build a Whatsit for @@Url
  DefMacro!("\\@Url Token", sub[gullet, args, state] {
    unpack_to_token!(args => cmd);
    let perc = vec!['%'];
    state.begin_semiverbatim(Some(&perc));
    let mut open = gullet.read_token(state).unwrap();
    let mut close;
    let url = if open == T_BEGIN!() {
      open = T_OTHER!("{");
      close = T_OTHER!("}");
      gullet.read_balanced(false, state)?
    } else {
      open = T_OTHER!(open.get_string());
      close = open.clone();
      gullet.read_until_token(close.clone(), state)?
    };
    state.end_semiverbatim()?;

    let toks : Vec<Token> = url.unlist().into_iter().filter(|t| t.get_catcode() != Catcode::SPACE).map(|t| T_OTHER!(t.get_string())).collect();

    let mut url_wrapped = vec![T_CS!("\\UrlFont"), T_CS!("\\UrlLeft")];
    url_wrapped.extend(toks.clone());
    url_wrapped.push(T_CS!("\\UrlRight"));
    let mut invocation_tokens = Invocation!(T_CS!("\\@@Url"),vec![
        Tokens!(T_OTHER!(cmd.to_string())),
        Tokens!(open),
        Tokens!(close),
        Tokens::new(toks),
        Tokens::new(url_wrapped)], gullet)?.unlist();
    invocation_tokens.push(T_CS!("\\endgroup"));
    Ok(Tokens::new(invocation_tokens))
  });

  // Define \Url, in case its used; won't be represented as nicely
  DefMacro!("\\Url", sub[gullet, args, state] {
    gullet.unread_one(T_OTHER!("\\Url"));
    Ok(Tokens!(T_CS!("\\@Url")))
  });

  // \@@Url cmd {open}{close}{url}{formattedurl}
  DefConstructor!("\\@@Url Undigested {}{} Semiverbatim {}",// Allow this to work in Math!
    "?#isMath(<ltx:XMWrap href='#href'>#5</ltx:XMWrap>) (<ltx:ref href='#href' class='#class'>#5</ltx:ref>)",
    properties => sub[stomach, args, state] {
      unpack!(args => cmd, open, close, url, formattedurl);
      let ltx_cmd = s!("ltx_{}", LEADING_BACKSLASH_RE.replace(&cmd.to_string(),""));
      Ok(map!(
        "href" => Stored::String(compose_url(&state.lookup_string("BASE_URL"), &url.to_string(), None)),
        // TODO: why was class a sub {}??
        "class"=> Stored::String(ltx_cmd)
      ))
    }
        // sizer     => "#5",
        // reversion => "#1#2#4#3");
  );

  // These are the expansions of \DeclareUrlCommand
  DefMacro!("\\path", "\\begingroup\\urlstyle{tt}\\@Url\\path");
  DefMacro!("\\url", "\\begingroup\\@Url\\url", locked => true);

  // \urldef{newcmd}\cmd{arg}
  // Kinda tricky, since we need to get the expansion of \cmd as the value of \newcmd
  // Along with the annoying \endgroup that must balance the one always preceding \Url!
  DefPrimitive!("\\urldef{}", sub[stomach, args, url_state] {
    unpack_to_string!(args => cmd);
    let expansion : Vec<Digested> = stomach.digest_next_body(Some(T_CS!("\\endgroup")), url_state)?;
    let gullet = stomach.get_gullet_mut();
    DefPrimitive!(&cmd, { Ok(expansion.clone()) });
    Ok(vec![])
  });
});
