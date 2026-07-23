//! standalone.sty — compile standalone sub-documents
//! Perl: standalone.sty.ltxml (40 lines).
//! NOTE: standalone.cls is handled separately; this is the .sty package.
use crate::prelude::*;

/// The `standalone.cls` options that load a same-named package
/// (standalone.cls L171/193/237/249/255, resolved at L562 and L611-620).
/// Every other option — `crop`, `multi`, `math`, `beamer`, `float`, `png`,
/// `border=`, `class=`, `10pt`/`11pt`/`12pt`, … — the class handles itself.
const CLASS_OPTION_PACKAGES: [&str; 5] = ["tikz", "pstricks", "preview", "varwidth", "multido"];

#[rustfmt::skip]
LoadDefinitions!({
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

  DefMacro!("\\@standalone@end@input", "\\egroup\\endinput");

  // Perl L21-23: DefPrimitiveI \@standalone@start@input — sets inPreamble = 0.
  DefPrimitive!("\\@standalone@start@input", {
    assign_value("inPreamble", false, None);
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
    Let!(T_CS!("\\begin{document}"), T_CS!("\\@standalone@start@input"));
    Let!(T_CS!("\\end{document}"),   T_CS!("\\@standalone@end@input"));
  });

  // Perl L35-36: AtBeginDocument — swap \documentclass to the intercept.
  // Native push to @at@begin@document so the hook fires at the same
  // lifecycle point Perl uses.
  at_begin_document(TokenizeInternal!(r"\let\documentclass\@standalone@documentclass"))?;

  // standalone.sty L1014: \includestandalone[opts]{file}. Treat as
  // \includegraphics{file} so the figure surfaces in the XML output.
  // Witness 2406.02722.
  DefMacro!("\\includestandalone[]{}", "\\includegraphics{#2}");
});
