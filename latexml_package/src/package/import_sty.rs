use crate::prelude::*;
use latexml_core::util::pathname;

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
    let paths = state::get_search_paths();
    state::push_value("lx@searchpaths@stack",
      latexml_core::common::store::Stored::Strings(
        std::rc::Rc::from(paths.iter().map(|s| latexml_core::common::arena::pin(s)).collect::<Vec<_>>())))?;
  });

  DefPrimitive!("\\lx@restore@paths", {
    if let Ok(Some(latexml_core::common::store::Stored::Strings(syms))) =
      state::pop_value("lx@searchpaths@stack") {
      let paths: Vec<String> = syms.iter()
        .map(|s| latexml_core::common::arena::with(*s, |t| t.to_string()))
        .collect();
      state::set_search_paths(paths);
    }
  });

  // Perl import.sty.ltxml L20-29: \lx@set@path OptionalMatch:* {}
  //   path = ToString(Expand(#2)); if relative, resolve vs SOURCEDIRECTORY.
  //   If * → replace SEARCHPATHS with [canonical(path)]
  //   else → prepend canonical(path) to existing SEARCHPATHS.
  DefPrimitive!("\\lx@set@path OptionalMatch:* {}", sub[(star, path_tks)] {
    let raw = Expand!(path_tks.clone()).to_string();
    let mut path = raw.trim().to_string();
    if path.is_empty() { return Ok(Vec::new()); }
    if !pathname::is_absolute(&path) {
      let source_dir = state::lookup_string("SOURCEDIRECTORY").to_string();
      if !source_dir.is_empty() {
        path = pathname::concat(&source_dir, &path);
      }
    }
    let canonical = pathname::canonical(&path);
    if star.is_some() {
      state::set_search_paths(vec![canonical]);
    } else {
      let mut new_paths = vec![canonical];
      new_paths.extend(state::get_search_paths());
      state::set_search_paths(new_paths);
    }
  });

  // Perl import.sty.ltxml L31-42: \lx@append@path OptionalMatch:* {}
  //   If SEARCHPATHS has entries, concat the first with path:
  //   new_lead = concat(lead_path, path); star → [new_lead], else → [new_lead, ...rest].
  //   If SEARCHPATHS is empty, this is a no-op (matches Perl's early-return).
  DefPrimitive!("\\lx@append@path OptionalMatch:* {}", sub[(star, path_tks)] {
    let raw = Expand!(path_tks.clone()).to_string();
    let path = raw.trim().to_string();
    if path.is_empty() { return Ok(Vec::new()); }
    let mut paths = state::get_search_paths();
    if paths.is_empty() { return Ok(Vec::new()); }
    let lead = paths.remove(0);
    let new_lead = pathname::concat(&lead, &path);
    if star.is_some() {
      state::set_search_paths(vec![new_lead]);
    } else {
      let mut new_paths = vec![new_lead];
      new_paths.extend(paths);
      state::set_search_paths(new_paths);
    }
  });

  // Wrap the input call in `\lx@save@paths … \lx@restore@paths` so each
  // \import / \subimport starts from the BASE search_paths (matching
  // Perl's AssignValue local-scoping). Without this, two consecutive
  // sibling `\subimport{Chapter/}{Abstract}` + `\subimport{Chapter/}{Poster}`
  // would concat Chapter/ onto the still-mutated lead from the first
  // call, producing "Chapter/Chapter/Poster" as the search target.
  DefMacro!("\\import OptionalMatch:* {}{}",
    "{\\lx@save@paths\\lx@set@path #1{#2} \\input{#3}\\lx@restore@paths}");
  DefMacro!("\\includefrom OptionalMatch:* {}",
    "{\\lx@save@paths\\lx@set@path #1{#2} \\include{#3}\\lx@restore@paths}");
  DefMacro!("\\subimport OptionalMatch:* {}{}",
    "{\\lx@save@paths\\lx@append@path #1{#2} \\input{#3}\\lx@restore@paths}");
  DefMacro!("\\subincludefrom OptionalMatch:* {}",
    "{\\lx@save@paths\\lx@append@path #1{#2} \\include{#3}\\lx@restore@paths}");
  Let!("\\inputfrom", "\\import");
  Let!("\\subinputfrom", "\\subimport");
});
