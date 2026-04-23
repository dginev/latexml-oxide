use latexml_package::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl ar5iv-bindings/fontawesome.sty.ltxml L16: RequireResource for the
  // FontAwesome 4 CDN stylesheet. Rust's RequireResource! takes a local
  // resource name only; the CDN URL is a separate post-processing concern
  // (LATEXML CSS resource handling). Skipped for now — the essential
  // feature is making `\faicon` / `\faXxx` CSes resolve so papers using
  // fontawesome don't hit undefined-CS cascades.

  // Perl L18: \faicon[opts]{name} — inline icon by name.
  DefConstructor!("\\faicon[]{}",
    "<ltx:inline-block aria:hidden='true' class='fa fa-#2'></ltx:inline-block>");

  // Perl L19-27: \faIconFromMacro{name} — lowercase + kebab-ify the CamelCase
  // macro suffix. e.g. faAlignLeft → fa-align-left. Used as the shared target
  // of all the \faXxx convenience macros.
  DefConstructor!("\\faIconFromMacro{}",
    "<ltx:inline-block aria:hidden='true' class='fa #name'></ltx:inline-block>",
    properties => sub[args] {
      let name = args[0].as_ref().map(|a| a.to_string()).unwrap_or_default();
      // Perl L24: s/([A-Z])/-$1/g then lc. Hyphenate before each uppercase
      // letter, then lowercase.
      let mut kebab = String::with_capacity(name.len() + 8);
      for c in name.chars() {
        if c.is_ascii_uppercase() {
          kebab.push('-');
          kebab.push(c.to_ascii_lowercase());
        } else {
          kebab.push(c);
        }
      }
      Ok(stored_map!("name" => kebab))
    });

  // BLOCKER (deferred): Perl L35-761 defines ~350 convenience macros of the
  // form `\faAdjust → \faIconFromMacro{faAdjust}`. Pure mechanical wrappers
  // — porting all is a large line-count but each is trivial. Deferred so
  // this cycle lands the infrastructure; follow-up can batch-add the
  // aliases. Papers that use bare `\faicon{adjust}` work today; papers
  // that use `\faAdjust` still hit undefined-CS until the aliases land.
});
