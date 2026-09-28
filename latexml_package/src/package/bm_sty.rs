use crate::prelude::*;

/// bm.sty's `\bm@define` (:307-316) `\xdef`s `\<name>` from its body, so a body that names
/// `\<name>` itself (`\DeclareBoldMathCommand{\nabla}{\nabla}`) makes it bold over its meaning
/// at that moment. Our body stays unexpanded, so that meaning is saved globally under a name of
/// its own, `\lx@bm@saved@<n>@<name>` (a new `<n>` per definition, as each `\xdef` sees the
/// meaning just before it: a second self-referential definition must not save the first's
/// wrapper under the same name), and the body refers to that; a plain `\gdef` would recurse.
fn bm_define(cs: Tokens, body: Tokens) -> Result<Tokens> {
  let Some(target) = cs
    .unlist()
    .into_iter()
    .find(|t| t.get_catcode() != Catcode::SPACE)
  else {
    Error!("expected", "\\bmdefine", "Missing the command to define");
    return Ok(Tokens::new(Vec::new()));
  };
  let mut body = body.unlist();
  let mut out = Vec::new();
  if body.contains(&target) {
    let n = lookup_int("lx@bm@saved@count") + 1;
    assign_value("lx@bm@saved@count", n, Some(Scope::Global));
    let saved = T_CS!(
      &target.with_cs_name(|name| { s!("\\lx@bm@saved@{n}@{}", name.trim_start_matches('\\')) })
    );
    out.extend([T_CS!("\\global"), T_CS!("\\let"), saved, target]);
    for token in body.iter_mut() {
      if *token == target {
        *token = saved;
      }
    }
  }
  out.extend([
    T_CS!("\\gdef"),
    target,
    T_BEGIN!(),
    T_CS!("\\bm"),
    T_BEGIN!(),
  ]);
  out.extend(body);
  out.extend([T_END!(), T_END!()]);
  Ok(Tokens::new(out))
}

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: bm.sty.ltxml
  // Since we're really punting the whole question of what fonts have
  // bold variants of which characters, this should be enough:
  DefConstructor!("\\bm{}", "#1", bounded => true, require_math => true, font => { forcebold => true });
  // bm.sty:229 `\def\bmdefine{\DeclareBoldMathCommand[bold]}`, whose `\bm@define` (:307-316)
  // `\xdef`s the command: global, and overwriting one already defined. Perl's `\newcommand`
  // (bm.sty.ltxml:22) kept a `\bmdefine` inside a group local and refused an existing name.
  DefMacro!("\\bmdefine{}{}", sub[(cs, body)] { bm_define(cs, body) });
  Let!("\\boldsymbol", "\\bm");

  // Should we make a distinction between bold & heavy?
  Let!("\\hm",          "\\bm");
  Let!("\\heavysymbol", "\\boldsymbol");
  Let!("\\hmdefine",    "\\bmdefine");
  Let!("\\heavymath",   "\\boldmath");

  // bm.sty L222-227: `\DeclareBoldMathCommand[<bold|heavy>]\<name>{<body>}`
  // declares `\name` as a bold-math wrapper around `body`. We collapse
  // both bold and heavy variants into our single `\bm` wrapper (we don't
  // distinguish bold vs heavy in the XML output anyway, mirroring the
  // \heavysymbol → \boldsymbol Let above). Surpass-Perl: Perl bm.sty.ltxml
  // doesn't carry `\DeclareBoldMathCommand` either, so this is a Rust-only
  // addition. Witness: arxiv-examples/1205.4484 (macros.tex defines
  // \boldlangle / \boldrangle / \boldlvert / \boldrvert via
  // \DeclareBoldMathCommand).
  DefMacro!("\\DeclareBoldMathCommand[]{}{}", sub[(_weight, cs, body)] { bm_define(cs, body) });
});
