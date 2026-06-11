use latexml_package::prelude::*;

LoadDefinitions!({
  RequirePackage!("keyval");

  DefKeyVal!("foo", "path", "Semiverbatim");

  DefConstructor!(
    "\\KVsimple OptionalKeyVals:foo",
    "<ltx:graphics graphic='none' options='#1'/>"
  );

  DefConstructor!(
    "\\KVcomplex OptionalKeyVals:foo",
    "<ltx:graphics graphic='none' imagewidth='&GetKeyVal(#1,width)' imageheight='&GetKeyVal(#1,height)'/>"
  );

  DefEnvironment!(
    "{KVenv} OptionalKeyVals:foo",
    "<ltx:text width='&GetKeyVal(#1,width)' height='&GetKeyVal(#1,height)'>#body</ltx:text>"
  );

  // KVstruct: hand-written replacement closure because opening two consecutive
  // ltx:text child elements inside an environment body triggers a double-free
  // in the libxml node management. Instead, we use absorb_string for inline text.
  {
    let replacement: ReplacementClosure = Rc::new(
      |document: &mut Document, args: &Vec<Option<Digested>>, props: &SymHashMap<Stored>| {
        document.open_element("ltx:text", None, None)?;
        // ?&GetKeyVal(#1,width)(Width: &GetKeyVal(#1,width))
        if let Some(width) = GetKeyVal(&args[0], "width") {
          let w_str = width.to_string();
          if !w_str.is_empty() && w_str != "false" {
            document.absorb_string(&format!("Width: {}", w_str), props)?;
          }
        }
        // &amp; (literal ampersand)
        document.absorb_string("&", props)?;
        // ?&GetKeyVal(#1,height)(Height: &GetKeyVal(#1,height))
        if let Some(height) = GetKeyVal(&args[0], "height") {
          let h_str = height.to_string();
          if !h_str.is_empty() && h_str != "false" {
            document.absorb_string(&format!("Height: {}", h_str), props)?;
          }
        }
        // #body
        if let Some(body_stored) = props.get("body") {
          let digested_opt: Option<Digested> = body_stored.into();
          if let Some(ref digested) = digested_opt {
            document.absorb(digested, None)?;
          }
        }
        document.close_element("ltx:text")?;
        Ok(())
      },
    );
    let options = ConstructorOptions::default();
    let name = "KVstruct".to_string();
    let cs = T_CS!(s!("\\{}", &name));
    let paramlist = parse_parameters("OptionalKeyVals:foo", &cs, true)?;
    def_environment(name, paramlist, Some(replacement), options);
  }

  DefConstructor!("\\KVauto RequiredKeyVals:foo", "<ltx:text>#1</ltx:text>");
});
