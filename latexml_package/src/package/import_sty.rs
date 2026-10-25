use latexml_core::util::pathname;

use crate::prelude::*;

/// import.sty:147-151 `\import@path@fix`: a non-empty directory ends in exactly one `/` (`\@ensure@one@trailing@slash`);
/// an empty one stays empty.
fn import_path_fix(path: &str) -> String {
  let path = path.trim();
  if path.is_empty() {
    String::new()
  } else {
    s!("{}/", path.trim_end_matches('/'))
  }
}

/// The current `\import@path` (import.sty:142, empty outside any import).
fn import_path() -> String { lookup_string("lx@import@path") }

/// import.sty:82-85 `\@import`: `\import@path` becomes `path`, and `\input@path` gets it in front of the enclosing
/// entries (TeX resolves each entry against the working directory, here the source directory). The `*` forms are
/// the same (import.sty:60-62 "Consume now-meaningless `*`"); Perl's `*` searched only the import's directory, which
/// loses the source directory the joined name is relative to (`\subimport*{nested/}{outer.tex}` reads
/// `nested/outer.tex`).
fn enter_import_path(path: String) {
  assign_value("lx@import@path", Stored::String(pin(&path)), None);
  let mut new_paths = Vec::new();
  if !path.is_empty() {
    let dir = if pathname::is_absolute(&path) {
      path.clone()
    } else {
      let source_dir = lookup_string("SOURCEDIRECTORY");
      if source_dir.is_empty() {
        path.clone()
      } else {
        pathname::concat(&source_dir, &path)
      }
    };
    new_paths.push(pathname::canonical(&dir));
  }
  new_paths.extend(get_search_paths());
  set_search_paths_local(new_paths);
}

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: import.sty.ltxml — `AssignValue(SEARCHPATHS => …)`, local-by-default.
  // The paths are saved before and restored after each import's input
  // (`\lx@import@save`/`\lx@import@restore` below), so sibling
  // `\subimport{Chapter/}{File}` calls each start from the base paths (the
  // second must NOT concat Chapter/ onto the first call's Chapter/). Witnesses:
  // arXiv:2604.09744, 2603.04457.
  //
  // The import's file is found as import.sty finds it (:82-88): `\import@path` is the directory, reset by `\import`
  // (:49) and extended by `\subimport` (:55, `\import@path#3`), and the file read is `\import@path #7`, with
  // `\input@path` = `{\import@path}` + the enclosing import's entries. A nested `\import{intro}{intro}` inside
  // `\import{chapters/}{chapters}` reads `intro/intro`, found under the enclosing entry as
  // `chapters/intro/intro.tex`, and a `\subimport{}{statements/x.tex}` there reads `intro/statements/x.tex`
  // (2408.09583: its chapters, then its statement files). Perl's `\input{#3}` against a search path made of the
  // first entry and the new directory (import.sty.ltxml:20-47) found neither (KPE #1171).

  // Perl import.sty.ltxml L20-29: \lx@set@path OptionalMatch:* {} — `\import`'s directory, relative to the source
  // directory unless absolute.
  DefPrimitive!("\\lx@set@path OptionalMatch:* {}", sub[(_star, path_tks)] {
    let path = import_path_fix(&Expand!(path_tks).to_string());
    enter_import_path(path);
  });

  // Perl import.sty.ltxml L31-42: \lx@append@path OptionalMatch:* {} — `\subimport`'s directory, appended to the
  // current `\import@path`.
  //   DIVERGENCE (OXIDIZED_DESIGN #137): an absolute `path` is used verbatim
  //   rather than concatenated, to match real LaTeX — see the body.
  DefPrimitive!("\\lx@append@path OptionalMatch:* {}", sub[(_star, path_tks)] {
    let raw = Expand!(path_tks).to_string();
    let raw = raw.trim();
    // OXIDIZED_DESIGN #137 — surpass Perl to match real LaTeX (pdflatex,
    // verified): an ABSOLUTE directory arg is used verbatim, not concatenated
    // onto the current import path (which yields an unresolvable `<lead>//abs/…`).
    // Perl's \lx@append@path (import.sty.ltxml L31-42) ALWAYS concats, so both
    // engines fail `\subimport*{/abs/}{file}` where pdflatex succeeds — issue
    // #697. \lx@set@path already special-cases absolute; mirror it here.
    let path = if pathname::is_absolute(raw) {
      import_path_fix(raw)
    } else {
      import_path_fix(&s!("{}{raw}", import_path()))
    };
    enter_import_path(path);
  });

  // `\@import`'s read (import.sty:86-88): `<command>{\import@path <file>}`. Where that file does not exist but the
  // bare name does, the bare name, as import.sty before v6 read it (`#1{#7}` with `\import@path` first on
  // `\input@path`; TeX tries the working directory first): an archive whose `\import{pictures/}{cantor.pdf_tex}`
  // ships `cantor.pdf_tex` at its top (1409.7023) or whose `\subimport{SM_quarkgluon/}{quarkgluon_main}` ships the
  // chapter beside the main file (1605.04692) compiled so on arXiv; TL 2025's v6.2 stops ("File not found").
  DefMacro!("\\lx@import@input{}{}", sub[(command, file)] {
    let mut toks = command.unlist();
    toks.push(T_BEGIN!());
    let path = import_path();
    let name = Expand!(file.clone()).to_string();
    let name = name.trim();
    if !path.is_empty()
      && (find_file(&s!("{path}{name}"), None).is_some() || find_file(name, None).is_none())
    {
      toks.extend(ExplodeText!(path));
    }
    toks.extend(file.unlist());
    toks.push(T_END!());
    Ok(Tokens::new(toks))
  });

  // No group around the input (import.sty:65-92): `\@sub@import` closes its own
  // group before `\@import` runs the `\input`/`\include` at the caller's level,
  // then restores `\input@path`/`\Ginput@path` by plain `\def`. Perl's binding
  // (import.sty.ltxml L44-47) wraps the input in `{…}`, so every definition the
  // imported file makes — its `\newcommand`s, and the packages it loads, whose
  // loaded-flags are global — was popped at the `}`: `\subimport{}{macros}` left
  // the paper's macros undefined, and a package loaded there (hyperref → etoolbox)
  // was "already loaded" but gone when biblatex asked for `\newbool` (arXiv
  // 2605.20598, `Fatal:TooManyErrors`). The search paths and `\import@path` are
  // saved before and restored after the input instead (import.sty:92-95), so each
  // sibling `\subimport{Chapter/}{…}` still starts from the base paths (witnesses
  // 2604.09744, 2603.04457).
  // OXIDIZED_DESIGN #280.
  //
  // KNOWN_PERL_ERRORS #56: `\includefrom`/`\subincludefrom` take TWO arguments
  // after the star — `\includefrom{dir/}{file}` — but Perl's prototypes declare
  // only one while their bodies reference `#3` (import.sty.ltxml L45/L47). The
  // undeclared `#3` expands to nothing, so `\include{}` includes nothing and the
  // file is dropped in silence: no error, no warning, no content. Real
  // `import.sty` takes both for all four (L57/L58 route `\includefrom` /
  // `\subincludefrom` through the same `\@doimport` as `\import`/`\subimport`;
  // `\@sub@import` L65 consumes the directory as #3 and `\@import` L82 the file
  // name as #7), so the arity below is the real package's, not Perl's typo.
  DefPrimitive!("\\lx@import@save", {
    let saved = lookup_value("SEARCHPATHS").unwrap_or(Stored::None);
    let mut stack = match lookup_value("lx@import@saved@paths") {
      Some(Stored::VecDequeStored(stack)) => stack,
      _ => VecDeque::new(),
    };
    stack.push_front(saved);
    stack.push_front(Stored::String(pin(import_path())));
    assign_value("lx@import@saved@paths", Stored::VecDequeStored(stack), Some(Scope::Global));
  });
  DefPrimitive!("\\lx@import@restore", {
    if let Some(Stored::VecDequeStored(mut stack)) = lookup_value("lx@import@saved@paths")
      && let Some(saved_import_path) = stack.pop_front()
      && let Some(saved) = stack.pop_front()
    {
      assign_value("lx@import@saved@paths", Stored::VecDequeStored(stack), Some(Scope::Global));
      assign_value("SEARCHPATHS", saved, Some(Scope::Local));
      assign_value("lx@import@path", saved_import_path, None);
    }
  });
  DefMacro!("\\import OptionalMatch:* {}{}",
    "\\lx@import@save\\lx@set@path #1{#2} \\lx@import@input{\\input}{#3}\\lx@import@restore");
  DefMacro!("\\includefrom OptionalMatch:* {}{}",
    "\\lx@import@save\\lx@set@path #1{#2} \\lx@import@input{\\include}{#3}\\lx@import@restore");
  DefMacro!("\\subimport OptionalMatch:* {}{}",
    "\\lx@import@save\\lx@append@path #1{#2} \\lx@import@input{\\input}{#3}\\lx@import@restore");
  DefMacro!("\\subincludefrom OptionalMatch:* {}{}",
    "\\lx@import@save\\lx@append@path #1{#2} \\lx@import@input{\\include}{#3}\\lx@import@restore");
  Let!("\\inputfrom", "\\import");
  Let!("\\subinputfrom", "\\subimport");
});
