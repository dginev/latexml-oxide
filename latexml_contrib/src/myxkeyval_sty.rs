use latexml_core::keyval::{self, KeyvalConfig};
use latexml_package::prelude::*;

LoadDefinitions!({
  RequirePackage!("xkeyval");

  // Define keys directly (matching Perl .ltxml DefKeyVal calls)
  keyval::define(KeyvalConfig {
    prefix: "myxkeyval",
    keyset: "scenario",
    key: "role",
    vtype: "",
    ..KeyvalConfig::default()
  })?;

  keyval::define(KeyvalConfig {
    prefix: "myxkeyval",
    keyset: "scenario",
    key: "country",
    vtype: "",
    kind: Some("command"),
    ..KeyvalConfig::default()
  })?;

  keyval::define(KeyvalConfig {
    prefix: "myxkeyval",
    keyset: "scenario",
    key: "color",
    vtype: "",
    kind: Some("choice"),
    choices: vec!["red", "yellow", "green"],
    ..KeyvalConfig::default()
  })?;

  keyval::define(KeyvalConfig {
    prefix: "myxkeyval",
    keyset: "scenario",
    key: "cross",
    vtype: "",
    kind: Some("boolean"),
    ..KeyvalConfig::default()
  })?;

  // \scenario RequiredKeyVals:myxkeyval|scenario
  {
    let replacement: ReplacementClosure = Rc::new(
      |document: &mut Document, args: &Vec<Option<Digested>>, _props: &SymHashMap<Stored>| {
        let mut para_attrs = HashMap::default();
        para_attrs.insert("class".to_string(), "scenario".to_string());
        document.open_element("ltx:para", Some(para_attrs), None)?;

        let get_kv =
          |key: &str| -> Option<String> { GetKeyVal(&args[0], key).map(|d| d.to_string()) };

        // role class uses "cross" value (matching Perl .ltxml)
        let mut a = HashMap::default();
        a.insert("class".to_string(), "role".to_string());
        document.open_element("ltx:text", Some(a), None)?;
        if let Some(val) = get_kv("cross") {
          document.absorb_string(&val, &SymHashMap::default())?;
        }
        document.close_element("ltx:text")?;

        let mut a = HashMap::default();
        a.insert("class".to_string(), "country".to_string());
        document.open_element("ltx:text", Some(a), None)?;
        if let Some(val) = get_kv("country") {
          document.absorb_string(&val, &SymHashMap::default())?;
        }
        document.close_element("ltx:text")?;

        let mut a = HashMap::default();
        a.insert("class".to_string(), "color".to_string());
        document.open_element("ltx:text", Some(a), None)?;
        if let Some(val) = get_kv("color") {
          document.absorb_string(&val, &SymHashMap::default())?;
        }
        document.close_element("ltx:text")?;

        let mut a = HashMap::default();
        a.insert("class".to_string(), "cross".to_string());
        document.open_element("ltx:text", Some(a), None)?;
        if let Some(val) = get_kv("cross") {
          document.absorb_string(&val, &SymHashMap::default())?;
        }
        document.close_element("ltx:text")?;

        document.close_element("ltx:para")?;
        Ok(())
      },
    );
    let cs = T_CS!("\\scenario");
    let paramlist = parse_parameters("RequiredKeyVals:myxkeyval|scenario", &cs, true)?;
    def_constructor(
      cs,
      paramlist,
      Some(replacement),
      ConstructorOptions::default(),
    );
  }
});
