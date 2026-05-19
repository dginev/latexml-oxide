use crate::engine::latex_constructs::*;
use crate::prelude::*;
#[rustfmt::skip]
LoadDefinitions!({
  // Basically, this package is similar to amsthm.sty (or theorem.sty)
  // with two main enhancements: endmarks, list of theorems
  RequirePackage!("amsthm");
  RequirePackage!("ifthen");

  // Re-redefine from amsthm from LaTeX; use same note font as head font
  DefRegister!("\\thm@notefont" => Tokens!(T_CS!("\\the"), T_CS!("\\thm@headfont")));

  // options
  DefConditional!("\\if@thref");
  DefConditional!("\\ifthm@inframe");
  DefConditional!("\\ifthm@tempif");

  // Declare options
  DeclareOption!("leqno", None);
  DeclareOption!("fleqn", None);
  DeclareOption!("amsmath", None);
  DeclareOption!("amsthm", None);
  DeclareOption!("hyperref", None);
  DeclareOption!("standard", sub {
    AssignValue!("thm@usestd" => true);
  });
  DeclareOption!("noconfig", None);
  DeclareOption!("framed", None);
  DeclareOption!("thmmarks", None);

  DeclareOption!("thref", sub {
    Let!("\\orig@label", "\\label");
    DefMacro!("\\label Semiverbatim []", "\\orig@label{#1}");
    // Perl ntheorem.sty.ltxml L60-63: `enterHorizontal => 1` — \thref
    // emits an inline <ltx:ref>, so like \ref it must enter horizontal
    // mode before absorbing into the surrounding paragraph.
    DefConstructor!("\\thref OptionalMatch:* Semiverbatim",
      "<ltx:ref labelref='#label' show='typerefnum' _force_font='true'/>",
      enter_horizontal => true,
      properties => sub[args] {
        let label = args[1].as_ref().map(|a| clean_label(&a.to_string(), None).into_owned()).unwrap_or_default();
        Ok(stored_map!("label" => label))
      }
    );
  });

  ProcessOptions!();

  // Registers
  DefRegister!("\\theoremindent"                => Dimension::new(0));
  DefRegister!("\\theoremrightindent"           => Dimension::new(0));
  DefRegister!("\\theorempreskipamount"         => Dimension::new(0));
  DefRegister!("\\theorempostskipamount"        => Dimension::new(0));
  DefRegister!("\\theoremframepreskipamount"    => Dimension::new(0));
  DefRegister!("\\theoremframepostskipamount"   => Dimension::new(0));
  DefRegister!("\\theoreminframepreskipamount"  => Dimension::new(0));
  DefRegister!("\\theoreminframepostskipamount" => Dimension::new(0));
  def_macro_noop("\\theorempreskip{}")?;
  def_macro_noop("\\theorempostskip{}")?;
  def_macro_noop("\\theoremframepreskip{}")?;
  def_macro_noop("\\theoremframepostskip{}")?;
  def_macro_noop("\\theoreminframepreskip{}")?;
  def_macro_noop("\\theoreminframepostskip{}")?;
  DefMacro!("\\None",                     "None");
  DefMacro!("\\NoneSymbol",               "None");
  DefMacro!("\\NoneKeyword",              "None");
  DefRegister!("\\shadecolor" => Tokens!());

  // Include a few other theorem parameters in definitions
  set_savable_theorem_parameters(vec![
    "\\thm@headfont", "\\thm@bodyfont", "\\thm@headpunct",
    "\\thm@styling", "\\thm@headstyling", "thm@swap",
    "\\thm@numbering", "\\thm@prework", "\\thm@postwork", "\\thm@symbol",
  ]);

  DefMacro!("\\theoremheaderfont{}", sub[(font)] {
    state::assign_register("\\thm@headfont",
      RegisterValue::Tokens(font), None, vec![])?;
    Ok(Tokens!())
  });

  DefMacro!("\\theoremseparator{}", sub[(punct)] {
    state::assign_register("\\thm@headpunct",
      RegisterValue::Tokens(punct), None, vec![])?;
    Ok(Tokens!())
  });
  DefMacro!("\\theoremsymbol{}", sub[(sym)] {
    state::assign_register("\\thm@symbol",
      RegisterValue::Tokens(sym), None, vec![])?;
    Ok(Tokens!())
  });

  DefMacro!("\\theoremprework{}", sub[(work)] {
    state::assign_register("\\thm@prework",
      RegisterValue::Tokens(work), None, vec![])?;
    Ok(Tokens!())
  });
  DefMacro!("\\theorempostwork{}", sub[(work)] {
    state::assign_register("\\thm@postwork",
      RegisterValue::Tokens(work), None, vec![])?;
    Ok(Tokens!())
  });
  DefMacro!("\\theoremnumbering{}", sub[(numbering)] {
    let numbering_str = numbering.to_string();
    let cs = T_CS!(s!("\\{numbering_str}"));
    state::assign_register("\\thm@numbering",
      RegisterValue::Tokens(Tokens::new(vec![cs])), None, vec![])?;
    Ok(Tokens!())
  });

  // theoremclass does nothing significant in our implementation
  DefPrimitive!("\\theoremclass{}", sub[(_class)] { });

  //======================================================================
  // End marks
  DefMacro!("\\TheoremSymbol", "\\@qedbox{\\the\\thm@symbol}");
  DefConstructor!("\\@qedbox{}", "<ltx:text class='ltx_align_floatright'>#1</ltx:text>",
    enter_horizontal => true);

  RawTeX!("\\newif\\ifsetendmark\\setendmarktrue");
  DefMacro!("\\NoEndMark", "\\global\\setendmarkfalse");
  DefMacro!("\\thm@doendmark", "\\ifsetendmark\\TheoremSymbol\\fi");
  DefRegister!("\\qedsymbol" => Tokens!());
  DefMacro!("\\qed", "\\@qedbox{\\the\\qedsymbol}");

  //======================================================================
  // Redefine \newtheorem to also create * variants
  Let!("\\orig@newtheorem", "\\newtheorem");
  DefMacro!("\\newtheorem OptionalMatch:* {}[]{}[]",
    "\\orig@newtheorem#1{#2}[#3]{#4}[#5]\
     \\ifx.#3.\\orig@newtheorem#1{#2*}[#2]{#4}[#5]\
     \\else\\orig@newtheorem#1{#2*}[#3]{#4}[#5]\\fi"
  );

  DefMacro!("\\Theoremname", "\\lx@thistheorem");
  Let!("\\renewtheorem", "\\newtheorem");

  //======================================================================
  // Style commands - newtheoremstyle and renewtheoremstyle
  DefPrimitive!("\\newtheoremstyle{}{}{}", sub[(name, _noopt, _opt)] {
    let style = name.to_string();
    let savable_keys = get_savable_keys();
    let mut saved: Vec<(String, Stored)> = Vec::new();
    for key in &savable_keys {
      if key.starts_with('\\') {
        let reg = LookupRegisterOrDefault!(key);
        let tokens = match reg {
          RegisterValue::Tokens(t) => t,
          _ => Tokens!(),
        };
        saved.push((key.clone(), Stored::Tokens(tokens)));
      } else {
        let val = state::lookup_value(key).unwrap_or(Stored::None);
        saved.push((key.clone(), val));
      }
    }
    save_theorem_style(&style, saved);
    let style_for_closure = style.clone();
    DefMacro!(
      T_CS!(s!("\\th@{style}")),
      None,
      Some(ExpansionBody::Closure(Rc::new(move |_args| {
        use_theorem_style(&style_for_closure);
        Ok(Tokens!())
      })))
    );
  });

  DefPrimitive!("\\renewtheoremstyle{}{}{}", sub[(name, _noopt, _opt)] {
    let style = name.to_string();
    let savable_keys = get_savable_keys();
    let mut saved: Vec<(String, Stored)> = Vec::new();
    for key in &savable_keys {
      if key.starts_with('\\') {
        let reg = LookupRegisterOrDefault!(key);
        let tokens = match reg {
          RegisterValue::Tokens(t) => t,
          _ => Tokens!(),
        };
        saved.push((key.clone(), Stored::Tokens(tokens)));
      } else {
        let val = state::lookup_value(key).unwrap_or(Stored::None);
        saved.push((key.clone(), val));
      }
    }
    save_theorem_style(&style, saved);
    let style_for_closure = style.clone();
    DefMacro!(
      T_CS!(s!("\\th@{style}")),
      None,
      Some(ExpansionBody::Closure(Rc::new(move |_args| {
        use_theorem_style(&style_for_closure);
        Ok(Tokens!())
      })))
    );
  });

  //======================================================================
  // Framing support
  // Perl: ntheorem.sty.ltxml lines 213-220
  DefConstructor!("\\lx@addframing", sub[document, _args, props] {
    let mut node = document.get_element().unwrap();
    document.set_attribute(&mut node, "framed", "rectangle")?;
    // Add padding from \FrameSep register
    if let Some(Stored::String(margin)) = props.get("margin") {
      let margin_str = arena::with(*margin, |s| s.to_string());
      let pad = s!("padding:{}pt;", margin_str);
      let existing = node.get_attribute("cssstyle").unwrap_or_default();
      let css = if existing.is_empty() { pad } else { s!("{};{}", existing, pad) };
      document.set_attribute(&mut node, "cssstyle", &css)?;
    }
  },
  properties => sub[_args] {
    let margin = LookupRegisterOrDefault!("\\FrameSep");
    let pt_val = match margin {
      RegisterValue::Dimension(d) => d.pt_value(None),
      _ => 9.0, // default 3*\fboxsep = 9pt
    };
    Ok(stored_map!("margin" => s!("{}", pt_val)))
  });

  // Perl: ntheorem.sty.ltxml lines 228-248
  // Executes \theoremframecommand on dummy text, captures result,
  // and copies background/frame attributes to the theorem.
  DefMacro!("\\lx@snapshot@framing", "\\lx@@snapshot@framing{\\theoremframecommand{foo}}");
  DefConstructor!("\\lx@@snapshot@framing{}", sub[document, args] {
    let mut theorem = document.get_element().unwrap();
    // Absorb the frame command result into a temporary capture element
    let capture = document.open_element("ltx:_Capture_", None, None)?;
    if let Some(frame_content) = args[0].as_ref() {
      document.absorb(frame_content, None)?;
    }
    document.close_element("ltx:_Capture_")?;
    // Extract attributes from the frame result
    if let Some(frame) = capture.get_first_child() {
      if let Some(bg) = frame.get_attribute("backgroundcolor") {
        document.set_attribute(&mut theorem, "backgroundcolor", &bg)?;
      }
      if let Some(css) = frame.get_attribute("cssstyle") {
        let existing = theorem.get_attribute("cssstyle").unwrap_or_default();
        let combined = if existing.is_empty() { css.clone() } else { s!("{};{}", existing, css) };
        document.set_attribute(&mut theorem, "cssstyle", &combined)?;
      }
      if let Some(framed) = frame.get_attribute("framed") {
        document.set_attribute(&mut theorem, "framed", &framed)?;
      }
      if let Some(fc) = frame.get_attribute("framecolor") {
        document.set_attribute(&mut theorem, "framecolor", &fc)?;
      }
    }
    document.remove_node(capture);
  },
  reversion => "");

  DefMacro!("\\newframedtheorem{}[]{}[]",
    "\\begingroup\\thm@styling{\\lx@addframing}\\newtheorem{#1}[#2]{#3}[#4]\\endgroup"
  );
  DefMacro!("\\newshadedtheorem{}[]{}[]",
    "\\begingroup\
     \\ifx\\theoremframecommand\\relax\\def\\theoremframecommand{\\colorbox{shadecolor}}\\fi\
     \\thm@styling{\\lx@snapshot@framing}\
     \\newtheorem{#1}[#2]{#3}[#4]\\endgroup"
  );

  //======================================================================
  // ntheorem builtin styles
  DefPrimitive!("\\lx@ntheorem@newtheoremstyle{}{}{}{}{}{}", sub[(
    name, headfont, bodyfont, headstyle, swap, numbering
  )] {
    let name_str = name.to_string();
    let swap_val = swap.eq_text("S");
    let symbol = LookupRegisterOrDefault!("\\thm@symbol");
    let symbol_tokens = match symbol {
      RegisterValue::Tokens(t) => t,
      _ => Tokens!(),
    };
    save_theorem_style(&name_str, vec![
      ("\\thm@bodyfont".into(), Stored::Tokens(bodyfont)),
      ("\\thm@headfont".into(), Stored::Tokens(headfont)),
      ("\\thm@headstyling".into(), Stored::Tokens(headstyle)),
      ("thm@swap".into(), Stored::Bool(swap_val)),
      ("\\thm@numbering".into(), Stored::Tokens(numbering)),
      ("\\thm@symbol".into(), Stored::Tokens(symbol_tokens)),
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

  RawTeX!(r"\lx@ntheorem@newtheoremstyle{plain}{\bfseries}{\itshape}{\lx@makerunin}{N}{\arabic}");
  RawTeX!(r"\lx@ntheorem@newtheoremstyle{break}{\bfseries}{\slshape}{}{N}{\arabic}");
  RawTeX!(r"\lx@ntheorem@newtheoremstyle{change}{\bfseries}{\slshape}{\lx@makerunin}{S}{\arabic}");
  RawTeX!(r"\lx@ntheorem@newtheoremstyle{margin}{\bfseries}{\slshape}{\lx@makerunin\lx@makeoutdent}{S}{\arabic}");
  RawTeX!(r"\lx@ntheorem@newtheoremstyle{marginbreak}{\bfseries}{\slshape}{\lx@makeoutdent}{S}{\arabic}");
  RawTeX!(r"\lx@ntheorem@newtheoremstyle{changebreak}{\bfseries}{\slshape}{}{S}{\arabic}");
  RawTeX!(r"\lx@ntheorem@newtheoremstyle{nonumberplain}{\bfseries}{\itshape}{\lx@makerunin}{N}{}");
  RawTeX!(r"\lx@ntheorem@newtheoremstyle{nonumberbreak}{\bfseries}{\slshape}{}{N}{}");
  RawTeX!(r"\lx@ntheorem@newtheoremstyle{empty}{}{}{\lx@makerunin}{N}{}");
  RawTeX!(r"\lx@ntheorem@newtheoremstyle{emptybreak}{}{}{}{N}{}");

  // Start off as plain style.
  use_theorem_style("plain");

  //======================================================================
  // Lists of Theorems
  def_macro_noop("\\addtheoremline OptionalMatch:* {}{}")?;
  def_macro_noop("\\addtotheoremfile[]{}")?;

  def_macro_noop("\\theoremlisttype{}")?;
  def_macro_noop("\\newtheoremlisttype{}{}{}{}")?;
  def_macro_noop("\\renewtheoremlisttype{}{}{}{}")?;

  DefConstructor!("\\listtheorems{}",
    "<ltx:TOC lists='#lists'/>",
    properties => sub[args] {
      let types_str = args[0].as_ref().map(|a| a.to_string()).unwrap_or_default();
      let lists: Vec<String> = types_str.split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| {
          if s == "all" { "thm".to_string() }
          else { s!("theorem:{s}") }
        })
        .collect();
      Ok(stored_map!("lists" => lists.join(" ")))
    }
  );

  def_macro_noop("\\theoremlistall")?;
  def_macro_noop("\\theoremlistallname")?;
  def_macro_noop("\\theoremlistalloptional")?;
  def_macro_noop("\\theoremlistalloptname")?;

  //======================================================================
  // Greek numbering
  DefMacro!("\\greek{}", sub[(ctr)] {
    let ctr_str = Expand!(ctr).to_string();
    let val = CounterValue!(&ctr_str).value_of();
    Ok(Tokens::new(ExplodeText!(&radix::radix_greek(val as i64))))
  });
  DefMacro!("\\Greek{}", sub[(ctr)] {
    let ctr_str = Expand!(ctr).to_string();
    let val = CounterValue!(&ctr_str).value_of();
    Ok(Tokens::new(ExplodeText!(&radix::radix_up_greek(val as i64))))
  });

  // Perl ntheorem.sty.ltxml L293: when option `standard` was given (sets
  // `thm@usestd`), input the actual TL `ntheorem.std` config file. That
  // file calls \newtheorem{theorem}{Theorem}, \newtheorem{lemma}{Lemma},
  // \newtheorem{definition}{Definition}, etc. — without it, papers using
  // [standard]{ntheorem} fail with `\begin{lemma}` undefined. Witness:
  // 0810.4249 (R=1→0).
  if state::lookup_bool("thm@usestd") {
    InputDefinitions!("ntheorem", noltxml => true, extension => Some(Cow::Borrowed("std")));
  }
});
