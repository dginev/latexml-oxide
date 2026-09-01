use latexml_package::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // hvfloat.sty — captions beside/after/around float objects.
  //
  // Raw-impossibility justification (perfect-kernel README protocol): raw
  // hvfloat BUILDS its floats manually out of (mini)boxes with `\@captype`
  // set (hvfloat.sty L545+ `\do@hvFloat` box assembly), so `\caption` runs
  // with NO float element open — our (and Perl's) `^^<ltx:caption>` float-up
  // finds no legal ancestor and every caption errors
  // `malformed:ltx:(toc)caption isn't allowed in <ltx:block>` (43 docs, the
  // largest single malformed cluster of sweep 16; real LaTeX only requires
  // `\@captype`). The box choreography is pure page layout; the SEMANTIC
  // content is exactly a float + object + caption + label — map it to the
  // real environment.
  //
  // \hvFloat*?[keys]{figure|table}{object}[shortcap]{caption}{label}
  // (hvfloat.sty L535-550). Placement keys (capPos, rotation, fullpage…)
  // are presentational — dropped.
  // hvfloat.sty's real dependency chain (caption/graphicx/…).
  RequirePackage!("caption");
  RequirePackage!("graphicx");
  // \hvFloat*?[keys]{type}{object}[shortcap]{caption}{label}
  // (hvfloat.sty L535-550), plus the multiFloat form where each sub-float
  // arrives as `+{type}{object}[short]{caption}` and ONE trailing {label}
  // (multi-default2s1c.tex L26-33; the naive `{}` prototype grabbed the
  // bare `+` as the environment name → `undefined:{+}` ×15 docs).
  // Placement keys are presentational — dropped.
  DefMacro!("\\hvFloat OptionalMatch:* []", sub[(_star, _opts)] {
    let plus = Tokens!(T_OTHER!("+"));
    let mut out: Vec<Token> = Vec::new();
    let mut subfloats: Vec<(Tokens, Tokens, Option<Tokens>, Tokens)> = Vec::new();
    let multi = loop {
      if read_match(&[&plus])?.is_some() {
        let ftype = read_arg(ExpansionLevel::Off)?;
        let obj = read_arg(ExpansionLevel::Off)?;
        let short = read_optional(None)?;
        let cap = read_arg(ExpansionLevel::Off)?;
        subfloats.push((ftype, obj, short, cap));
      } else if subfloats.is_empty() {
        break false;
      } else {
        break true;
      }
    };
    if !multi {
      let ftype = read_arg(ExpansionLevel::Off)?;
      let obj = read_arg(ExpansionLevel::Off)?;
      let short = read_optional(None)?;
      let cap = read_arg(ExpansionLevel::Off)?;
      subfloats.push((ftype, obj, short, cap));
    }
    let label = read_arg(ExpansionLevel::Off)?;
    for (i, (ftype, obj, short, cap)) in subfloats.into_iter().enumerate() {
      out.extend(Tokenize!(TeXString::assembled(s!(
        "\\begin{{{0}}}\\centering ", ftype.to_string().trim()
      ))).unlist());
      out.extend(obj.unlist());
      out.extend(Tokenize!(TeXString::assembled("\\caption[".to_string())).unlist());
      if let Some(sc) = short {
        out.extend(sc.unlist());
      }
      out.push(T_OTHER!("]"));
      out.push(T_BEGIN!());
      out.extend(cap.unlist());
      out.push(T_END!());
      if i == 0 {
        out.extend(Tokenize!(TeXString::assembled("\\label".to_string())).unlist());
        out.push(T_BEGIN!());
        out.extend(label.clone().unlist());
        out.push(T_END!());
      }
      out.extend(Tokenize!(TeXString::assembled(s!(
        "\\end{{{0}}}", ftype.to_string().trim()
      ))).unlist());
    }
    Ok(Tokens::new(out))
  });
  // Companion setup macros (presentational).
  def_macro_noop("\\hvFloatSet{}")?;
  def_macro_noop("\\hvFloatSetDefaults")?;
  def_macro_noop("\\hvSet{}")?;
  // Two-object variant \hvFloat with sub-floats is rare in the manuals;
  // the main form covers the corpus. Registers used by the demos:
  DefRegister!("\\hvObjectWidth" => Dimension!("0pt"));
  DefRegister!("\\hvCapWidth" => Dimension!("0pt"));
});
