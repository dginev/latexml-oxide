use crate::package::*;
//**********************************************************************
// Semi-Undocumented stuff
//**********************************************************************

LoadDefinitions!(outer_stomach, state, {
  DefMacro!("\\@ifnextchar DefToken {}{}", sub[gullet, (token, t_if, t_else), state] {
    let next = gullet.read_non_space(state);
    // NOTE: Not actually substituting, but collapsing ## pairs!!!!
    // use \egroup for $next, if we've fallen off end?
    let next_test = next.as_ref().unwrap_or(&T_END!());
    let which = if XEquals!(&token, next_test) {
      t_if
    } else {
      t_else
    };
    let mut result = which.substitute_parameters(&[]).unlist();
    if let Some(t_next) = next {
      result.push(t_next);
    }
    result
  });
  Let!("\\kernel@ifnextchar", "\\@ifnextchar");
  Let!("\\@ifnext", "\\@ifnextchar"); // ????

  // Hacky version matches multiple chars! but does NOT expand
  DefMacro!(r"\@ifnext@n {}{}{}", sub[gullet,(tokens,if_toks,else_toks),state] {
    let mut toks = VecDeque::from(tokens.unlist());
    let mut read = Vec::new();
    while let Some(t) = gullet.read_token(state) {
      if t == toks[0] {
        toks.pop_front();
        read.push(t);
      } else {
        read.push(t);
        break;
      }
    }
    let mut result = if toks.is_empty() {
      if_toks.unlist()
    } else {
      else_toks.unlist()
    };
    result.extend(read);
    Ok(Tokens::new(result))
  });

  DefMacro!("\\@ifstar {}{}", sub[gullet,(if_toks,else_toks),state] {
  let next_opt = gullet.read_non_space(state);
  if Some(T_OTHER!("*")) == next_opt {
    Ok(if_toks)
  } else {
    let mut result = else_toks.unlist();
    if let Some(next) = next_opt {
      result.push(next);
    }
    Ok(Tokens::new(result))
  }});

  DefMacro!("\\@dblarg {}", r"\kernel@ifnextchar[{#1}{\@xdblarg{#1}}");
  DefMacro!("\\@xdblarg {}{}", r"#1[{#2}]{#2}");

  DefMacro!("\\@testopt{}{}", sub[gullet,(cmd, option),state] {
    if gullet.if_next(T_OTHER!("["), state)? {
      Ok(cmd)
    } else {
      Ok(Tokens!(cmd.unlist(), T_OTHER!("["), option.unlist(), T_OTHER!("]")))
    }
  });
  RawTeX!(
    r###"
  \def\@protected@testopt#1{%%
    \ifx\protect\@typeset@protect
      \expandafter\@testopt
    \else
      \@x@protect#1%
    \fi}
  "###
  );

  Let!("\\l@ngrel@x", "\\relax"); // Never actually used anywhere, but...
  DefMacro!("\\@star@or@long{}", r"\@ifstar{\let\l@ngrel@x\relax#1}{\let\l@ngrel@x\long#1}");

  // maybe this is easiest just to punt.
  RawTeX!(
    r###"
  \def\in@#1#2{%
  \def\in@@##1#1##2##3\in@@{%
    \ifx\in@##2\in@false\else\in@true\fi}%
  \in@@#2#1\in@\in@@}
  \newif\ifin@
  "###
  );

  DefMacro!("\\@ifdefinable DefToken {}", sub[gullet, (token, iftoken), state] {
    if is_definable(&token, state) {
      iftoken.unlist()
    } else {
      let mut s = ExplodeText!(token.to_string());
      let slash = s.remove(0);
      DefMacro!(T_CS!("\\reserved@a"), None, Tokens::new(s), state);
      vec![T_CS!("\\@notdefinable")]
    }
  });

  Let!("\\@@ifdefinable", "\\@ifdefinable");

  DefMacro!("\\@rc@ifdefinable DefToken {}", sub[gullet, (token, iftoken), state] {
    state.let_i(&T_CS!("\\@ifdefinable"), T_CS!("\\@@ifdefinable"), None, gullet);
    iftoken.unlist()
  });

  DefMacro!("\\@notdefinable", None, r###"\@latex@error{%
    Command \@backslashchar\reserved@a\space
    already defined.
    Or name \@backslashchar\@qend... illegal,
    see p.192 of the manual}
  "###);

  //======================================================================
  // Hair
  DefPrimitive!("\\makeatletter", {
    AssignCatcode!('@', Catcode::LETTER, Some(Scope::Local));
  });
  DefPrimitive!("\\makeatother", {
    AssignCatcode!('@', Catcode::OTHER, Some(Scope::Local));
  });

  //**********************************************************************
  // Sundry (is this ams ?)
  DefMacro!("\\textprime", "\u{00B4}"); // ACUTE ACCENT

  Let!("\\endgraf", "\\par");
  Let!("\\endline", "\\cr");
  //**********************************************************************
  // Should be defined in each (or many) package, but it"s not going to
  // get set correctly or maintained, so...
  DefMacro!("\\fileversion", "");
  DefMacro!("\\filedate", "");

  // Ultimately these may be overridden by babel, or otherwise,
  // various of these are defined in various places by different classes.
  DefMacro!("\\chaptername", "Chapter");
  DefMacro!("\\partname", "Part");
  // The rest of these are defined in some classes, but not most.
  //DefMacroI("\sectionname",       undef, "Section");
  //DefMacroI("\subsectionname",    undef, "Subsection");
  //DefMacroI("\subsubsectionname", undef, "Subsubsection");
  //DefMacroI("\paragraphname",     undef, "Paragraph");
  //DefMacroI("\subparagraphname",  undef, "Subparagraph");

  DefMacro!("\\appendixname", "Appendix");
  // These aren"t defined in LaTeX,
  // these definitions will give us more meaningful typerefnum"s
  DefMacro!("\\sectiontyperefname", "\\lx@sectionsign\\lx@ignorehardspaces");
  DefMacro!("\\subsectiontyperefname", "\\lx@sectionsign\\lx@ignorehardspaces");
  DefMacro!("\\subsubsectiontyperefname", "\\lx@sectionsign\\lx@ignorehardspaces");
  DefMacro!("\\paragraphtyperefname", "\\lx@paragraphsign\\lx@ignorehardspaces");
  DefMacro!("\\subparagraphtyperefname", "\\lx@paragraphsign\\lx@ignorehardspaces");
});
