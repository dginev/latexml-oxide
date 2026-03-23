use crate::prelude::*;
LoadDefinitions!({
  // Not sure that ltx:p is the best to use here, but ... (see also \vbox, \vtop)
  // This should be fairly compact vertically.
  DefConstructor!("\\@shortstack@cr",
    "</ltx:p><ltx:p>",
    properties   => { stored_map!("isBreak" => true) },
    reversion    => Tokens!(T_CS!("\\\\"), T_CR!()),
    before_digest => { egroup()?; },
    after_digest  => { bgroup(); });

  DefConstructor!("\\shortstack[]{}  OptionalMatch:* [Dimension]",
  "<ltx:inline-block align='#align'><ltx:p>#2</ltx:p></ltx:inline-block>",
  bounded      => true,
  sizer        => "#2",
  before_digest => { reenter_text_mode(false);
    // Rebind \\ and its aliases to shortstack line break
    Let!("\\\\", "\\@shortstack@cr");
    Let!("\\lx@hidden@cr", "\\@shortstack@cr");
    Let!("\\lx@newline", "\\@shortstack@cr");
    AssignRegister!("\\baselineskip" , Glue::new_spec("-1pt", None, None, None, None).into());
    AssignRegister!("\\lineskip"     , Glue::new_spec("3pt", None, None, None, None).into());
    bgroup(); },
  after_digest => sub[_whatsit] {
    egroup()?; },
  // Note: does not get layout=vertical, since linebreaks are explicit
  properties => sub[args] {
    let align = args[0].as_ref().map(|a| {
      match a.to_string().as_str() {
        "l" => "left", "r" => "right", _ => ""
      }
    }).unwrap_or("");
    Ok(stored_map!("align" => align, "vattach" => "bottom"))
  },
  mode => "text");

  //======================================================================
  // C.14.1 The picture Environment
  // Perl: latex_constructs.pool.ltxml lines 4927-5185
  //======================================================================

  // Registers
  DefRegister!("\\unitlength" => Dimension!("1pt"));
  DefRegister!("\\@wholewidth" => Dimension!("0.4pt"));
  DefRegister!("\\@halfwidth" => Dimension!("0.2pt"));

  // \thinlines / \thicklines — set \@wholewidth register
  DefMacro!("\\thinlines", "\\@wholewidth 0.4pt\\relax");
  DefMacro!("\\thicklines", "\\@wholewidth 0.8pt\\relax");
  DefMacro!("\\linethickness{}", "\\@wholewidth #1\\relax");
  DefMacro!("\\arrowlength{}", None);
  DefMacro!("\\qbeziermax", "500");
  DefMacro!("\\@killglue", "\\unskip\\@whiledim \\lastskip >\\z@\\do{\\unskip}");

  // {picture} environment: (width,height) with optional (origin-x,origin-y)
  DefEnvironment!("{picture} Pair OptionalPair",
    "<ltx:picture fill='none' stroke='none'>\
      #body\
    </ltx:picture>",
    mode => "text",
    before_digest => {
      // Perl: before_picture — Let \raisebox to \pic@raisebox
      Let!("\\raisebox", "\\pic@raisebox");
    }
  );

  // \put(x,y){content}
  DefMacro!("\\put Pair {}", "\\lx@pic@put(#1){#2\\relax}");
  DefConstructor!("\\lx@pic@put Pair {}",
    "<ltx:g>#2</ltx:g>",
    alias => "\\put",
    mode  => "text"
  );

  // \line(slope){length}
  DefConstructor!("\\line Pair {Float}",
    "<ltx:line points='0,0 0,0' stroke='black'/>",
    alias => "\\line"
  );

  // \vector(slope){length} — like \line but with arrow terminator
  DefConstructor!("\\vector Pair {Float}",
    "<ltx:line points='0,0 0,0' stroke='black' terminators='->'/>",
    alias => "\\vector"
  );

  // \circle*{diameter} — filled or unfilled circle
  DefConstructor!("\\circle OptionalMatch:* {Float}",
    "<ltx:circle x='0' y='0' r='0' fill='none' stroke='black'/>",
    alias => "\\circle"
  );

  // \oval[radius](width,height)[part]
  DefConstructor!("\\oval [Float] Pair []",
    "<ltx:rect x='0' y='0' width='0' height='0' rx='0' stroke='black' fill='none'/>",
    alias => "\\oval"
  );

  // \qbezier[N](p1)(p2)(p3)
  DefConstructor!("\\qbezier [Number] Pair Pair Pair",
    "<ltx:bezier points='0,0 0,0 0,0' stroke='black'/>",
    alias => "\\qbezier"
  );

  // \multiput(pos)(delta){n}{body} — Perl expands to n \put commands via macro.
  // Simplified: just place the body at the initial position (full loop requires runtime).
  // TODO: full multiput loop expansion
  DefMacro!("\\multiput Pair Pair {}{}", "\\put(#1){#4}");

  // Box commands for picture mode (simplified)
  DefMacro!("\\pic@makebox Pair [] {}", "#3");
  DefMacro!("\\pic@framebox Pair [] {}", "#3");
  DefMacro!("\\frame{}", "#1");
  DefMacro!("\\dashbox{Float} Pair [] {}", "#4");

  // \pic@raisebox — simplified raisebox for picture mode
  DefConstructor!("\\pic@raisebox{Dimension}[Dimension][Dimension]{}",
    "<ltx:g y='#1'>#4</ltx:g>",
    alias => "\\raisebox"
  );

  // Perl: latex_constructs.pool.ltxml line 4862
  // Stubs for color/xcolor packages (overridden when color.sty is loaded)
  Let!("\\set@color", "\\relax");
  Let!("\\color@begingroup", "\\relax");
  Let!("\\color@endgroup", "\\relax");
  Let!("\\color@setgroup", "\\relax");
  Let!("\\color@hbox", "\\relax");
  Let!("\\color@vbox", "\\relax");
  Let!("\\color@endbox", "\\relax");

  // Perl: latex_constructs.pool.ltxml line 5802
  DefMacro!("\\ignorespacesafterend", None);

  // Perl: latex_constructs.pool.ltxml line 5027
  // Pre-define \Gin@driver so graphics.sty doesn't error when loaded from disk
  DefMacro!("\\Gin@driver", "");
});
