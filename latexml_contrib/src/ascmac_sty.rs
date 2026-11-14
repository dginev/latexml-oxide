//! Stub for ascmac.sty — a Japanese LaTeX2e add-on (part of `jsclasses`
//! family) that provides boxed environments such as `{itembox}`,
//! `{screen}`, `{shadebox}`, `{boxnote}` (TeX Live: platex-tools/ascmac).
//!
//! Witness: 2601.09339 (`\usepackage{ascmac, fancybox}` +
//! `\begin{itembox}[l]{title}` body `\end{itembox}`), chemobabel-en/-ja
//! (`{screen}` inside a figure). The boxes are block boxes through
//! `insert_block`; the itembox title is kept as an `ltx:note`.
use latexml_package::prelude::*;

LoadDefinitions!({
  // The four boxed environments are BLOCK BOXES: their body goes through the
  // kernel's `insert_block` (Perl TeX_Box.pool.ltxml insertBlock; as framed.sty
  // and mdframed do), which picks the container the context admits — a
  // `<ltx:block>` inside a `<ltx:figure>`, a `<ltx:para>`-wrapped block in the
  // flow, an inline-block in a picture. The earlier fixed
  // `<ltx:para><ltx:p>#body</ltx:p></ltx:para>` was schema-invalid whenever the
  // box sat in a figure (chemobabel-en/-ja: `\begin{figure}…\begin{screen}`;
  // RUST-ONLY — Perl runs ascmac.sty raw and the content flows as figure
  // panels). Guard: `perfect_kernel_batch56::ascmac_screen_inside_a_figure_is_a_block`.
  // ascmac.sty: `screen` = rounded frame, `shadebox` = shadowed frame,
  // `boxnote` = note frame, `itembox[align]{title}` = frame with a title.
  DefEnvironment!(
    "{itembox}[]{}",
    sub[document, args, props] {
      document.maybe_close_element("ltx:p")?;
      if let Some(Stored::Digested(body)) = props.get("body") {
        let mut attrs: HashMap<String, String> = HashMap::default();
        attrs.insert("framed".to_string(), "rectangle".to_string());
        attrs.insert("class".to_string(), "ltx_ascmac_itembox".to_string());
        let blocks = insert_block(document, body, attrs)?;
        // The title becomes the FIRST child of the box's content (as the raw
        // itembox typesets it — a label above the body), so the author's
        // caption stays with its box and in order. `insert_block` returns the
        // box's CONTENT nodes (for a text body: the one `<p>`), so the note is
        // opened inside the first of them and moved before whatever is there;
        // with nothing returned (empty body) it is emitted at the current point.
        // `args` is 0-based: [0] = the optional alignment, [1] = the title.
        if let Some(Some(title)) = args.get(1) {
          let saved = document.get_node().clone();
          let host = blocks.first().cloned();
          if let Some(h) = host.as_ref() {
            document.set_node(h);
          }
          let attrs = string_map!("role" => "itembox-title");
          document.open_element("ltx:note", Some(attrs), None)?;
          document.absorb(title, None)?;
          if let (Some(mut note), Some(mut h)) = (document.close_element("ltx:note")?, host) {
            note.unlink();
            match h.get_first_child() {
              Some(mut first) => {
                first.add_prev_sibling(&mut note)?;
              },
              None => {
                h.add_child(&mut note)?;
              },
            }
          }
          document.set_node(&saved);
        }
      }
      Ok(())
    },
    mode => "internal_vertical"
  );
  DefEnvironment!(
    "{screen}",
    sub[document, _args, props] {
      document.maybe_close_element("ltx:p")?;
      if let Some(Stored::Digested(body)) = props.get("body") {
        let mut attrs: HashMap<String, String> = HashMap::default();
        attrs.insert("framed".to_string(), "rectangle".to_string());
        attrs.insert("class".to_string(), "ltx_ascmac_screen".to_string());
        insert_block(document, body, attrs)?;
      }
      Ok(())
    },
    mode => "internal_vertical"
  );
  DefEnvironment!(
    "{boxnote}",
    sub[document, _args, props] {
      document.maybe_close_element("ltx:p")?;
      if let Some(Stored::Digested(body)) = props.get("body") {
        let mut attrs: HashMap<String, String> = HashMap::default();
        attrs.insert("framed".to_string(), "rectangle".to_string());
        attrs.insert("class".to_string(), "ltx_ascmac_boxnote".to_string());
        insert_block(document, body, attrs)?;
      }
      Ok(())
    },
    mode => "internal_vertical"
  );
  DefEnvironment!(
    "{shadebox}",
    sub[document, _args, props] {
      document.maybe_close_element("ltx:p")?;
      if let Some(Stored::Digested(body)) = props.get("body") {
        let mut attrs: HashMap<String, String> = HashMap::default();
        attrs.insert("framed".to_string(), "rectangle".to_string());
        attrs.insert("class".to_string(), "ltx_ascmac_shadebox".to_string());
        insert_block(document, body, attrs)?;
      }
      Ok(())
    },
    mode => "internal_vertical"
  );
});
