use latexml_core::util::pathname;

use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: import.sty.ltxml — `AssignValue(SEARCHPATHS => …)`, local-by-default.
  // The paths are saved before and restored after each import's input
  // (`\lx@import@save`/`\lx@import@restore` below), so sibling
  // `\subimport{Chapter/}{File}` calls each start from the base paths (the
  // second must NOT concat Chapter/ onto the first call's Chapter/). Witnesses:
  // arXiv:2604.09744, 2603.04457.

  // Perl import.sty.ltxml L20-29: \lx@set@path OptionalMatch:* {}
  //   path = ToString(Expand(#2)); if relative, resolve vs SOURCEDIRECTORY.
  //   If * → replace SEARCHPATHS with [canonical(path)]
  //   else → prepend canonical(path) to existing SEARCHPATHS.
  DefPrimitive!("\\lx@set@path OptionalMatch:* {}", sub[(star, path_tks)] {
    let raw = Expand!(path_tks).to_string();
    let mut path = raw.trim().to_string();
    if path.is_empty() { return Ok(Vec::new()); }
    if !pathname::is_absolute(&path) {
      let source_dir = lookup_string("SOURCEDIRECTORY");
      if !source_dir.is_empty() {
        path = pathname::concat(&source_dir, &path);
      }
    }
    let canonical = pathname::canonical(&path);
    // LOCAL, restored by `\lx@import@save`/`\lx@import@restore` around the input,
    // matching Perl's default-local `AssignValue(SEARCHPATHS…)`.
    if star.is_some() {
      set_search_paths_local(vec![canonical]);
    } else {
      let mut new_paths = vec![canonical];
      new_paths.extend(get_search_paths());
      set_search_paths_local(new_paths);
    }
  });

  // Perl import.sty.ltxml L31-42: \lx@append@path OptionalMatch:* {}
  //   If SEARCHPATHS has entries, concat the first with path:
  //   new_lead = concat(lead_path, path); star → [new_lead], else → [new_lead, ...rest].
  //   If SEARCHPATHS is empty, this is a no-op (matches Perl's early-return).
  //   DIVERGENCE (OXIDIZED_DESIGN #137): an absolute `path` is used verbatim
  //   rather than concatenated, to match real LaTeX — see the body.
  DefPrimitive!("\\lx@append@path OptionalMatch:* {}", sub[(star, path_tks)] {
    let raw = Expand!(path_tks).to_string();
    let path = raw.trim().to_string();
    if path.is_empty() { return Ok(Vec::new()); }
    let mut paths = get_search_paths();
    if paths.is_empty() { return Ok(Vec::new()); }
    let lead = paths.remove(0);
    // OXIDIZED_DESIGN #137 — surpass Perl to match real LaTeX (pdflatex,
    // verified): an ABSOLUTE directory arg is used verbatim, not concatenated
    // onto the lead search path (which yields an unresolvable `<lead>//abs/…`).
    // Perl's \lx@append@path (import.sty.ltxml L31-42) ALWAYS concats, so both
    // engines fail `\subimport*{/abs/}{file}` where pdflatex succeeds — issue
    // #697. \lx@set@path already special-cases absolute; mirror it here.
    // Relative args are unchanged (the common `\subimport*{sub/}{file}` case).
    let new_lead = if pathname::is_absolute(&path) {
      pathname::canonical(&path)
    } else {
      pathname::concat(&lead, &path)
    };
    // LOCAL (see `\lx@set@path`): restored after the input.
    if star.is_some() {
      set_search_paths_local(vec![new_lead]);
    } else {
      let mut new_paths = vec![new_lead];
      new_paths.extend(paths);
      set_search_paths_local(new_paths);
    }
  });

  // No group around the input (import.sty:65-92): `\@sub@import` closes its own
  // group before `\@import` runs the `\input`/`\include` at the caller's level,
  // then restores `\input@path`/`\Ginput@path` by plain `\def`. Perl's binding
  // (import.sty.ltxml L44-47) wraps the input in `{…}`, so every definition the
  // imported file makes — its `\newcommand`s, and the packages it loads, whose
  // loaded-flags are global — was popped at the `}`: `\subimport{}{macros}` left
  // the paper's macros undefined, and a package loaded there (hyperref → etoolbox)
  // was "already loaded" but gone when biblatex asked for `\newbool` (arXiv
  // 2605.20598, `Fatal:TooManyErrors`). The search paths are saved before and
  // restored after the input instead, so each sibling `\subimport{Chapter/}{…}`
  // still starts from the base paths (witnesses 2604.09744, 2603.04457).
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
    assign_value("lx@import@saved@paths", Stored::VecDequeStored(stack), Some(Scope::Global));
  });
  DefPrimitive!("\\lx@import@restore", {
    if let Some(Stored::VecDequeStored(mut stack)) = lookup_value("lx@import@saved@paths")
      && let Some(saved) = stack.pop_front()
    {
      assign_value("lx@import@saved@paths", Stored::VecDequeStored(stack), Some(Scope::Global));
      assign_value("SEARCHPATHS", saved, Some(Scope::Local));
    }
  });
  DefMacro!("\\import OptionalMatch:* {}{}",
    "\\lx@import@save\\lx@set@path #1{#2} \\input{#3}\\lx@import@restore");
  DefMacro!("\\includefrom OptionalMatch:* {}{}",
    "\\lx@import@save\\lx@set@path #1{#2} \\include{#3}\\lx@import@restore");
  DefMacro!("\\subimport OptionalMatch:* {}{}",
    "\\lx@import@save\\lx@append@path #1{#2} \\input{#3}\\lx@import@restore");
  DefMacro!("\\subincludefrom OptionalMatch:* {}{}",
    "\\lx@import@save\\lx@append@path #1{#2} \\include{#3}\\lx@import@restore");
  Let!("\\inputfrom", "\\import");
  Let!("\\subinputfrom", "\\subimport");
});
