use latexml_package::prelude::*;

LoadDefinitions!({
  // mdframed.sty:382 reads its options with `\ProcessKeyvalOptions`, which marks them processed: `\@curroptions` stays as it was.
  key_options_processed()?;
  Warn!(
    "missing_file",
    "mdframed.sty",
    "mdframed.sty is only minimally stubbed and will not be interpreted raw."
  );
  // mdframed.sty:154-169 `\define@key{mdf}{framemethod}`: the method, lowercased, picks the package to load.
  DefMacro!("\\lx@mdf@framemethod{}", sub[(method)] {
    let given = method.to_string();
    let (name, package) = match given.trim().to_lowercase().as_str() {
      "default" | "tex" | "latex" | "none" | "0" => (Some("default"), None),
      "pgf" | "tikz" | "1" => (Some("tikz"), Some("tikz".to_string())),
      "pstricks" | "ps" | "2" | "postscript" => (Some("pstricks"), Some("pst-node".to_string())),
      _ => (None, Some(given.trim().to_string())),
    };
    let mut code = String::new();
    if let Some(name) = name {
      code.push_str(&format!("\\def\\mdf@@framemethod{{{name}}}"));
    }
    // In the body (`\newmdenv[framemethod=…]`, `\mdfsetup` after `\begin{document}`) the load is mdframed's
    // `\RequirePackage`, `\@onlypreamble`: pdflatex reports "Can be used only in preamble" and goes on with the
    // packages it has, and so does the binding, with an Info (the frame is ours whatever the method, so nothing the
    // author wrote is lost).
    if let Some(package) = package {
      if lookup_bool_sym(pin!("inPreamble")) {
        code.push_str(&format!("\\mdf@LoadFile@IfExist{{{package}}}"));
      } else {
        Info!(
          "unexpected",
          "framemethod",
          s!("mdframed's framemethod={given} cannot load {package} after the preamble (LaTeX: Can be used only in preamble); the frame is drawn either way")
        );
      }
    }
    Ok(TokenizeInternal!(TeXString::assembled(code)))
  });
  // mdframed.sty:72-77 loads kvoptions, xparse, etoolbox, zref-abspage, needspace and color, and so does the
  // binding: a document or class that uses `\needspace`, zref or `\ifpdf` (iftex, under zref-base's pdftexcmds)
  // without loading them itself relies on these.
  RequirePackage!("kvoptions");
  RequirePackage!("xparse");
  RequirePackage!("etoolbox");
  // zref-abspage has no binding: it is read raw, so only where raw loading is on (as `--preload=ar5iv.sty` has it);
  // otherwise its load would only add a missing-file warning.
  if lookup_bool("INCLUDE_STYLES") {
    RequirePackage!("zref-abspage");
  }
  RequirePackage!("needspace");
  RequirePackage!("color");
  // mdframed.sty:150-190 the frame method: `framemethod=tikz` (or `pgf`, `1`, the void options `tikz`/`TikZ`) loads
  // tikz, `framemethod=pstricks` (`ps`, `postscript`, `2`, the void options `ps`/`pstricks`/`PSTricks`) loads
  // pst-node, any other value but the default ones loads the package of that name if it exists
  // (`\mdf@LoadFile@IfExist`); the `xcolor` key loads xcolor (mdframed.sty:369-380, whose option list is never
  // passed on). Documents and classes draw with the tikz the option loaded (witness 2101.02817:
  // `[framemethod=TikZ]{mdframed}`, then `{tikzpicture}`). The frame drawing itself stays ours, so the other
  // keys (mdframed.sty defines them in family `mdf`) are accepted and dropped, here and in `\mdfsetup`.
  RawTeX!(
    r"\def\mdf@LoadFile@IfExist#1{\IfFileExists{#1.sty}{\RequirePackage{#1}}{\PackageWarning{mdframed}{The file #1
 does not exist\MessageBreak but needed by mdframed}}}
\SetupKeyvalOptions{family=mdf,prefix=mdf@}
\def\mdf@@framemethod{default}
\define@key{mdf}{framemethod}[default]{\lx@mdf@framemethod{#1}}
\DeclareVoidOption{tikz}{\setkeys{mdf}{framemethod=tikz}}
\DeclareVoidOption{TikZ}{\setkeys{mdf}{framemethod=tikz}}
\DeclareVoidOption{ps}{\setkeys{mdf}{framemethod=pstricks}}
\DeclareVoidOption{pstricks}{\setkeys{mdf}{framemethod=pstricks}}
\DeclareVoidOption{PSTricks}{\setkeys{mdf}{framemethod=pstricks}}
\define@key{mdf}{xcolor}[]{\RequirePackage{xcolor}}
\kv@set@family@handler{mdf}{}
\DeclareDefaultOption{}
\ProcessKeyvalOptions*\relax"
  );
  // mdframed.sty:591 `\newmdtheoremenv[mdframed-opts]{env}[numbered like]
  // {caption}[within]` = `\newtheorem` inside a frame (presentational);
  // the no-op it replaced left every such theorem environment undefined
  // (beautynote: theorem, lemma, definition, proposition, problem).
  // Guard: `perfect_kernel_batch56::mdframed_theorem_environments_are_theorems`.
  DefMacro!("\\newmdtheoremenv [] {} [] {} []", sub[(_opts, env, like, caption, within)] {
    let mut toks = vec![T_CS!("\\newtheorem"), T_BEGIN!()];
    toks.extend(env.unlist());
    toks.push(T_END!());
    if let Some(like) = like {
      toks.push(T_OTHER!("["));
      toks.extend(like.unlist());
      toks.push(T_OTHER!("]"));
    }
    toks.push(T_BEGIN!());
    toks.extend(caption.unlist());
    toks.push(T_END!());
    if let Some(within) = within {
      toks.push(T_OTHER!("["));
      toks.extend(within.unlist());
      toks.push(T_OTHER!("]"));
    }
    Ok(Tokens::new(toks))
  });
  // mdframed.sty:616-660 `\mdtheorem[opts]{env}[like]{caption}[within]` defines
  // the numbered `env` and the unnumbered `env*` (rbt-mathnotes.sty:766-795 builds
  // its thm/lem/cor/prop/defn with it; class census 2026-09-24).
  DefMacro!("\\mdtheorem [] {} [] {} []", sub[(opts, env, like, caption, within)] {
    let mut toks =
      Invocation!(T_CS!("\\newmdtheoremenv"), vec![opts, Some(env.clone()), like, Some(caption.clone()), within])
        .unlist();
    toks.extend([T_CS!("\\newtheorem"), T_OTHER!("*"), T_BEGIN!()]);
    toks.extend(env.unlist());
    toks.extend([T_OTHER!("*"), T_END!(), T_BEGIN!()]);
    toks.extend(caption.unlist());
    toks.push(T_END!());
    Ok(Tokens::new(toks))
  });
  // `\newmdenv[opts]{name}` defines a new environment `name` that wraps
  // `mdframed` (mdframed.sty L578-585:
  //   \newenvironment{#2}{\mdfsetup{#1}\begin{mdframed}}{\end{mdframed}}).
  // `\mdfsetup` only reads the keys, so the body reduces to a mdframed wrapper.
  // Surpass-Perl: ar5iv-bindings/mdframed.sty.ltxml L22 also no-ops this,
  // leaving the user's custom env undefined (Perl then errors with
  // `{name} is not defined`). Faithfully porting the real definer makes
  // the custom env work. Witness arXiv:2002.06879
  // (`\newmdenv[...]{mdfigure}` then `\begin{mdfigure}`).
  DefMacro!(
    "\\newmdenv[]{}",
    "\\newenvironment{#2}{\\mdfsetup{#1}\\begin{mdframed}}{\\end{mdframed}}"
  );
  DefMacro!(
    "\\renewmdenv[]{}",
    "\\renewenvironment{#2}{\\mdfsetup{#1}\\begin{mdframed}}{\\end{mdframed}}"
  );
  def_macro_noop("\\surroundwithmdframed[]{}")?;
  // mdframed.sty:1312-1347 `\mdfsubtitle[opts]{text}`: its own paragraph
  // (`\par\addvspace…\par`) in `\mdf@subtitlefont` (default `\bfseries`,
  // the `subtitlefont` key); the frame rules/skips are presentation. The
  // former no-op dropped the subtitle text (batch 56aj).
  RawTeX!(
    r"\providecommand\mdf@subtitlefont{\normalfont\bfseries}
\providecommand\mdf@subsubtitlefont{\normalfont}
\newcommand\mdfsubtitle[2][]{\par\noindent{\mdf@subtitlefont #2}\par}
\newcommand\mdfsubsubtitle[2][]{\par\noindent{\mdf@subsubtitlefont #2}\par}"
  );
  def_macro_noop("\\mdfapptodefinestyle{}{}")?;
  // mdframed.sty:386 `\mdfsetup` = `\kvsetkeys{mdf}`: a `framemethod` set there loads its package too.
  DefMacro!("\\mdfsetup{}", "\\kvsetkeys{mdf}{#1}");
  def_macro_noop("\\mdfdefinestyle{}{}")?;
  DefRegister!("\\mdflength" => Dimension::new(0));
  // The frame is inserted through `insert_block` (Perl TeX_Box.pool.ltxml:449
  // `insertBlock`, the shape `framed.sty.ltxml:21-26` and our `framed_sty.rs`
  // use): the body is built inside `ltx:_CaptureBlock_` and the container is
  // then chosen from content AND context — `logical-block` in flow (theorems,
  // nested frames: arXiv 2506.03074, 2402.07712, 1712.00062), the inline
  // variant inside a float (arXiv 1907.05772), a block child auto-closing
  // when backmatter follows (biblatex-juradiss `\printbibliography`). A
  // hand-rolled `open_element`/`absorb`/`maybe_close_element` with
  // `_autoclose` (the round-2 rewrite) let a box's stray nested-list state
  // leak into a low-level `\begin{list}` item and closed the OUTER list after
  // its first item (cnltx-example's `{example}` frames its output in
  // `\mdframed`; schulmathematik 1→40 errors, sweep 45; pdflatex clean,
  // RUST-ONLY). The capture-and-rename isolates that state.
  // Guards: `perfect_kernel_batch56::mdframed_inside_low_level_list_keeps_items`,
  // `perfect_kernel_gemini::{mdframed_block_bibliography_juradiss, mdframed_in_float_and_nested}`.
  DefEnvironment!(
    "{mdframed}[]",
    sub[document, _args, props] {
      document.maybe_close_element("ltx:p")?;
      if let Some(Stored::Digested(body)) = props.get("body") {
        let mut attrs: HashMap<String, String> = HashMap::default();
        attrs.insert("framed".to_string(), "rectangle".to_string());
        if let Some(Stored::String(framecolor)) = props.get("framecolor") {
          attrs.insert("framecolor".to_string(), to_string(*framecolor));
        }
        insert_block(document, body, attrs)?;
      }
      Ok(())
    },
    properties => sub[_args] {
      let mut props = SymHashMap::default();
      if let Some(font) = lookup_font()
        && let Some(color) = font.get_color() {
          props.insert("framecolor", Stored::from(color.to_attribute()));
        }
      Ok(props)
    },
    // mdframed bodies routinely contain multi-paragraph content
    // (theorems, displayed equations, multiple `$$..$$` blocks). The
    // DefEnvironment default of restricted_horizontal makes
    // BOUND_MODE never end with "vertical", so tex_math.rs:467's
    // `$$` → display-math check stays false: each `$$` is parsed as
    // open + immediate close, leaving body content in text mode and
    // cascading "Script _/^ can only appear in math mode" on subscripts.
    // Witness 2402.07712 (eqnarray + multiple `$$..$$` in mdframed).
    mode => "internal_vertical"
  );
});
