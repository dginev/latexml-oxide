use crate::prelude::*;

LoadDefinitions!({
  // Perl: graphics.sty.ltxml — base graphics package
  // Package options: draft, final, hiderotate, hidescale, hiresbb
  // (most are no-ops for LaTeXML)

  // == Scaling boxes ==

  // \scalebox{xscale}[yscale]{content}
  // Perl: DefConstructor('\Gscale@box {Float} [Float] {}', ...)
  // For now, simplified: wrap content in inline-block with scale attributes
  DefConstructor!("\\scalebox{} []{}", "<ltx:inline-block xscale='#1' yscale='#yscale'>#3</ltx:inline-block>",
    mode => "restricted_horizontal", enter_horizontal => true,
    properties => sub[args] {
      let yscale = args[1].as_ref().map(|a| a.to_attribute())
        .unwrap_or_else(|| args[0].as_ref().map(|a| a.to_attribute()).unwrap_or_default());
      Ok(stored_map!("yscale" => yscale))
    });
  Let!("\\Gscale@box", "\\scalebox");

  // \resizebox{width}{height}{content}
  DefMacro!("\\resizebox", "\\leavevmode\\@ifstar{\\Gscale@@box\\totalheight}{\\Gscale@@box\\height}");

  // Simplified \Gscale@@box — just passes through content for now
  DefConstructor!("\\Gscale@@box{}{}{}{}", "#4",
    mode => "restricted_horizontal", enter_horizontal => true);

  // == Rotation ==

  // Rotation keyvals
  DefKeyVal!("Grot", "origin", "");
  DefKeyVal!("Grot", "x", "Dimension");
  DefKeyVal!("Grot", "y", "Dimension");
  DefKeyVal!("Grot", "units", "");

  DefConstructor!("\\rotatebox OptionalKeyVals:Grot {Float}{}", "<ltx:inline-block angle='#2'>#3</ltx:inline-block>",
    mode => "restricted_horizontal", enter_horizontal => true);

  DefMacro!("\\Grot@erotate", "\\rotatebox[]");

  DefConstructor!("\\reflectbox{}", "<ltx:inline-block xscale='-1'>#1</ltx:inline-block>",
    mode => "restricted_horizontal", enter_horizontal => true);

  // == Graphics path and inclusion ==

  DefConstructor!("\\graphicspath{}", "",
    after_digest => sub[_whatsit] {
      // TODO: push paths to GRAPHICSPATHS
    });

  // Perl: DefMacro('\includegraphics OptionalMatch:* [][] Semiverbatim',
  //   '\@includegraphics#1[#2][#3]{#4}');
  DefMacro!("\\includegraphics OptionalMatch:* [][] Semiverbatim",
    "\\@includegraphics#1[#2][#3]{#4}");

  DefConstructor!("\\@includegraphics OptionalMatch:* [][] Semiverbatim",
    "<ltx:graphics graphic='#graphic' candidates='#candidates' options='#options'/>",
    enter_horizontal => true,
    properties => sub[args] {
      let path = args[3].as_ref().map(|a| a.to_attribute()).unwrap_or_default();
      let path = path.trim().to_string();
      Ok(stored_map!("graphic" => path.clone(), "candidates" => path, "options" => ""))
    },
    alias => "\\includegraphics");

  DefConstructor!("\\DeclareGraphicsExtensions{}", "");
  DefConstructor!("\\DeclareGraphicsRule{}{}{} Undigested", "");

  // == Gin internal macros (Perl: RawTeX block, lines 311-324) ==

  Let!("\\Gin@decode", "\\@empty");
  DefMacro!("\\Gin@exclamation", "!");
  Let!("\\Gin@page", "\\@empty");
  DefMacro!("\\Gin@pagebox", "cropbox");
  DefConditional!("\\ifGin@interpolate");
  Let!("\\Gin@log", "\\wlog");
  Let!("\\Gin@req@sizes", "\\relax");
  DefMacro!("\\Gin@scalex", "1");
  Let!("\\Gin@scaley", "\\Gin@exclamation");
  // These reference macros that may not exist yet, so define them
  DefMacro!("\\Gin@nat@height", "");
  DefMacro!("\\Gin@nat@width", "");
  Let!("\\Gin@req@height", "\\Gin@nat@height");
  Let!("\\Gin@req@width", "\\Gin@nat@width");
  Let!("\\Gin@viewport@code", "\\relax");

  // Perl: DefConditional('\ifGin@clip');
  DefConditional!("\\ifGin@clip");
  // Perl: DefMacro('\Gin@i [][]{}', '');
  DefMacro!("\\Gin@i[][]{}", "");

  // Perl: DefPrimitive('\Gscale@div DefToken Dimension Dimension', sub { ... })
  // \Gscale@div{\cs}{\dima}{\dimb} : \cs = \dima / \dimb
  DefMacro!("\\Gscale@div{}{}{}", "");

  // Perl: \set@color defined elsewhere but referenced by graphics
  // Provide a no-op fallback if not already defined
  DefMacro!("\\set@color", "");
});
