//! `latex_constructs` section 11: C.11 Moving Information Around
//!
//! Split verbatim from the single `LoadDefinitions!` body (2026-09-03); the
//! definitions run in the original order from `super::load_definitions`.

use super::*;

#[rustfmt::skip]
pub(crate) fn load() -> Result<()> {
  // ======================================================================
  // C.11 Moving Information Around
  // ======================================================================

  //======================================================================
  // C.11.1 Files
  //======================================================================
  def_primitive_noop("\\nofiles")?;

  // Perl: DefPrimitive('\listfiles', undef) — no-op. Required so the
  // autoload trigger for `\listfiles` (engine/tex.rs) gets overridden
  // after LaTeX.pool loads; otherwise the trigger re-expands itself
  // after each pool load, creating a unique mouth-source per iteration
  // that eventually trips the 50M arena::pin sentinel (arxiv 1311.6082).
  def_primitive_noop("\\listfiles")?;

  //======================================================================
  // C.11.2 Cross-References
  //======================================================================

  // \label attaches a label to the nearest parent that can accept a labels attribute
  // but only those that have an xml:id (but should this require a refnum and/or title ???)
  // Note that latex essentially allows redundant labels, but we can record only one!!!
  DefConstructor!("\\lx@label Semiverbatim", sub[document, _olabel, props] {
    if let Some(savenode) = document.float_to_label() {
      let mut labels : HashMap<String,bool> = HashMap::default();
      if let Some(label) = props.get("label") {
        labels.insert(label.to_string(), true);
      }
      for label in document.node_get_attribute("labels").unwrap_or_default().split_whitespace() {
        labels.insert(label.to_string(), true);
      }
      let mut sorted_labels: Vec<String> = labels.into_keys().collect();
      sorted_labels.sort();
      document.node_set_attribute("labels", &sorted_labels.join(" "))?;
      document.set_node(&savenode);
    }
  },
  // Perl L3847-3848: disappear in tex=/content-tex unless outside DUAL_BRANCH.
  // Empty reversion: \label contributes no visible content to tex= attributes.
  reversion => "",
  properties => {stored_map!("alignmentSkippable" => true, "alignmentPreserve" => true)},
  after_digest => sub[whatsit] {
    if let Some(arg1) = whatsit.get_arg(1) {
      maybe_note_label(&arg1.to_string());
    }
    let label = match whatsit.get_arg(1) {
      Some(labeld) => clean_label(&labeld.to_string(), None).into_owned(),
      None => String::new()
    };
    let scope = label.replace("LABEL:","label:");
    let label_key = s!("LABEL@{}", label);
    whatsit.set_property("label", label);

    let ctr_key_opt = with_value("current_counter", |val_opt| val_opt
      .map(|ctr| s!("scopes_for_counter:{}", ctr)));
    if let Some(ctr_key) = ctr_key_opt {
      // TODO: we should probably improve the ergonomics here to avoid the vec![]
      unshift_value(&ctr_key, vec![scope.clone()]);
      activate_scope(pin(scope));
      begin_mode("text")?;
      let current_label = digest(Tokens!(T_CS!("\\@currentlabel")))?;
      assign_value(&label_key, current_label, Some(Scope::Global));
      end_mode("text")?;
    }
  }
  );
  // Perl L3862: Let('\label', '\lx@label'). The canonical label constructor is
  // \lx@label; \label is an alias. Saving \lx@label (not the mutable \label)
  // in eqnarray/ams rearrangeable bindings prevents an infinite
  // \lx@eqnarray@save@label recursion under nested align/gather (2008.13358).
  let_i(&T_CS!("\\label"), &T_CS!("\\lx@label"), Some(Scope::Global));

  // If a node has been labeled, but still hasn't yet got an id by afterClose:late,
  // we'd better generate an id for it.
  Tag!("ltx:*", after_close_late => sub[document, node] {
    if node.has_attribute("labels") && !node.has_attribute("xml:id") {
      document.generate_id(node, "")?;
    }
  });

  // # These will get filled in during postprocessing.
  // # * is added to accommodate hyperref
  // Perl latex_constructs.pool.ltxml L3873-3878: sizer => '()',
  //   robust => 1, enterHorizontal => 1.
  DefConstructor!("\\ref OptionalMatch:* Semiverbatim",
    "<ltx:ref ?#1(class='ltx_nolink')() labelref='#label' _force_font='true'/>",
    sizer => "()",
    robust => true,
    enter_horizontal => true,
    properties => sub[args] {
      unpack_opt_ref!(args => _star, label_opt);
      let label = label_opt.as_ref().unwrap().to_string();
      Ok(stored_map!("label" => Stored::String(pin(clean_label(&label, None)))))
  });

  // "page" does not make sense in xml.  If the user really wants, they will need:
  // \usepackage{latexml} ... \iflatexml alternate\else page \pageref{label}\fi
  Let!("\\pageref", "\\ref");

  // \@setref is from latex.ltx kernel. LaTeXML redefines \ref directly,
  // so \@setref is normally bypassed — but some packages call it directly.
  // The body below IS the latex.ltx kernel definition: if #1 is \relax
  // (undefined ref), show "??"; otherwise apply #2 to #1 with a \null guard.
  RawTeX!("\\def\\@setref#1#2#3{\\ifx#1\\relax ??\\else\\expandafter#2#1\\null\\fi}");

  // ======================================================================
  //  C.11.3 Bibliography and Citation
  // ======================================================================

  // Note that it's called \refname in LaTeX's article, but \bibname in report & book.
  // And likewise, mixed up in various other classes!

  // \thebibliography@ID empty default lives in `latex_constructs_rust_only.rs`
  // section 4 (per-bibliography runtime value is reassigned inside the
  // \bibliography constructor body at L2085).
  // Perl: latex_constructs.pool.ltxml L3891 — initial empty value
  def_macro_noop("\\the@lx@bibliography@ID")?;

  DefMacro!(
    "\\bibliography Semiverbatim",
    r#"\lx@ifusebbl{#1}{\input{\jobname.bbl}}{\lx@bibliography{#1}}"#
  );

  DefMacro!("\\lx@ifusebbl{}{}{}", sub[(bib_files_tks, bbl_clause, bib_clause)] {
    let bib_files = Expand!(bib_files_tks).to_string();
    let jobname = Expand!(T_CS!("\\jobname")).to_string();
    if bib_files.is_empty() {
      // Perl stops here (`latex_constructs.pool.ltxml` L3901,
      // `return unless $bib_files;`) and so did this port — silently, in both
      // engines. But real `latex.ltx` ends `\bibliography` with an
      // UNCONDITIONAL `\@input@{\jobname.bbl}`, so `\bibliography{}` beside a
      // shipped `<jobname>.bbl` renders the full reference list under
      // pdflatex (measured: "References / [1] A. Uthor. A paper. 2020.", and
      // the `\cite` resolves to [1]). Following latex.ltx here recovers the
      // references instead of dropping them: 7 papers in the 2605+2606
      // sandboxes, including the GWTC-5 LIGO set, all of which ship a
      // jobname-matching `.bbl`. Witness 2605.27226 (repro
      // `docs/parity/bib_absence_2026-07-29/repros/f3_empty_arg_bbl/`).
      // OXIDIZED_DESIGN #86; audit family F3(a).
      if FindFile!(&jobname, type => "bbl").is_some() {
        return Ok(bbl_clause);
      }
      return Ok(Tokens!());
    }
    let bbl_path = FindFile!(&jobname, type => "bbl");
    // BIB_CONFIG is a list of phases; with bibconfig=bbl,bib: try bbl first, fall back to bib.
    // Default (bibtex option) is ['bib', 'bbl']; nobibtex sets ['bbl'].
    let default_bib_config: Rc<[SymStr]> = Rc::new([pin("bib"), pin("bbl")]);
    let bib_config = match lookup_value("BIB_CONFIG") {
      Some(Stored::Strings(v)) => v,
      _ => default_bib_config,
    };
    if bib_config.is_empty() {
      Info!("missing", "bib_config", "BIB_CONFIG was empty, ignoring bibliography phase.");
      return Ok(Tokens!());
    }
    // Perl `\lx@ifusebbl` (latex_constructs.pool.ltxml L3967-3983) acts on the
    // FIRST configured phase ONLY (`$$bib_config[0]`); it does NOT fall through
    // to later phases. The earlier Rust port iterated all phases as a fallback
    // chain, which — with a `bbl,bib` config and no `\jobname.bbl` on disk —
    // fell through to the `bib` phase and emitted a spurious empty
    // `\lx@bibliography` placeholder, i.e. a SECOND, content-less "References"
    // section, whenever the real entries arrived via a differently-named
    // `.bbl` processed as content. Witness 2107.03065 (refs.bbl, not
    // <jobname>.bbl): Perl emits 1 bibliography, the old Rust 2. Mirror Perl:
    // branch on the first phase only.
    let first_is_bbl = with(bib_config[0], |s| s == "bbl");
    if first_is_bbl {
      if bbl_path.is_some() {
        return Ok(bbl_clause);
      }
      // bbl-first precedence (e.g. ar5iv's bibconfig=bbl,bib) but no
      // <jobname>.bbl on disk: fall through to a configured 'bib' phase, BUT
      // only when the .bib files actually exist. Emitting \lx@bibliography
      // unconditionally would add an empty placeholder "References"; when the
      // real entries instead arrive via a manual \input{refs.bbl} (witness
      // 2107.03065, which ships refs.bbl and NO refs.bib) that produces a
      // duplicate, content-less bibliography. Requiring a real .bib delivers
      // true "bbl-over-bib" precedence (prefer .bbl, else .bib) without the
      // double. Witness 2605.16562 (refs.bib, no .bbl) now gets a bibliography.
      let bib_in_config = bib_config.iter().any(|p| with(*p, |s| s == "bib"));
      let all_bibs_exist = bib_in_config
        && bib_files
          .split(',')
          .map(str::trim)
          .filter(|bf| !bf.is_empty())
          .all(|bf| FindFile!(bf, type => "bib").is_some());
      if all_bibs_exist {
        return Ok(bib_clause);
      }
      Info!("expected", "bbl", "Couldn't find bbl file, bibliography may be empty.");
      Ok(Tokens!())
    } else {
      // 'bib' phase — check if .bib files exist
      let mut missing_bibs = String::new();
      for bf in bib_files.split(',').map(str::trim).filter(|bf| !bf.is_empty()) {
        let bib_path = FindFile!(bf, type => "bib");
        if bib_path.is_none() {
          if !missing_bibs.is_empty() {
            missing_bibs.push(',');
          }
          missing_bibs.push_str(bf);
        }
      }
      if missing_bibs.is_empty() || bbl_path.is_none() {
        Ok(bib_clause)
      } else {
        Info!("expected", missing_bibs, s!("Couldn't find all bib files, using {jobname}.bbl instead"));
        Ok(bbl_clause)
      }
    }
  });

  AssignMapping!("BACKMATTER_ELEMENT", "ltx:bibliography" => "ltx:section");
  AssignMapping!("BACKMATTER_ELEMENT", "ltx:index"        => "ltx:section");

  DefConstructor!("\\lx@bibliography [] Semiverbatim",
    "<ltx:bibliography files='#2' xml:id='#id' bibstyle='#bibstyle' citestyle='#citestyle' sort='#sort' lists='#lists'><ltx:title font='#titlefont' _force_font='true'>#title</ltx:title></ltx:bibliography>",
    after_digest => sub[whatsit] {
      bgroup();
      begin_bibliography(whatsit)?;
      let _ = egroup();
    },
    before_construct => sub[doc,whatsit] {
      adjust_backmatter_element(doc, whatsit)?;
    },
    properties => sub[args] {
      // The chapterbib/bibunits unit name (`[#1]`) is a LIST IDENTIFIER, matched
      // against each `<ltx:bibref inlist=...>` to select the entries for this
      // per-unit bibliography, so `lists` must be the RAW source string. The unit
      // name is an included file's basename; `\lx@cb@unitname` explodes it to
      // catcode-12 OTHER tokens (so `_` is not a subscript — 1611.05798). But the
      // args here are DIGESTED, and a bare `.to_string()` would render that `_`
      // through the active OT1 font (slot 0x5F = `˙` U+02D9), giving
      // `lists="main˙paper"` while the citation side keeps `inlist="main_paper"`
      // (from the CITE_UNIT string) — the mismatch drops every entry (0 cited, empty
      // References). So `.revert()` to the source tokens FIRST, then `.to_string()`
      // — do NOT drop the revert. This matches Perl, which builds `lists` from the
      // arg's ToString (source `_`). Rust-only; witness arXiv 2605.15421 (0 -> 101).
      unpack_opt_ref!(args => unit_opt);
      let lists = match unit_opt.as_ref() {
        Some(u) => u.revert()?.to_string(),
        None => String::new(),
      };
      Ok(stored_map!("lists" => Stored::String(pin(&lists))))
    }
  );

  DefConstructor!("\\bibstyle{}", sub[document, _whatsit, props] {
    let style = prop_string!(props, "style");
    set_bibstyle(&style);
    if let Some(mut bib) = document.findnode("//ltx:bibliography", None) {
      if let Some(Stored::String(bs)) = lookup_value("BIBSTYLE") {
        with(bs, |s| document.set_attribute(&mut bib, "bibstyle", s))?;
      }
      if let Some(Stored::String(cs)) = lookup_value("CITE_STYLE") {
        with(cs, |s| document.set_attribute(&mut bib, "citestyle", s))?;
      }
      if let Some(Stored::String(so)) = lookup_value("CITE_SORT") {
        with(so, |s| document.set_attribute(&mut bib, "sort", s))?;
      }
    }
  },
    after_digest => sub[whatsit] {
      let style = whatsit.get_arg(1).map(|a| a.to_string()).unwrap_or_default();
      assign_value("BIBSTYLE", pin(&style), Some(Scope::Global));
      if let Some((cs, so)) = lookup_bibstyle_params(&style) {
        assign_value("CITE_STYLE", pin(cs), None);
        assign_value("CITE_SORT", pin(so), None);
      } else {
        Info!("unexpected", style, s!("Unknown bibstyle '{style}', it will be ignored"));
      }
    },
    properties => sub[args] {
      unpack_opt_ref!(args => style_opt);
      let style = style_opt.as_ref().map_or(String::new(), |s| s.to_string());
      Ok(stored_map!("style" => Stored::String(pin(&style))))
    }
  );

  // Record the bibliographystyle name globally BEFORE dispatching to \bibstyle,
  // so it reaches the <ltx:bibliography> node even when a package layer drops the
  // name at \bibstyle: natbib's `nobibstyle`/`[numbers]` does
  // `\let\bibstyle\@gobble`, and its author-year \bibstyle has no `\bibstyle@`
  // preset for a numeric style, so `\bibstyle{ieeetr}` otherwise vanishes.
  // MakeBibliography reads this name to number an unsorted numeric style
  // (ieeetr/IEEEtran/unsrt) in citation order (html_feedback #5930/#6095, the
  // natbib/revtex arm of #6294). CITE_STYLE is deliberately NOT set here —
  // natbib owns numbers-vs-author-year via its options, and the plain-path
  // \bibstyle DefConstructor already sets CITE_STYLE/CITE_SORT for known styles.
  DefPrimitive!("\\lx@record@bibstyle{}", sub[(style)] {
    assign_value("BIBSTYLE", pin(Expand!(style).to_string()), Some(Scope::Global));
  });
  DefMacro!(
    "\\bibliographystyle Semiverbatim",
    "\\lx@record@bibstyle{#1}\\bibstyle{#1}"
  );

  DefConditional!("\\if@lx@inbibliography");
  // Should be an environment, but people seem to want to misuse it.
  DefConstructor!("\\thebibliography",
  "<ltx:bibliography xml:id='#id'><ltx:title font='#titlefont' _force_font='true'>#title</ltx:title><ltx:biblist>",
    before_digest => {
        before_digest_bibliography() },
    after_digest => sub[whatsit] {
      // NOTE that in some perverse situations (revtex?)
      // it seems to be allowable to omit the argument
      // It's ignorable for latexml anyway, so we'll just read it if its there.
      skip_spaces()?;
      if if_next(T_BEGIN!())? {
        read_arg(ExpansionLevel::Off)?;
      }
      begin_bibliography(whatsit)?;
    },
    before_construct => sub[doc,whatsit] {
      adjust_backmatter_element(doc, whatsit)?;
    },
    locked => true
  );

  // Close the bibliography
  DefConstructor!("\\endthebibliography", sub[document,_whatsit,_props] {
    document.maybe_close_element("ltx:biblist")?;
    document.maybe_close_element("ltx:bibliography")?;
  },
    // Disarm the `setupPseudoBibitem` redirection that `\thebibliography`
    // installed — the same three `\let`s `\restoring@bibitem` performs, minus
    // its trailing `\bibitem`. Perl (latex_constructs.pool.ltxml L4014-4017)
    // has no teardown: it relies on `\begin`/`\end{thebibliography}` popping
    // the group the `\let`s were made in. That covers hand-written
    // bibliographies, but NOT the bare-CS `\thebibliography …
    // \endthebibliography` pair that the biblatex `.bbl` rebuilder expands to
    // (`biblatex_sty.rs::bib_as_thebibliography`), which opens no group. There
    // the redirection outlived the bibliography, so the next `\par` — a blank
    // line after `\printbibliography` — still expanded to
    // `\par@in@bibliography` and deposited a stray empty `\save@bibitem{}`
    // OUTSIDE the biblist (`Error:malformed:ltx:bibitem <ltx:bibitem> isn't
    // allowed in <ltx:p>`, witness arXiv 2605.17646: 59 bibitems where Perl
    // and the `.bbl` both say 58).
    //
    // Restoring here is a no-op for the grouped case (the `\let`s are undone
    // again when the group pops) and for the bare `\thebibliography` with no
    // closer at all (this never runs) — the two shapes Perl's comment at
    // `\thebibliography` calls out. See KNOWN_PERL_ERRORS #57.
    //
    // Gated on still being armed, for the same reason `\restoring@bibitem`
    // only runs once: a real `\bibitem` inside the bibliography has already
    // restored all three, and an `\endthebibliography` reached with no arming
    // at all would otherwise copy an UNDEFINED `\save@par` onto `\par`.
    after_digest => sub[_whatsit] {
      if x_equals(&T_CS!("\\bibitem"), &T_CS!("\\restoring@bibitem")) {
        Let!("\\bibitem", "\\save@bibitem");
        Let!("\\par", "\\save@par");
        Let!("\\\\", "\\save@backbackslash");
      }
    },
    locked=>true);
  Let!("\\saved@endthebibliography", "\\endthebibliography");
  // auto close the bibliography and contained biblist.
  Tag!("ltx:biblist",      auto_close => true);
  // arXiv-fork: the bibliography joins the navigation TOC (its xml:id is
  // assigned by the bibliography machinery itself, so only inlist here).
  Tag!("ltx:bibliography", auto_close => true,
    after_open => sub[document, node] {
      document.set_attribute(node, "inlist", "toc")?;
  });

  DefMacro!("\\par@in@bibliography", {
    skip_spaces()?;
    if let Some(tok) = read_token()? {
      // If next token is another \par, or a REAL \bibitem,
      // then this \par expands into what followed
      // Else, put it back, and start a bibitem.
      if tok == T_CS!("\\par") || tok == T_CS!("\\bibitem") {
        Ok(Tokens!(tok))
      } else {
        unread_one(tok);
        Ok(Tokens!(T_CS!("\\save@bibitem"), T_BEGIN!(), T_END!()))
      }
    } else {
      Ok(Tokens!(T_CS!("\\save@bibitem"), T_BEGIN!(), T_END!()))
    }
  });
  def_macro_noop("\\vskip@in@bibliography Glue")?;
  DefMacro!("\\item@in@bibliography", "\\save@bibitem{}");

  // If we hit a real \bibitem, put \par & \bibitem back to correct defn, and then \bibitem.
  // A bibitem with now key or label...
  //
  // Porting note: careful with the escaping rules. In perl we had a '\let\\\\\save@...'
  // but if we use the r## 'raw string literal' in Rust, the extra \\ escape is not needed.
  DefMacro!(
    "\\restoring@bibitem",
    r#"\let\bibitem\save@bibitem\let\par\save@par\let\\\save@backbackslash\bibitem"#
  );

  // Perl latex_constructs.pool.ltxml L4126 uses parent counter `@lx@bibliography`
  // (declared at L3890). The `@lx@` prefix prevents the counter helper macro
  // `\the<parent>` from colliding with `\thebibliography` (the env constructor).
  NewCounter!("@bibitem", "@lx@bibliography", idprefix => "bib");
  DefMacro!("\\the@bibitem", "\\arabic{@bibitem}");
  DefMacro!("\\@biblabel{}", "[#1]");
  DefMacro!("\\fnum@@bibitem", "{\\@biblabel{\\the@bibitem}}");
  // Hack for abused bibliographies; see below
  DefMacro!(
    "\\bibitem",
    r#"\if@lx@inbibliography\else\expandafter\lx@mung@bibliography\expandafter{\@currenvir}\fi\lx@bibitem"#,
    locked=>true);
  // Perl latex_constructs.pool.ltxml L4134-4162: enterHorizontal => 1 + afterDigest.
  DefConstructor!("\\lx@bibitem[] Semiverbatim",
    "<ltx:bibitem key='#key' xml:id='#id'>#tags<ltx:bibblock>",
    enter_horizontal => true,
    after_digest => sub[whatsit] {
      // Perl #2409: prune previous \lx@bibitem whatsit if it was auto-opened
      // with no tag/key body, and reuse its ID (avoids empty bibitem elements).
      let pruned_prev = with_box_list(|list| {
        if let Some(prev) = list.last()
          && let DigestedData::Whatsit(prev_ws_cell) = prev.data() {
            let prev_ws = prev_ws_cell.borrow();
            let defn = prev_ws.get_definition();
            let cs_str = defn.get_cs().to_string();
            if cs_str == "\\lx@bibitem"
              && prev_ws.get_arg(1).is_none()
              && prev_ws.get_arg(2).is_none_or(|a| a.is_empty().unwrap_or(true))
            {
              return true;
            }
          }
        false
      });
      if pruned_prev {
        with_box_list_mut_vec(|list| { list.pop(); });
        Info!("empty", "bibitem",
          "Encountered an empty \\bibitem, likely auto-opened without need. Pruning and reusing its id.");
      }
      let tag_opt = whatsit.get_arg(1);
      let key = if let Some(key) = whatsit.get_arg(2) {
        clean_bib_key(&key.to_string())
      } else { String::default() };
      if let Some(tag) = tag_opt {
        let mut properties = if pruned_prev {
          RefCurrentID!("@bibitem")?
        } else {
          RefStepID!("@bibitem")?
        };
        properties.insert("key", key.into());
        let mut tag_tokens = vec![
            T_BEGIN!(), T_CS!("\\def"), T_CS!("\\the@bibitem"), T_BEGIN!()];
        tag_tokens.extend(Revert!(tag));
        tag_tokens.push(T_END!());
        tag_tokens.extend(
          Invocation!(T_CS!("\\lx@make@tags"), vec![T_OTHER!("@bibitem")]).unlist());
        tag_tokens.push(T_END!());
        properties.insert("tags",
          digest(tag_tokens)?.into());
        whatsit.set_properties(properties);
      } else {
        let mut properties = RefStepCounter!("@bibitem")?;
        properties.insert("key", key.into());
        whatsit.set_properties(properties);
      }
    }
  );

  // Prune a phantom keyless bibitem auto-opened for `.bbl` PREAMBLE content — the macro
  // definitions / blank line an ACM-Reference-Format-style `.bbl` places between
  // `\begin{thebibliography}` and the first `\bibitem`. The blank line makes
  // `\par@in@bibliography` open a keyless `\lx@bibitem` for that preamble, rendering as a
  // spurious empty "(N)" entry before the real references. The digest-time prune in the
  // `\lx@bibitem` afterDigest only inspects the IMMEDIATELY-previous box, which the
  // preamble whitespace displaces, so it misses this one — scrub it here after
  // construction instead. A real `\bibitem` always carries a key; a keyless bibitem whose
  // `<bibblock>`s are all whitespace is the phantom. SHARED with Perl (both engines emit
  // it) — a surpass. Witness arXiv 2605.03143. OXIDIZED_DESIGN #155.
  Tag!("ltx:bibitem", after_close_late => sub[document, node] {
    let has_key = node.get_attribute("key").is_some_and(|k| !k.trim().is_empty());
    if !has_key {
      let blank = node
        .get_child_elements()
        .iter()
        .filter(|c| document::get_node_qname(c) == pin!("ltx:bibblock"))
        .all(|bb| bb.get_content().trim().is_empty());
      if blank {
        document.remove_node(node.clone());
      }
    }
  });

  // This attempts to handle the case where folks put \bibitem's within an enumerate or such.
  // We try to close the list and open the bibliography
  DefMacro!("\\lx@mung@bibliography{}", sub[(env)] {
    let tag = env.to_string();
    let mut tokens = Vec::new();
    // If we're in some sort of list environment, maybe we can recover
    if tag == "enumerate" || tag == "itemize" || tag == "description" {
      tokens.extend(Invocation!("\\end", vec![env]).unlist());
      tokens.extend(vec![
        T_CS!("\\let"),
        T_CS!(format!("\\end{tag}")),
        T_CS!("\\endthebibliography"),
        T_CS!("\\let"),
        T_CS!(format!("\\end{{{tag}}}")),
        T_CS!("\\end{thebibliography}")
      ]);
    }
    // else ? it probably isn't going to work??
    //Now, try to open {thebibliography}
    tokens.push(T_CS!("\\lx@mung@bibliography@pre"));
    tokens.push(T_CS!("\\thebibliography"));
    Ok(Tokens::new(tokens))
  });
  // Perl: maybeCloseElement($tag) if tag =~ /^ltx:(?:itemize|enumerate|description)$/
  DefConstructor!("\\lx@mung@bibliography@pre", sub[document] {
    let parent     = document.get_node();
    let tag_sym    = model::get_node_qname(parent);
    with(tag_sym, |tag|
      if tag == "ltx:itemize" || tag == "ltx:enumerate" || tag == "ltx:description" {
        document.maybe_close_element(tag)
      } else { Ok(None) }
    )?;
  });

  // Perl latex_constructs.pool.ltxml L4187-4189: enterHorizontal => 1.
  DefConstructor!("\\lx@bibnewblock", sub[document] {
  if document.is_openable("ltx:bibblock") {
    document.open_element("ltx:bibblock",None,None)?;
  }}, enter_horizontal => true);
  Let!("\\newblock", "\\lx@bibnewblock");
  Tag!("ltx:bibitem",  auto_open => true, auto_close => true);
  Tag!("ltx:bibblock", auto_open => true, auto_close => true);
  // NOTE: `ltx:block` deliberately has NO `auto_close` Tag, matching Perl (no
  // `Tag('ltx:block', autoClose=>1)` anywhere in the .ltxml sources). An
  // author's explicit `<ltx:block>` wrapper — e.g. a `DefEnvironment` body
  // template `<ltx:block>#body</ltx:block>` — must survive a paragraph break in
  // its body, holding the successive `<ltx:p>`s (issue #508). A prior blanket
  // `auto_close => true` here let `\par`'s `maybe_close_element("ltx:para")`
  // climb through such a block and over-close it, splitting the body — the #508
  // bug. It was added to mask a wayward `<ltx:block>` in `insert_block` (arXiv
  // 2302.11635), but Perl's `insertBlock` (TeX_Box.pool L516) produces that same
  // plain `<ltx:block>` and Perl reports the identical malformed-close there —
  // so the blanket Tag was an unfaithful band-aid, not parity. See
  // `base_utilities.rs::insert_block`.

  //----------------------------------------------------------------------
  // We've got the same problem as LaTeX: Lather, Rinse, Repeat.
  // It would be nice to know the bib info at digestion time
  //  * whether author lists will collapse
  //  * whether there are "a","b".. extensions on the year.
  // We could process the bibliography first, (IF it is a separate *.bib!)
  // but won't know which entries are included (and so can't resolve the a/b/c..)
  // until we've finished looking at (all of) the source(s) that will refer to them!
  //
  // We can do this in 2 passes, however
  //  (1) convert (latexml) both the source document(s) and the bibliography
  //  (2) extract the required bibitems and integrate (latexmlpost) it into the documents.
  // [Note that for mult-document sites, step (2) becomes 2 stages: scan and integrate]
  //
  // Here's the general layout.
  //   <ltx:cite> contains everything that the citations produce,
  //     including parens, pre-note, punctunation that precede the <ltx:bibcite>
  //     and punctuation, post-note, parens, that follow it.
  //   <ltx:bibcite show="string" bibrefs="keys" sep="" yysep="">phrases</ltx:bibcite>
  //     encodes the actual citation.
  //
  //     bibrefs : lists the bibliographic keys that will be used
  //     show    : gives the pattern for formatting using data from the bibliography
  //       It can contain:
  //         authors or fullauthors
  //         year
  //         number
  //         phrase1,phrase2,... selects one of the phrases from the content of the <ltx:bibref>
  //     This format is used as follows:
  //       If author and year is present, and a subset of the citations share the same authors,
  //         then the format is used, but the year is repeated for each citation in the subset,
  //         as a link to the bib entry.
  //       Otherwise, the format is applied to each entry.
  //
  // The design is intended to support natbib, as well as plain LaTeX.

  AssignValue!("CITE_STYLE", "numbers");
  AssignValue!("CITE_OPEN", T_OTHER!("["));
  AssignValue!("CITE_CLOSE", T_OTHER!("]"));
  AssignValue!("CITE_SEPARATOR", T_OTHER!(","));
  AssignValue!("CITE_YY_SEPARATOR", T_OTHER!(","));
  AssignValue!("CITE_NOTE_SEPARATOR", T_OTHER!(","));

  // Perl latex_constructs.pool.ltxml L4238: `\@cite{#1}{#2}` — kernel default
  // citation formatter, wraps `#1` (citation list) + optional `#2` (note) in
  // brackets via `\if@tempswa`. Used by `\@citex` (latex_dump.pool.ltxml).
  // The dump path captures this body, but the NODUMP path lacks it — port
  // here for source-org parity (CLAUDE.md priority 3).
  DefMacro!("\\@cite{}{}", "[{#1\\if@tempswa , #2\\fi}]");

  // Perl latex_constructs.pool.ltxml L4239-4241: DefConstructor('\@@cite []{}', ...,
  //   alias => '\cite', mode => 'restricted_horizontal', enterHorizontal => 1)
  DefConstructor!("\\@@cite[]{}", "<ltx:cite ?#1(class='ltx_citemacro_#1')>#2</ltx:cite>",
    alias => "\\cite", mode => "text", enter_horizontal => true);

  // \@@bibref{what to show}{bibkeys}{phrase1}{phrase2}
  // Perl latex_constructs.pool.ltxml L4244-4251: enterHorizontal => 1.
  DefConstructor!("\\@@bibref Semiverbatim Semiverbatim {}{}",
    "<ltx:bibref show='#1' bibrefs='#bibrefs' inlist='#bibunit' separator='#separator'
      yyseparator='#yyseparator'>#3#4</ltx:bibref>",
    enter_horizontal => true,
    properties => sub[args] {
      unref!(args => _show, keys, _phrase1, _phrase2);
      Ok(stored_map!("bibrefs" => clean_bib_key(&keys.to_string()),
        "separator" => match lookup_tokens("CITE_SEPARATOR") {
          Some(sep) => digest(sep)?.to_string(),
          None => String::new() },
        "yyseparator" => match lookup_tokens("CITE_YY_SEPARATOR") {
          Some(yysep) => digest(yysep)?.to_string(),
          None => String::new() },
        "bibunit" => match lookup_value("CITE_UNIT") {
          Some(Stored::String(s)) => to_string(s),
          _ => String::new() }
      ))
    }
  );

  // Simple container for any phrases used in the bibref
  // Perl latex_constructs.pool.ltxml L4254-4255: enterHorizontal => 1.
  DefConstructor!("\\@@citephrase{}", "<ltx:bibrefphrase>#1</ltx:bibrefphrase>",
    mode => "text", enter_horizontal => true);

  DefMacro!("\\cite[] Semiverbatim", sub[(post_opt, keys)] {
    // let style = state::lookup_tokens("CITE_STYLE").unwrap_or(NO_TOKENS);
    let open = lookup_tokens("CITE_OPEN");
    let open = open.unwrap_or(NO_TOKENS);
    let close = lookup_tokens("CITE_CLOSE").unwrap_or(NO_TOKENS);
    let mut post_tokens = match post_opt {
      Some(tks) => tks.unlist(),
      None => Vec::new()
    };
    if !post_tokens.is_empty() {
      let ns = lookup_tokens("CITE_NOTE_SEPARATOR").unwrap_or(NO_TOKENS);
      let mut post_wrapped = ns.unlist();
      post_wrapped.push(T_SPACE!());
      post_wrapped.extend(post_tokens);
      post_tokens = post_wrapped;
    }
    let bibref = Invocation!(T_CS!("\\@@bibref"),
      vec![Tokens!(), keys, Tokens!(), Tokens!()]);
    let mut arg_tokens = open.unlist();
    arg_tokens.extend(bibref.unlist());
    arg_tokens.extend(post_tokens);
    arg_tokens.extend(close.unlist());

    Ok(Invocation!(T_CS!("\\@@cite"),
      vec![Tokens::new(Explode!("cite")), Tokens::new(arg_tokens)]))
  }, robust => true, locked => true);

  // Perl L4271-4278: \nocite — defer to document end for MakeBibliography.
  // The key is EXPANDED here, as latex.ltx's `\nocite` writes it through
  // `\protected@write\@auxout{}{\string\citation{#1}}` at the call site:
  // Perl (:4214) and the former port deferred the raw tokens, so a key held
  // in a transient macro — tufte-common.def:934 `\@for\@temp@bibkeyx:=
  // \@tufte@citations\do{…\bibentry{\@temp@bibkeyx}}` inside a `\marginpar`
  // (bibentry.sty:64 `\bibentry` = `\nocite`) — was expanded at `\end
  // {document}` when the loop variable no longer existed ("`\@temp@bibkeyx`
  // is not defined", tufte sample-book/-handout; KPE #171). Guard:
  // `perfect_kernel_batch54::nocite_expands_its_key_at_the_call_site`.
  DefMacro!("\\nocite{}", sub[args] {
    let key = args.first().map(|a| a.revert().unwrap_or_default()).unwrap_or_default();
    let key = do_expand_partially(key).unwrap_or_default();
    let mut toks = vec![T_CS!("\\lx@mark@nocite"), T_BEGIN!()];
    toks.extend(key.unlist());
    toks.push(T_END!());
    let _ = push_value("@at@end@document", Stored::Tokens(Tokens::new(toks)));
    Ok(Tokens!())
  });
  DefConstructor!(
    "\\lx@mark@nocite Semiverbatim",
    "<ltx:cite><ltx:bibref show='nothing' bibrefs='#bibrefs' inlist='#bibunit'/></ltx:cite>",
    properties => sub[args] {
      let key = args[0].as_ref().map(|a| a.to_attribute()).unwrap_or_default();
      // Perl CleanBibKey: trim + remove internal spaces
      let bibrefs: String = key.chars().filter(|c| !c.is_whitespace()).collect();
      let bibunit = lookup_value("CITE_UNIT")
        .map(|v| v.to_string()).unwrap_or_default();
      Ok(stored_map!("bibrefs" => bibrefs, "bibunit" => bibunit))
    }
  );

  // #======================================================================
  // # C.11.4 Splitting the input
  // #======================================================================
  // NOTE: do NOT `Let!(\@@input, \input)` here. The Let in
  // `latex_bootstrap.rs:48` already aliased `\@@input` to the raw
  // TeX `\input` (the engine-init version from `tex_file_io.rs`)
  // BEFORE the dump load installed latex.ltx's redefined `\input`
  // (`\@ifnextchar\bgroup\@iinput\@@input`). Doing the Let again
  // here would re-alias `\@@input` to THAT redefined `\input` —
  // a self-recursive macro that loops at the false branch:
  // `\@@input snippet` → `\@@input` (itself) → infinite recursion
  // → TokenLimit. Triggered by `\verbatimlisting{snippet}` in
  // tests/tokenize/verb.tex.
  // LaTeX's \input is a bit different...

  // Input, now
  DefPrimitive!("\\ltx@input {}", sub[(arg)] { Input!(&Expand!(arg).to_string()); });
  DefMacro!("\\input", "\\@ifnextchar\\bgroup\\@iinput\\@@input");
  Let!("\\@iinput", "\\ltx@input");
  DefMacro!(
    "\\@input{}",
    "\\IfFileExists{#1}{\\@@input\\@filef@und}{\\typeout{No file #1.}}"
  );
  DefMacro!(
    "\\@input@{}",
    "\\InputIfFileExists{#1}{}{\\typeout{No file #1.}}"
  );

  DefMacro!("\\quote@name{}", "\"\\quote@@name#1\\@gobble\"\"");
  DefMacro!("\\quote@@name{} Match:\"", "#1\\quote@@name");
  DefMacro!("\\unquote@name{}", "\\quote@@name#1\\@gobble\"");

  // Perl L4313-4315: \include — input a file, respecting \includeonly
  DefPrimitive!("\\include{}", sub[(path)] {
    let path_str = Expand!(path).to_string();
    // Check if \includeonly restricts inclusion
    let table = lookup_value("including@only");
    let should_include = match table {
      None => true, // no \includeonly — include everything
      Some(Stored::HashString(map)) => map.contains_key(&path_str),
      _ => true,
    };
    if should_include {
      Input!(&path_str);
    }
  });

  // Perl L4303-4311: \includeonly — restrict which files \include loads
  DefPrimitive!("\\includeonly{}", sub[(paths)] {
    let paths_str = Expand!(paths).to_string();
    let mut map = rustc_hash::FxHashMap::default();
    for part in paths_str.split(',') {
      let trimmed = part.trim().to_string();
      if !trimmed.is_empty() {
        map.insert(trimmed, "1".to_string());
      }
    }
    assign_value("including@only", Stored::HashString(map), Scope::Global);
  });

  // {filecontents}/{filecontents*} environments + cache_filecontents
  // helper live in `latex_constructs_rust_only.rs` (Rust-only impl, not
  // a Perl latex_*.pool.ltxml export). Identical block previously
  // duplicated here removed.

  Tag!("ltx:indexphrase", after_close => sub[_document, node] {
    add_index_phrase_key(node)?;
  });
  Tag!("ltx:glossaryphrase", after_close => sub[_document, node] {
    add_index_phrase_key(node)?;
  });

  // \@index[style][inlist]{phrases} → <ltx:indexmark>
  DefConstructor!("\\@index[][]{}", "^<ltx:indexmark style='#1' inlist='#2'>#3</ltx:indexmark>",
    bounded => true,
    mode => "restricted_horizontal",
    sizer => 0
  );

  // \@indexphrase[sortkey]{phrase} → <ltx:indexphrase>
  DefConstructor!("\\@indexphrase[]{}", "<ltx:indexphrase key='#key' _standalone_font='true'>#2</ltx:indexphrase>",
    properties => sub[args] {
      // Perl (CleanIndexKey($_[1])) keys off the DIGESTED sort key, which is
      // right for `\index{LaTeX@\LaTeX}`-style keys but renders a literal
      // `_` (catcode-OTHER, from process_index_phrases' `\@sanitize`
      // neutralization) through the OT1 slot 0x5F as `˙` — the chapterbib
      // `lists=` trap (see `\@bibliography` above, 2605.15421). A sort key
      // holding one of the sanitized specials is a plain makeindex string
      // (tcolorbox `tag_if_active:TF`), so key it off the SOURCE tokens then.
      let key = args[0].as_ref()
        .map(|a| {
          let reverted = a.revert().unwrap_or_default();
          let has_sanitized_special = reverted.unlist_ref().iter().any(|t| {
            t.get_catcode() == Catcode::OTHER
              && t.with_str(|s| matches!(s, "_" | "^" | "&" | "#" | "$" | "~"))
          });
          let raw = if has_sanitized_special { reverted.to_string() } else { a.to_string() };
          clean_index_key(&raw)
        })
        .unwrap_or_default();
      if key.is_empty() {
        Ok(stored_map!())
      } else {
        Ok(stored_map!("key" => key))
      }
    }
  );

  // \@indexsee{key} → <ltx:indexsee>
  // Perl carries name => DigestIf('\seename') so the post-processor can
  // print the italic "see" word in front of the cross-reference.
  DefConstructor!("\\@indexsee{}", "<ltx:indexsee key='#key' name='#name' _standalone_font='true'>#1</ltx:indexsee>",
    properties => sub[args] {
      let key = args[0].as_ref()
        .map(|a| clean_index_key(&a.to_string()))
        .unwrap_or_default();
      let mut props = stored_map!("key" => key);
      if let Some(name) = DigestIf!("\\seename")? {
        props.insert("name", name.into());
      }
      Ok(props)
    }
  );

  // \@indexseealso{key} → <ltx:indexsee>
  DefConstructor!("\\@indexseealso{}", "<ltx:indexsee key='#key' name='#name' _standalone_font='true'>#1</ltx:indexsee>",
    properties => sub[args] {
      let key = args[0].as_ref()
        .map(|a| clean_index_key(&a.to_string()))
        .unwrap_or_default();
      let mut props = stored_map!("key" => key);
      if let Some(name) = DigestIf!("\\alsoname")? {
        props.insert("name", name.into());
      }
      Ok(props)
    }
  );

  // \index{phrases} — expand to \@index via process_index_phrases.
  // Perl: latex_constructs.pool.ltxml L4454 uses the SanitizedVerbatim
  // parameter type so that `\index{a_b}`, `\index{with spaces}`, etc. don't
  // fail tokenization on chars that normally have non-OTHER catcodes.
  DefMacro!("\\index SanitizedVerbatim", sub[(phrases)] {
    process_index_phrases(Tokens::new(phrases.revert()))
  });

  DefMacro!("\\indexname", "Index");
  // Perl latex_constructs.pool.ltxml L4567-4585: theindex generates the
  // "Index" title, computes a document-relative xml:id (the root the
  // post-processor's per-entry idx.* anchors chain off — without it,
  // see-refs have nothing to resolve to), and registers as a backmatter
  // element so a \printindex inside the last open section relocates to
  // the document/backmatter level instead of nesting (the BACKMATTER_ELEMENT
  // mapping for ltx:index already exists alongside ltx:bibliography's).
  DefEnvironment!("{theindex}",
  "<ltx:index xml:id='#id'><ltx:title font='#titlefont' _force_font='true'>#title</ltx:title>#body</ltx:index>",
  before_digest => {
    Let!("\\item", "\\index@item");
    Let!("\\subitem", "\\index@subitem");
    Let!("\\subsubitem", "\\index@subsubitem");
    // Perl L4519: `\dotfill` between an index phrase and its page list opens
    // the `ltx:indexrefs` separator (index styles that use `term \dotfill page`).
    Let!("\\dotfill", "\\index@dotfill");
  },
  // Must RETURN the digested `\index@done` whatsit (no trailing `;`) so it is
  // appended to the env body and CONSTRUCTED — it closes the trailing
  // indexphrase/indexrefs and unwinds the open indexlist levels (do_index_item
  // level 0). With a discarding `;` the whatsit was dropped, so `\end{theindex}`
  // force-closed the still-open indexphrase/indexlist and errored "Closing tag
  // ltx:index whose open descendents do not auto-close". Cf. the titlepage env.
  before_digest_end => { digest(Tokens!(T_CS!("\\index@done")))? },
  after_digest_begin => sub[whatsit] {
    note_backmatter_element(whatsit, "ltx:index");
    let docid: String = Expand!(T_CS!("\\thedocument@ID")).to_string();
    let id = if docid.is_empty() {
      "idx".to_string()
    } else {
      s!("{docid}.idx")
    };
    whatsit.set_property("id", id);
    if let Some(title) = DigestIf!("\\indexname")? {
      if let Some(titlefont) = title.get_font()? {
        whatsit.set_property("titlefont", titlefont);
      }
      whatsit.set_property("title", title);
    }
  },
  before_construct => sub[doc, whatsit] {
    adjust_backmatter_element(doc, whatsit)?;
  });

  def_primitive_noop("\\indexspace")?;
  // OXIDIZED_DESIGN #163: Perl noops `\makeindex`/`\makeglossary` entirely
  // (latex_constructs.pool L4531-4532) — but real latex.ltx `\makeindex` also
  // `\newwrite`s the `\@indexfile` stream, and raw doc.sty/l3doc.cls-style
  // code then writes `\protected@write\@indexfile{…}` DIRECTLY. With the
  // stream never allocated that raw write errors `undefined \@indexfile`
  // (SHARED with Perl 0.8.8, same-host verified on l3doc's own saveenv.tex —
  // Perl lands at 101 errors + fatal; 14 TL-doc bundles incl. l3kernel's own
  // manuals). Allocate the stream (guarded, once) while keeping everything
  // else nooped: no `\openout`, and crucially NO kernel-style redefinition of
  // `\index` — the semantic `\index SanitizedVerbatim` above stays in charge.
  // Writes to the allocated-but-unopened stream go to the log, as real TeX
  // does when no file is open — harmless.
  DefMacro!(
    "\\makeindex",
    "\\ifdefined\\@indexfile\\else\\csname newwrite\\endcsname\\@indexfile\\fi"
  );
  DefMacro!(
    "\\makeglossary",
    "\\ifdefined\\@glossaryfile\\else\\csname newwrite\\endcsname\\@glossaryfile\\fi"
  );
  // \printindex removed — not in Perl engine (defined in makeidx.sty.ltxml)

  // Perl latex_constructs.pool.ltxml L4481-4493 — `\glossary{}` uses a
  // closure constructor that guards on the current node: if we're inside
  // `ltx:p` or `ltx:text`, emit a Warn and skip the element entirely
  // (schema disallows `ltx:glossaryphrase` in those parents). Otherwise
  // insert the element. The earlier Rust port used a static template that
  // unconditionally produced `<ltx:glossaryphrase>`, which surfaced as
  // `Error:malformed:ltx:glossaryphrase isn't allowed in <ltx:p>` on
  // papers calling `\glossary{...}` in the body flow (where most papers
  // actually use it). Witnesses: arXiv:cs/9809003, math/9608214,
  // nucl-th/9311001.
  DefConstructor!("\\glossary{}", sub[document, args, props] {
    use latexml_core::document::get_node_qname;
    let current = document.get_node().clone();
    let current_name = with(get_node_qname(&current), |s| s.to_string());
    let parent_name = if current_name == "#PCDATA" {
      match current.get_parent() { Some(p) => {
        with(get_node_qname(&p), |s| s.to_string())
      } _ => { current_name }}
    } else { current_name };
    // Beyond the ltx:p/ltx:text guard, verify the SCHEMA actually admits a
    // glossaryphrase here. The doc-family `\changes`→`\glossary` path fires
    // at DOCUMENT level (between sections, during Building), where the
    // static check passed but insertion still produced
    // `malformed:ltx:glossaryphrase isn't allowed in <ltx:document>` —
    // 9 TL doc bundles (abntex2, the biblatex-* style manuals, pixelart…).
    let in_flow = parent_name.starts_with("ltx:p")
      || parent_name == "ltx:text"
      || !document.is_openable("ltx:glossaryphrase");
    if in_flow {
      Warn!("unexpected", "glossary",
        "glossary support is not yet ready for use in the main text flow.");
    } else {
      let key = prop_string!(props, "key");
      let mut attrs: rustc_hash::FxHashMap<String, String> = rustc_hash::FxHashMap::default();
      attrs.insert("role".to_string(), "glossary".to_string());
      attrs.insert("key".to_string(), key);
      let body: Vec<&Digested> = match args.first().and_then(|a| a.as_ref()) {
        Some(d) => vec![d],
        None => Vec::new(),
      };
      document.insert_element("ltx:glossaryphrase", body, Some(attrs))?;
    }
  },
    properties => sub[args] {
      let key = args[0].as_ref()
        .map(|a| clean_index_key(&a.to_string()))
        .unwrap_or_default();
      Ok(stored_map!("key" => key))
    },
    sizer => 0
  );

  // Standard English caption names set by babel-english.ldf's
  // \captionsenglish hook (and by letter.cls for the letter-specific
  // ones). Documents that pull in `babel` indirectly via blindtext /
  // tocbibind / sectsty without reaching the \selectlanguage path leave
  // these as undefined. Perl LaTeXML's babel.def.ltxml shim quietly
  // absorbs these CSes, but our raw-load of babel.sty surfaces
  // \setlocalecaption stubs without their \captionsenglish backings.
  // Provide English defaults here (NOT in latex_base.rs — that file
  // is skipped on the dump path so defs there don't survive). Witness
  // 2110.05865 (article + blindtext pulls babel; 8 undefined captions).
  DefMacro!("\\bibname", "Bibliography");
  DefMacro!("\\seename", "see");
  DefMacro!("\\alsoname", "see also");
  DefMacro!("\\glossaryname", "Glossary");
  DefMacro!("\\pagename", "Page");
  // NOTE: the letter-class captions `\ccname`/`\enclname`/`\headtoname` are NOT
  // defaulted here. Perl LaTeXML defines them NOWHERE (not in the base, babel,
  // or letter.cls.ltxml — its babel shim merely absorbs undefined references),
  // so they are undefined for a plain article unless the author defines them.
  // Pre-defining `\ccname` (= "cc", the carbon-copy label) unconditionally
  // silently blocked an author's `\newcommand{\ccname}` (`\@ifdefinable` sees it
  // as already-defined → the redefinition no-ops, keeping "cc"). Witness
  // 1706.00283 repurposes `\ccname` as a `c_i` constant-generator
  // (`\newcommand{\ccname}[1]{\cc\ccdef{#1}}`); with the default present its
  // `\ccname\ccT` minted nothing and every `\cc*` constant was undefined (10
  // errors). Real letters get these captions from letter.cls; defaulting them
  // for every article is non-faithful and clobbers author macros (cf. the
  // iopart `\revised` and pgfmath `\real` clobber traps).
  // Additional babel-english.ldf captions (\captionsenglish hook).
  DefMacro!("\\prefacename", "Preface");
  DefMacro!("\\proofname", "Proof");
  DefMacro!("\\abstractname", "Abstract");
  DefMacro!("\\indexname", "Index");

  //======================================================================
  // Perl: latex_constructs.pool.ltxml L4536-4564 — index constructors

  // Perl latex_constructs.pool.ltxml L4477: `Tag('ltx:indexentry', autoClose => 1)`.
  // `doIndexItem` opens a new `ltx:indexentry` for each `\item`/`\subitem` while a
  // sibling entry may still be open (consecutive items at the same level) and
  // closes an `ltx:indexlist` whose `indexentry` children are still open (at level
  // descent / `\index@done`). Both rely on `indexentry` auto-closing; without this
  // Tag the builder errors "<ltx:indexentry> isn't allowed in <ltx:indexentry>"
  // and "Closing tag ltx:indexlist whose open descendents do not auto-close".
  // Witness arXiv:1205.0533 (makeidx + `\input` of a multi-level `.ind`): 102
  // errors / Fatal → 0.
  Tag!("ltx:indexentry", auto_close => true);

  // Helper: close an open indexphrase element
  DefConstructor!("\\index@dotfill", sub[document] {
    if document.is_closeable("ltx:indexphrase").is_some() {
      document.close_element("ltx:indexphrase")?;
    }
    document.open_element("ltx:indexrefs", None, None)?;
  });

  DefConstructor!("\\index@item", sub[document] {
    do_index_item(document, 1)?;
  });
  DefConstructor!("\\index@subitem", sub[document] {
    do_index_item(document, 2)?;
  });
  DefConstructor!("\\index@subsubitem", sub[document] {
    do_index_item(document, 3)?;
  });
  DefConstructor!("\\index@done", sub[document] {
    do_index_item(document, 0)?;
  });

  //======================================================================
  // C.11.6 Terminal Input and Output
  //======================================================================
  // Perl latex_constructs.pool.ltxml L4538-4541: `Note(ToString($stuff))` — called
  // UNCONDITIONALLY. `Note!` does the log-always / stderr-if-`$VERBOSITY>=0` split
  // itself, so no `current_verbosity()` guard here — the old guard dropped
  // `\typeout` from the log under `--quiet` (the #763 log-floor bug).
  //
  // latex.ltx L1185-1188: the argument is written under
  // `\set@display@protect` (L1438 `\let\protect\string`), so a robust
  // command in it is written by NAME, never run. Expanding it with
  // `\protect`=`\relax` (as `make_generic_message` once did, and as Perl's
  // primitive-`\small` world never notices) walked into raw KOMA's
  // `\DeclareRobustCommand\small` (scrsize10pt.clo:62-72) — whose
  // `\@setfontsize\small…` re-expands `\small` forever; pdflatex's
  // `\edef\x{\small}` overflows the same way — from hvextern.sty:325
  // `\hv@ex@typeout{Running BodyVerbatim with fontsize=\small,…}`
  // (witness hvextern manual, `Fatal:Timeout:PushbackLimit`). Guard:
  // `perfect_kernel_batch53::typeout_writes_robust_commands_by_name`.
  DefPrimitive!("\\typeout{}", sub[(stuff)] {
    bgroup();
    let_i(&T_CS!("\\protect"), &T_CS!("\\string"), None);
    let content = Expand!(stuff);
    egroup()?;
    Note!(s!("{content}"));
  });
  def_primitive_noop("\\typein[]{}")?;

  Ok(())
}
