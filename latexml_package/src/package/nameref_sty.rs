use crate::prelude::*;
LoadDefinitions!({
  // Perl: loads these packages
  RequirePackage!("refcount");
  RequirePackage!("gettitlestring");
  RequirePackage!("ltxcmds");

  // We want to display the actual "name" of the labeled structure (e.g. \section),
  //   which is accessible via show="title"
  //
  // TODO: The star forms prevent nested double links.
  // Perl L28 passes `enterHorizontal => 1` so \nameref in vertical mode
  // transitions into horizontal like \ref does. Rust was missing it, so
  // a bare `\nameref{…}` at the start of a cell or a potential vmode
  // position could skip the mode flip.
  DefConstructor!("\\nameref OptionalMatch:* Semiverbatim",
  "<ltx:ref ?#1(class='ltx_refmacro_nameref ltx_nolink')(class='ltx_refmacro_nameref')\
    show='title' labelref='#label' _force_font='true'/>",
  enter_horizontal => true,
  properties => sub[args] {
    let label_arg = args[1].as_ref().map(ToString::to_string).unwrap_or_default();
    Ok(stored_map!(
      "label" => clean_label(&label_arg, None)))
  });

  DefMacro!("\\Nameref", "\\nameref"); //\def\Nameref#1{‘\nameref{#1}’ on page~\pageref{#1}}
  DefMacro!("\\Sectionformat{}{}", "#1");
  DefMacro!("\\Ref", "\\ref"); // can be improved if "varioref.sty" is loaded?
  //The original nameref docs say: "Overload an AMS LaTEX command, which uses \newlabel. Sigh!"
  DefMacro!("\\slabel", "\\label");
  // We can improve if we had \vpageref
  DefMacro!("\\vnameref", "\\nameref");
  // nameref.sty:189-192, defined unconditionally: the title-capture hook. The
  // binding (Perl nameref.sty.ltxml too) reimplements `\nameref` and never
  // defined it, but memoir.cls:7020-7026 sets `\NR@nopatch@sectioning`,
  // requires nameref, and routes its own `\M@gettitle` (heads, `\PoemTitle`,
  // memoir.cls:3079/3147/3754/5376) through `\NR@gettitle` — srbook-mem ×3,
  // serbian-apostrophe ×2 had it as their sole error. Guard:
  // `perfect_kernel_batch54::nameref_gettitle_records_the_title`.
  DefMacro!(
    "\\NR@gettitle{}",
    "\\GetTitleString{#1}\\let\\@currentlabelname\\GetTitleStringResult"
  );
  RawTeX!(r"\providecommand*{\@currentlabelname}{}");
});
