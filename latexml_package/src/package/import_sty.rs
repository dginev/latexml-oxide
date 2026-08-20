use latexml_core::util::pathname;

use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: import.sty.ltxml — `AssignValue(SEARCHPATHS => …)`, local-by-default,
  // reverted by the `{…}` group each `\import`/`\subimport` wraps its body in.
  // SEARCHPATHS is now a group-scoped value in Rust too (state::get/set with
  // `Scope::Local`), so no explicit save/restore stack is needed — the group
  // handles the revert, faithful to Perl. Witnesses: arXiv:2604.09744,
  // 2603.04457 (sibling `\subimport{Chapter/}{File}` calls — the second must
  // NOT concat Chapter/ onto the first call's Chapter/).

  // Mark the `{…}` each `\import`/`\subimport` opens as a LaTeXML subfile scope.
  // OXIDIZED_DESIGN #65 (#311): the group is a LaTeXML artifact — the real
  // import.sty never groups the input (`\@import` restores `\input@path`/
  // `\Ginput@path` by plain `\def` AFTER the `\input`, at the caller's level;
  // "input files must have balanced grouping", L42). Naming the region lets
  // `require_package` give a package loaded in there the outermost-level lifetime
  // real LaTeX would (content.rs `is_scope_active(subfile_scope_here())`). The
  // marker is `Scope::Local`, so the region ends with the group. (This primitive
  // formerly also saved SEARCHPATHS; that is now handled by group-scoping the
  // value itself — see `\lx@set@path`/`\lx@append@path`.) Mirror of
  // `standalone_sty.rs`'s inline `activate_scope(subfile_scope_here())`.
  DefPrimitive!("\\lx@activate@subfile@scope", {
    activate_scope(subfile_scope_here());
  });

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
    // LOCAL: reverted by the enclosing `\import`/`\subimport` `{…}` group,
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
    // LOCAL (see `\lx@set@path`): the `{…}` group reverts it.
    if star.is_some() {
      set_search_paths_local(vec![new_lead]);
    } else {
      let mut new_paths = vec![new_lead];
      new_paths.extend(paths);
      set_search_paths_local(new_paths);
    }
  });

  // Each `\import`/`\subimport` wraps its body in a `{…}` group (with
  // `\lx@activate@subfile@scope` naming the subfile scope). The path change is LOCAL, so the
  // group reverts it at `}` — each sibling starts from the BASE search paths,
  // exactly as Perl's default-local `AssignValue(SEARCHPATHS…)` does. Without
  // group-local paths, two sibling `\subimport{Chapter/}{Abstract}` +
  // `\subimport{Chapter/}{Poster}` would concat Chapter/ onto the first call's
  // still-mutated lead → "Chapter/Chapter/Poster". Witnesses 2604.09744, 2603.04457.
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
  DefMacro!("\\import OptionalMatch:* {}{}",
    "{\\lx@activate@subfile@scope\\lx@set@path #1{#2} \\input{#3}}");
  DefMacro!("\\includefrom OptionalMatch:* {}{}",
    "{\\lx@activate@subfile@scope\\lx@set@path #1{#2} \\include{#3}}");
  DefMacro!("\\subimport OptionalMatch:* {}{}",
    "{\\lx@activate@subfile@scope\\lx@append@path #1{#2} \\input{#3}}");
  DefMacro!("\\subincludefrom OptionalMatch:* {}{}",
    "{\\lx@activate@subfile@scope\\lx@append@path #1{#2} \\include{#3}}");
  Let!("\\inputfrom", "\\import");
  Let!("\\subinputfrom", "\\subimport");
});
