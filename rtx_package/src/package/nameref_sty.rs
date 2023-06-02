use crate::package::*;
LoadDefinitions!(state, {
  // TODO:
  // RequirePackage!("refcount");
  // RequirePackage!("gettitlestring");
  // RequirePackage!("ltxcmds");

  // We want to display the actual "name" of the labeled structure (e.g. \section),
  //   which is accessible via show="title"
  //
  // TODO: The star forms prevent nested double links.
  DefConstructor!("\\nameref OptionalMatch:* Semiverbatim",
    "<ltx:ref ?#1(class='ltx_refmacro_nameref ltx_nolink')(class='ltx_refmacro_nameref')\
      show='title' labelref='#label' _force_font='true'/>",
    properties => sub[_stomach,args,_state] {
      let label_arg = args[1].as_ref().map(ToString::to_string).unwrap_or_default();
      Ok(stored_map!(
        "label" => clean_label(&label_arg, None)))
    });

  DefMacro!("\\Nameref", "\\nameref");   //\def\Nameref#1{‘\nameref{#1}’ on page~\pageref{#1}}
  DefMacro!("\\Sectionformat{}{}", "#1");
  DefMacro!("\\Ref", "\\ref");            // can be improved if "varioref.sty" is loaded?
  //The original nameref docs say: "Overload an AMS LaTEX command, which uses \newlabel. Sigh!"
  DefMacro!("\\slabel", "\\label");
  // We can improve if we had \vpageref
  DefMacro!("\\vnameref", "\\nameref");

});
