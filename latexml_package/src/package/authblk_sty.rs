use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: authblk.sty.ltxml — 100 lines
  // Author/affiliation blocks with mark-based association

  // Font/separator macros — Perl L22-27
  DefMacro!("\\Affilfont", "\\normalfont");
  DefMacro!("\\Authfont",  "\\normalfont");
  DefMacro!("\\Authsep",   ",");
  DefMacro!("\\Authand",   " and ");
  DefMacro!("\\Authands",  ", and ");
  DefMacro!("\\authorcr",  "\\\\");

  // Bookkeeping — Perl L30-38
  DefConditional!("\\ifnewaffil");
  DefRegister!("\\affilsep" =>  Dimension::from_str("1em")?);
  DefRegister!("\\@affilsep" => Dimension::from_str("1em")?);
  NewCounter!("Maxaffil");
  RawTeX!("\\setcounter{Maxaffil}{2}");
  NewCounter!("authors");
  NewCounter!("affil");
  NewCounter!("@affil");
  DefMacro!("\\the@affil", "affil\\arabic{@affil}");

  // \author — Perl L40-46
  // Splits on \and and comma, creates one \lx@ab@author per author
  DefMacro!("\\author[]{}", sub[(opt_mark, authors)] {
    let mark_toks = opt_mark.unwrap_or_default();
    let parts = split_tokens(authors, vec![T_CS!("\\and"), T_OTHER!(",")]);
    let mut result = Vec::new();
    for part in parts {
      // \lx@ab@author[mark]{name}
      result.push(T_CS!("\\lx@ab@author"));
      result.push(T_OTHER!("["));
      result.extend(mark_toks.unlist_ref().iter().cloned());
      result.push(T_OTHER!("]"));
      result.push(T_BEGIN!());
      result.extend(part.unlist());
      result.push(T_END!());
    }
    result
  });

  DefMacro!("\\lx@ab@author[]{}",
    "\\@add@frontmatter{ltx:creator}[role=author]{\\@personname{#2}\\lx@split@authormark{#1}}");

  // Mark splitting — Perl L50-54
  // Splits comma-separated marks into individual \lx@authormark calls
  DefMacro!("\\lx@split@authormark{}", sub[(marks)] {
    let mark_str = marks.to_string();
    let mark_str = mark_str.trim();
    if mark_str.is_empty() {
      // No explicit mark — use auto-generated \the@affil
      vec![T_CS!("\\lx@authormark"), T_BEGIN!(), T_CS!("\\the@affil"), T_END!()]
    } else {
      let mut result = Vec::new();
      for mark in mark_str.split(',') {
        let mark = mark.trim();
        if !mark.is_empty() {
          result.push(T_CS!("\\lx@authormark"));
          result.extend(Explode!(mark));
        }
      }
      result
    }
  });

  // Perl L56-58: empty element, mark text only as attribute (not content)
  DefConstructor!("\\lx@authormark{}",
    "^ <ltx:contact role='affiliationmark' _mark='#1'></ltx:contact>");

  // \affil — Perl authblk.sty.ltxml L60-69 is a single DefConstructor
  // whose `afterDigest` reads `\the@affil` + StepCounter('@affil') inline
  // to auto-generate the mark when `#1` is absent. Rust splits that into
  // a DefMacro wrapper (which expands the counter+token glue at gullet
  // time) delegating to a hidden `\lx@ab@affil` DefConstructor with a
  // pre-resolved mark arg. The split is required because the Rust
  // constructor API doesn't expose a `Digest(T_CS('\the@affil'))`
  // equivalent inside `after_digest` with a writable counter step.
  // Intentional kind divergence (DefConstructor → DefMacro wrapper);
  // see WISDOM #44 — the observable XML is identical.
  DefMacro!("\\affil[]{}", sub[(opt_mark, body)] {
    let mark_toks = opt_mark.unwrap_or_default();
    let mark_str = mark_toks.to_string();
    let mark_str = mark_str.trim().to_string();
    let mut result = Vec::new();
    if mark_str.is_empty() {
      // No optional mark — use \the@affil and step counter
      result.push(T_CS!("\\lx@ab@affil"));
      result.push(T_OTHER!("["));
      result.push(T_CS!("\\the@affil"));
      result.push(T_OTHER!("]"));
      result.push(T_BEGIN!());
      result.extend(body.unlist_ref().iter().cloned());
      result.push(T_END!());
      result.push(T_CS!("\\stepcounter"));
      result.push(T_BEGIN!());
      result.extend(Explode!("@affil"));
      result.push(T_END!());
    } else {
      result.push(T_CS!("\\lx@ab@affil"));
      result.push(T_OTHER!("["));
      result.extend(mark_toks.unlist_ref().iter().cloned());
      result.push(T_OTHER!("]"));
      result.push(T_BEGIN!());
      result.extend(body.unlist_ref().iter().cloned());
      result.push(T_END!());
    }
    result
  });

  DefConstructor!("\\lx@ab@affil[]{}",
    "^ <ltx:note role='affiliationtext' mark='#1'>#2</ltx:note>");

  // DOM surgery: after ltx:document closes, relocate affiliation text
  // into matching author contact elements.
  // Perl L71-91: Tag('ltx:document', afterClose => \&authblkRelocateAffil)
  Tag!("ltx:document", after_close => sub[document, _node] {
    authblk_relocate_affil(document)?;
  });

  // Note formatting — Perl L95-96
  DefMacro!("\\AB@authnote{}",  "\\textsuperscript{\\normalfont#1}");
  DefMacro!("\\AB@affilnote{}", "\\textsuperscript{\\normalfont#1}");
});

/// Perl L73-91: authblkRelocateAffil
/// Moves affiliation text from ltx:note elements into matching ltx:contact elements.
fn authblk_relocate_affil(document: &mut Document) -> Result<()> {
  // Find all affiliationmark contacts and affiliationtext notes
  let author_nodes = document.findnodes(".//ltx:contact[@role='affiliationmark' and @_mark]", None);
  let affil_nodes = document.findnodes(".//ltx:note[@role='affiliationtext']", None);

  // Build mark → affil_node mapping, unlinking affil nodes from DOM
  let mut mark_to_affil: rustc_hash::FxHashMap<String, Node> = rustc_hash::FxHashMap::default();
  for mut affil_node in affil_nodes {
    affil_node.unlink();
    if let Some(mark) = affil_node.get_attribute("mark") {
      mark_to_affil.insert(mark, affil_node);
    }
  }

  // Process each affiliationmark: change role, clone affil children into it
  for mut author_node in author_nodes {
    document.set_attribute(&mut author_node, "role", "affiliation")?;
    let mark = author_node.get_attribute("_mark").unwrap_or_default();
    if let Some(affil_node) = mark_to_affil.get(&mark) {
      let children = affil_node.get_child_nodes();
      document.append_clone(&mut author_node, children)?;
    }
  }

  // D3b: affil_nodes were unlinked above and never reattached — the
  // detached subtrees would leave dangling idstore entries when the
  // HashMap drops them. append_clone has now consumed the originals'
  // ids via modify_id suffix mapping, so it's safe to recursively
  // unrecord.
  for (_, affil_node) in mark_to_affil.into_iter() {
    document.unrecord_node_ids(&affil_node);
  }

  Ok(())
}
