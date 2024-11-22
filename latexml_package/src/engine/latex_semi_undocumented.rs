use crate::prelude::*;
//**********************************************************************
// Semi-Undocumented stuff
//**********************************************************************

LoadDefinitions!({
  DefMacro!("\\@ifnextchar DefToken {}{}", sub[(token, t_if, t_else)] {
    let next = gullet::read_non_space()?;
    // NOTE: Not actually substituting, but collapsing ## pairs!!!!
    // use \egroup for $next, if we've fallen off end?
    let next_test = match next {
      Some(ref n) => XEquals!(&token, n),
      None => XEquals!(&token, &*TOKEN_END)
    };
    let which = if next_test {
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
  DefMacro!("\\@ifnext@n {}{}{}", sub[(tokens,if_toks,else_toks)] {
    let mut toks = VecDeque::from(tokens.unlist());
    let mut read = Vec::new();

    while let Some(t) = gullet::read_token()? {
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

  DefMacro!("\\@ifstar {}{}", sub[(if_toks,else_toks)] {
  let next_opt = gullet::read_non_space()?;
  if next_opt == Some(T_OTHER!("*")) {
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

  DefMacro!("\\@testopt{}{}", sub[(cmd, option)] {
    if gullet::if_next(T_OTHER!("["))? {
      Ok(cmd)
    } else {
      Ok(Tokens!(cmd.unlist(), T_OTHER!("["), option.unlist(), T_OTHER!("]")))
    }
  });
  TeX!(
    r"
  \def\@protected@testopt#1{%%
    \ifx\protect\@typeset@protect
      \expandafter\@testopt
    \else
      \@x@protect#1%
    \fi}"
  );

  Let!("\\l@ngrel@x", "\\relax"); // Never actually used anywhere, but...
  DefMacro!(
    "\\@star@or@long{}",
    r"\@ifstar{\let\l@ngrel@x\relax#1}{\let\l@ngrel@x\long#1}"
  );

  // maybe this is easiest just to punt.
  TeX!(
    r"
  \def\in@#1#2{%
  \def\in@@##1#1##2##3\in@@{%
    \ifx\in@##2\in@false\else\in@true\fi}%
  \in@@#2#1\in@\in@@}
  \newif\ifin@"
  );

  DefMacro!("\\IfFileExists{}{}{}", sub[(file, if_tks, else_tks)] {
    let file_string = Expand!(file).to_string();
    if let Some(_) = find_file(&file_string, None) {
      let found_str = s!("\"{file_string}\" ");
      def_macro(T_CS!("\\@filef@und"), None, Some(found_str.into()), None)?;
      if_tks 
    } else {
      else_tks
    } 
  });

  DefMacro!("\\InputIfFileExists{}{}{}", sub[(file, if_tks, else_tks)] {
    let file_tks = Expand!(file);
    let file_string = file_tks.to_string();
    if let Some(_) = find_file(&file_string, None) {
      let found_str = s!("\"{file_string}\" ");
      def_macro(T_CS!("\\@filef@und"), None, Some(found_str.into()), None)?;
      Tokens!(if_tks, T_CS!("\\@addtofilelist"), T_BEGIN!(), file_tks.clone(), T_END!(),
        T_CS!("\\ltx@input"), T_BEGIN!(), file_tks, T_END!())
    } else { 
      else_tks
    } 
  });

  DefMacro!("\\@ifdefinable DefToken {}", sub[(token, iftoken)] {
    if is_definable(&token) {
      iftoken.unlist()
    } else {
      let token_str = token.to_string();
      let mut s = ExplodeText!(token_str);
      if token_str.starts_with('\\') { // drop leading slash
        s.remove(0);
      }
      DefMacro!(T_CS!("\\reserved@a"), None, Tokens::new(s));
      vec![T_CS!("\\@notdefinable")]
    }
  });

  Let!("\\@@ifdefinable", "\\@ifdefinable");

  DefMacro!("\\@rc@ifdefinable DefToken {}", sub[(_token, iftoken)] {
    Let!("\\@ifdefinable", "\\@@ifdefinable");
    iftoken.unlist()
  });

  DefMacro!(
    "\\@notdefinable",
    None,
    r###"\@latex@error{%
    Command \@backslashchar\reserved@a\space
    already defined.
    Or name \@backslashchar\@qend... illegal, see p.192 of the manual}
  "###
  );

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

  DefMacro!("\\appendixname", "Appendix");
  // These aren"t defined in LaTeX,
  // these definitions will give us more meaningful typerefnum"s
  DefMacro!(
    "\\sectiontyperefname",
    "\\lx@sectionsign\\lx@ignorehardspaces"
  );
  DefMacro!(
    "\\subsectiontyperefname",
    "\\lx@sectionsign\\lx@ignorehardspaces"
  );
  DefMacro!(
    "\\subsubsectiontyperefname",
    "\\lx@sectionsign\\lx@ignorehardspaces"
  );
  DefMacro!(
    "\\paragraphtyperefname",
    "\\lx@paragraphsign\\lx@ignorehardspaces"
  );
  DefMacro!(
    "\\subparagraphtyperefname",
    "\\lx@paragraphsign\\lx@ignorehardspaces"
  );

  DefPrimitive!("\\@@end", { gullet::flush() });

  // DG: TODO Maybe split these out?
  //%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
  // Expl3 "Experimental LaTeX 3" is no longer Experimental!
  // It is beginning to be built into latex.ltx
  // We WILL need a new strategy to keep up; probably based in some form
  // of pre-read/pre-processed latex.ltx !
  //
  // For now, a few macros required by other packages will be included:
  // DefMacro!(T_CS!("\\hook_gput_code:nnn"), '{}{}{}', '');
  DefMacro!("\\NewHook{}", None);
  DefMacro!("\\NewReversedHook{}", None);
  DefMacro!("\\NewMirroredHookPair{}{}", None);
  DefMacro!("\\ActivateGenericHook{}", None);
  DefMacro!("\\DisableGenericHook{}", None);
  DefMacro!("\\AddToHook{}[]{}", None);
  DefMacro!("\\AddToHookNext{}{}", None);
  DefMacro!("\\ClearHookNext{}", None);
  DefMacro!("\\RemoveFromHook{}[]", None);
  DefMacro!("\\SetDefaultHookLabel{}", None);
  DefMacro!("\\PushDefaultHookLabel{}", None);
  DefMacro!("\\PopDefaultHookLabel", None);
  DefMacro!("\\UseHook{}", None);
  DefMacro!("\\UseOneTimeHook{}", None);
  DefMacro!("\\ShowHook{}", None);
  DefMacro!("\\LogHook{}", None);
  DefMacro!("\\DebugHooksOn", None);
  DefMacro!("\\DebugHooksOff", None);
  DefMacro!("\\DeclareHookRule{}{}{}{}", None);
  DefMacro!("\\DeclareDefaultHookRule{}{}{}", None);
  DefMacro!("\\ClearHookRule{}{}{}", None);
  DefMacro!("\\IfHookEmptyTF{}{}{}", "#3");
  DefMacro!("\\IfHookExistsTF{}{}{}", "#3");
  DefMacro!("\\MakeTextLowercase", "\\lowercase");
  DefMacro!("\\MakeTextUppercase", "\\uppercase");

  DefConditional!("\\if@includeinrelease");
  Let!("\\@kernel@after@enddocument", "\\@empty");
  Let!("\\@kernel@after@enddocument@afterlastpage", "\\@empty");
  Let!("\\@kernel@before@begindocument", "\\@empty");
  Let!("\\@kernel@after@begindocument", "\\@empty");
  Let!("\\conditionally@traceon", "\\@empty");
  Let!("\\conditionally@traceoff", "\\@empty");
});
