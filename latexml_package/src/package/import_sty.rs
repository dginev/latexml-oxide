use latexml_core::util::pathname;

use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: import.sty.ltxml — uses `AssignValue(SEARCHPATHS => …)` which
  // auto-scopes via the frame stack. Rust's `state::set_search_paths`
  // mutates the global VecDeque without frame scoping. To recover the
  // "local-by-default" semantic, save/restore search_paths around each
  // `\import` / `\subimport` body via a state-stored stack. Witnesses:
  // arXiv:2604.09744, 2603.04457 (papers with multiple sibling
  // `\subimport{Chapter/}{File}` calls — second call would otherwise
  // concat Chapter/ onto Chapter/ from the first call).

  DefPrimitive!("\\lx@save@paths", {
    // OXIDIZED_DESIGN #65 (#311): the `{…}` below is a LaTeXML artifact — the
    // real import.sty never groups the input (`\@import` restores
    // `\input@path`/`\Ginput@path` by plain `\def` AFTER the `\input`, at the
    // caller's level; "input files must have balanced grouping", L42). Name the
    // region so `require_package` gives a package loaded in there the
    // outermost-level lifetime real LaTeX would. Every caller invokes this
    // immediately after `{`, and `activate_scope`'s marker is `Scope::Local`, so
    // the region ends with that group.
    activate_scope(pin_static(SUBFILE_SCOPE));
    let paths = get_search_paths();
    push_value("lx@searchpaths@stack",
      Stored::Strings(
        Rc::from(paths.iter().map(pin).collect::<Vec<_>>())))?;
  });

  DefPrimitive!("\\lx@restore@paths", {
    if let Ok(Some(Stored::Strings(syms))) =
      pop_value("lx@searchpaths@stack") {
      let paths: Vec<String> = syms.iter()
        .map(|s| with(*s, |t| t.to_string()))
        .collect();
      set_search_paths(paths);
    }
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
    if star.is_some() {
      set_search_paths(vec![canonical]);
    } else {
      let mut new_paths = vec![canonical];
      new_paths.extend(get_search_paths());
      set_search_paths(new_paths);
    }
  });

  // Perl import.sty.ltxml L31-42: \lx@append@path OptionalMatch:* {}
  //   If SEARCHPATHS has entries, concat the first with path:
  //   new_lead = concat(lead_path, path); star → [new_lead], else → [new_lead, ...rest].
  //   If SEARCHPATHS is empty, this is a no-op (matches Perl's early-return).
  DefPrimitive!("\\lx@append@path OptionalMatch:* {}", sub[(star, path_tks)] {
    let raw = Expand!(path_tks).to_string();
    let path = raw.trim().to_string();
    if path.is_empty() { return Ok(Vec::new()); }
    let mut paths = get_search_paths();
    if paths.is_empty() { return Ok(Vec::new()); }
    let lead = paths.remove(0);
    let new_lead = pathname::concat(&lead, &path);
    if star.is_some() {
      set_search_paths(vec![new_lead]);
    } else {
      let mut new_paths = vec![new_lead];
      new_paths.extend(paths);
      set_search_paths(new_paths);
    }
  });

  // Wrap the input call in `\lx@save@paths … \lx@restore@paths` so each
  // \import / \subimport starts from the BASE search_paths (matching
  // Perl's AssignValue local-scoping). Without this, two consecutive
  // sibling `\subimport{Chapter/}{Abstract}` + `\subimport{Chapter/}{Poster}`
  // would concat Chapter/ onto the still-mutated lead from the first
  // call, producing "Chapter/Chapter/Poster" as the search target.
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
    "{\\lx@save@paths\\lx@set@path #1{#2} \\input{#3}\\lx@restore@paths}");
  DefMacro!("\\includefrom OptionalMatch:* {}{}",
    "{\\lx@save@paths\\lx@set@path #1{#2} \\include{#3}\\lx@restore@paths}");
  DefMacro!("\\subimport OptionalMatch:* {}{}",
    "{\\lx@save@paths\\lx@append@path #1{#2} \\input{#3}\\lx@restore@paths}");
  DefMacro!("\\subincludefrom OptionalMatch:* {}{}",
    "{\\lx@save@paths\\lx@append@path #1{#2} \\include{#3}\\lx@restore@paths}");
  Let!("\\inputfrom", "\\import");
  Let!("\\subinputfrom", "\\subimport");
});
