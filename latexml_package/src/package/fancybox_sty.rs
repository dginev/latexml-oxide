use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // These could be made to depend on \fboxsep, \fboxrule, \cornersize
  DefMacro!("\\cornersize OptionalMatch:* {}", None);

  // Perl fancybox.sty.ltxml L25-36: DefConstructor(... mode => 'internal_vertical').
  // The mode declaration was dropped in the Rust stub — add it back so the
  // constructors pair like the Perl originals when they appear in
  // paragraph-mode contexts.
  DefConstructor!("\\shadowbox MoveableBox",
    "<ltx:text cssstyle='border:1px solid black; box-shadow: 5px 5px 10px black;'>#1</ltx:text>",
    mode => "internal_vertical");
  DefConstructor!("\\doublebox MoveableBox",
    "<ltx:text cssstyle='border:3px double black;'>#1</ltx:text>",
    mode => "internal_vertical");
  DefConstructor!("\\ovalbox MoveableBox",
    "<ltx:text cssstyle='border:1px solid black;border-radius:5px;'>#1</ltx:text>",
    mode => "internal_vertical");
  DefConstructor!("\\Ovalbox MoveableBox",
    "<ltx:text cssstyle='border:2px solid black;border-radius:5px;'>#1</ltx:text>",
    mode => "internal_vertical");

  // Perl fancybox.sty.ltxml L38-46: {Sbox} stashes its digested body
  // globally under the `Sbox` state value; \TheSbox pops it and
  // replays the stored content in place. Prior Rust stub was an empty
  // env + no-op macro, so `\sbox{…}{foo}\TheSbox` lost `foo`.
  DefEnvironment!("{Sbox}", "",
    after_digest_body => sub[whatsit] {
      if let Ok(Some(body)) = whatsit.get_body() {
        assign_value("Sbox", Stored::Digested(body), Some(Scope::Global));
      }
    });
  DefPrimitive!("\\TheSbox", {
    let stashed = lookup_value("Sbox");
    assign_value("Sbox", Stored::None, Some(Scope::Global));
    if let Some(Stored::Digested(body)) = stashed {
      return Ok(vec![body]);
    }
  });

  // fancybox ships its own verbatim layer (fancybox.sty "Verbatim" section);
  // {VerbatimOut}{file} captures its body verbatim to a file that demos
  // re-\input (fancybox-doc writes \jobname.ex1…). Store into the in-memory
  // filecontents cache instead of a real \write stream.
  DefPrimitive!("\\lx@fancybox@VerbatimOut", {
    skip_spaces()?;
    let filename_toks = read_arg(ExpansionLevel::Full)?;
    let filename = filename_toks.to_string();
    // `\VerbatimEnvironment` (below) redirects the end-scan to the WRAPPING
    // environment's `\end{...}` — the fancyvrb idiom for user-defined
    // verbatim-writing environments (fancybox-doc's {example} wraps
    // {VerbatimOut}{\jobname.tmp} and the author writes \end{example}).
    let end_env = match lookup_value("lx@verbatimout@envname") {
      Some(Stored::String(sym)) => with(sym, |v| v.to_string()),
      _ => "VerbatimOut".to_string(),
    };
    assign_value("lx@verbatimout@envname", Stored::None, Some(Scope::Global));
    let end_marker = s!("\\end{{{end_env}}}");
    read_raw_line(); // discard remainder of the \begin line
    let mut lines: Vec<String> = Vec::new();
    loop {
      // fancyvrb's end check matches a line that IS `\end{<name>}` (leading
      // whitespace allowed, trailing content defeats it — fancybox-doc
      // deliberately writes `\end{VerbatimOut}%.` INSIDE a captured body).
      match read_raw_line() {
        Some(line) if line.trim() != end_marker.as_str() => lines.push(line),
        _ => break,
      }
    }
    let n = lines.len();
    Info!("note", "filecontents", s!("Cached VerbatimOut for {filename} ({n} lines)"));
    assign_value(&s!("{filename}_contents"), Stored::from(lines.join("\n")), Some(Scope::Global));
    endgroup()?;
  });
  assign_meaning(
    &T_CS!("\\VerbatimOut"),
    lookup_meaning(&T_CS!("\\lx@fancybox@VerbatimOut")).unwrap_or(Stored::None),
    Some(Scope::Global),
  );
  def_macro_noop("\\endVerbatimOut")?;
  // Verbatim-in-footnotes toggle — presentational, no-op.
  def_macro_noop("\\VerbatimFootnotes")?;
  // `\VerbatimEnvironment`: record the CURRENT environment name so the
  // VerbatimOut scanner above stops at ITS \end (fancyvrb semantics).
  DefPrimitive!("\\VerbatimEnvironment", {
    let env = do_expand(Tokens!(T_CS!("\\@currenvir")))?.to_string();
    if !env.is_empty() {
      assign_value("lx@verbatimout@envname", Stored::String(pin(&env)), Some(Scope::Global));
    }
  });
  // \LVerbatimInput{file}: fancybox's LR-mode verbatim file input — route
  // through verbatim.sty's reader (the demos input what {VerbatimOut} wrote).
  RequirePackage!("verbatim");
  DefMacro!("\\LVerbatimInput{}", "\\verbatiminput{#1}");
  DefMacro!("\\BVerbatimInput{}", "\\verbatiminput{#1}");
  DefMacro!("\\VerbatimInput{}", "\\verbatiminput{#1}");
});
