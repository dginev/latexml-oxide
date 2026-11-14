//! standalone.sty — compile standalone sub-documents
//! Perl: standalone.sty.ltxml (40 lines).
//! NOTE: standalone.cls is handled separately; this is the .sty package.
use latexml_core::util::image::image_candidates;

use crate::prelude::*;

/// The `standalone.cls` options that load a same-named package
/// (standalone.cls L171/193/237/249/255, resolved at L562 and L611-620).
/// Every other option — `crop`, `multi`, `math`, `beamer`, `float`, `png`,
/// `border=`, `class=`, `10pt`/`11pt`/`12pt`, … — the class handles itself.
const CLASS_OPTION_PACKAGES: [&str; 5] = ["tikz", "pstricks", "preview", "varwidth", "multido"];

#[rustfmt::skip]
LoadDefinitions!({
  // standalone.sty:172-211 `mode=`: how `\includestandalone` includes a sub-file (`tex`, the default, L211).
  if let Some(opts) = lookup_vecdeque("opt@standalone.sty") {
    for opt in opts.iter() {
      let opt = opt.to_string();
      let (key, value) = opt.split_once('=').map_or((opt.trim(), "true"), |(k, v)| (k.trim(), v.trim()));
      if key == "mode" {
        assign_value("standalone_mode", Stored::String(pin(unbraced(value))), Some(Scope::Global));
      }
      // standalone.sty:97-144: `subpreambles` (or `sortsubpreambles`/`printsubpreambles`) keeps the sub-files'
      // preambles for the main one
      if key.ends_with("subpreambles") && value != "false" {
        assign_value("standalone_subpreambles", Stored::Bool(true), Some(Scope::Global));
      }
    }
  }
  // standalone.sty:26-39: in a main document `\ifstandalone` (and `\ifstandalonebeamer`) is false, `\onlyifstandalone`
  // gobbles its argument and `\IfStandalone{<standalone>}{<main>}` takes the second (11 papers `\ifstandalone`:
  // 1310.0809, 2010.10238, 2011.05064; `\onlyifstandalone` 2208.05046, 2403.07821; `\IfStandalone` 2208.05046); :707-709
  // an environment `{standalone}[<opt>]` that keeps its body (2310.18279). Perl's binding defines none
  // (KNOWN_PERL_ERRORS #806).
  RawTeX!(r"\@ifundefined{ifstandalone}{\expandafter\newif\csname ifstandalone\endcsname\standalonefalse}{}
\@ifundefined{ifstandalonebeamer}{\expandafter\newif\csname ifstandalonebeamer\endcsname\standalonebeamerfalse}{}
\@ifundefined{onlyifstandalone}{\let\onlyifstandalone\@gobble}{}
\@ifundefined{IfStandalone}{\let\IfStandalone\@secondoftwo}{}
\@ifundefined{standalone}{\newenvironment{standalone}[1][]{}{}}{}");
  // standalone.sty:296-304 `\standaloneconfig{<keys>}` sets the package's keys as its options do (`\setkeys
  // {standalone.sty}`): `mode=` (`tex`, `image`, `build`, `buildmissing`, `buildnew`; 17 papers: 1905.04614,
  // 1910.10882, 2210.09924) is the mode `\includestandalone` follows, and `subpreambles` keeps the sub-files'
  // preambles; the other keys shape a build or the class and are read without effect, their values unexpanded as
  // `\setkeys` stores them (`build={latexoptions={… -jobname \quote\buildjobname\quote}}`, 1703.03769's copy).
  DefPrimitive!("\\standaloneconfig Undigested", sub[(keys)] {
    for item in top_level_items(&keys.to_string()) {
      let (key, value) = item.split_once('=').map_or((item.trim(), "true"), |(k, v)| (k.trim(), v.trim()));
      if key == "mode" {
        assign_value("standalone_mode", Stored::String(pin(unbraced(value))), None);
      }
      if key.ends_with("subpreambles") && value != "false" {
        assign_value("standalone_subpreambles", Stored::Bool(true), None);
      }
    }
  });
  // standalone.sty:255 reads its options with `\ProcessOptionsX`, which marks them processed: `\@curroptions` stays as it was.
  key_options_processed()?;
  // BEYOND PERL (the Perl standalone.sty.ltxml omits these): the real
  // standalone.sty has exactly TWO *unconditional* `\RequirePackage`s —
  // `xkeyval` (L107) and `currfile` (L305). (Every other require is guarded:
  // engine probes `ifpdf`/`ifluatex`/`ifxetex`/`shellesc`, and the
  // `\IfFileExists`/option-gated `varwidth`/`trimclip`/`adjustbox`/`gincltex`/
  // `filemod-expmin`.) Restore just those two, which real LaTeX always provides:
  //   * `xkeyval` defines `\define@key` (and the wider keyval family);
  //   * `currfile` `\RequirePackage{filehook}` (currfile.sty L30), and filehook
  //     defines the package-file hooks `\AtEndOfPackageFile`/`\AtBeginOfPackageFile`/…
  // sTeX 3.x leans on both: `\AtEndOfPackageFile{graphicx}{\define@key{Gin}
  // {archive}{…}}` (stex.sty L2134) — without them the hook is undefined and its
  // deferred body runs prematurely (`\define@key` undefined). Rust ships bindings
  // for all three. Witness: raw stex.sty under ar5iv. (Unconditional — a binding
  // emulates the package identically regardless of INCLUDE_STYLES; both requires
  // resolve to always-available package bindings.)
  RequirePackage!("xkeyval");
  RequirePackage!("currfile");
  // standalone.sty:1012-1013 `\IfFileExists{gincltex.sty}{\RequirePackage{gincltex}…}` (TeX Live has it): gincltex.sty:43
  // `\DeclareGraphicsRule{.tex}{tex}{.tex}{}`, so `\includegraphics{x.tex}` beside standalone reads the file as TeX
  // (graphicx_sty.rs `\lx@Gin@include`); handed to the image converter it was lost (2511.17298, 2305.04329,
  // 2408.06999, 2305.16841). gincltex has no binding, so it is read raw: only where raw styles are (amsrefs_sty.rs the
  // same). Guard: `perfect_kernel_batch70a::standalone_loads_gincltex`.
  // A sub-file gincltex boxes (`\gincltex@boxfile`'s `\input`, gincltex.sty:45-56, from its reader `\Gread@tex` or its
  // include `\Ginclude@tex`, :57-90) is the sub-file `\includestandalone` reads (standalone.sty:1014-1021 hands it to
  // `\includegraphics`): standalone's `\sa@documentclass` (:602-646) gobbles its preamble, so its packages never load
  // in the main document's body — 2512.06733's `\includegraphics{figures/c4_ex1.tex}` ran the sub-file's
  // `\usepackage{cleveref}` after `\begin{document}`, and the begin-document half of cleveref never came
  // ("`\lx@cleverrefnum@@` undefined" ×8).
  // Guard `perfect_kernel_batchint20::gincltex_subfile_preamble_is_skipped`.
  if lookup_bool("INCLUDE_STYLES") {
    RequirePackage!("gincltex");
    if IsDefined!(&T_CS!("\\gincltex@boxfile")) {
      RawTeX!(
        r"\let\lx@standalone@gincltex@boxfile\gincltex@boxfile
\def\gincltex@boxfile#1{\lx@standalone@skippreamble\lx@standalone@gincltex@boxfile{#1}}"
      );
    }
  }

  DefMacro!("\\@standalone@end@input", "\\egroup\\endinput");
  // The sub-file `\includestandalone` reads next has its preamble skipped (in the group around it).
  DefPrimitive!("\\lx@standalone@skippreamble", {
    assign_value("standalone_skip_preamble", Stored::Bool(true), None);
  });

  // Perl L21-23: DefPrimitiveI \@standalone@start@input — sets inPreamble = 0.
  DefPrimitive!("\\@standalone@start@input", {
    assign_value("inPreamble", false, None);
    assign_value("lx@subpreamble@begindocument", Stored::Bool(false), None);
  });

  // Perl L24-33: DefPrimitive \@standalone@documentclass[]{} — open a
  // group, mark inPreamble = 1, RequirePackage each comma-separated entry
  // of the OPTIONAL `[]` argument (Perl binds `$packages = $_[1]`, the
  // optional), and alias \begin{document}/\end{document} to the start/end
  // input primitives so the sub-document is injected as a bounded scope
  // inside the outer document.
  //
  // OXIDIZED_DESIGN #63: NEITHER argument is a package list, so both are
  // gated. The mandatory class name is ignored outright — the parent already
  // loaded a class, and requiring it warned `missing_file:article` on a
  // `\documentclass{article}` child (#293). The optional list holds class
  // OPTIONS; Perl requires all of them for every class, so
  // `\documentclass[12pt]{article}` warns `missing_file:12pt` (#309).
  // standalone.sty L604-614 consults a subfile's options only when the
  // subfile's class is literally `standalone`, so we require them only there
  // — and only the ones standalone.cls turns into a package load, which is
  // what makes `\documentclass[tikz]{standalone}` work (upstream LaTeXML#1432,
  // the reason this loop exists).
  // The optional argument is read as `OptionalKeyVals`, NOT as a raw string we
  // comma-split ourselves: a class option list IS a keyval list, and every
  // option here has a valued form — `\sa@boolorvalue` accepts `varwidth=5cm`
  // and `tikz=true` exactly as it accepts bare `varwidth`/`tikz`
  // (standalone.sty L815-824), and `border={1pt 2pt}` puts a brace group in the
  // list. Splitting on `,` and matching the whole item missed every valued form
  // — `[varwidth=5cm]{standalone}` then lost the package and reported
  // `Error:undefined:{varwidth}` where pdflatex is clean. Reusing the engine's
  // keyval reader gets brace-aware splitting and key/value separation for free,
  // and keeps this on the same parser `\documentclass`/`\usepackage` options
  // already flow through instead of a second, weaker one.
  DefPrimitive!("\\@standalone@documentclass OptionalKeyVals {}", sub[(options_kv, class_tks)] {
    bgroup();
    // OXIDIZED_DESIGN #65 (#311): the bracket just opened is a LaTeXML artifact —
    // real standalone.sty *gobbles* the child preamble (`\sa@gobble`), so
    // nothing loads inside a group there, and LaTeXML only executes that
    // preamble to make `\documentclass[tikz]{standalone}` work (#63). Name the
    // region with the engine's own scope machinery (Perl's named scopes, e.g.
    // `section:4`/`label:foo`, State.pm L965-975) so `require_package` can give a package
    // loaded in here the outermost-level lifetime real LaTeX would have given
    // it. `activate_scope` marks StashActive with `Scope::Local`, so the region
    // ends exactly when this bracket pops — we do not have to arrange that. A
    // group the AUTHOR wrote carries no scope and is untouched: `{\usepackage
    // {amsthm}}` must still leave `\theoremstyle` undefined, as it does in
    // pdflatex ("Loading a class or package in a group", then "Undefined control
    // sequence") and in Perl. Guards:
    // `06_cluster_regressions::author_written_group_around_usepackage_still_loses_the_package`
    // and `100_stale_autoload_no_runaway` (same boundary, fresh process).
    activate_scope(subfile_scope_here());
    assign_value("inPreamble", true, None);
    // its `\AtBeginDocument` code is not run (`at_document_hook`, sect02.rs; 2405.01743)
    assign_value("lx@subpreamble@begindocument", Stored::Bool(true), None);
    if class_tks.to_string().trim() == "standalone"
      && let Some(kv) = options_kv.as_ref()
    {
      // Match on the KEY, so `varwidth` and `varwidth=5cm` behave alike. An
      // absent optional yields no pairs ⇒ nothing required.
      for (key, _value) in kv.get_pairs() {
        if CLASS_OPTION_PACKAGES.contains(&key.trim()) {
          RequirePackage!(key.trim());
        }
      }
    }
    // A sub-file of the standalone class names its cropped environments in its preamble (`\standaloneenv
    // {circuitikz}`, 2505.21582), which standalone.sty never runs (see below); run here, it marks nothing
    // (standalone.sty:802 `\let\standaloneenv\@gobble`).
    if class_tks.to_string().trim() == "standalone" && !IsDefined!(&T_CS!("\\standaloneenv")) {
      Let!(T_CS!("\\standaloneenv"), T_CS!("\\@gobble"));
    }
    Let!(T_CS!("\\begin{document}"), T_CS!("\\@standalone@start@input"));
    Let!(T_CS!("\\end{document}"),   T_CS!("\\@standalone@end@input"));
    // standalone.sty:602-646 `\sa@documentclass` gobbles the sub-file's preamble to its `\begin{document}`: it never
    // runs there, so a sub-file's `\title`, its own packages (`\usepackage{emoji}`, LuaTeX only; 2403.17633) and
    // `\input`s (2508.06316 `config-gfx`) are not the main document's. With `subpreambles` the preambles are kept for the
    // main preamble of the next run (`\subpreamble`, :654-680), so they run in place, in the bracket above. Done for a
    // sub-file `\includestandalone` reads; a plain `\input` keeps running the preamble, as Perl does, which renders
    // children whose packages only their own preamble loads (#311, OXIDIZED_DESIGN_DIVERGENCES #63, #65, #463).
    if matches!(lookup_value("standalone_skip_preamble"), Some(Stored::Bool(true)))
      && !matches!(lookup_value("standalone_subpreambles"), Some(Stored::Bool(true)))
    {
      loop {
        let Some(token) = read_token()? else {
          // pdflatex: "File ended while scanning use of \sa@gobble"
          Error!("expected", "\\begin{document}",
            "the standalone sub-file ended while its preamble was skipped, before its \\begin{document}");
          break;
        };
        if token == T_CS!("\\begin") {
          let name = read_arg(ExpansionLevel::Off)?;
          if name.to_string().trim() == "document" {
            let mut begin = vec![T_CS!("\\begin"), T_BEGIN!()];
            begin.extend(name.unlist());
            begin.push(T_END!());
            unread(Tokens::new(begin));
            break;
          }
        }
      }
    }
  });

  // Perl L35-36: AtBeginDocument — swap \documentclass to the intercept.
  // Native push to @at@begin@document so the hook fires at the same
  // lifecycle point Perl uses.
  at_begin_document(TokenizeInternal!(r"\let\documentclass\@standalone@documentclass"))?;

  // standalone.sty:1014-1093 `\includestandalone[opts]{file}`, by its mode (the `mode` key, else the package option,
  // else `tex`): `tex`, and `build` without shell escape, inputs `file.tex`, set in a box and scaled to the requested
  // width, height or scale as gincltex does (gincltex.sty:45-67); `image` includes the image; `image|tex`,
  // `buildmissing` and `buildnew` (which compares file times, :1078-1092) include `file.pdf` when it exists, else the
  // `.tex`. The
  // `.tex` is found as a graphic is (`\graphicspath`). Before, every call was `\includegraphics{file}`, whose `.tex`
  // candidate nothing renders: the figure was lost (2406.02722, 2412.12317, 2505.19304, 2608.05283, 2504.17583, …;
  // html_feedback HF12). Perl's binding lacks the command.
  DefMacro!("\\includestandalone[]{}", sub[(opts, file)] {
    // standalone.sty:1017 expands the name (`\edef\@tempa{{#2…}}`)
    let file = Expand!(file).to_string();
    let base = file.trim();
    let mut mode = lookup_string("standalone_mode");
    let mut gin: Vec<(String, String)> = Vec::new();
    for item in top_level_items(&opts.map(|o| o.to_string()).unwrap_or_default()) {
      // (xkeyval's `\setkeys` strips a value's one brace level: `width={0.5\textwidth}`)
      let (key, value) = item.split_once('=').map_or((item.as_str(), ""), |(k, v)| (k.trim(), unbraced(v)));
      if key == "mode" {
        mode = value.to_string();
      } else {
        gin.push((key.to_string(), value.to_string()));
      }
    }
    // as a graphic is found (`\graphicspath`), or as `\input` finds it (a `filecontents` file)
    let tex = image_candidates(&s!("{base}.tex"))
      .split(',')
      .next()
      .filter(|found| !found.is_empty())
      .map(str::to_string)
      .or_else(|| find_file(&s!("{base}.tex"), None).map(|_| base.to_string()))
      .unwrap_or_default();
    let image_exists = !image_candidates(&s!("{base}.pdf")).is_empty();
    let use_tex = !tex.is_empty()
      && match mode.as_str() {
        "image" => false,
        "image|tex" | "buildmissing" | "buildnew" => !image_exists,
        _ => true,
      };
    let value = |key: &str| gin.iter().find(|(k, _)| k == key).map(|(_, v)| v.clone()).filter(|v| !v.is_empty());
    let source = if !use_tex {
      let options = gin
        .iter()
        .map(|(k, v)| if v.is_empty() { k.clone() } else { s!("{k}={v}") })
        .collect::<Vec<_>>()
        .join(",");
      s!("\\includegraphics[{options}]{{{base}}}")
    } else {
      let body = s!("{{\\lx@standalone@skippreamble\\input{{{tex}}}}}");
      match (value("width"), value("height"), value("scale")) {
        (None, None, None) => body,
        (None, None, Some(scale)) => s!("\\scalebox{{{scale}}}{body}"),
        (width, height, _) => s!(
          "\\resizebox{{{}}}{{{}}}{body}",
          width.unwrap_or_else(|| s!("!")),
          height.unwrap_or_else(|| s!("!"))
        ),
      }
    };
    Ok(mouth::tokenize_internal(TeXString::assembled(source)))
  });
});

/// A keyval value without one balanced outer brace pair, as xkeyval reads it (`{0.5\textwidth}`; not `{a}{b}`).
pub(crate) fn unbraced(value: &str) -> &str {
  let value = value.trim();
  if !value.starts_with('{') || !value.ends_with('}') {
    return value;
  }
  let mut depth = 0usize;
  for (at, c) in value.char_indices() {
    match c {
      '{' => depth += 1,
      '}' => {
        depth = depth.saturating_sub(1);
        if depth == 0 && at + 1 < value.len() {
          return value;
        }
      },
      _ => {},
    }
  }
  value[1..value.len() - 1].trim()
}

/// The items of a keyval list at brace depth 0.
pub(crate) fn top_level_items(list: &str) -> Vec<String> {
  let (mut items, mut current, mut depth) = (Vec::new(), String::new(), 0usize);
  for c in list.chars() {
    match c {
      '{' => depth += 1,
      '}' => depth = depth.saturating_sub(1),
      ',' if depth == 0 => {
        items.push(std::mem::take(&mut current));
        continue;
      },
      _ => {},
    }
    current.push(c);
  }
  items.push(current);
  items
    .into_iter()
    .map(|i| i.trim().to_string())
    .filter(|i| !i.is_empty())
    .collect()
}
