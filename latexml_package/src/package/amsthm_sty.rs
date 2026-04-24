use crate::engine::latex_constructs::*;
use crate::prelude::*;
#[rustfmt::skip]
LoadDefinitions!({
  RequirePackage!("amsgen");

  Let!("\\nonslanted", "\\upshape");
  DefMacro!("\\nopunct", "");

  // Redefine from LaTeX; notes go in normal font, not headfont
  DefRegister!("\\thm@notefont" => Tokens!(
    T_CS!("\\fontseries"), T_CS!("\\mddefault"), T_CS!("\\upshape")
  ));

  // amsthm also saves headfont and headformatter!
  set_savable_theorem_parameters(vec![
    "\\thm@headfont", "\\thm@bodyfont", "\\thm@headpunct",
    "\\thm@styling", "\\thm@headstyling", "\\thm@headformatter", "thm@swap",
  ]);

  // extra stubs for internals that show up in arXiv
  DefRegister!("\\thm@preskip"  => Glue::new(0));
  DefRegister!("\\thm@postskip" => Glue::new(0));
  DefMacro!("\\thm@space@setup", "\\thm@preskip=\\topsep \\thm@postskip=\\thm@preskip");

  // activate a certain theorem style
  DefPrimitive!("\\theoremstyle{}", sub[(style_tok)] {
    let style = style_tok.to_string();
    let style_cs = T_CS!(s!("\\th@{style}"));
    if is_defined(&s!("\\th@{style}")) {
      state::assign_register("\\thm@style",
        RegisterValue::Tokens(mouth::tokenize(&style)),
        None, vec![])?;
      stomach::digest(Tokens::new(vec![style_cs]))?;
    } else {
      Warn!("undefined", "theoremstyle",
        s!("Unknown theorem style '{style}', reverting to 'plain'."));
      state::assign_register("\\thm@style",
        RegisterValue::Tokens(mouth::tokenize("plain")),
        None, vec![])?;
      stomach::digest(Tokens::new(vec![T_CS!("\\th@plain")]))?;
    }
  });

  DefMacro!("\\swapnumbers", sub[_args] {
    let cur = state::lookup_value("thm@swap")
      .map(|v| match v {
        Stored::Int(n) => n != 0,
        Stored::Bool(b) => b,
        _ => false,
      })
      .unwrap_or(false);
    AssignValue!("thm@swap" => !cur);
    Ok(Tokens!())
  });

  // \newtheoremstyle{name}{spaceabove}{spacebelow}{bodyfont}{indent}{headfont}{headpunct}{spaceafter}{headspec}
  DefPrimitive!("\\newtheoremstyle{}{}{}{}{}{}{}{}{}", sub[(
    name, _spaceabove, _spacebelow, bodyfont, _indent, headfont, headpunct, spaceafter, headspec
  )] {
    let name_str = name.to_string();

    let mut headformatter_stored = Stored::Tokens(Tokens!());
    if !headspec.is_empty() {
      // If spec given, create formatter macro
      let formatter_cs = T_CS!(s!("\\format@title@theoremstyle@{name_str}"));
      let preamble = mouth::tokenize_internal(
        "\\@ifempty{#1}{\\let\\thmname\\@gobble}{\\let\\thmname\\@iden}\
         \\@ifempty{#2}{\\let\\thmnumber\\@gobble}{\\let\\thmnumber\\@iden}\
         \\@ifempty{#3}{\\let\\thmnote\\@gobble}{\\let\\thmnote\\@iden}"
      );
      let mut full_body = preamble.unlist();
      full_body.extend(headspec.unlist());
      let params = parse_parameters("{}{}{}", &formatter_cs, true)?;
      DefMacro!(
        formatter_cs,
        params,
        Some(ExpansionBody::Tokens(Tokens::new(full_body)))
      );
      headformatter_stored = Stored::Tokens(Tokens::new(vec![formatter_cs]));
    }

    // Determine headstyling from spaceafter
    let spaceafter_str = spaceafter.to_string();
    let headstyling = if spaceafter_str.contains("\\newline") {
      Tokens!()
    } else {
      Tokens!(T_CS!("\\lx@makerunin"))
    };

    save_theorem_style(&name_str, vec![
      ("\\thm@bodyfont".into(), Stored::Tokens(bodyfont)),
      ("\\thm@headfont".into(), Stored::Tokens(headfont)),
      ("\\thm@headpunct".into(), Stored::Tokens(headpunct)),
      ("\\thm@headformatter".into(), headformatter_stored),
      ("\\thm@headstyling".into(), Stored::Tokens(headstyling)),
    ]);
    let name_for_closure = name_str.clone();
    DefMacro!(
      T_CS!(s!("\\th@{name_str}")),
      None,
      Some(ExpansionBody::Closure(Rc::new(move |_args| {
        use_theorem_style(&name_for_closure);
        Ok(Tokens!())
      })))
    );
  });

  // The default theorem styles
  RawTeX!(r"\newtheoremstyle{plain}{}{}{\itshape}{}{\bfseries}{.}{}{}");
  RawTeX!(r"\newtheoremstyle{definition}{}{}{\normalfont}{}{\bfseries}{.}{}{}");
  RawTeX!(r"\newtheoremstyle{remark}{}{}{\normalfont}{}{\itshape}{.}{}{}");
  RawTeX!(r"\theoremstyle{plain}");

  DefMacro!("\\thmname{}", "#1");
  DefMacro!("\\thmnumber{}", "#1");
  DefMacro!("\\thmnote{}", "#1");

  DefMacro!("\\thmhead{}{}{}", "");
  DefMacro!("\\swappedhead{}{}{}", "");
  DefMacro!("\\thmheadnl", "");

  //======================================================================
  // Proofs

  AssignValue!("QED@stack" => Stored::VecDequeStored(std::collections::VecDeque::new()));
  DefMacro!("\\pushQED{}", sub[(qed)] {
    let _ = push_value("QED@stack", Stored::Tokens(qed));
    Ok(Tokens!())
  });
  DefMacro!("\\popQED", sub[_args] {
    if let Ok(Some(Stored::Tokens(t))) = pop_value("QED@stack") {
      Ok(t)
    } else {
      Ok(Tokens!())
    }
  });

  // QED symbol — Perl amsthm.sty.ltxml has `enterHorizontal => 1`.
  // Without it, a bare \qed at end of proof in vertical mode emits the
  // U+220E text node outside any <ltx:p>, producing structurally
  // invalid bare text in a vertical-mode container.
  DefMacro!("\\qed", "\\ltx@qed");
  DefConstructor!("\\ltx@qed",
    "?#isMath(<ltx:XMTok role='PUNCT'>\u{220E}</ltx:XMTok>)(\u{220E})",
    enter_horizontal => true,
    reversion => "\\qed"
  );
  Let!("\\mathqed",    "\\qed");
  Let!("\\textsquare", "\\qed");
  Let!("\\qedsymbol",  "\\qed");
  Let!("\\openbox",    "\\qed");

  DefMacro!("\\qedhere", sub[_args] {
    let t = pop_value("QED@stack");
    let _ = push_value("QED@stack", Stored::Tokens(Tokens!()));
    if let Ok(Some(Stored::Tokens(tokens))) = t {
      Ok(tokens)
    } else {
      Ok(Tokens!())
    }
  });

  DefMacro!("\\proofname", "Proof");
  DefPrimitive!("\\th@proof", {
    state::assign_register("\\thm@headfont",
      RegisterValue::Tokens(Tokens!(T_CS!("\\itshape"))), None, vec![])?;
    state::assign_register("\\thm@bodyfont",
      RegisterValue::Tokens(Tokens!(T_CS!("\\normalfont"))), None, vec![])?;
  });

  // Proof environment
  DefConstructor!("\\@proof OptionalUndigested",
    "<ltx:proof class='#class'><ltx:title font='#titlefont' _force_font='true' class='#titleclass'>#title</ltx:title>#body",
    before_digest => {
      stomach::digest(Tokens::new(vec![T_CS!("\\th@proof")]))?;
    },
    after_digest => sub[whatsit] {
      let _ = push_value("QED@stack", Stored::Tokens(Tokens!(T_CS!("\\qed"))));
      stomach::digest(mouth::tokenize_internal("\\the\\thm@bodyfont"))?;
    },
    properties => sub[args] {
      let mut title_tokens = vec![T_BEGIN!(), T_CS!("\\the"), T_CS!("\\thm@headfont")];
      if let Some(Some(ref arg)) = args.first() {
        title_tokens.extend(arg.revert()?.unlist());
      } else {
        title_tokens.push(T_CS!("\\proofname"));
      }
      title_tokens.push(T_OTHER!("."));
      title_tokens.push(T_END!());
      let title = stomach::digest(Tokens::new(title_tokens))?;
      // Perl: [$title->unlist]->[1]->getFont — get font from first content box.
      // The template engine treats the `"font"` prop as the
      // element-font attribute (auto-binds to font= attr regardless of
      // placeholder name in the template).
      let titlefont = title.get_font().ok().flatten().map(|f| f.into_owned());
      let mut map = SymHashMap::default();
      map.insert("title", title.into());
      if let Some(f) = titlefont {
        map.insert("font", Stored::Font(Rc::new(f)));
      }
      map.insert("titleclass", "ltx_runin".into());
      Ok(map)
    }
  );

  DefConstructor!("\\end@proof", sub[document, _args] {
    document.maybe_close_element("ltx:proof")?;
  },
  before_digest => {
    if let Ok(Some(Stored::Tokens(qed))) = pop_value("QED@stack") {
      if !qed.is_empty() {
        return Ok(vec![stomach::digest(qed)?]);
      }
    }
    Ok(vec![])
  });

  Let!("\\proof",    "\\@proof");
  Let!("\\endproof", "\\end@proof");
  // Also need to handle \begin{proof} / \end{proof}
  Let!("\\begin{proof}", "\\begin{@proof}");
  Let!("\\end{proof}",   "\\end{@proof}");

  // from older versions of amsthm.sty
  DefPrimitive!("\\theorembodyfont{}", sub[(font)] {
    state::assign_register("\\thm@bodyfont",
      RegisterValue::Tokens(font), None, vec![])?;
  });
  DefPrimitive!("\\theoremheaderfont{}", sub[(font)] {
    state::assign_register("\\thm@headfont",
      RegisterValue::Tokens(font), None, vec![])?;
  });
  DefRegister!("\\theorempreskipamount"  => Glue::new(0));
  DefRegister!("\\theorempostskipamount" => Glue::new(0));
});
