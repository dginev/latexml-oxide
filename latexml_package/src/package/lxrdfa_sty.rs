use crate::prelude::*;

/// Extract RDFa attributes from a KeyVals Digested argument.
/// Perl: RDFAttributes (L106-122) — processes about/resource for \ref{label} or #id.
fn rdf_attributes_from_digested(kv: &Digested) -> HashMap<String, String> {
  let mut attrs = HashMap::default();
  if let DigestedData::KeyVals(ref kvs) = *kv.data() {
    for (key, val) in kvs.get_pairs() {
      if key == "about" || key == "resource" {
        // Perl L111-121: check if value is a \ref Whatsit or #id string
        let digested_opt = val.clone().undigested();
        let is_ref = digested_opt.as_ref().is_some_and(|d| {
          if let DigestedData::Whatsit(ref w) = *d.data() {
            let cs = w.borrow().get_definition().get_cs_name().to_string();
            cs == "\\ref" || cs == "\\ref "
          } else {
            false
          }
        });
        if is_ref {
          // Extract label from \ref's second argument (the label text)
          if let Some(ref d) = digested_opt
            && let DigestedData::Whatsit(ref w) = *d.data()
          {
            let label = w
              .borrow()
              .get_arg(2)
              .map(|a| s!("LABEL:{}", a.to_string()))
              .unwrap_or_else(|| {
                w.borrow()
                  .get_arg(1)
                  .map(|a| s!("LABEL:{}", a.to_string()))
                  .unwrap_or_default()
              });
            attrs.insert(s!("{}labelref", key), label);
            continue;
          }
        }
        let val_str = val.to_string();
        if val_str.starts_with('#') && val_str.len() > 1 {
          attrs.insert(s!("{}idref", key), val_str[1..].to_string());
        } else {
          attrs.insert(key.clone(), val_str);
        }
      } else {
        let val_str = val.to_string();
        if !val_str.is_empty() {
          attrs.insert(key.clone(), val_str);
        }
      }
    }
  }
  attrs
}

/// Extract RDFa attributes from an ArgWrap (handles ArgWrap::KV directly).
fn rdf_attributes_from_argwrap(arg: &ArgWrap) -> HashMap<String, String> {
  match arg {
    ArgWrap::KV(kvs) => {
      let mut attrs = HashMap::default();
      for (key, val) in kvs.get_pairs() {
        process_rdf_key_value(key, val, &mut attrs);
      }
      attrs
    },
    _ => {
      // Try converting to Digested
      match arg.clone().undigested() {
        Some(d) => rdf_attributes_from_digested(&d),
        _ => HashMap::default(),
      }
    },
  }
}

/// Process a single RDF key-value pair, handling \ref and #id patterns.
fn process_rdf_key_value(key: &str, val: &ArgWrap, attrs: &mut HashMap<String, String>) {
  if key == "about" || key == "resource" {
    // Check for \ref Whatsit in digested value
    if let Some(d) = val.clone().undigested() {
      if let Some(label) = extract_ref_label(&d) {
        attrs.insert(s!("{}labelref", key), label);
        return;
      }
      // Check for List containing a \ref Whatsit
      if let DigestedData::List(ref l) = *d.data() {
        for item in &l.borrow().boxes {
          if let Some(label) = extract_ref_label(item) {
            attrs.insert(s!("{}labelref", key), label);
            return;
          }
        }
      }
    }
    let val_str = val.to_string();
    if val_str.starts_with('#') && val_str.len() > 1 {
      attrs.insert(s!("{}idref", key), val_str[1..].to_string());
    } else if val_str.starts_with("\\ref{") && val_str.ends_with('}') {
      // Preamble fallback: \ref{label} as string → labelref
      let label = &val_str[5..val_str.len() - 1];
      attrs.insert(s!("{}labelref", key), s!("LABEL:{}", label));
    } else {
      attrs.insert(key.to_string(), val_str);
    }
  } else {
    let val_str = val.to_string();
    if !val_str.is_empty() {
      attrs.insert(key.to_string(), val_str);
    }
  }
}

/// Check if a Digested is a \ref Whatsit and extract its label.
fn extract_ref_label(d: &Digested) -> Option<String> {
  if let DigestedData::Whatsit(ref w) = *d.data() {
    let wb = w.borrow();
    let cs = wb.get_definition().get_cs_name().to_string();
    if cs == "\\ref" || cs == "\\ref " {
      let label = wb
        .get_arg(2)
        .map(|a| s!("LABEL:{}", a.to_string()))
        .or_else(|| wb.get_arg(1).map(|a| s!("LABEL:{}", a.to_string())))
        .unwrap_or_default();
      return Some(label);
    }
  }
  None
}

/// Set RDFa attributes directly on a node (bypassing model validation).
fn set_rdf_attrs_on_node(node: &mut Node, attrs: &HashMap<String, String>) {
  for (key, val) in attrs {
    let _ = node.set_attribute(key, val);
  }
}

LoadDefinitions!({
  // Perl lxRDFa.sty.ltxml L20-26 + L28: `labels` option overrides \label
  // and \lx@longtable@label to also emit a dcterms:alternative RDFa
  // triple alongside the original \label. Rust omitted the option, so
  // `\usepackage[labels]{lxRDFa}` silently didn't tag labels. Ported as
  // Perl-equivalent DeclareOption + ProcessOptions. The expansion uses
  // \lxRDFa which is defined later in the same load block — LaTeXML
  // expands options lazily (at document time, not at package-load
  // time), so forward-reference is fine.
  DeclareOption!("labels", {
    Let!("\\lxRDF@original@label", "\\label");
    Let!("\\lxRDF@originallx@longtable@label", "\\lx@longtable@label");
    DefMacro!(
      "\\label Semiverbatim",
      "\\lxRDF@original@label{#1}\\lxRDFa{property=dcterms:alternative,content=#1}"
    );
    DefMacro!(
      "\\lx@longtable@label Semiverbatim",
      "\\lxRDF@originallx@longtable@label{#1}\\lxRDFa{property=dcterms:alternative,content=#1}"
    );
  });
  ProcessOptions!();

  // DefKeyVal for the RDFa keyval family
  DefKeyVal!("RDFa", "about", "Semiverbatim");
  DefKeyVal!("RDFa", "resource", "Semiverbatim");
  DefKeyVal!("RDFa", "typeof", "Semiverbatim");
  DefKeyVal!("RDFa", "property", "Semiverbatim");
  DefKeyVal!("RDFa", "rel", "Semiverbatim");
  DefKeyVal!("RDFa", "rev", "Semiverbatim");
  DefKeyVal!("RDFa", "content", "Semiverbatim");
  DefKeyVal!("RDFa", "datatype", "Semiverbatim");

  // \lxRDFaPrefix{prefix}{url}
  DefPrimitive!("\\lxRDFaPrefix{}{}", sub[(prefix, url)] {
    let p = do_expand(prefix)?.to_string();
    let u = do_expand(url)?.to_string();
    let entry = if p.is_empty() { u } else { s!("{}: {}", p, u) };
    let _ = push_value("RDFa_prefixes", entry);
    Ok(Vec::new())
  });

  // \lxRDFa[xpath]{keyvals} — add RDFa attributes to current/specified node
  DefConstructor!("\\lxRDFa OptionalSemiverbatim RequiredKeyVals:RDFa",
    sub[document, args, _props] {
      let xpath_str = args.first()
        .and_then(|a| a.as_ref())
        .map(|a| a.to_string())
        .unwrap_or_default();

      let mut target_nodes: Vec<Node>;
      let savenode;
      if !xpath_str.is_empty() {
        savenode = document.get_node().clone();
        target_nodes = document.findnodes(&xpath_str, Some(&savenode));
      } else {
        savenode = document.float_to_attribute("property")
          .unwrap_or_else(|| document.get_node().clone());
        target_nodes = match document.get_element() {
          Some(el) => vec![el],
          None => Vec::new(),
        };
      }

      if let Some(kv) = args.get(1).and_then(|a| a.as_ref()) {
        let attrs = rdf_attributes_from_digested(kv);
        for node in &mut target_nodes {
          set_rdf_attrs_on_node(node, &attrs);
        }
      }

      document.set_node(&savenode);
    }
  );

  // \lxRDFAnnotate{keyvals}{text}
  DefConstructor!("\\lxRDFAnnotate RequiredKeyVals:RDFa {}",
    "<ltx:text>#2</ltx:text>",
    enter_horizontal => true,
    after_construct => sub[document, whatsit] {
      if let Some(kv) = whatsit.get_arg(1) {
        let attrs = rdf_attributes_from_digested(kv);
        // Only set on ltx:text elements (the element we constructed), not auto-opened parents
        for node in document.get_constructed_nodes() {
          let qname = document::get_node_qname(node);
          if qname == pin!("ltx:text") {
            let mut n = node.clone();
            set_rdf_attrs_on_node(&mut n, &attrs);
          }
        }
      }
    }
  );

  // \lxRDF — preamble version: store as frontmatter entry
  // Perl L184-194: push(@{ LookupValue('frontmatter')->{'ltx:rdf'} }, ['ltx:rdf', {%attr}, undef])
  DefPrimitive!("\\lxRDF@preamble[] RequiredKeyVals:RDFa", sub[args] {
    if let Some(kv) = args.get(1) {
      let mut attrs = rdf_attributes_from_argwrap(kv);
      if !attrs.contains_key("about") && !attrs.contains_key("aboutlabelref")
        && !attrs.contains_key("aboutidref") {
        attrs.insert(s!("about"), String::new());
      }
      // Store in frontmatter hash under "ltx:rdf" key, matching Perl
      with_value_mut("frontmatter", |val_opt| {
        let frontmatter = match val_opt {
          Some(&mut Stored::HashTagData(ref mut frnt)) => frnt,
          _ => return Ok::<(), Error>(()),
        };
        let tag = s!("ltx:rdf");
        let empty_content = Digested::from(List::new(Vec::new()));
        let entry = document::tag::TagData {
          tag: tag.clone(),
          attr: attrs,
          content: vec![document::tag::TagContent::Box(empty_content)],
        };
        let f_entry = frontmatter.entry(tag).or_insert_with(Vec::new);
        f_entry.push(entry);
        Ok(())
      })?;
    }
    Ok(Vec::new())
  });

  // \lxRDF — body version (create <ltx:rdf> element)
  // Perl lxRDFa.sty.ltxml L197-210 sets `alias => '\lxRDF'` so that a
  // \lxRDF{...} body-form invocation reverts to the user-facing `\lxRDF`
  // in the tex= attribute rather than the internal `\lxRDF@body` variant.
  DefConstructor!("\\lxRDF@body[] RequiredKeyVals:RDFa",
    sub[document, args, _props] {
      let savenode = document.float_to_element("ltx:rdf", false);
      if let Ok(Some(ref save)) = savenode {
        let mut rdf = document.open_element("ltx:rdf", None, None)?;
        if let Some(kv) = args.get(1).and_then(|a| a.as_ref()) {
          let mut attrs = rdf_attributes_from_digested(kv);
          if !attrs.contains_key("about") && !attrs.contains_key("aboutlabelref")
            && !attrs.contains_key("aboutidref") {
            attrs.insert(s!("about"), String::new());
          }
          set_rdf_attrs_on_node(&mut rdf, &attrs);
        }
        if let Some(content) = args.first().and_then(|a| a.as_ref()) {
          document.absorb(content, None)?;
        }
        document.close_element("ltx:rdf")?;
        document.set_node(save);
      }
    },
    alias => "\\lxRDF"
  );

  Let!("\\lxRDF", "\\lxRDF@preamble");
  let _ = push_value(
    "@at@begin@document",
    Tokens!(T_CS!("\\let"), T_CS!("\\lxRDF"), T_CS!("\\lxRDF@body")),
  );

  // Add prefix= attribute when document opens
  Tag!("ltx:document", after_open => sub[_document, node] {
    if let Some(Stored::VecDequeStored(prefixes)) = lookup_value("RDFa_prefixes") {
      let prefix_strs: Vec<String> = prefixes.iter().map(|s| s.to_string()).collect();
      if !prefix_strs.is_empty() {
        let _ = node.set_attribute("prefix", &prefix_strs.join(" "));
      }
    }
  });
});
