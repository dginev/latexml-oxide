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
  //
  // OXIDIZED_DESIGN #65 (#311): this is also where the standalone group OPENS
  // (Perl opens it back at \@standalone@documentclass). The child's
  // \begin{document} has just been consumed, so:
  //   * restore the real \begin{document} at the CALLER's level — the alias
  //     installed by \@standalone@documentclass is no longer inside a group
  //     that would undo it;
  //   * only then `bgroup`, so the group brackets the child's CONTENT and not
  //     its preamble;
  //   * alias \end{document} INSIDE the group, exactly as before, so the same
  //     `\egroup` (via \@standalone@end@input) undoes it.
  DefPrimitive!("\\@standalone@start@input", {
    Let!(T_CS!("\\begin{document}"), T_CS!("\\lx@standalone@saved@begindocument"));
    assign_value("inPreamble", false, None);
    bgroup();
    Let!(T_CS!("\\end{document}"), T_CS!("\\@standalone@end@input"));
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
  // OXIDIZED_DESIGN #65 (#311): NO `bgroup()` here. Perl opens the group at the
  // child's `\documentclass`, so the child's whole PREAMBLE runs inside it — and
  // a package loaded there registers document-level hooks that outlive the group
  // while its `\newif` conditionals do not. Real `standalone.sty` closes its own
  // `\begingroup` (L616) immediately before `\begin{document}` (L680), which is
  // where we open ours instead (`\@standalone@start@input`).
  DefPrimitive!("\\@standalone@documentclass OptionalKeyVals {}", sub[(options_kv, class_tks)] {
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
    // Stash the real \begin{document} so \@standalone@start@input can put it
    // back: with the group no longer wrapping the preamble, nothing else would.
    // Saving/restoring here is safe under nesting — the restore happens at the
    // child's own \begin{document}, before any nested subfile can intercept.
    // \end{document} is aliased later, inside the group, so `\egroup` undoes it.
    Let!(T_CS!("\\lx@standalone@saved@begindocument"), T_CS!("\\begin{document}"));
    Let!(T_CS!("\\begin{document}"), T_CS!("\\@standalone@start@input"));
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
