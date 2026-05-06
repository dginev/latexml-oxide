// glossaries — package binding for the LaTeX `glossaries` package.
//
// TODO(strict-perl-parity): Migrate this binding to a strict translation
// of `glossaries.sty.ltxml` (~127 lines). The Perl shim is built around
// `InputDefinitions('glossaries', type => 'sty', noltxml => 1)`, which
// raw-loads the actual TL `glossaries.sty` (8702 lines) and only
// overrides:
//   * `\@gls@link` — wrap typesetting output in `<ltx:glossaryref>`
//   * `\glsdohyperlink` / `\glsdonohyperlink` — drop hyperref wrapping
//   * `\glsdisablehyper` — disable hyperref pipeline
//   * `\glspostlinkhook` — `\xspace`
//   * `\@newglossaryentryposthook` — feed entry data to `\lx@glossaries@newentry{}{}
//     RequiredKeyVals`
//   * `\printglossary` / `\printnoidxglossary` — emit `<ltx:glossary>`
//
// The current Rust port hand-rolls `\newglossaryentry`,
// `\longnewglossaryentry`, `\newacronym`, `\gls`, `\Gls`, `\glspl`,
// `\Glspl`, `\glssymbol`, `\printglossary`, etc., plus stubs for the
// `\<gls|Gls>entry<field>` family and the `\acr*` family. This is
// because `glossaries.sty` uses heavy expl3 / datatools that the
// Rust raw-load pipeline currently can't ingest cleanly. Once the
// Rust translation is good enough to raw-load `glossaries.sty`,
// drop all the homegrown reimplementations and replace this file
// with a near line-for-line port of `glossaries.sty.ltxml`.

use crate::prelude::*;

// Helper: store a glossary entry field in state
fn glo_store(label: &str, field: &str, value: &str) {
  if !value.is_empty() {
    state::assign_value(
      &s!("glo@{label}@{field}"),
      Stored::String(arena::pin(value)),
      Some(Scope::Global),
    );
  }
}

// Helper: look up a glossary entry field from state
fn glo_lookup(label: &str, field: &str) -> String {
  state::lookup_value(&s!("glo@{label}@{field}"))
    .map(|s| s.to_string())
    .unwrap_or_default()
}

// Helper: capitalize first letter of a string
fn capitalize_first(s: &str) -> String {
  let mut chars = s.chars();
  match chars.next() {
    None => String::new(),
    Some(c) => c.to_uppercase().to_string() + chars.as_str(),
  }
}

// Helper: store a full glossary entry from extracted key-value pairs
// and register the glossary type. Returns the sorted list of (role, value) pairs
// for XML emission.
fn store_glossary_entry(
  label: &str,
  entry_type: &str,
  fields: &[(&str, String)],
) -> Vec<(String, String)> {
  // Store the type
  glo_store(label, "type", entry_type);

  // Register this glossary type
  let types_key = "glossary_types";
  let mut types: Vec<String> = state::lookup_value(types_key)
    .map(|s| {
      s.to_string()
        .split(',')
        .filter(|s| !s.is_empty())
        .map(String::from)
        .collect()
    })
    .unwrap_or_default();
  if !types.contains(&entry_type.to_string()) {
    types.push(entry_type.to_string());
    state::assign_value(
      types_key,
      Stored::String(arena::pin(types.join(","))),
      Some(Scope::Global),
    );
  }

  // Store all fields and collect non-empty ones for XML emission
  let mut phrases: Vec<(String, String)> = Vec::new();
  for (field, value) in fields {
    if !value.is_empty() {
      glo_store(label, field, value);
      phrases.push((field.to_string(), value.clone()));
    }
  }

  // Sort by role name (matching Perl's `sort keys %$hash`)
  phrases.sort_by(|a, b| a.0.cmp(&b.0));
  phrases
}

// Helper: build <glossarydefinition> XML with <glossaryphrase> children
fn build_glossary_definition(
  document: &mut Document,
  label: &str,
  entry_type: &str,
  phrases: &[(String, String)],
) -> Result<()> {
  let mut attrs = HashMap::default();
  attrs.insert("inlist".into(), entry_type.to_string());
  attrs.insert("key".into(), label.to_string());
  document.open_element("ltx:glossarydefinition", Some(attrs), None)?;

  for (role, value) in phrases {
    let mut phrase_attrs = HashMap::default();
    phrase_attrs.insert("key".into(), label.to_string());
    phrase_attrs.insert("role".into(), role.clone());
    document.open_element("ltx:glossaryphrase", Some(phrase_attrs), None)?;
    document.absorb_string(value, &Default::default())?;
    document.close_element("ltx:glossaryphrase")?;
  }

  document.close_element("ltx:glossarydefinition")?;
  Ok(())
}

// Helper: build tokens for \lx@glossaryref{list}{key}{text}
fn gls_ref_tokens(list: &str, key: &str, text: &str) -> Tokens {
  let mut toks = vec![T_CS!("\\lx@glossaryref")];
  toks.push(T_BEGIN!());
  toks.extend(ExplodeText!(list));
  toks.push(T_END!());
  toks.push(T_BEGIN!());
  toks.extend(ExplodeText!(key));
  toks.push(T_END!());
  toks.push(T_BEGIN!());
  toks.extend(ExplodeText!(text));
  toks.push(T_END!());
  Tokens::new(toks)
}

// Determine the display text for \gls{key}
fn gls_text(key: &str) -> String {
  let entry_type = glo_lookup(key, "type");
  let is_acronym = entry_type == "acronym";
  let used_key = s!("glo@{key}@used");
  let used = state::lookup_value(&used_key)
    .map(|s| matches!(s, Stored::Bool(true)))
    .unwrap_or(false);

  let text = if is_acronym && !used {
    // First use of acronym with long-short style: "long (short)"
    let long = glo_lookup(key, "long");
    let short = glo_lookup(key, "short");
    if !long.is_empty() && !short.is_empty() {
      s!("{long} ({short})")
    } else if !short.is_empty() {
      short
    } else {
      glo_lookup(key, "name")
    }
  } else if is_acronym {
    // Subsequent use: short form
    let short = glo_lookup(key, "short");
    if !short.is_empty() {
      short
    } else {
      glo_lookup(key, "name")
    }
  } else {
    // Regular entry: text or name
    let text = glo_lookup(key, "text");
    if !text.is_empty() {
      text
    } else {
      glo_lookup(key, "name")
    }
  };

  // Mark as used
  state::assign_value(&used_key, Stored::Bool(true), Some(Scope::Global));
  text
}

// Determine the display text for \glspl{key} (plural)
fn gls_plural_text(key: &str) -> String {
  let entry_type = glo_lookup(key, "type");
  let is_acronym = entry_type == "acronym";
  let used_key = s!("glo@{key}@used");
  let used = state::lookup_value(&used_key)
    .map(|s| matches!(s, Stored::Bool(true)))
    .unwrap_or(false);

  let text = if is_acronym && !used {
    // First use of acronym plural: "longs (shorts)"
    let longpl = glo_lookup(key, "longplural");
    let shortpl = glo_lookup(key, "shortplural");
    if !longpl.is_empty() && !shortpl.is_empty() {
      s!("{longpl} ({shortpl})")
    } else {
      let plural = glo_lookup(key, "plural");
      if !plural.is_empty() {
        plural
      } else {
        glo_lookup(key, "name") + "s"
      }
    }
  } else if is_acronym {
    let shortpl = glo_lookup(key, "shortplural");
    if !shortpl.is_empty() {
      shortpl
    } else {
      glo_lookup(key, "short") + "s"
    }
  } else {
    let plural = glo_lookup(key, "plural");
    if !plural.is_empty() {
      plural
    } else {
      let text = glo_lookup(key, "text");
      if !text.is_empty() {
        text + "s"
      } else {
        glo_lookup(key, "name") + "s"
      }
    }
  };

  state::assign_value(&used_key, Stored::Bool(true), Some(Scope::Global));
  text
}

LoadDefinitions!({
  // Perl glossaries.sty.ltxml L19: RequirePackage('xspace').
  // \glspostlinkhook (L44) expands to \xspace, and many acronym-first-use
  // paths invoke \xspace, so the package must be loaded up front.
  RequirePackage!("xspace");

  // Mirror raw glossaries.sty's transitive dependency on amsmath.
  // Perl's binding raw-loads the actual glossaries.sty (via
  // `InputDefinitions('glossaries', type => 'sty', noltxml => 1)`),
  // which `\RequirePackage{datatool-base}`, which
  // `\RequirePackage{amsmath}`. Our hand-rolled binding skips the
  // raw-load (see file header), so the chain is broken and any paper
  // that uses glossaries plus an amsmath-defined CS like
  // \DeclareMathOperator without an explicit \usepackage{amsmath}
  // hits "Error:undefined:\DeclareMathOperator" cascading through
  // every operator the paper declares (e.g. canvas paper 2303.16633:
  // 15 such errors, 0 in Perl).
  RequirePackage!("amsmath");

  // ======================================================================
  // Options
  // ======================================================================
  DeclareOption!("acronyms", "");
  DeclareOption!("toc", "");
  DeclareOption!("section", "");
  DeclareOption!("numberedsection", "");
  DeclareOption!("nonumberlist", "");
  DeclareOption!("nopostdot", "");
  DeclareOption!("nomain", "");
  DeclareOption!("style", "");
  ProcessOptions!();

  // When "acronyms" option is loaded, register the "acronym" glossary type.
  // The "main" type is always registered.
  {
    state::assign_value(
      "glossary_types",
      Stored::String(arena::pin("main")),
      Some(Scope::Global),
    );
    // Check if acronyms option was given
    let raw_options = state::lookup_value("package_options:glossaries")
      .map(|s| s.to_string())
      .unwrap_or_default();
    if raw_options.contains("acronyms") {
      state::assign_value(
        "glossary_types",
        Stored::String(arena::pin("main,acronym")),
        Some(Scope::Global),
      );
    }
  }

  // ======================================================================
  // Setup macros (no-ops and stubs)
  // ======================================================================
  DefMacro!("\\makenoidxglossaries", "");
  DefMacro!("\\makeglossaries", "");
  DefMacro!("\\glsnoidxstripaccents", "");
  // glossaries.sty `\glsenableentrycount` enables per-entry usage counting;
  // the `\gls` family then routes through `\cgls` etc. to record usage.
  // Rust's stub of `\gls` doesn't track usage, so this is a no-op — but it
  // must be defined or 2309.05205 (and any paper using glossaries v4+ entry
  // counting) hits an undefined-CS error.
  DefMacro!("\\glsenableentrycount", "");
  DefMacro!("\\setacronymstyle{}", "");
  DefMacro!("\\glsdisablehyper", "");
  DefMacro!("\\glsdohyperlink{}{}", "#2");
  DefMacro!("\\glsdonohyperlink{}{}", "#2");
  DefMacro!("\\glspostlinkhook", "");
  DefMacro!("\\glsaddall OptionalKeyVals", "");
  DefMacro!("\\glsadd OptionalKeyVals Semiverbatim", "");
  DefMacro!("\\newglossary OptionalMatch:* {}{}{}{}", "");
  DefMacro!("\\glslink{}{}", "#2");
  // glossaries.sty defines a `\<gls|Gls>entry<field>` family for read-only
  // access to entry fields (used outside `\gls{}` typesetting context, e.g.
  // in section headings). The capitalized `\Gls*` variants pipe the result
  // through `\makefirstuc`. Perl's glossaries.sty.ltxml gets these by
  // raw-loading the actual TL `glossaries.sty` (`InputDefinitions(noltxml=1)`,
  // L18). Rust's port stubs them as no-ops to preserve the same CS coverage —
  // the contents would otherwise expand the entry hash, but the typesetting
  // path uses `\gls`/`\Gls` (which Rust reimplements above), so dropping the
  // expansion here is harmless. Driver: 1806.05262 calls `\Glsentrytext{nbs}`
  // in a section heading.
  DefMacro!("\\glsentrytext Semiverbatim", "");
  DefMacro!("\\Glsentrytext Semiverbatim", "");
  DefMacro!("\\glsentrylong Semiverbatim", "");
  DefMacro!("\\Glsentrylong Semiverbatim", "");
  DefMacro!("\\glsentryshort Semiverbatim", "");
  DefMacro!("\\Glsentryshort Semiverbatim", "");
  DefMacro!("\\glsentryname Semiverbatim", "");
  DefMacro!("\\Glsentryname Semiverbatim", "");
  DefMacro!("\\glsentrydesc Semiverbatim", "");
  DefMacro!("\\Glsentrydesc Semiverbatim", "");
  DefMacro!("\\glsentrysymbol Semiverbatim", "");
  DefMacro!("\\Glsentrysymbol Semiverbatim", "");
  DefMacro!("\\glsentryfirst Semiverbatim", "");
  DefMacro!("\\Glsentryfirst Semiverbatim", "");
  DefMacro!("\\glsentryplural Semiverbatim", "");
  DefMacro!("\\Glsentryplural Semiverbatim", "");
  DefMacro!("\\glsentryfirstplural Semiverbatim", "");
  DefMacro!("\\Glsentryfirstplural Semiverbatim", "");
  DefMacro!("\\glsentryshortpl Semiverbatim", "");
  DefMacro!("\\Glsentryshortpl Semiverbatim", "");
  DefMacro!("\\glsentrylongpl Semiverbatim", "");
  DefMacro!("\\Glsentrylongpl Semiverbatim", "");
  DefMacro!("\\glsentryfull Semiverbatim", "");
  DefMacro!("\\Glsentryfull Semiverbatim", "");
  DefMacro!("\\glsentryfullpl Semiverbatim", "");
  DefMacro!("\\Glsentryfullpl Semiverbatim", "");
  // \acr* family — the real glossaries.sty defines short/long/full plus
  // their `pl` (plural) and uppercase-first (`\Acr*`) variants. Perl's
  // glossaries.sty.ltxml gets these via `InputDefinitions(noltxml=1)`
  // raw-load of the actual TL glossaries.sty source. Rust's port stubs
  // them here as no-ops to mirror the same set of bound CSes
  // (driver paper: arXiv:1801.10219 invokes `\acrfullpl`).
  DefMacro!("\\acrshort Semiverbatim", "");
  DefMacro!("\\acrshortpl Semiverbatim", "");
  DefMacro!("\\Acrshort Semiverbatim", "");
  DefMacro!("\\Acrshortpl Semiverbatim", "");
  DefMacro!("\\acrlong Semiverbatim", "");
  DefMacro!("\\acrlongpl Semiverbatim", "");
  DefMacro!("\\Acrlong Semiverbatim", "");
  DefMacro!("\\Acrlongpl Semiverbatim", "");
  DefMacro!("\\acrfull Semiverbatim", "");
  DefMacro!("\\acrfullpl Semiverbatim", "");
  DefMacro!("\\Acrfull Semiverbatim", "");
  DefMacro!("\\Acrfullpl Semiverbatim", "");

  // \glsresetall[<glossaries>] — resets the "first use" flag for all
  // entries. We don't track first-use state, so it's a safe no-op.
  // Mirrors TL glossaries.sty L3370 `\newcommand*{\glsresetall}[1][...]`.
  DefMacro!("\\glsresetall []", "");
  DefMacro!("\\glsresetempty []", "");
  // \loadglsentries[<gls-type>]{<file>} — TL glossaries.sty L3543 expands
  // to `\input{#2}`. We stub it as a no-op rather than `\input`-ing the
  // entries file: Perl LaTeXML's glossaries.sty.ltxml uses `InputDefinitions
  // (noltxml=1)` to raw-load the actual TL `.sty` and override only the
  // `\@newglossaryentryposthook` (which then calls
  // `\lx@glossaries@newentry{}{} RequiredKeyVals` with already-flat tokens).
  // Rust's binding hand-rolls `\newglossaryentry{} RequiredKeyVals`, so it
  // can't accept the user-source's `}\n{` whitespace between args that the
  // raw TeX `\def\newglossaryentry#1#2{...}` happily skips. Until the Rust
  // binding is refactored to follow Perl's raw-load+hook pattern,
  // `\loadglsentries` is a no-op — sufficient for the common case where
  // `\acrshort{label}` etc. don't depend on the entry being pre-defined.
  // Driver paper: arXiv:1806.05262 (`\loadglsentries{definitions}` →
  // 2 errors → 0 errors).
  DefMacro!("\\loadglsentries []{}", "");

  // glossaries-internal macros that might be called
  DefMacro!("\\warn@noprintglossary", "");
  // glossary title macros
  DefMacro!("\\glossaryname", "Glossary");
  DefMacro!("\\acronymname", "Acronyms");

  // ======================================================================
  // \lx@glossaryref{list}{key}{text}
  // The constructor that wraps glossary references in <ltx:glossaryref>
  // ======================================================================
  DefConstructor!(
    "\\lx@glossaryref{}{}{}",
    "<ltx:glossaryref inlist='#1' key='#2'>#3</ltx:glossaryref>",
    enter_horizontal => true
  );

  // ======================================================================
  // \newglossaryentry{label} RequiredKeyVals
  // ======================================================================
  DefConstructor!("\\newglossaryentry{} RequiredKeyVals",
    sub [document, args] {
      let label = args[0].as_ref().map(|d| d.to_string()).unwrap_or_default();
      let entry_type = glo_lookup(&label, "type");
      let entry_type = if entry_type.is_empty() { "main".to_string() } else { entry_type };

      // Collect the phrases that were stored during after_digest
      let phrases_key = s!("glo@{label}@_phrases");
      let phrases_str = state::lookup_value(&phrases_key)
        .map(|s| s.to_string())
        .unwrap_or_default();
      let phrases: Vec<(String, String)> = if phrases_str.is_empty() {
        Vec::new()
      } else {
        phrases_str
          .split('\x1F') // unit separator
          .filter(|s| !s.is_empty())
          .filter_map(|pair| {
            let mut parts = pair.splitn(2, '\x1E'); // record separator
            let role = parts.next()?;
            let value = parts.next().unwrap_or("");
            if value.is_empty() { None } else { Some((role.to_string(), value.to_string())) }
          })
          .collect()
      };

      build_glossary_definition(document, &label, &entry_type, &phrases)?;
    },
    after_digest => sub[whatsit] {
      let label = whatsit.get_arg(1).map(|d| d.to_string()).unwrap_or_default();

      // Extract key-value pairs from the RequiredKeyVals argument
      let mut fields: Vec<(&str, String)> = Vec::new();
      if let Some(kv_arg) = whatsit.get_arg(2) {
        if let DigestedData::KeyVals(ref kv) = kv_arg.data() {
          let hash = kv.get_hash_digested();
          // Extract known fields
          for field in &["name", "description", "text", "plural", "first", "firstplural",
                        "sort", "symbol", "symbolplural", "counter", "see", "parent",
                        "prefix", "short", "shortplural", "long", "longplural"] {
            if let Some(value) = hash.get(*field) {
              if !value.is_empty() {
                fields.push((field, value.clone()));
              }
            }
          }
        }
      }

      // Compute sort default: sort defaults to name if not provided
      let has_sort = fields.iter().any(|(f, _)| *f == "sort");
      if !has_sort {
        if let Some(name) = fields.iter().find(|(f, _)| *f == "name").map(|(_, v)| v.clone()) {
          fields.push(("sort", name));
        }
      }

      // Determine entry type (default: main)
      let entry_type = glo_lookup(&label, "type");
      let entry_type = if entry_type.is_empty() { "main" } else { &entry_type };

      // Store all fields and get sorted phrases for XML
      let phrases = store_glossary_entry(&label, entry_type, &fields);

      // Serialize phrases for the constructor to read back
      let phrases_str: String = phrases
        .iter()
        .map(|(role, value)| s!("{role}\x1E{value}"))
        .collect::<Vec<_>>()
        .join("\x1F");
      state::assign_value(
        &s!("glo@{label}@_phrases"),
        Stored::String(arena::pin(&phrases_str)),
        Some(Scope::Global),
      );

      Ok(Vec::new())
    }
  );

  // ======================================================================
  // \longnewglossaryentry{label}{kv}{description}
  // Same as \newglossaryentry but with description as a separate argument
  // ======================================================================
  DefConstructor!("\\longnewglossaryentry{} RequiredKeyVals {}",
    sub [document, args] {
      let label = args[0].as_ref().map(|d| d.to_string()).unwrap_or_default();
      let entry_type = glo_lookup(&label, "type");
      let entry_type = if entry_type.is_empty() { "main".to_string() } else { entry_type };

      let phrases_key = s!("glo@{label}@_phrases");
      let phrases_str = state::lookup_value(&phrases_key)
        .map(|s| s.to_string())
        .unwrap_or_default();
      let phrases: Vec<(String, String)> = if phrases_str.is_empty() {
        Vec::new()
      } else {
        phrases_str
          .split('\x1F')
          .filter(|s| !s.is_empty())
          .filter_map(|pair| {
            let mut parts = pair.splitn(2, '\x1E');
            let role = parts.next()?;
            let value = parts.next().unwrap_or("");
            if value.is_empty() { None } else { Some((role.to_string(), value.to_string())) }
          })
          .collect()
      };

      build_glossary_definition(document, &label, &entry_type, &phrases)?;
    },
    after_digest => sub[whatsit] {
      let label = whatsit.get_arg(1).map(|d| d.to_string()).unwrap_or_default();

      let mut fields: Vec<(&str, String)> = Vec::new();

      // Extract key-value pairs
      if let Some(kv_arg) = whatsit.get_arg(2) {
        if let DigestedData::KeyVals(ref kv) = kv_arg.data() {
          let hash = kv.get_hash_digested();
          for field in &["name", "text", "plural", "first", "firstplural",
                        "sort", "symbol", "symbolplural", "counter", "see", "parent",
                        "prefix", "short", "shortplural", "long", "longplural"] {
            if let Some(value) = hash.get(*field) {
              if !value.is_empty() {
                fields.push((field, value.clone()));
              }
            }
          }
        }
      }

      // Get description from arg 3
      let desc = whatsit.get_arg(3).map(|d| d.to_string()).unwrap_or_default();
      if !desc.is_empty() {
        fields.push(("description", desc));
      }

      // Compute sort default
      let has_sort = fields.iter().any(|(f, _)| *f == "sort");
      if !has_sort {
        if let Some(name) = fields.iter().find(|(f, _)| *f == "name").map(|(_, v)| v.clone()) {
          fields.push(("sort", name));
        }
      }

      let entry_type = glo_lookup(&label, "type");
      let entry_type = if entry_type.is_empty() { "main" } else { &entry_type };

      let phrases = store_glossary_entry(&label, entry_type, &fields);

      let phrases_str: String = phrases
        .iter()
        .map(|(role, value)| s!("{role}\x1E{value}"))
        .collect::<Vec<_>>()
        .join("\x1F");
      state::assign_value(
        &s!("glo@{label}@_phrases"),
        Stored::String(arena::pin(&phrases_str)),
        Some(Scope::Global),
      );

      Ok(Vec::new())
    }
  );

  // ======================================================================
  // \newacronym{label}{short}{long/description}
  // Defines an acronym entry
  // ======================================================================
  DefConstructor!("\\newacronym{}{}{}",
    sub [document, args] {
      let label = args[0].as_ref().map(|d| d.to_string()).unwrap_or_default();

      let phrases_key = s!("glo@{label}@_phrases");
      let phrases_str = state::lookup_value(&phrases_key)
        .map(|s| s.to_string())
        .unwrap_or_default();
      let phrases: Vec<(String, String)> = if phrases_str.is_empty() {
        Vec::new()
      } else {
        phrases_str
          .split('\x1F')
          .filter(|s| !s.is_empty())
          .filter_map(|pair| {
            let mut parts = pair.splitn(2, '\x1E');
            let role = parts.next()?;
            let value = parts.next().unwrap_or("");
            if value.is_empty() { None } else { Some((role.to_string(), value.to_string())) }
          })
          .collect()
      };

      build_glossary_definition(document, &label, "acronym", &phrases)?;
    },
    after_digest => sub[whatsit] {
      let label = whatsit.get_arg(1).map(|d| d.to_string()).unwrap_or_default();
      let short = whatsit.get_arg(2).map(|d| d.to_string()).unwrap_or_default();
      let long = whatsit.get_arg(3).map(|d| d.to_string()).unwrap_or_default();

      // Set type before storing
      glo_store(&label, "type", "acronym");

      let fields: Vec<(&str, String)> = vec![
        ("description", long.clone()),
        ("long", long.clone()),
        ("longplural", s!("{long}s")),
        ("name", short.clone()),
        ("short", short.clone()),
        ("shortplural", s!("{short}s")),
        ("sort", short.clone()),
        ("text", short.clone()),
      ];

      let phrases = store_glossary_entry(&label, "acronym", &fields);

      let phrases_str: String = phrases
        .iter()
        .map(|(role, value)| s!("{role}\x1E{value}"))
        .collect::<Vec<_>>()
        .join("\x1F");
      state::assign_value(
        &s!("glo@{label}@_phrases"),
        Stored::String(arena::pin(&phrases_str)),
        Some(Scope::Global),
      );

      Ok(Vec::new())
    }
  );

  // ======================================================================
  // \gls, \Gls, \glspl, \Glspl, \glssymbol
  // Runtime macros that expand to \lx@glossaryref{list}{key}{text}
  // ======================================================================
  {
    let gls_cs = T_CS!("\\gls");
    let gls_params = parse_parameters("Semiverbatim", &gls_cs, true)?;
    let gls_closure: ExpansionClosure = Rc::new(move |args: Vec<ArgWrap>| {
      let key = args[0].to_string();
      let entry_type = glo_lookup(&key, "type");
      let list = if entry_type.is_empty() {
        "main".to_string()
      } else {
        entry_type
      };
      let text = gls_text(&key);
      Ok(gls_ref_tokens(&list, &key, &text))
    });
    def_macro(
      gls_cs,
      gls_params,
      ExpansionBody::Closure(gls_closure),
      None,
    )?;
  }

  {
    let cs = T_CS!("\\Gls");
    let params = parse_parameters("Semiverbatim", &cs, true)?;
    let closure: ExpansionClosure = Rc::new(move |args: Vec<ArgWrap>| {
      let key = args[0].to_string();
      let entry_type = glo_lookup(&key, "type");
      let list = if entry_type.is_empty() {
        "main".to_string()
      } else {
        entry_type
      };
      let text = capitalize_first(&gls_text(&key));
      Ok(gls_ref_tokens(&list, &key, &text))
    });
    def_macro(cs, params, ExpansionBody::Closure(closure), None)?;
  }

  {
    let cs = T_CS!("\\glspl");
    let params = parse_parameters("Semiverbatim", &cs, true)?;
    let closure: ExpansionClosure = Rc::new(move |args: Vec<ArgWrap>| {
      let key = args[0].to_string();
      let entry_type = glo_lookup(&key, "type");
      let list = if entry_type.is_empty() {
        "main".to_string()
      } else {
        entry_type
      };
      let text = gls_plural_text(&key);
      Ok(gls_ref_tokens(&list, &key, &text))
    });
    def_macro(cs, params, ExpansionBody::Closure(closure), None)?;
  }

  {
    let cs = T_CS!("\\Glspl");
    let params = parse_parameters("Semiverbatim", &cs, true)?;
    let closure: ExpansionClosure = Rc::new(move |args: Vec<ArgWrap>| {
      let key = args[0].to_string();
      let entry_type = glo_lookup(&key, "type");
      let list = if entry_type.is_empty() {
        "main".to_string()
      } else {
        entry_type
      };
      let text = capitalize_first(&gls_plural_text(&key));
      Ok(gls_ref_tokens(&list, &key, &text))
    });
    def_macro(cs, params, ExpansionBody::Closure(closure), None)?;
  }

  {
    let cs = T_CS!("\\glssymbol");
    let params = parse_parameters("Semiverbatim", &cs, true)?;
    let closure: ExpansionClosure = Rc::new(move |args: Vec<ArgWrap>| {
      let key = args[0].to_string();
      let entry_type = glo_lookup(&key, "type");
      let list = if entry_type.is_empty() {
        "main".to_string()
      } else {
        entry_type
      };
      let symbol = glo_lookup(&key, "symbol");
      Ok(gls_ref_tokens(&list, &key, &symbol))
    });
    def_macro(cs, params, ExpansionBody::Closure(closure), None)?;
  }

  // \glsfirst, \Glsfirst, \glsfirstplural, \Glsfirstplural — emit the
  // entry's `first` form (long form for acronyms, falls back to `text`
  // when no separate first-use form was declared). Same wrapping as
  // \gls. Several arXiv papers (e.g. 2303.16633) use \glsfirst inside
  // their definitions; without the binding the CS hits the undefined
  // path. Perl's Bruce-binding raw-loads glossaries.sty which provides
  // these via \newrobustcmd*; we mirror the user-facing behaviour
  // (return formatted entry text + glossaryref wrapping).
  {
    let cs = T_CS!("\\glsfirst");
    let params = parse_parameters("Semiverbatim", &cs, true)?;
    let closure: ExpansionClosure = Rc::new(move |args: Vec<ArgWrap>| {
      let key = args[0].to_string();
      let entry_type = glo_lookup(&key, "type");
      let list = if entry_type.is_empty() { "main".to_string() } else { entry_type };
      let mut text = glo_lookup(&key, "first");
      if text.is_empty() {
        text = gls_text(&key);
      }
      Ok(gls_ref_tokens(&list, &key, &text))
    });
    def_macro(cs, params, ExpansionBody::Closure(closure), None)?;
  }
  {
    let cs = T_CS!("\\Glsfirst");
    let params = parse_parameters("Semiverbatim", &cs, true)?;
    let closure: ExpansionClosure = Rc::new(move |args: Vec<ArgWrap>| {
      let key = args[0].to_string();
      let entry_type = glo_lookup(&key, "type");
      let list = if entry_type.is_empty() { "main".to_string() } else { entry_type };
      let mut text = glo_lookup(&key, "first");
      if text.is_empty() {
        text = gls_text(&key);
      }
      Ok(gls_ref_tokens(&list, &key, &capitalize_first(&text)))
    });
    def_macro(cs, params, ExpansionBody::Closure(closure), None)?;
  }
  {
    let cs = T_CS!("\\glsfirstplural");
    let params = parse_parameters("Semiverbatim", &cs, true)?;
    let closure: ExpansionClosure = Rc::new(move |args: Vec<ArgWrap>| {
      let key = args[0].to_string();
      let entry_type = glo_lookup(&key, "type");
      let list = if entry_type.is_empty() { "main".to_string() } else { entry_type };
      let mut text = glo_lookup(&key, "firstplural");
      if text.is_empty() {
        text = gls_plural_text(&key);
      }
      Ok(gls_ref_tokens(&list, &key, &text))
    });
    def_macro(cs, params, ExpansionBody::Closure(closure), None)?;
  }
  {
    let cs = T_CS!("\\Glsfirstplural");
    let params = parse_parameters("Semiverbatim", &cs, true)?;
    let closure: ExpansionClosure = Rc::new(move |args: Vec<ArgWrap>| {
      let key = args[0].to_string();
      let entry_type = glo_lookup(&key, "type");
      let list = if entry_type.is_empty() { "main".to_string() } else { entry_type };
      let mut text = glo_lookup(&key, "firstplural");
      if text.is_empty() {
        text = gls_plural_text(&key);
      }
      Ok(gls_ref_tokens(&list, &key, &capitalize_first(&text)))
    });
    def_macro(cs, params, ExpansionBody::Closure(closure), None)?;
  }
  // \glsdesc / \Glsdesc — emit the entry's description.
  {
    let cs = T_CS!("\\glsdesc");
    let params = parse_parameters("Semiverbatim", &cs, true)?;
    let closure: ExpansionClosure = Rc::new(move |args: Vec<ArgWrap>| {
      let key = args[0].to_string();
      let entry_type = glo_lookup(&key, "type");
      let list = if entry_type.is_empty() { "main".to_string() } else { entry_type };
      let text = glo_lookup(&key, "description");
      Ok(gls_ref_tokens(&list, &key, &text))
    });
    def_macro(cs, params, ExpansionBody::Closure(closure), None)?;
  }
  {
    let cs = T_CS!("\\Glsdesc");
    let params = parse_parameters("Semiverbatim", &cs, true)?;
    let closure: ExpansionClosure = Rc::new(move |args: Vec<ArgWrap>| {
      let key = args[0].to_string();
      let entry_type = glo_lookup(&key, "type");
      let list = if entry_type.is_empty() { "main".to_string() } else { entry_type };
      let text = glo_lookup(&key, "description");
      Ok(gls_ref_tokens(&list, &key, &capitalize_first(&text)))
    });
    def_macro(cs, params, ExpansionBody::Closure(closure), None)?;
  }

  // ======================================================================
  // \printglossary, \printnoidxglossary, \printnoidxglossaries, \printglossaries
  // ======================================================================
  DefConstructor!("\\lx@printglossary{}{}",
    "<ltx:glossary xml:id='#1' lists='#2'><ltx:title>#title</ltx:title></ltx:glossary>",
    properties => sub[args] {
      let glo_type = args[1].as_ref().map(|d| d.to_string()).unwrap_or_else(|| "main".into());
      // Look up title macro for this type
      let title = if glo_type == "acronym" {
        "Acronyms".to_string()
      } else {
        "Glossary".to_string()
      };
      let title_digested = digest_text(Tokens::new(ExplodeText!(&title)))?;
      Ok(stored_map!("title" => title_digested))
    }
  );

  {
    // \printglossary OptionalKeyVals → expand to \lx@printglossary{id}{type}
    let cs = T_CS!("\\printglossary");
    let params = parse_parameters("OptionalKeyVals", &cs, true)?;
    let closure: ExpansionClosure = Rc::new(move |args: Vec<ArgWrap>| {
      let glo_type = if !args.is_empty() {
        let kv_str = args[0].to_string();
        // Extract type= from keyvals
        if let Some(pos) = kv_str.find("type=") {
          let rest = &kv_str[pos + 5..];
          rest.split(',').next().unwrap_or("main").trim().to_string()
        } else {
          "main".to_string()
        }
      } else {
        "main".to_string()
      };
      let docid = state::lookup_value("docid")
        .map(|s| s.to_string())
        .unwrap_or_default();
      let id = if docid.is_empty() {
        s!("glo.{glo_type}")
      } else {
        s!("{docid}.glo.{glo_type}")
      };

      let mut toks = vec![T_CS!("\\lx@printglossary")];
      toks.push(T_BEGIN!());
      toks.extend(ExplodeText!(&id));
      toks.push(T_END!());
      toks.push(T_BEGIN!());
      toks.extend(ExplodeText!(&glo_type));
      toks.push(T_END!());
      Ok(Tokens::new(toks))
    });
    def_macro(cs, params, ExpansionBody::Closure(closure), None)?;
  }

  {
    // \printnoidxglossary — same as \printglossary
    let cs = T_CS!("\\printnoidxglossary");
    let params = parse_parameters("OptionalKeyVals", &cs, true)?;
    let closure: ExpansionClosure = Rc::new(move |args: Vec<ArgWrap>| {
      let glo_type = if !args.is_empty() {
        let kv_str = args[0].to_string();
        if let Some(pos) = kv_str.find("type=") {
          let rest = &kv_str[pos + 5..];
          rest.split(',').next().unwrap_or("main").trim().to_string()
        } else {
          "main".to_string()
        }
      } else {
        "main".to_string()
      };
      let docid = state::lookup_value("docid")
        .map(|s| s.to_string())
        .unwrap_or_default();
      let id = if docid.is_empty() {
        s!("glo.{glo_type}")
      } else {
        s!("{docid}.glo.{glo_type}")
      };
      let mut toks = vec![T_CS!("\\lx@printglossary")];
      toks.push(T_BEGIN!());
      toks.extend(ExplodeText!(&id));
      toks.push(T_END!());
      toks.push(T_BEGIN!());
      toks.extend(ExplodeText!(&glo_type));
      toks.push(T_END!());
      Ok(Tokens::new(toks))
    });
    def_macro(cs, params, ExpansionBody::Closure(closure), None)?;
  }

  {
    // \printnoidxglossaries — prints all registered glossary types
    let cs = T_CS!("\\printnoidxglossaries");
    let closure: ExpansionClosure = Rc::new(move |_args: Vec<ArgWrap>| {
      let types_str = state::lookup_value("glossary_types")
        .map(|s| s.to_string())
        .unwrap_or_else(|| "main".to_string());
      let types: Vec<&str> = types_str.split(',').filter(|s| !s.is_empty()).collect();

      let mut toks = Vec::new();
      for glo_type in types {
        let docid = state::lookup_value("docid")
          .map(|s| s.to_string())
          .unwrap_or_default();
        let id = if docid.is_empty() {
          s!("glo.{glo_type}")
        } else {
          s!("{docid}.glo.{glo_type}")
        };
        toks.push(T_CS!("\\lx@printglossary"));
        toks.push(T_BEGIN!());
        toks.extend(ExplodeText!(&id));
        toks.push(T_END!());
        toks.push(T_BEGIN!());
        toks.extend(ExplodeText!(glo_type));
        toks.push(T_END!());
      }
      Ok(Tokens::new(toks))
    });
    def_macro(cs, None, ExpansionBody::Closure(closure), None)?;
  }

  {
    // \printglossaries — same as \printnoidxglossaries
    let cs = T_CS!("\\printglossaries");
    let closure: ExpansionClosure = Rc::new(move |_args: Vec<ArgWrap>| {
      let types_str = state::lookup_value("glossary_types")
        .map(|s| s.to_string())
        .unwrap_or_else(|| "main".to_string());
      let types: Vec<&str> = types_str.split(',').filter(|s| !s.is_empty()).collect();

      let mut toks = Vec::new();
      for glo_type in types {
        let docid = state::lookup_value("docid")
          .map(|s| s.to_string())
          .unwrap_or_default();
        let id = if docid.is_empty() {
          s!("glo.{glo_type}")
        } else {
          s!("{docid}.glo.{glo_type}")
        };
        toks.push(T_CS!("\\lx@printglossary"));
        toks.push(T_BEGIN!());
        toks.extend(ExplodeText!(&id));
        toks.push(T_END!());
        toks.push(T_BEGIN!());
        toks.extend(ExplodeText!(glo_type));
        toks.push(T_END!());
      }
      Ok(Tokens::new(toks))
    });
    def_macro(cs, None, ExpansionBody::Closure(closure), None)?;
  }
});
