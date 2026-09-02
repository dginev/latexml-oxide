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
      let ftype_str = ftype.to_string();
      let ftype_name = ftype_str.trim();
      if ftype_name.is_empty() {
        // hvfloat.sty L630-636 `\hvFloat@ii`: an EMPTY float type forces
        // `nonFloat,onlyText` — no float, no counter, no "Figure:" label;
        // the object is typeset with the caption as plain text beside it
        // (L780-782 `\ifhv@nonFloat\ifhv@onlyText \hv@longCap`). The former
        // `\begin{}` emission gave `undefined:{}` and ran the class
        // `\caption` outside any float (hvfloat.tex L1475 `onlyText=true`
        // demo; under raw KOMA that ERROR-defined `\@captype` and every later
        // `\captionof` recursed).
        out.push(T_CS!("\\par"));
        out.extend(obj.unlist());
        out.push(T_CS!("\\par"));
        out.extend(cap.unlist());
        if i == 0 && !label.to_string().trim().is_empty() {
          out.push(T_CS!("\\label"));
          out.push(T_BEGIN!());
          out.extend(label.clone().unlist());
          out.push(T_END!());
        }
        out.push(T_CS!("\\par"));
        continue;
      }
      out.extend(Tokenize!(TeXString::assembled(s!(
        "\\begin{{{0}}}\\centering ", ftype_name
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
  // hvfloat.sty L309/L328: `\setDefaults` is the real name, `\hvFloatSetDefaults`
  // its `\let` alias; L19-22 `\fileversion`/`\hvFloatFileVersion`.
  def_macro_noop("\\setDefaults")?;
  DefMacro!("\\hvFloatFileVersion", "2.54");
  // hvfloat.sty L381-400: `\figcaption`/`\tabcaption`/`\tabcaptionbelow` set
  // `\@captype` in a group and run `\caption` — the `\captionof` shape
  // (caption.sty binding), which is what our `\caption` needs to find a type.
  DefMacro!("\\figcaption[]{}", r"\captionof{figure}[#1]{#2}");
  DefMacro!("\\tabcaption[]{}", r"\captionof{table}[#1]{#2}");
  DefMacro!("\\tabcaptionbelow[]{}", r"\captionof{table}[#1]{#2}");
  // Two-object variant \hvFloat with sub-floats is rare in the manuals;
  // the main form covers the corpus. Registers used by the demos
  // (hvfloat.sty L86-99 `\newlength`/`\newsavebox`):
  DefRegister!("\\hvObjectWidth" => Dimension!("0pt"));
  DefRegister!("\\hvCapWidth" => Dimension!("0pt"));
  DefRegister!("\\hvWideWidth" => Dimension!("0pt"));
  DefRegister!("\\hvMaxCapWidth" => Dimension!("0pt"));
  DefRegister!("\\hvFloatFullWidth" => Dimension!("0pt"));
  DefRegister!("\\hvMultiFloatSkip" => Dimension!("0pt"));
  DefRegister!("\\hvNonFloatTopSkip" => Dimension!("0pt"));
  RawTeX!(r"\newsavebox\hvObjectBox\newsavebox\hvCaptionBox\newsavebox\hvOBox");
  // hvfloat.sty L24-36: the package options are `\newif` switches
  // (`\hv@fboxfalse` is used by the manual's demos, L70-71 loads hyperref on
  // `[hyperref]`); L306-307 `\defhvstyle{name}{keys}` = `\@namedef{hv@name}`
  // with `\hvDefFloatStyle` as its alias.
  RawTeX!(r"\newif\ifhv@fbox\newif\ifhv@hyperref\newif\ifhv@nostfloats\newif\ifhv@tugboat\newif\ifhv@forceLeft
    \def\defhvstyle#1#2{\@namedef{hv@#1}{#2}}\let\hvDefFloatStyle\defhvstyle");
  DeclareOption!("fbox", "\\hv@fboxtrue");
  DeclareOption!("hyperref", "\\hv@hyperreftrue");
  DeclareOption!("nostfloats", "\\hv@nostfloatstrue");
  DeclareOption!("no-stfloats", "\\hv@nostfloatstrue");
  ProcessOptions!();
  RawTeX!(r"\ifhv@hyperref\RequirePackage{hyperref}\fi");
  // hvfloat.sty L55.
  RequirePackage!("ifoddpage");
  // hvfloat.sty L1264-1266: `\newenvironment{hvFloatEnv}[1][\textwidth]
  // {\minipage{#1}}{\endminipage}` — a plain minipage the manual wraps
  // `\captionof` demos in (hvfloat.tex L1531; was `undefined:{hvFloatEnv}`).
  DefMacro!("\\hvFloatEnv[]", r"\minipage{\ifx.#1.\textwidth\else#1\fi}");
  DefMacro!("\\endhvFloatEnv", r"\endminipage");
});
