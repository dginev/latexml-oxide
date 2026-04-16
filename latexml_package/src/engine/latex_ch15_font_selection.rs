use crate::prelude::*;

LoadDefinitions!({
  //**********************************************************************
  // C.15 Font Selection
  //**********************************************************************
  //======================================================================
  // C.15.1 Changing the Type Style
  //======================================================================
  // Text styles.

  DefMacro!("\\rmdefault", "cmr");
  DefMacro!("\\sfdefault", "cmss");
  DefMacro!("\\ttdefault", "cmtt");
  DefMacro!("\\bfdefault", "bx");
  DefMacro!("\\mddefault", "m");
  DefMacro!("\\itdefault", "it");
  DefMacro!("\\sldefault", "sl");
  DefMacro!("\\scdefault", "sc");
  DefMacro!("\\updefault", "n");
  DefMacro!("\\encodingdefault", "OT1");
  DefMacro!("\\familydefault", "\\rmdefault");
  DefMacro!("\\seriesdefault", "\\mddefault");
  DefMacro!("\\shapedefault", "\\updefault");

  Let!("\\mediumseries", "\\mdseries");
  Let!("\\normalshape", "\\upshape");

  // ? DefMacro("\\f@encoding','cm');
  DefMacro!("\\f@family", "cmr");
  DefMacro!("\\f@series", "m");
  DefMacro!("\\f@shape", "n");
  DefMacro!("\\f@size", "10");

  // These do NOT immediately effect the font!
  DefMacro!("\\fontfamily{}", "\\edef\\f@family{#1}");
  DefMacro!("\\fontseries{}", "\\edef\\f@series{#1}");
  DefMacro!("\\fontshape{}", "\\edef\\f@shape{#1}");

  // For fonts not allowed in math!!!
  // Perl L5226: \not@math@alphabet@@ checks if we're in math mode
  // LaTeX kernel also defines \not@math@alphabet (2 args) — stub both
  DefPrimitive!("\\not@math@alphabet{}{}", "");
  DefPrimitive!("\\not@math@alphabet@@ {}", sub[(c)] {
    if lookup_bool("IN_MATH") {
      let c = c.to_string();
      let message = s!("Command {:?} invalid in math mode", c);
      Warn!("unexpected", c, message);
    }
    Ok(vec![])
  });

  // These DO immediately effect the font!
  DefMacro!(
    "\\mdseries",
    "\\not@math@alphabet@@{\\mddefault}\\fontseries{\\mddefault}\\selectfont"
  );
  DefMacro!(
    "\\bfseries",
    "\\not@math@alphabet@@{\\bfdefault}\\fontseries{\\bfdefault}\\selectfont"
  );

  DefMacro!(
    "\\rmfamily",
    "\\not@math@alphabet@@{\\rmdefault}\\fontfamily{\\rmdefault}\\selectfont"
  );
  DefMacro!(
    "\\sffamily",
    "\\not@math@alphabet@@{\\sfdefault}\\fontfamily{\\sfdefault}\\selectfont"
  );
  DefMacro!(
    "\\ttfamily",
    "\\not@math@alphabet@@{\\ttdefault}\\fontfamily{\\ttdefault}\\selectfont"
  );

  DefMacro!(
    "\\upshape",
    "\\not@math@alphabet@@{\\updefault}\\fontshape{\\updefault}\\selectfont"
  );
  DefMacro!(
    "\\itshape",
    "\\not@math@alphabet@@{\\itdefault}\\fontshape{\\itdefault}\\selectfont"
  );
  DefMacro!(
    "\\slshape",
    "\\not@math@alphabet@@{\\sldefault}\\fontshape{\\sldefault}\\selectfont"
  );
  DefMacro!(
    "\\scshape",
    "\\not@math@alphabet@@{\\scdefault}\\fontshape{\\scdefault}\\selectfont"
  );

  DefMacro!(
    "\\normalfont",
    "\\fontfamily{\\rmdefault}\\fontseries{\\mddefault}\\fontshape{\\updefault}\\selectfont"
  );
  DefMacro!(
    "\\verbatim@font",
    "\\fontfamily{\\ttdefault}\\fontseries{\\mddefault}\\fontshape{\\updefault}\\selectfont"
  );

  Let!("\\reset@font", "\\normalfont");
  // Perl latex_constructs.pool.ltxml L5251
  DefMacro!("\\@fontswitch{}{}", "\\ifmmode #2\\relax\\else #1 \\fi");

  // Perl: latex_constructs.pool.ltxml L5759-5764 — picture font stubs
  DefPrimitive!("\\OMX", None, font => { family => "cmex10" });
  DefPrimitive!("\\tenln", None, font => { family => "line10" });
  DefPrimitive!("\\tenlnw", None, font => { family => "linew10" });
  DefPrimitive!("\\tencirc", None, font => { family => "lcircle10" });
  DefPrimitive!("\\tencircw", None, font => { family => "lcirclew10" });

  // Perl: latex_constructs.pool.ltxml L5777-5779
  Let!("\\nocorr", "\\relax");
  Let!("\\check@icl", "\\@empty");
  Let!("\\check@icr", "\\@empty");
  Let!("\\curr@math@size", "\\@empty");

  DefPrimitive!("\\selectfont", {
    let family = Expand!(T_CS!("\\f@family")).to_string();
    let series = Expand!(T_CS!("\\f@series")).to_string();
    let shape = Expand!(T_CS!("\\f@shape")).to_string();
    if let Some(sh) = font::lookup_font_family(&family) {
      MergeFont!(sh.clone());
    } else {
      let message = s!("Unrecognized font family {:?}.", family);
      Info!("unexpected", family, message);
    }
    if let Some(sh) = font::lookup_font_series(&series) {
      MergeFont!(sh.clone());
    } else {
      let message = s!("Unrecognized font series {:?}.", series);
      Info!("unexpected", series, message);
    }
    if let Some(sh) = font::lookup_font_shape(&shape) {
      MergeFont!(sh.clone());
    } else {
      let message = s!("Unrecognized font shape {:?}.", shape);
      Info!("unexpected", shape, message);
    }
    Ok(Vec::new())
  });

  DefMacro!(
    "\\usefont{}{}{}{}",
    "\\fontencoding{#1}\\fontfamily{#2}\\fontseries{#3}\\fontshape{#4}\\selectfont"
  );

  // If these series or shapes appear in math, they revert it to roman, medium, upright (?)
  DefConstructor!("\\textmd@math{}", "<ltx:text _noautoclose='1'>#1</ltx:text>", mode => "text",
    bounded      => true, font => { series => "medium" }, alias => "\\textmd",
    before_digest => { DefMacro!("\\f@series", "m"); });
  DefConstructor!("\\textbf@math{}", "<ltx:text _noautoclose='1'>#1</ltx:text>", mode => "text",
    bounded      => true, font => { series => "bold" }, alias => "\\textbf",
    before_digest => { DefMacro!("\\f@series", "b"); });
  DefConstructor!("\\textrm@math{}", "<ltx:text _noautoclose='1'>#1</ltx:text>",
    mode => "text", bounded => true, font => { family => "serif" }, alias => "\\textrm",
    before_digest => { DefMacro!("\\f@family", "cm"); });
  DefConstructor!("\\textsf@math{}", "<ltx:text _noautoclose='1'>#1</ltx:text>", mode => "text",
    bounded      => true, font => { family => "sansserif" }, alias => "\\textsf",
    before_digest => { DefMacro!("\\f@family", "cmss"); });
  DefConstructor!("\\texttt@math{}", "<ltx:text _noautoclose='1'>#1</ltx:text>", mode => "text",
    bounded      => true, font => { family => "typewriter" }, alias => "\\texttt",
    before_digest => { DefMacro!("\\f@family", "cmtt"); });
  DefConstructor!("\\textup@math{}", "<ltx:text _noautoclose='1'>#1</ltx:text>", mode => "text",
    bounded      => true, font => { shape => "upright" }, alias => "\\textup",
    before_digest => { DefMacro!("\\f@shape", ""); });
  DefConstructor!("\\textit@math{}", "<ltx:text _noautoclose='1'>#1</ltx:text>", mode => "text",
    bounded      => true, font => { shape => "italic" }, alias => "\\textit",
    before_digest => { DefMacro!("\\f@shape", "i"); });
  DefConstructor!("\\textsl@math{}", "<ltx:text _noautoclose='1'>#1</ltx:text>", mode => "text",
    bounded      => true, font => { shape => "slanted" }, alias => "\\textsl",
    before_digest => { DefMacro!("\\f@shape", "sl"); });
  DefConstructor!("\\textsc@math{}", "<ltx:text _noautoclose='1'>#1</ltx:text>", mode => "text",
    bounded      => true, font => { shape => "smallcaps" }, alias => "\\textsc",
    before_digest => { DefMacro!("\\f@shape", "sc"); });
  DefConstructor!("\\textnormal@math{}", "<ltx:text _noautoclose='1'>#1</ltx:text>", mode =>
  "text",   bounded => true, font => { family => "serif", series => "medium", shape => "upright"
  }, alias => "\\textnormal",   before_digest => {
    DefMacro!("\\f@family", "cmtt");
    DefMacro!("\\f@series", "m");
    DefMacro!("\\f@shape",  "n"); });

  // These really should be robust! which is a source of expand timing issues!
  DefMacro!("\\textmd{}",     "\\ifmmode\\textmd@math{#1}\\else{\\mdseries #1}\\fi",       protected => true);
  DefMacro!("\\textbf{}",     "\\ifmmode\\textbf@math{#1}\\else{\\bfseries #1}\\fi",       protected => true);
  DefMacro!("\\textrm{}",     "\\ifmmode\\textrm@math{#1}\\else{\\rmfamily #1}\\fi",       protected => true);
  DefMacro!("\\textsf{}",     "\\ifmmode\\textsf@math{#1}\\else{\\sffamily #1}\\fi",       protected => true);
  DefMacro!("\\texttt{}",     "\\ifmmode\\texttt@math{#1}\\else{\\ttfamily #1}\\fi",       protected => true);
  DefMacro!("\\textup{}",     "\\ifmmode\\textup@math{#1}\\else{\\upshape #1}\\fi",        protected => true);
  DefMacro!("\\textit{}",     "\\ifmmode\\textit@math{#1}\\else{\\itshape #1}\\fi",        protected => true);
  DefMacro!("\\textsl{}",     "\\ifmmode\\textsl@math{#1}\\else{\\slshape #1}\\fi",        protected => true);
  DefMacro!("\\textsc{}",     "\\ifmmode\\textsc@math{#1}\\else{\\scshape #1}\\fi",        protected => true);
  DefMacro!("\\textnormal{}", "\\ifmmode\\textnormal@math{#1}\\else{\\normalfont #1}\\fi", protected => true);

  // Perl: latex_constructs.pool.ltxml line 5365
  // \DeclareOldFontCommand{\cmd}{text-font-switch}{math-font-cmd}
  // Defines \cmd to use text-font-switch in text mode, math-font-cmd in math mode.
  DefPrimitive!("\\DeclareOldFontCommand{}{}{}", sub[(cmd, font, mathcmd)] {
    // cmd contains a CS token like \bf; get the first token
    let cmd_cs = *cmd.unlist_ref().first()
      .ok_or("DeclareOldFontCommand: expected a CS token")?;
    let font_toks = font.clone();
    let math_toks = mathcmd.clone();
    DefMacro!(cmd_cs, None, ExpansionBody::Closure(Rc::new(move |_args| {
      if lookup_bool("IN_MATH") {
        Ok(math_toks.clone())
      } else {
        Ok(font_toks.clone())
      }
    })));
    Ok(Vec::new())
  });

  // Perl L5333-5339: \DeclareTextFontCommand — creates a text font command.
  // Simplified: \cmd{} → {\font #1} (group with font change).
  DefPrimitive!("\\DeclareTextFontCommand DefToken {}", sub[(cmd, font)] {
    let cs = cmd;
    let font_rev: Tokens = font;
    // Build expansion: {<font> #1}
    let mut expansion = vec![T_BEGIN!()];
    expansion.extend(font_rev.unlist());
    expansion.push(T_PARAM!());
    expansion.push(T_OTHER!("1"));
    expansion.push(T_END!());
    let params = parse_parameters("{}", &cs, false)?;
    def_macro(cs, params,
      Some(ExpansionBody::Tokens(Tokens::new(expansion))), None)?;
  });

  // Perl L5373: \newfont{cmd}{fontname} — legacy LaTeX font command
  DefMacro!("\\newfont{}{}", "\\font#1=#2\\relax");
  // Perl L5375: \normalcolor — default no-op (overridden by color.sty)
  Let!("\\normalcolor", "\\relax");

  // Perl L5364: \math@version default
  DefMacro!("\\math@version", "normal");

  // Perl L5341-5348: \mathversion — switches between bold/normal math fonts
  DefPrimitive!("\\mathversion{}", sub[(version)] {
    let v = version.to_string();
    match v.trim() {
      "bold" => { MergeFont!(forcebold => true); },
      "normal" => { MergeFont!(forcebold => false); },
      _ => {},
    }
  });
});
