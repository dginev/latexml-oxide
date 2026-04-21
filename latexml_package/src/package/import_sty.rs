use crate::prelude::*;
use latexml_core::util::pathname;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: import.sty.ltxml

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

  DefMacro!("\\import OptionalMatch:* {}{}", "{\\lx@set@path #1{#2} \\input{#3}}");
  DefMacro!("\\includefrom OptionalMatch:* {}", "{\\lx@set@path #1{#2} \\include{#3}}");
  DefMacro!("\\subimport OptionalMatch:* {}{}", "{\\lx@append@path #1{#2} \\input{#3}}");
  DefMacro!("\\subincludefrom OptionalMatch:* {}", "{\\lx@append@path #1{#2} \\include{#3}}");
  Let!("\\inputfrom", "\\import");
  Let!("\\subinputfrom", "\\subimport");
});
