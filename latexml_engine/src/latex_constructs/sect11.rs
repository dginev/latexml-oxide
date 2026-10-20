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
    whatsit.set_property("label", label);

    let ctr_key_opt = with_value("current_counter", |val_opt| val_opt
      .map(|ctr| s!("scopes_for_counter:{}", ctr)));
    if let Some(ctr_key) = ctr_key_opt {
      // TODO: we should probably improve the ergonomics here to avoid the vec![]
      unshift_value(&ctr_key, vec![scope.clone()]);
      activate_scope(pin(scope));
      // Perl (latex_constructs.pool.ltxml:3802) also digests `\@currentlabel` into `LABEL@<label>`, a value nothing
      // reads (neither here nor in Perl). `\@currentlabel` is `\the<ctr>` unexpanded (Package.pm:765, dialect.rs),
      // where LaTeX holds the text as it was when the counter stepped (latex.ltx:14963 `\protected@edef`): read
      // here, after a list whose `label=` redefined `\theenumi` locally, it reached the document's own `\theenumi`,
      // which may loop (`\renewcommand\theenumi{\RegularTheEnumi}` with `\def\RegularTheEnumi{\theenumi}`:
      // 2609.13327, Fatal Recursion; Perl hangs, KNOWN_PERL_ERRORS #496). The unread digest is not made.
    }
  }
  );
  // Perl L3862: Let('\label', '\lx@label'). The canonical label constructor is
  // \lx@label; \label is an alias. Saving \lx@label (not the mutable \label)
  // in eqnarray/ams rearrangeable bindings prevents an infinite
  // \lx@eqnarray@save@label recursion under nested align/gather (2008.13358).
  let_i(&T_CS!("\\label"), &T_CS!("\\lx@label"), Some(Scope::Global));
  // The type a `\label[type]{key}` gives its object's references (cleveref.sty `\label[type]`), where a site that moves
  // a trailing `\label` reads the optional itself (`typed_label`). The kernel's `\label` has no type: nothing;
  // cleveref's binding reads it.
  DefMacro!("\\lx@label@type{}", "");

  // A reference shows its target's tags as its `show` names them: a held tag (`\lx@tag@intags@held`) it prints has
  // its deferred diagnostics replayed once the document is read (logger.rs `replay_shown_deferred`, ruling 7e).
  Tag!("ltx:ref", after_open => sub[_document, node] {
    if let (Some(label), Some(show)) = (node.get_attribute("labelref"), node.get_attribute("show")) {
      util::logger::note_label_use(&label, &show);
    }
  });

  // If a node has been labeled, but still hasn't yet got an id by afterClose:late,
  // we'd better generate an id for it.
  Tag!("ltx:*", after_close_late => sub[document, node] {
    if node.has_attribute("labels") && !node.has_attribute("xml:id") {
      document.generate_id(node, "")?;
    }
    if let Some(labels) = node.get_attribute("labels") {
      tie_held_tags(node, &labels);
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
  // The kernel's `\ref`, its robust wrapper and the constructor that wrapper calls, which nameref.sty restores at
  // `\begin{document}` (`nameref_sty.rs`; 1811.01873).
  Let!("\\lx@kernel@ref", "\\ref");
  Let!("\\lx@kernel@ref@", "\\ref ");

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

  // `locked`, beyond Perl (which lets a raw `\def\bibliography` take over —
  // and then renders nothing): classes and packages redefine it to write
  // `\bibdata` to the `.aux` and `\@input{\jobname.bbl}` — a file only a
  // bibtex run produces (paper.cls:933 for sfee, ltugboat.cls:1683,
  // abntex2cite.sty:375, gbt7714.sty, the curve/fcavtex/nxuthesis/thuthesis/
  // uafthesis classes) — and the `.bib` list lives nowhere else (the `.aux`
  // is never re-read), so the reference list vanished at 0 errors. The
  // binding's route prefers a shipped `.bbl` and otherwise runs the `.bib`
  // session, which is what every such wrapper means; bindings outrank raw
  // (OXIDIZED_DESIGN_DIVERGENCES #236). A wrapper that extends the original
  // is refused with an Info, as `\thebibliography`'s is.
  DefMacro!(
    "\\bibliography Semiverbatim",
    r#"\lx@ifusebbl{#1}{\input{\jobname.bbl}}{\lx@bibliography{#1}}"#,
    locked => true
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

  // `class` is the state value `lx@bibliography@class` (Rust-only): a binding that needs a bibliography the
  // document does not print sets `ltx_nodisplay` in a group around it (biblatex's `\fullcite` with no
  // `\printbibliography`, biblatex_sty.rs `blx_unprinted_bibliography`). `#preamble` (Rust-only) is
  // `\lx@bibliography@preamble`, typeset after the heading as `\thebibliography`'s below: the `.bbl` a `.bib`
  // becomes is a `thebibliography` that typesets it (natbib.sty:1063-1066 `\bibpreamble`; biblatex's
  // `\printbibliography[prenote=…]`, biblatex.sty:10002). MakeBibliography appends the list after it.
  DefConstructor!("\\lx@bibliography [] Semiverbatim",
    "<ltx:bibliography files='#2' xml:id='#id' class='#bibclass' bibstyle='#bibstyle' citestyle='#citestyle' sort='#sort' lists='#lists'><ltx:title font='#titlefont' _force_font='true'>#title</ltx:title>#preamble</ltx:bibliography>",
    after_digest => sub[whatsit] {
      record_bibliography_normalfont()?;
      bgroup();
      begin_bibliography_clean(whatsit)?;
      digest_bibliography_preamble(whatsit)?;
      setup_pseudo_bibitem()?;
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
      let mut props = stored_map!("lists" => Stored::String(pin(&lists)));
      if let Some(class) = lookup_value("lx@bibliography@class") {
        props.insert("bibclass", class);
      }
      Ok(props)
    }
  );

  // The style is a name, read Semiverbatim as `\bibliographystyle`'s is: digested, `SciPost_bibstyle` erred (2412.10240;
  // latex.ltx:17773 `\let\bibstyle\@gobble` typesets nothing; Perl pool:3972 `{}` the same, KNOWN_PERL_ERRORS #966). Guard:
  // `perfect_kernel_batch73a::bibstyle_name_is_not_typeset`.
  DefConstructor!("\\bibstyle Semiverbatim", sub[document, _whatsit, props] {
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
  // What a package typesets between the bibliography's heading and its list: natbib's
  // `thebibliography` runs `\bibpreamble` there (natbib.sty:1063-1066), which apacite extends with
  // `\bibliographyprenote` and `\nocitemeta`'s note (apacite.sty:1835-1850). Empty in the kernel;
  // natbib points it at `\bibpreamble`. Digested after the heading and before the list's `\bibitem`
  // redirection is armed (`setup_pseudo_bibitem`), it becomes the `Para.model` content the schema allows before
  // `ltx:biblist` (LaTeXML-structure.rnc:237). Perl never typesets it (KPE #325). Guard
  // `perfect_kernel_gemini::bibpreamble_is_printed`.
  DefMacro!("\\lx@bibliography@preamble", None);
  // Its counterpart at the list's end: natbib's `thebibliography` ends with `\bibitem@fin\bibpostamble`
  // before `\endlist` (natbib.sty:1079-1086). Empty in the kernel; natbib points it at `\bibpostamble`. A
  // preamble that opens an environment (`\begin{multicols}{2}`) is closed by its postamble
  // (`\end{multicols}`, 2009.08029); without it the environment outlived the bibliography and met the
  // `\endgroup` of `\end{thebibliography}`. Guard `perfect_kernel_batch64y::bibpostamble_closes_the_preamble`.
  DefMacro!("\\lx@bibliography@postamble", None);
  // A package's title for the bibliography, in place of `\bibsection`'s or `\refname`/`\bibname`
  // (`begin_bibliography_clean`): biblatex's `\printbibliography[title=…]` (biblatex.sty:9694), defined in its group.
  // Empty in the kernel.
  DefMacro!("\\lx@bibliography@title", None);
  // Should be an environment, but people seem to want to misuse it.
  DefConstructor!("\\thebibliography",
  "<ltx:bibliography xml:id='#id'><ltx:title font='#titlefont' _force_font='true' _own_title='#owntitle'>#title</ltx:title>#preamble<ltx:biblist>",
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
      // The heading first, then the preamble, then the list's redirection — the order natbib's
      // `thebibliography` typesets them (natbib.sty:1063-1066), so a font switch in the preamble
      // (`\bibpreamble{\small …}`, apacite's `\bibliographytypesize`) reaches the entries but
      // not the title, whose font `begin_bibliography_clean` reads from its digested box.
      begin_bibliography_clean(whatsit)?;
      // a continuation's own title (`\refname`, the class's `\bibsection`), which its close drops when no heading
      // took its place (`drop_own_bibliography_heading`)
      if lookup_bool("lx_bibliography_is_continued") {
        whatsit.set_property("owntitle", "true");
      }
      digest_bibliography_preamble(whatsit)?;
      setup_pseudo_bibitem()?;
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
    before_digest => { Digest!(T_CS!("\\lx@bibliography@postamble"))? },
    // Disarm the `setupPseudoBibitem` redirection that `\thebibliography`
    // installed — the same three `\let`s `\restoring@bibitem` performs, minus
    // its trailing `\bibitem`. Perl (latex_constructs.pool.ltxml L4014-4017)
    // has no teardown: it relies on `\begin`/`\end{thebibliography}` popping
    // the group the `\let`s were made in. That covers hand-written
    // bibliographies, but NOT the bare-CS `\thebibliography …
    // \endthebibliography` pair that the biblatex `.bbl` rebuilder expanded to
    // (`bib_as_thebibliography`, replaced by the `.bbl` reader in batch 56jc),
    // which opens no group. There
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
      // a heading inside it that no `\bibitem` followed (`\lx@bibliography@sectioned`) leaves nothing to continue: a
      // later lonely `\bibitem` is reported and a later bibliography numbers afresh
      assign_value("lx_bibliography_sectioned", false, Some(Scope::Global));
    },
    locked=>true);
  Let!("\\saved@endthebibliography", "\\endthebibliography");
  // the kernel's own, for `\lx@mung@bibliography` when a class has `\let` its list over it
  Let!("\\lx@kernel@thebibliography", "\\thebibliography");
  // auto close the bibliography and contained biblist.
  // …and reopen a list for a `\bibitem` that follows what closed it: a float or a closed
  // list between the entries (a figure is no `ltx:biblist` child) left the next
  // `ltx:bibitem` with no list to go in — an Error, Perl's too (it declares `autoClose`
  // only, latex_constructs.pool.ltxml:4020), where pdflatex prints one list. Half
  // openability (Perl's fractional `autoOpen`, as `ltx:picture`'s) keeps it off every path
  // a full auto-opener serves: text in the bibliography still opens an `ltx:para`, only
  // the bibliography entries (`ltx:bibitem`, `ltx:bibentry`) open a list. Witnesses
  // astro-ph9806175, 2103.15495, 0801.2060.
  Tag!("ltx:biblist",      auto_open => 0.5, auto_close => true);
  // arXiv-fork: the bibliography joins the navigation TOC (its xml:id is
  // assigned by the bibliography machinery itself, so only inlist here) —
  // unless it is hidden (`ltx_nodisplay`, `lx@bibliography@class`).
  Tag!("ltx:bibliography", auto_close => true,
    after_open => sub[document, node] {
      if !node
        .get_attribute("class")
        .is_some_and(|class| class.split_whitespace().any(|c| c == "ltx_nodisplay"))
      {
        document.set_attribute(node, "inlist", "toc")?;
      }
      // opened by a `\bibitem` outside any bibliography (`\lx@mung@bibliography@pre{implicit}`)
      if lookup_bool("lx_bibliography_opened_implicitly") {
        assign_value("lx_bibliography_opened_implicitly", false, Some(Scope::Global));
        document.set_attribute(node, "_implicit", "true")?;
      }
      // …or reopened after a heading inside the author's `{thebibliography}` (`\lx@bibliography@sectioned`)
      if lookup_bool("lx_bibliography_opened_continued") {
        assign_value("lx_bibliography_opened_continued", false, Some(Scope::Global));
        document.set_attribute(node, "_continued", "true")?;
      }
    },
    after_close => sub[document, node] {
      prune_phantom_entries(document, node);
      remove_emptied_lists(document, node);
      let absorbed = absorb_bibliography_heading(document, node)?;
      if !absorbed && node.has_attribute("_implicit") {
        head_implicit_bibliography(document, node)?;
      }
      if node.has_attribute("_continued") {
        // a continuation the heading before it did not head (a `\paragraph{Note.}` and its text between the entries)
        // is the list going on, as pdflatex prints it: no `\refname` title, no entry in the TOC of its own
        if !absorbed {
          drop_own_bibliography_heading(document, node)?;
        }
        prune_interrupted_bibliography(document, node)?;
      }
    });
  // …and those a bibliography's close did not reach: after it (2403.05651's nested `{thebibliography}` ends the outer
  // one, the blank line before its `\end` makes an entry under `<document>`). None removes itself, so the close walk
  // reaches every open element (Document.pm:1243-1247).
  Tag!("ltx:document", after_close => sub[document, node] {
    prune_phantom_entries(document, node);
    remove_emptied_lists(document, node);
  });

  DefMacro!("\\par@in@bibliography", {
    skip_spaces()?;
    if let Some(tok) = read_token()? {
      // If next token is another \par, or a REAL \bibitem,
      // then this \par expands into what followed
      // Else, put it back, and start a bibitem.
      // …and `\item`, an entry start as `\bibitem` is (Perl latex_constructs.pool.ltxml:4041-4049 names `\par` and
      // `\bibitem` only): a blank line before it makes no entry of its own.
      // Only the bibliography's own list's `\item` (`{thebibliography}`'s): in one a `\bibitem` opened out of an author's
      // list (`\lx@mung@bibliography`) the `\item` is that list's, which keeps Perl's path.
      let own_item = tok == T_CS!("\\item") && !lookup_bool("lx_bibliography_from_list");
      if tok == T_CS!("\\par") || tok == T_CS!("\\bibitem") || own_item {
        Ok(Tokens!(tok))
      } else {
        unread_one(tok);
        Ok(Tokens!(T_CS!("\\lx@pseudo@bibitem")))
      }
    } else {
      Ok(Tokens!(T_CS!("\\lx@pseudo@bibitem")))
    }
  });
  // The entry the recovery opens for a paragraph no `\bibitem` or `\item` starts (Perl's `\save@bibitem{}`,
  // latex_constructs.pool.ltxml:4047): only a TeX `\item` or `\bibitem` steps the bibliography's counter, so it takes
  // an unnumbered id and no label — an aas `\references` paragraph prints none (aasms4.sty `\refpar`), a `.bbl`
  // preamble's none at all (2605.03143) — and pruning it, empty (`prune_phantom_entries`), leaves no gap.
  DefConstructor!("\\lx@pseudo@bibitem", "<ltx:bibitem xml:id='#id'><ltx:bibblock>",
    enter_horizontal => true,
    properties => { RefStepID!("@bibitem") },
    after_digest => sub[_whatsit] {
      // one right after another (blank lines around a float's material, 0801.2060): the first is empty, pruned
      // as `\lx@bibitem` prunes its empty predecessor (Perl #2403), not placed to be pruned later
      let empty_prev = with_box_list(|list| {
        list.last().is_some_and(|prev| matches!(prev.data(), DigestedData::Whatsit(ws)
          if ws.borrow().get_definition().get_cs().to_string() == "\\lx@pseudo@bibitem"))
      });
      if empty_prev {
        with_box_list_mut_vec(|list| { list.pop(); });
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
    r#"\if@lx@inbibliography\else\expandafter\lx@mung@bibliography\expandafter{\@currenvir}\fi\lx@bibitem@autoloads"#,
    locked=>true);
  // An autoload the `[label]` calls loads its package before the entry is read, where Perl's digestion of the label
  // as the `\lx@bibitem`'s argument loaded it (latex_constructs.pool.ltxml:4134): the label is typeset only in the
  // entry's tags, where a package load came too late — the binding of crckapb.sty gives its bibliography's
  // `\citeauthoryear` to natbib (OXIDIZED_DESIGN_DIVERGENCES #731), whose parser, loaded inside the tags, took the
  // label `\the@bibitem` as its own number, `\NAT@wrout{\the@bibitem}` for ever (astro-ph0007279: Fatal
  // Stomach:Recursion). A package that takes `\bibitem` over (natbib) reads the entry itself. The label is peeked,
  // and put back as it was; a `\par` ends the peek, as it ends the non-long `\@lbibitem`'s argument (latex.ltx).
  DefMacro!("\\lx@bibitem@autoloads", {
    let kernel_bibitem = lookup_meaning(&T_CS!("\\bibitem"));
    let par = T_CS!("\\par");
    let mut peeked: Vec<Token> = Vec::new();
    let mut fired = false;
    if let Some(open) = read_token()? {
      peeked.push(open);
      if open == T_OTHER!("[") {
        let mut level = 0usize;
        while let Some(t) = read_token()? {
          peeked.push(t);
          match t.get_catcode() {
            Catcode::BEGIN => level += 1,
            Catcode::END => level = level.saturating_sub(1),
            _ if level == 0 && t == T_OTHER!("]") => break,
            _ if t == par => break,
            _ => {},
          }
          if let Some(package) = crate::tex::unfired_autoload_package(&t) {
            crate::tex::fire_autoload(&t, &package)?;
            fired = true;
          }
        }
      }
    }
    unread(Tokens::new(peeked));
    let taken_over = fired && !matches!((lookup_meaning(&T_CS!("\\bibitem")), kernel_bibitem),
      (Some(Stored::Expandable(now)), Some(Stored::Expandable(before))) if Rc::ptr_eq(&now, &before));
    Ok(Tokens!(if taken_over { T_CS!("\\bibitem") } else { T_CS!("\\lx@bibitem") }))
  });
  // Perl latex_constructs.pool.ltxml L4134-4162: enterHorizontal => 1 + afterDigest. The `[label]` is read
  // undigested (Perl `[]` digests it): it is typeset only in the entry's tags, each form where TeX prints it
  // (OXIDIZED_DESIGN_DIVERGENCES #750).
  DefConstructor!("\\lx@bibitem OptionalUndigested Semiverbatim",
    "<ltx:bibitem key='#key' xml:id='#id' _labelled='#labelled'>#tags<ltx:bibblock>",
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
            // …and the recovery's own entry (`\lx@pseudo@bibitem`), empty when the box before this one: Perl's
            // `\save@bibitem{}` was an `\lx@bibitem` this prune caught; pruned only later, at the bibliography's
            // close, it is placed first, where it may not go (0801.2060).
            if cs_str == "\\lx@pseudo@bibitem" {
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
        properties.insert("key", key.clone().into());
        // the author's own label (`\bibitem[Baker(2001)]{}`): a printed entry however empty (`is_phantom_entry`)
        properties.insert("labelled", "true".into());
        // The label is typeset where TeX prints it, form by form. The entry's own label, `\fnum@@bibitem` =
        // `{\@biblabel{\the@bibitem}}`, is printed here, as `\@lbibitem`'s `\item[\@biblabel{#1}\hfill]`
        // (latex.ltx:17746-17749). The forms a reference prints (refnum, …) are TeX's `\b@key`, which `\@lbibitem`
        // writes to the .aux (`\bibcite{key}{label}`) and only a `\cite` of the key expands: typeset only once some
        // `\cite` prints the key (`cite_prints_keys`), at once when one already has, through the deferred-print list
        // `b@key` (latexml_core `defer_print`; OXIDIZED_DESIGN_DIVERGENCES #750). Under mn2e/mnras without natbib
        // (`\def\@biblabel#1{}`, mn2e.cls:1370, mnras.cls:1636) a `\bibitem[\protect\citeauthoryear{…}{…}]{key}`
        // label errs only where a `\cite` prints it, as in pdflatex (0812.1037, 1105.2318, 1306.6339, 1508.03599). Perl
        // (latex_constructs.pool.ltxml:4096-4102) digests every form here. Repros index-bib/bibitem_label_*.
        // Under natbib (its own or any copy: `\NAT@ifcmd`, natbib.sty:809) the label is natbib's to parse at the
        // `\bibitem` (`\@lbibitem` writes the parsed author and year; `\citeauthoryear` is its delimited parser,
        // natbib.sty:1014 `#1#2#3(@)(@)\@nil#4`): no `\cite` expands the raw label. It is digested here once, as the
        // optional argument it was before 70c, and the tags are made from what it digested to, all forms at once;
        // read raw, out of that context, a copy's parser reads past the label's end (1003.3017's natbib_vw.sty:622;
        // repro index-bib/mn2e_natbib_copy/natbib_copy_citeauthoryear_label_cited, pdflatex and Perl 0 errors).
        if lookup_meaning(&T_CS!("\\NAT@ifcmd")).is_some() {
          let label = digest(Revert!(tag))?;
          let mut tag_tokens = vec![T_BEGIN!(), T_CS!("\\def"), T_CS!("\\the@bibitem"), T_BEGIN!()];
          tag_tokens.extend(Revert!(label));
          tag_tokens.push(T_END!());
          tag_tokens.extend(Invocation!(T_CS!("\\lx@make@tags"), vec![T_OTHER!("@bibitem")]).unlist());
          tag_tokens.push(T_END!());
          properties.insert("tags", digest(tag_tokens)?.into());
          whatsit.set_properties(properties);
          return Ok(Vec::new());
        }
        let mut define_label = vec![T_CS!("\\def"), T_CS!("\\the@bibitem"), T_BEGIN!()];
        define_label.extend(Revert!(tag));
        define_label.push(T_END!());
        let mut printed_here = Vec::new();
        let mut printed_by_cite = vec![T_CS!("\\begingroup")];
        printed_by_cite.extend(define_label.iter().copied());
        let mut cite_forms = false;
        for (here, form) in tag_forms(&Tokens!(T_OTHER!("@bibitem")))? {
          if here {
            printed_here.extend(form.unlist());
          } else {
            cite_forms = true;
            printed_by_cite.extend(form.unlist());
          }
        }
        printed_by_cite.push(T_CS!("\\endgroup"));
        let mut tag_tokens = vec![T_BEGIN!()];
        tag_tokens.extend(define_label);
        tag_tokens.extend([T_CS!("\\lx@tags"), T_BEGIN!()]);
        tag_tokens.extend(printed_here);
        if cite_forms {
          tag_tokens.extend(deferred_print_tokens(&cite_list(&key), Tokens::new(printed_by_cite)).unlist());
        }
        tag_tokens.extend([T_END!(), T_END!()]);
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
  // construction instead. The phantom: a keyless, unlabelled bibitem whose `<bibblock>`s hold no
  // text and no element but a page break (`is_phantom_entry`). SHARED with Perl (both engines emit
  // it) — a surpass. Witness arXiv 2605.03143. OXIDIZED_DESIGN #155. Done when the bibliography
  // closes (`prune_phantom_entries`), not in the entry's own close: a node its after-close hook
  // removes ends the document's close walk there (Document.pm:1243-1247, `parentNode` of a
  // removed node is a fragment), leaving the lists and the bibliography above it unclosed.

  // The bibliography a natbib `\bibitem` in no list opened (natbib_sty.rs `\lx@NAT@bibitem@check`): each later
  // `\bibitem` there is as lonely as the first (LaTeX's "Lonely \item"; repro index-bib/natbib_bibitem_in_document_body).
  DefPrimitive!("\\lx@bibliography@lonely", {
    assign_value("lx_bibliography_lonely", true, Some(Scope::Global));
  });
  // …until the next sectioning command, which closes it (`ltx:bibliography` auto-closes): a `\bibitem` after that is
  // lonely again and opens one of its own, one Error each as LaTeX's (a natbib `\bibitem` before and after a `\section`:
  // pdflatex 2). Read at digestion, where the bibliography's close (a construction) cannot tell it. The hook also runs
  // from `\maketitle` (sect05.rs `\lx@maketitle@body`), after which a `\bibitem` is as lonely. Repros
  // index-bib/natbib_bibitem_in_document_body, index-bib/natbib_bibitem_in_size_environment_twice; guard
  // `natbib_bibitem_in_the_document_body_is_reported`.
  DefPrimitive!("\\lx@bibliography@unlonely", {
    if lookup_bool("lx_bibliography_lonely") {
      assign_value("lx_bibliography_lonely", false, Some(Scope::Global));
      Digest!("\\@lx@inbibliographyfalse")?;
      // natbib's pending entry stop (`\BibitemShut{NoStop}`'s `\@bibstop`, natbib.sty:880-905) goes with it: the
      // bibliography's `\lx@bibliography@postamble` that would have spent it never runs, and the next `\bibitem`'s
      // `\bibitem@fin` would put the first bibliography's "." at the head of the second.
      Let!("\\@bibstop", "\\@undefined");
      if x_equals(&T_CS!("\\bibitem"), &T_CS!("\\restoring@bibitem")) {
        Let!("\\bibitem", "\\save@bibitem");
        Let!("\\par", "\\save@par");
        Let!("\\\\", "\\save@backbackslash");
      }
    }
  });
  AddToMacro!("\\@startsection@hook", "\\lx@bibliography@unlonely");
  // A heading inside the author's `{thebibliography}` (`\section*{References}` under `\begin{thebibliography}`, mostly
  // revtex4/4-1 papers; a topical `\section*`/`\subsection*` between groups of entries, hep-th0109182): pdflatex prints
  // it inside the list and the entries after it stay the list's; here the unit closes the bibliography
  // (`ltx:bibliography` auto-closes), so the bibliography state ends with it — the `\bibitem` after it opens the next
  // bibliography in that unit (`\lx@mung@bibliography`), which takes its heading (`absorb_bibliography_heading`) and the
  // author's `\end{thebibliography}` closes once. Perl keeps `\if@lx@inbibliography` and errs on every later entry
  // (`<ltx:bibitem> isn't allowed in <ltx:section>`, KNOWN_PERL_ERRORS #855). Witnesses 1702.06834, 2402.15720,
  // 2110.15342 (`\section*`), 2307.13534 (`\subsection*`), hep-th0109182; repro index-bib/heading_inside_thebibliography;
  // guards `perfect_kernel_batch68w::heading_inside_thebibliography_*`.
  DefPrimitive!("\\lx@bibliography@sectioned", {
    if x_equals(&T_CS!("\\if@lx@inbibliography"), &T_CS!("\\iftrue")) && lookup_bool("lx_bibliography_author_env") {
      Digest!("\\@lx@inbibliographyfalse")?;
      assign_value("lx_bibliography_sectioned", true, Some(Scope::Global));
      // A bibliography interrupted before its first entry (nothing digested in it yet, `holds_nothing_yet`, nor
      // `@bibitem` stepped; not itself a continuation) is the pruned one (`prune_interrupted_bibliography`): the
      // continuation takes its number (`begin_bibliography_clean`), so its entries are `bib.bibN`, as in the one
      // bibliography pdflatex prints (Perl's ids are fixed at digestion, latex_constructs.pool.ltxml:3877-3883).
      // `@bibitem` alone misses a labelled `\bibitem[S01]{a}` (`RefStepID`) and a paragraph's entry.
      let interrupted_empty = CounterValue!("@bibitem").value_of() == 0
        && !lookup_bool("lx_bibliography_is_continued")
        && holds_nothing_yet();
      assign_value("lx_bibliography_sectioned_empty", interrupted_empty, Some(Scope::Global));
      // the stop the entry before the heading asked for (natbib's `\BibitemShut`, natbib.sty:880-905) ends that entry,
      // which the next `\bibitem`'s `\bibitem@fin` would put at the head of the next bibliography's first
      if is_defined_token(&T_CS!("\\bibitem@fin")) {
        Digest!("\\bibitem@fin")?;
      }
      Let!("\\@bibstop", "\\@undefined");
      if x_equals(&T_CS!("\\bibitem"), &T_CS!("\\restoring@bibitem")) {
        Let!("\\bibitem", "\\save@bibitem");
        Let!("\\par", "\\save@par");
        Let!("\\\\", "\\save@backbackslash");
      }
    }
  });
  AddToMacro!("\\@startsection@hook", "\\lx@bibliography@sectioned");
  // The bibliography a `\bibitem` opens out of an author's list, in the group standing for that list (below): an
  // `\item` there is that list's, on Perl's path, not one of the bibliography's own (hep-ph9804413's `{enumerate}` of
  // `\bibitem`s; repro index-bib/kernel_bibitem_list_items_stay).
  DefPrimitive!("\\lx@bibliography@from@list", {
    assign_value("lx_bibliography_from_list", true, None);
  });
  // This attempts to handle the case where folks put \bibitem's within an enumerate or such.
  // We try to close the list and open the bibliography
  DefMacro!("\\lx@mung@bibliography{}", sub[(env)] {
    let tag = env.to_string();
    let mut tokens = Vec::new();
    // If we're in some sort of list environment, maybe we can recover
    if tag == "enumerate" || tag == "itemize" || tag == "description" {
      // Ending the list ends its group, with the definitions the author made in it, which TeX keeps until their own
      // `\end{<list>}` (2205.07302: a `\newcommand{\enquote}` in the `{description}`; RED repro
      // index-bib/bibitem_list_local_definition — the list's whole body is one captured whatsit).
      tokens.extend(Invocation!("\\end", vec![env.clone()]).unlist());
      tokens.extend(vec![
        // the group the author's `\end{<list>}` closes, its own having been ended above, which also undoes the
        // `\let`s, so a later list of that kind is itself again (KNOWN_PERL_ERRORS #513)
        T_CS!("\\begingroup"),
        T_CS!("\\lx@bibliography@from@list"),
        T_CS!("\\let"),
        T_CS!(format!("\\end{tag}")),
        T_CS!("\\endthebibliography"),
        T_CS!("\\let"),
        T_CS!(format!("\\end{{{tag}}}")),
        T_CS!("\\end{thebibliography}"),
      ]);
    }
    // else ? it probably isn't going to work??
    // …unless a heading inside it closed the bibliography (`\lx@bibliography@sectioned`): the environment is the kernel's,
    // its end the author's own `\end{thebibliography}`.
    else if tag == "thebibliography" && lookup_bool("lx_bibliography_sectioned") {
    } else if tag == "thebibliography" {
      // (OXIDIZED_DESIGN_DIVERGENCES #484, KNOWN_PERL_ERRORS #569) In `{thebibliography}` with no bibliography open,
      // the class's own list stands in the kernel's:
      // `\let\thebibliography\bibreferences` (ajour, cjour, IEEE-Con-Sys-mag: 0711.2301, astro-ph0005341, 2310.12571;
      // `\let` ignores the lock, State.pm assignMeaning), whose list reopened here per `\bibitem`, nested without end
      // (Perl --includestyles: the same loop; run 336's PushbackLimit Fatals). Its own end closes its list now
      // (ajour.cls/cjour.cls `\endreferences` = `\endlist`), the kernel's bibliography making the environment's end
      // its own (`before_digest_bibliography`).
      tokens.push(T_CS!("\\endthebibliography"));
    }
    // In another environment (a class's `{references}` list: astro-ph0002202, gr-qc0207046; an author's
    // `\begin{list}{}{}`), its end closes the kernel's bibliography first and then runs as its own (the class's `\endlist`
    // ending its list's frame), so what follows the environment follows the bibliography, as pdflatex prints it
    // (OXIDIZED_DESIGN_DIVERGENCES #484, KNOWN_PERL_ERRORS #569). The saved end is `\csname end<env>\endcsname`,
    // `\relax` for an environment with none (`{small}`).
    else if !tag.is_empty() {
      // saved once per environment, as its own marker says (`lx@mung@saved@<env>@done`): a second `\bibitem` that
      // opens a bibliography in the same one (after a `\section` closed the first) must not save the end it made,
      // which would call itself — nor may the saved end itself be the test, being `\relax` for an environment with
      // none (`{small}`). Repros index-bib/natbib_bibitem_in_document_body (`\enddocument`) and
      // index-bib/natbib_bibitem_in_size_environment_twice (`{small}`).
      tokens.extend(mouth::tokenize_internal(TeXString::assembled(format!(
        r"\expandafter\ifx\csname lx@mung@saved@{tag}@done\endcsname\relax\expandafter\let\csname lx@mung@saved@{tag}@done\endcsname\@empty\expandafter\let\csname lx@mung@saved@end@{tag}\expandafter\endcsname\csname end{tag}\endcsname\expandafter\def\csname end{tag}\endcsname{{\saved@endthebibliography\csname lx@mung@saved@end@{tag}\endcsname}}\fi"
      ))).unlist());
    }
    // Then open the kernel's bibliography: not `\thebibliography`, which a class's `\let` may have made its own list
    // (the loop above). Opened by the `\bibitem` alone — not by an author's `{thebibliography}` — it is headed as the
    // PDF heads it, not with `\refname` (`head_implicit_bibliography`).
    tokens.extend(mouth::tokenize_internal(TeXString::assembled(
      r"\let\lx@mung@saved@bibliography\bibliography\let\lx@mung@saved@bibliographystyle\bibliographystyle".to_string(),
    )).unlist());
    tokens.push(T_CS!("\\lx@mung@bibliography@pre"));
    tokens.push(T_BEGIN!());
    if tag != "thebibliography" {
      tokens.extend(Explode!("implicit "));
    }
    // …and the list that environment opened is closed first, when the `\bibitem` is in it: Kluwer's
    // `\normallatexbib` `{chapthebibliography}` is `\section*{References}`, a `\list` and an `\item[]` the entries are
    // typeset in (cargese.cls:1859-1890; kapproc.cls, edbk.sty alike), so the bibliography opened inside that item —
    // `<ltx:section>`/`<ltx:bibliography> isn't allowed in <ltx:item>`, Perl's too (KNOWN_PERL_ERRORS #887); its entries
    // are the bibliography's, as in the PDF. The class's `\endlist` then finds it closed (`\endlx@list` closes only
    // what is open). Witnesses hep-th0212109 (cargese), cond-mat0112379 cond-mat0110649 (kapproc), cond-mat0002157
    // (edbk); repro index-bib/kluwer_chapthebibliography; guard
    // `perfect_kernel_batch69a::kluwer_chapthebibliography_closes_its_list`.
    if !matches!(tag.as_str(), "" | "thebibliography" | "enumerate" | "itemize" | "description")
      && lookup_string("lx@list@currenvir") == tag
    {
      tokens.extend(Explode!("list "));
    }
    let continued = lookup_bool("lx_bibliography_sectioned");
    if continued {
      tokens.extend(Explode!("continued"));
    }
    tokens.push(T_END!());
    tokens.push(T_CS!("\\lx@kernel@thebibliography"));
    // The continuation is entered by `\lx@bibitem`, not by the `\bibitem` its setup armed (`setup_pseudo_bibitem`):
    // what that `\restoring@bibitem` undoes is undone here, or every later blank line or `\\` in an entry would open
    // an entry of its own (`\section*{References}\bibitem{a} A.,\\ J. 1.` and an erratum paragraph: one entry, as
    // without the heading; hep-th0109182 [17]).
    if continued {
      tokens.extend(mouth::tokenize_internal(TeXString::assembled(
        r"\let\bibitem\save@bibitem\let\par\save@par\let\\\save@backbackslash".to_string(),
      )).unlist());
    }
    // …whose setup (`before_digest_bibliography`, Perl latex_constructs.pool.ltxml:3839-3840) makes `\bibliography` and
    // `\bibliographystyle` do nothing, for the bibliography environment's group. Opened here without one of its own,
    // that would last to the end of the enclosing group — the document's — and drop the author's real bibliography
    // read later from the `.bbl` (Perl drops it too for the kernel's `\bibitem`); they are theirs again, so that
    // bibliography opens its own `{thebibliography}`, as pdflatex prints it. Repros
    // index-bib/natbib_bibitem_then_real_bibliography, index-bib/kernel_bibitem_then_real_bibliography.
    tokens.extend(mouth::tokenize_internal(TeXString::assembled(
      r"\let\bibliography\lx@mung@saved@bibliography\let\bibliographystyle\lx@mung@saved@bibliographystyle".to_string(),
    )).unlist());
    // A bibliography opened for a `\bibitem` outside `{thebibliography}` ends where the environment around it does (or
    // the document, or a heading), never at a bibliography list's own end: what that end does — natbib's
    // `\bibitem@fin` stop (natbib.sty:1080) — is not done (`lx_bibliography_no_list_end`). Guards
    // `perfect_kernel_batch64ze::natbib_lonely_entries_spend_no_stop`,
    // `perfect_kernel_batch64ze::natbib_lonely_entry_closed_by_its_environment_spends_no_stop`.
    if tag != "thebibliography" {
      tokens.push(T_CS!("\\lx@bibliography@nolistend"));
    }
    Ok(Tokens::new(tokens))
  });
  DefPrimitive!("\\lx@bibliography@nolistend", {
    assign_value("lx_bibliography_no_list_end", true, Some(Scope::Global));
  });
  // Perl: maybeCloseElement($tag) if tag =~ /^ltx:(?:itemize|enumerate|description)$/
  // The unit the `\bibitem` opens the bibliography in is marked: the PDF prints its heading over the entries, the
  // bibliography's own being LaTeXML's (`absorb_bibliography_heading`).
  DefConstructor!("\\lx@mung@bibliography@pre{}", sub[document, args, _props] {
    // whether the bibliography opened next is the `\bibitem`'s own (`head_implicit_bibliography`): set
    // afresh here, so one that failed to open is not taken for the next
    let how = args.first().and_then(|a| a.as_ref()).map(|a| a.to_string()).unwrap_or_default();
    let how: Vec<&str> = how.split_whitespace().collect();
    assign_value("lx_bibliography_opened_implicitly", how.contains(&"implicit"), Some(Scope::Global));
    assign_value("lx_bibliography_opened_continued", how.contains(&"continued"), Some(Scope::Global));
    let parent     = document.get_node();
    let tag_sym    = model::get_node_qname(parent);
    let closed = with(tag_sym, |tag|
      if tag == "ltx:itemize" || tag == "ltx:enumerate" || tag == "ltx:description" {
        document.maybe_close_element(tag)
      } else { Ok(None) }
    )?;
    // the environment's own list, from inside its item (`\lx@mung@bibliography`) — unless that list was the insertion
    // point, closed above (a list around it is not the environment's)
    if closed.is_none() && how.contains(&"list") {
      let mut node = Some(document.get_node().clone());
      while let Some(current) = node {
        let qname = model::get_node_qname(&current);
        if qname == pin!("ltx:itemize") || qname == pin!("ltx:enumerate") || qname == pin!("ltx:description") {
          document.close_node_internal(&current)?;
          // holding only the class's `\item[]`, it was the entries' frame alone
          if is_empty_block(&current) {
            document.remove_node(current);
          }
          break;
        }
        if !document::can_auto_close(&current) {
          break;
        }
        node = current.get_parent();
      }
    }
    let mut node = Some(document.get_node().clone());
    while let Some(current) = node {
      if is_heading_unit(&current) {
        let mut unit = current;
        document.set_attribute(&mut unit, "_bibliography_opened", "true")?;
        break;
      }
      node = current.get_parent();
    }
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

  // Perl latex_constructs.pool.ltxml:4182: DefConstructor('\@@cite []{}', ...,
  //   alias => '\cite', mode => 'restricted_horizontal', enterHorizontal => 1)
  // Named `\lx@@cite`, `\@@cite` its alias: documents save `\cite` as `\@@cite` (a name LaTeX leaves free), and
  // the bindings' `\cite`, calling `\@@cite`, then called itself without end (2305.06365, revtex4-2 + natbib,
  // `PushbackLimit`; Perl alike, KPE #476; OXIDIZED_DESIGN_DIVERGENCES #443).
  DefConstructor!("\\lx@@cite[]{}", "<ltx:cite ?#1(class='ltx_citemacro_#1')>#2</ltx:cite>",
    alias => "\\cite", mode => "text", enter_horizontal => true);
  Let!("\\@@cite", "\\lx@@cite");

  // \@@bibref{what to show}{bibkeys}{phrase1}{phrase2}
  // Perl latex_constructs.pool.ltxml L4244-4251: enterHorizontal => 1.
  DefConstructor!("\\@@bibref Semiverbatim Semiverbatim {}{}",
    "<ltx:bibref show='#1' bibrefs='#bibrefs' inlist='#bibunit' separator='#separator'
      yyseparator='#yyseparator'>#3#4</ltx:bibref>",
    enter_horizontal => true,
    properties => sub[args] {
      unref!(args => _show, keys, _phrase1, _phrase2);
      let bibrefs = clean_bib_key(&keys.to_string());
      cite_prints_keys(&bibrefs)?;
      Ok(stored_map!("bibrefs" => bibrefs,
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

    Ok(Invocation!(T_CS!("\\lx@@cite"),
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
  // latex.ltx:17786 `\nocite` opens with `\@bsphack` (:9276, `\relax`), where
  // a number scan ends; queued as it expands, the key was cited from a false `\ifnum0=1%`
  // branch (`peeks_by_futurelet`; guard
  // `sweep125_roots::binding_assignments_wait_for_a_number_scan`).
  DefMacro!("\\nocite{}", sub[args] {
    let key = args.first().map(|a| a.revert().unwrap_or_default()).unwrap_or_default();
    let key = do_expand_partially(key).unwrap_or_default();
    let mut toks = vec![T_CS!("\\lx@mark@nocite"), T_BEGIN!()];
    toks.extend(key.unlist());
    toks.push(T_END!());
    // Once `\end{document}` has read the list (`\lx@enddocument@hooks`), a `\nocite` its own hooks run
    // (wsemclassic.cls:302-306, :324 `\AtEndDocument{\makebib}` → `\nocite{*}`) marks its keys where it stands, as
    // LaTeX's `\citation` is written at once; pushed, it was never read (KNOWN_PERL_ERRORS #447; Perl alike).
    // Repro index-bib/nocite_in_atenddocument_cites.
    if lookup_bool("lx@enddocument@hooks@fired") {
      return Ok(Tokens::new(toks));
    }
    let _ = push_value("@at@end@document", Stored::Tokens(Tokens::new(toks)));
    Ok(Tokens!())
  }, peeks_by_futurelet => true);
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
  // The file name is expanded as `\set@curr@file` (latex.ltx) does it, inside a
  // `\csname` (`\ifincsname` true): underscore's first-aid `_` stays `_` there
  // (arXiv 2606.31852 `\input{sections/logic_T}`).
  DefPrimitive!("\\ltx@input {}", sub[(arg)] {
    Input!(&::latexml_core::gullet::expand_as_csname_text(arg)?.to_string());
  });
  DefMacro!("\\input", "\\@ifnextchar\\bgroup\\@iinput\\@@input");
  Let!("\\@iinput", "\\ltx@input");
  DefMacro!(
    "\\@input{}",
    "\\IfFileExists{#1}{\\@@input\\@filef@und}{\\typeout{No file #1.}}"
  );
  // Perl's `\@input@` (pool:4230) with `\typeout{No file #1.}` in the
  // missing-file branch; `\lx@makeindex@output` (latex_constructs_rust_only)
  // stands in for a missing makeindex output file there and falls back to
  // that `\typeout` for any other file. OXIDIZED_DESIGN_DIVERGENCES #318.
  DefMacro!(
    "\\@input@{}",
    "\\InputIfFileExists{#1}{}{\\lx@makeindex@output{#1}}"
  );

  DefMacro!("\\quote@name{}", "\"\\quote@@name#1\\@gobble\"\"");
  // latex.ltx:9665 `\def\quote@@name#1"{#1\quote@@name}`: the argument is delimited by the `"`, so the loop drops
  // every `"` of the name (`\quote@name{a"b}` = `"ab"`). Perl's `{} Match:"` (latex_constructs.pool.ltxml:4233) read one
  // token and then demanded a `"`: "Missing argument Match:\"" for every name longer than one token. latex.ltx:9702
  // `\@iffileonpath` quotes through it, and VTeX's arXiv configs `\let\IfFileExists\@iffileonpath` (1312.4083 213
  // errors, 2001.03064 76, 2006.14814 76, 1504.06523 63). KNOWN_PERL_ERRORS #924. Guard
  // `perfect_kernel_batch70b::quote_name_is_delimited_by_the_quote`.
  DefMacro!("\\quote@@name Until:\"", "#1\\quote@@name");
  DefMacro!("\\unquote@name{}", "\\quote@@name#1\\@gobble\"");

  // Perl L4313-4315: \include — input a file, respecting \includeonly
  DefPrimitive!("\\include{}", sub[(path)] {
    // latex.ltx:9557-9562: `\include` strips a `.tex` extension
    // (`\@strip@tex@ext`, :9577-9585) before `\@include` appends one, so
    // `\include{preamble.tex}` reads `preamble.tex`; appending blindly asked
    // for `preamble.tex.tex` ("No file"), and a whole preamble of amsmath,
    // siunitx, physics … never loaded (arXiv 2605.05505, 2605.24122 flooded
    // into `Fatal:TooManyErrors`). Guard:
    // `perfect_kernel_batch56::include_strips_a_tex_extension`.
    let expanded = ::latexml_core::gullet::expand_as_csname_text(path)?.to_string();
    let path_str = expanded.strip_suffix(".tex").unwrap_or(&expanded).to_string();
    // Check if \includeonly restricts inclusion
    let table = lookup_value("including@only");
    let should_include = match table {
      None => true, // no \includeonly — include everything
      Some(Stored::HashString(map)) => map.contains_key(&path_str),
      _ => true,
    };
    if should_include {
      // latex.ltx:9557→9612→9730: `\include` reaches the file through
      // `\@input@{#1.tex}` = `\InputIfFileExists{…}{}{\typeout{No file …}}`,
      // so a missing chapter file is a note, not an error (uiucthesis
      // thesis-ex ships without its `\include`d chapters; pdflatex prints
      // "No file 1-introduction.tex."). Perl's `\include` opens it hard
      // (SHARED). Guard: `perfect_kernel_batch56::sweep46_single_name_gaps`.
      unread(Tokenize!(TeXString::assembled(format!(
        "\\InputIfFileExists{{{path_str}.tex}}{{}}{{\\typeout{{No file {path_str}.tex.}}}}"
      ))));
    }
  });

  // Perl L4303-4311: \includeonly — restrict which files \include loads
  DefPrimitive!("\\includeonly{}", sub[(paths)] {
    let paths_str = Expand!(paths).to_string();
    let mut map = rustc_hash::FxHashMap::default();
    for part in paths_str.split(',') {
      // latex.ltx:9565-9575: each entry is `\@strip@tex@ext`ed too.
      let trimmed = part.trim();
      let trimmed = trimmed.strip_suffix(".tex").unwrap_or(trimmed).to_string();
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
  // The style is the makeindex encap STRING, read back from the argument's
  // tokens: digested as text, the braces of an encap with arguments
  // (hypdoc's `hdpindex{main}`, hypdoc.sty:344-353) went through the OT1 slots
  // 0x7B/0x7D as `hdpindex–main˝`. Perl's string is the token's own.
  // Perl (latex_constructs.pool.ltxml:4409-4412) digests the entry in a neutral font and reverts
  // it to nothing: a formula holding an `\index` kept `{\@index{\@indexphrase{…}}}` in its tex=.
  // The neutral font keeps the text encoding (an index is set in the document's: a T1 entry's `<`
  // read `¡` in OT1, as in Perl).
  // The entry is the string `\@wrindex`/`\@wrglossary` write, read under `\@sanitize`
  // (latex.ltx:17720-17740, :1778), where `&` is an ordinary character: it is `\&` here, as natbib
  // lets it for its labels (natbib.sty.ltxml:632) — robustglossary's `formula&explanation` entries
  // (robustsample.tex:45/:59) raised `Stray alignment "&"` and lost the `&`. And it is one brace
  // level in, as the argument's braces put it (tex.web §358 `align_state`): in an enclosing
  // alignment's cell its `&` is no column end (a column end is the character, gullet.rs
  // `is_column_end`, so the `\&` meaning alone let `\index{x&y}` in an `align` end the cell and
  // splice its template into the entry). A tabular inside the entry still ends its own cells.
  // Guard: `perfect_kernel_batch59::index_entry_ampersand_is_literal`.
  // `inlist` is the list's NAME, from its source tokens (`type_name`): the entry digests in the document's encoding,
  // which under LGR reads a digested `persons` back in Greek letters.
  DefConstructor!("\\@index[][]{}", "^<ltx:indexmark style='#style' inlist='#list'>#3</ltx:indexmark>",
    before_digest => {
      let encoding = LookupFont!().and_then(|f| f.get_encoding().map(|e| e.to_string()));
      neutralize_font();
      if let Some(encoding) = encoding {
        MergeFont!(encoding => encoding);
      }
      Let!(T_ALIGN!(), T_CS!("\\&"));
      increment_align_group_count();
    },
    after_digest => {
      decrement_align_group_count();
    },
    properties => sub[args] {
      let style = args[0].as_ref()
        .map(|a| a.revert().unwrap_or_default().to_string())
        .unwrap_or_default();
      let mut props = if style.is_empty() { stored_map!() } else { stored_map!("style" => style) };
      if let Some(list) = args[1].as_ref().map(type_name).filter(|list| !list.is_empty()) {
        props.insert("list", list.into());
      }
      Ok(props)
    },
    bounded => true,
    mode => "restricted_horizontal",
    reversion => "",
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
  // The entry is typeset only where the index is printed (user ruling 2026-10-18, OXIDIZED_DESIGN_DIVERGENCES #738):
  // TeX writes it raw to the `.idx` (latex.ltx:17713-17726) and typesets nothing without makeindex's `.ind`, so an
  // entry's `_` (`\index{API!SELF_HEALING}`) is no TeX error until `\printindex` prints it (1112.3972). The entry is kept
  // at its place (`\lx@deferred@print`) and digested when `\printindex`/`{theindex}` prints the list, its mark then
  // where `\index` stood. Guards: `perfect_kernel_batch73a::{index_entry_is_typeset_only_when_printed,
  // printed_index_keeps_its_entries}`.
  DefMacro!("\\index SanitizedVerbatim", sub[(phrases)] {
    let (entry, formed) = reread_sanitized_entry(&Tokens::new(phrases.revert()));
    let chars = IndexChars::for_index_entry(&entry);
    Ok(deferred_print_tokens("idx", process_index_phrases(entry, &chars, None, &formed)?))
  });
  // The anchor of a deferred entry (`deferred_print_tokens`): what the entry digested to when its list was printed, at
  // the place the entry was made; nothing when it never was.
  DefConstructor!("\\lx@deferred@print Semiverbatim", sub[document, args, _props] {
    let id = args.first().and_then(|a| a.as_ref()).map(|a| a.to_string()).unwrap_or_default();
    if let Some(content) = deferred_print_content(id.trim()) {
      document.absorb(&content, None)?;
    }
  },
    reversion => "",
    sizer => 0);
  // `\lx@defer@print{list}{entry}`: the TeX-level `deferred_print_tokens`, for a binding's entry (nomencl_sty.rs).
  DefMacro!("\\lx@defer@print{}{}", sub[(list, entry)] {
    Ok(deferred_print_tokens(list.to_string().trim(), entry))
  });
  // `\lx@deferred@print@list{list}`: print the list — digest its deferred entries (`resolve_deferred_prints`).
  DefPrimitive!("\\lx@deferred@print@list{}", sub[(list)] {
    resolve_deferred_prints(Some(list.to_string().trim()))?;
  });

  // `\lx@index@inlist{list}{phrases}` — `\index` into the named list `list` (imakeidx's `\index[name]{…}`,
  // imakeidx_sty.rs): its marks are collected by an `ltx:index` listing `list`, not by the main index.
  DefMacro!("\\lx@index@inlist{} SanitizedVerbatim", sub[(list, phrases)] {
    let list = list.to_string();
    let (entry, formed) = reread_sanitized_entry(&Tokens::new(phrases.revert()));
    let chars = IndexChars::for_index_entry(&entry);
    Ok(deferred_print_tokens(list.trim(), process_index_phrases(entry, &chars, Some(list.trim()), &formed)?))
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
    // The index is printed: its deferred entries are typeset (OXIDIZED_DESIGN_DIVERGENCES #738).
    resolve_deferred_prints(Some("idx"))?;
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
    if let Some(title) = digest_in_group_if(T_CS!("\\indexname"))? {
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

  // `\glossary` writes a makeindex entry exactly as `\index` does —
  // latex.ltx's `\@wrglossary` is `\@wrindex` onto the `.glo` file — so it
  // is read and split the same way, into an invisible `ltx:indexmark` in
  // list `glo` (the file extension names the list, as `idx`, `toc` and
  // `lof` do). A makeindex stand-in's `<ltx:index lists="glo">` collects the
  // marks (`\lx@makeindex@output`, post MakeIndex). Perl (pool:4424-4436)
  // inserts an `ltx:glossaryphrase` only outside `ltx:p`/`ltx:text` and
  // otherwise DROPS the entry with `Warning:unexpected:glossary`: every
  // doc.sty `\changes` (doc.sty:626-650) was lost that way (source2e alone
  // 3,518 warnings, about 70 TeX Live manuals: abntex2, the biblatex-* style
  // manuals, pixelart…, where the document-level insertion had also erred
  // `malformed:ltx:glossaryphrase isn't allowed in <ltx:document>`), and the
  // arXiv papers that call `\glossary` in running text (cs/9809003,
  // math/9608214, nucl-th/9311001) warned once per call. KNOWN_PERL_ERRORS #281,
  // OXIDIZED_DESIGN_DIVERGENCES #318.
  // Until `\makeglossary` (latex.ltx:17727-17736; doc.sty's `\RecordChanges`)
  // allocated `\@glossaryfile`, `\glossary` is latex.ltx:17742's
  // `\@bsphack\begingroup\@sanitize\@index`: the entry is read and dropped
  // (`\@index`, :17726), never typeset — the arXiv papers above call none, and
  // guitar.dtx:397-399's `\changes` text `\protected@edef`s to `\def{\relax}`
  // (repro `index/glossary_digests_argument_guitar.tex`). After it, the entry is typeset only where the glossary is
  // printed (the `.gls` stand-in, `\lx@makeindex@output`), as an `\index` entry is (OXIDIZED_DESIGN_DIVERGENCES #738,
  // main-session call 2026-10-18 under user ruling 1): latex.ltx:17737-17741 `\@wrglossary` writes it raw to the
  // `.glo`. Guard: `perfect_kernel_batch73a::glossary_entry_is_typeset_only_when_printed`.
  DefMacro!("\\glossary SanitizedVerbatim", sub[(phrases)] {
    if !is_defined_token(&T_CS!("\\@glossaryfile")) {
      return Ok(Tokens!());
    }
    let (entry, formed) = reread_sanitized_entry(&Tokens::new(phrases.revert()));
    Ok(deferred_print_tokens("glo", process_index_phrases(entry, &IndexChars::in_force(), Some("glo"), &formed)?))
  });

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
  // PARTIAL expansion, as Perl's `\typeout ExpandedPartially`
  // (latex_constructs.pool.ltxml:4538) and as the `\edef` of latex.ltx's
  // `\typeout`: a `\protected` command is written, not run. Full expansion ran
  // the peek-based argument grabber of an xparse `s` command
  // (`\NewDocumentCommand\foo{s}{…\IfBooleanTF{#1}…}`) in an expansion-only
  // context, handing `\IfBooleanTF` an empty argument — expl3's expandable
  // `cmd/if-boolean` error, silent here (`\??? Match:?` under the strict
  // delimiter check). Guard `perfect_kernel_batch56::typeout_leaves_protected_commands_unexpanded`.
  DefPrimitive!("\\typeout{}", sub[(stuff)] {
    bgroup();
    let_i(&T_CS!("\\protect"), &T_CS!("\\string"), None);
    let content = do_expand_partially(stuff)?;
    egroup()?;
    Note!(s!("{content}"));
  });
  def_primitive_noop("\\typein[]{}")?;

  Ok(())
}

/// Record, globally as `lx@bibliography@normalfont`, the font the document's `\normalfont` selects where
/// `\bibliography` stands — latex.ltx's `\fontencoding\encodingdefault`, `\familydefault`, `\seriesdefault`,
/// `\shapedefault` (:14113-14121), each expanded now — as the `\normalfont` body, `\fontencoding{…}\lx@normalfont@base
/// \fontfamily{…}\fontseries{…}\fontshape{…}\selectfont` (sect13.rs), the `.bib` session (bib_session.rs) digests its
/// entries in. That session runs after the document, in its live state, so its own
/// `\normalfont` read the defaults of the document's END: cmpj3.sty's `\ukrainianpart` after the bibliography
/// (`\renewcommand\familydefault{\sfdefault}`, 2603.23615) set the entries sans where pdflatex's bibliography,
/// typeset in place, is serif. OXIDIZED_DESIGN_DIVERGENCES #779; guard
/// `06_cluster_bibliography::bib_entries_take_the_defaults_where_the_bibliography_stands`.
fn record_bibliography_normalfont() -> Result<()> {
  let mut tokens = Vec::new();
  for (default, select) in [
    ("\\encodingdefault", "\\fontencoding"),
    ("\\familydefault", "\\fontfamily"),
    ("\\seriesdefault", "\\fontseries"),
    ("\\shapedefault", "\\fontshape"),
  ] {
    if lookup_definition(&T_CS!(default))?.is_none() {
      return Ok(());
    }
    let code = Expand!(T_CS!(default)).to_string();
    tokens.push(T_CS!(select));
    tokens.push(T_BEGIN!());
    tokens.extend(Explode!(code.trim()));
    tokens.push(T_END!());
    if select == "\\fontencoding" {
      // as `\normalfont` (sect13.rs): an unknown family resets to LaTeXML's serif, not the font in force
      tokens.push(T_CS!("\\lx@normalfont@base"));
    }
  }
  tokens.push(T_CS!("\\selectfont"));
  assign_value(
    "lx@bibliography@normalfont",
    Tokens::new(tokens),
    Some(Scope::Global),
  );
  Ok(())
}

/// A labelled node answers, for each role whose tags are held (`type_tag_deferred`, `\lx@tag@intags@held`; ruling 7e),
/// for the tag a reference to its labels prints: its own tag of that role, else the nearest ancestor's — CrossRef's
/// walk up for a show word the labelled entry lacks (`\section*` under a numbered chapter prints the chapter's).
/// Every label counts, however it was set (`\lx@label`, longtable's `labels=`, a panel merge), as the node closes.
/// The held diagnostics are raised if a reference shows the role (logger.rs `replay_shown_deferred`). Witness
/// biblatex-gost-examples.
fn tie_held_tags(node: &Node, labels: &str) {
  let roles: Vec<String> = with_mapping_keys("type_tag_deferred", |keys| {
    keys.into_iter().map(to_string).collect()
  });
  if roles.is_empty() {
    return;
  }
  // A node's element children, walked lazily (an ancestor may be wide: the document root).
  let elements = |parent: &Node| {
    std::iter::successors(parent.get_first_child(), |child| child.get_next_sibling())
      .filter(|child| child.get_type() == Some(NodeType::ElementNode))
  };
  for role in &roles {
    let mut cursor = Some(node.clone());
    while let Some(current) = cursor {
      let tag = elements(&current)
        .filter(|child| child.get_name() == "tags")
        .find_map(|tags| {
          elements(&tags).find(|tag| tag.get_attribute("role").as_deref() == Some(role))
        });
      if let Some(tag) = tag {
        if let Some(id) = tag
          .get_attribute("_deferred")
          .and_then(|id| id.parse().ok())
        {
          for label in labels.split_whitespace() {
            util::logger::defer_to_label(label, role, id);
          }
        }
        break;
      }
      cursor = current.get_parent();
    }
  }
}

/// The units a bibliography may take the heading of: those whose model takes `BackMatter.class`
/// (LaTeXML-structure.rnc:69-163), which a bibliography closes as section-level back matter
/// (`adjust_backmatter_element`) — an `\appendix`'s unit too (`\appendix \section*{References}` then the entries in
/// a `{quote}`, astro-ph0503283).
const BIBLIOGRAPHY_HEADED_UNITS: &[&str] = &[
  "ltx:part",
  "ltx:chapter",
  "ltx:appendix",
  "ltx:section",
  "ltx:subsection",
  "ltx:subsubsection",
  "ltx:paragraph",
  "ltx:subparagraph",
];

fn is_heading_unit(node: &Node) -> bool {
  with(model::get_node_qname(node), |q| {
    BIBLIOGRAPHY_HEADED_UNITS.contains(&q)
  })
}

fn is_heading_part(node: &Node) -> bool {
  with(model::get_node_qname(node), |q| {
    matches!(q, "ltx:tags" | "ltx:title" | "ltx:toctitle")
  })
}

/// The text of a unit's title without its number tag, spaces collapsed, a closing `.`/`:` and case ignored; empty
/// when it has none.
fn heading_text(unit: &Node) -> String {
  let Some(title) = unit
    .get_child_elements()
    .into_iter()
    .find(|c| with(model::get_node_qname(c), |q| q == "ltx:title"))
  else {
    return String::new();
  };
  let mut text = String::new();
  for child in title.get_child_nodes() {
    if !with(model::get_node_qname(&child), |q| q == "ltx:tag") {
      text.push_str(&child.get_content());
    }
  }
  let text = text.split_whitespace().collect::<Vec<_>>().join(" ");
  text.trim_end_matches(['.', ':']).to_lowercase()
}

/// A paragraph or list with no text and nothing but paragraphs and lists inside: Pandoc's `{CSLReferences}` list
/// whose `\bibitem`s opened the bibliography, a `\phantomsection`'s paragraph.
fn is_empty_block(node: &Node) -> bool {
  with(model::get_node_qname(node), |q| {
    matches!(
      q,
      "ltx:para" | "ltx:p" | "ltx:itemize" | "ltx:enumerate" | "ltx:description" | "ltx:item"
    )
  }) && node.get_content().trim().is_empty()
    && node.get_child_elements().iter().all(is_empty_block)
}

/// A bibliography right after a unit that heads it takes that unit's place (user 2026-10-06), else the document has
/// two headings, the first over nothing: the author's `\section{References}` over the `\bibitem`s that open the
/// bibliography — Pandoc's citeproc `{CSLReferences}` (2609.02899), `\bibitem`s in an `{enumerate}` (Perl the same,
/// KNOWN_PERL_ERRORS #512) — or before `{thebibliography}` (OXIDIZED_DESIGN_DIVERGENCES #456). The unit heads it when
/// titled as the bibliography is; or, holding nothing else, when the bibliography has no title
/// (`\renewcommand\refname{}`) or the `\bibitem`s opened the bibliography in it — the PDF prints that heading over
/// them, not `\refname`. Its heading (tags, title, toctitle) and labels replace the bibliography's own, its other content goes
/// before the entries, and the unit is removed. Whether it did.
fn absorb_bibliography_heading(document: &mut Document, bibliography: &Node) -> Result<bool> {
  if bibliography
    .get_attribute("class")
    .is_some_and(|class| class.split_whitespace().any(|c| c == "ltx_nodisplay"))
  {
    return Ok(false);
  }
  // The unit just before the bibliography in document order: its previous sibling, or that one's last unit, …
  let Some(mut unit) = bibliography.get_prev_element_sibling() else {
    return Ok(false);
  };
  if !is_heading_unit(&unit) {
    return Ok(false);
  }
  while let Some(last) = unit.get_last_element_child()
    && is_heading_unit(&last)
  {
    unit = last;
  }
  let unit_title = heading_text(&unit);
  if unit_title.is_empty() {
    return Ok(false);
  }
  let (heading, rest): (Vec<Node>, Vec<Node>) = unit
    .get_child_elements()
    .into_iter()
    .partition(is_heading_part);
  let (emptied, body): (Vec<Node>, Vec<Node>) = rest.into_iter().partition(is_empty_block);
  let bibliography_title = heading_text(bibliography);
  let opened_here = unit.get_attribute("_bibliography_opened").is_some();
  // Titled as the bibliography, the unit heads it with what it holds; else only when it holds nothing else, as a
  // `\section{Conclusion}` with its text before an untitled bibliography stays the conclusion.
  if !(unit_title == bibliography_title
    || ((bibliography_title.is_empty() || opened_here) && body.is_empty()))
  {
    return Ok(false);
  }
  if !body
    .iter()
    .all(|n| document::can_contain_node(bibliography, n))
  {
    return Ok(false);
  }
  for own in bibliography
    .get_child_elements()
    .into_iter()
    .filter(is_heading_part)
  {
    document.remove_node(own);
  }
  // after the bibliography's front matter (an abstract, a date), where its model has the heading
  let anchor = bibliography
    .get_child_elements()
    .into_iter()
    .find(|child| !is_front_matter(child));
  let mut target = bibliography.clone();
  for mut node in heading.into_iter().chain(body) {
    node.unlink();
    match &anchor {
      Some(anchor) => {
        anchor.clone().add_prev_sibling(&mut node)?;
      },
      None => {
        target.add_child(&mut node)?;
      },
    }
  }
  // the unit's labels, those in the paragraphs it is left with, and the bibliography's own
  let mut labels: Vec<String> = unit.get_attribute("labels").into_iter().collect();
  for block in &emptied {
    collect_labels(block, &mut labels);
  }
  labels.extend(bibliography.get_attribute("labels"));
  if !labels.is_empty() {
    document.set_attribute(&mut target, "labels", &labels.join(" "))?;
  }
  document.remove_node(unit);
  Ok(true)
}

/// The bibliography a heading inside the author's `{thebibliography}` interrupted before any entry
/// (`\lx@bibliography@sectioned`): `\begin{thebibliography}{99}\section*{References}\bibitem…`, the revtex4/4-1 papers'
/// shape (1702.06834, 2402.15720, 2110.15342). pdflatex prints the environment's heading and then the author's, the
/// same twice; the bibliography that took the author's heading (`absorb_bibliography_heading`) is the one, and the
/// first, holding no entry and nothing but its heading, goes, its labels with the second. Kept when its heading is
/// another's than the author's (a "References" over the topical `\section*`s of hep-th0109182): the PDF prints both.
fn prune_interrupted_bibliography(document: &mut Document, bibliography: &Node) -> Result<()> {
  // the element before it in document order, past the heading of a unit it opens (a frame's, 1301.0681)
  let mut current = bibliography.clone();
  let previous = loop {
    match current.get_prev_element_sibling() {
      Some(previous) if is_heading_part(&previous) => current = previous,
      Some(previous) => break previous,
      None => match current.get_parent() {
        Some(parent) if is_heading_unit(&parent) => current = parent,
        _ => return Ok(()),
      },
    }
  };
  if document::get_node_qname(&previous) != pin!("ltx:bibliography")
    || !document
      .findnodes("descendant::ltx:bibitem", Some(&previous))
      .is_empty()
    || !previous.get_child_elements().iter().all(|child| {
      is_heading_part(child)
        || (with(model::get_node_qname(child), |q| q == "ltx:biblist")
          && child.get_content().trim().is_empty())
    })
  {
    return Ok(());
  }
  // (titles compared as `absorb_bibliography_heading` compares a unit's with the bibliography's: case, spaces and a
  // closing `.`/`:` ignored — the ruling of 2026-10-06 that a unit titled as the bibliography heads it)
  let title = heading_text(&previous);
  if !title.is_empty() && title != heading_text(bibliography) {
    return Ok(());
  }
  let mut labels: Vec<String> = Vec::new();
  collect_labels(&previous, &mut labels);
  labels.extend(bibliography.get_attribute("labels"));
  let mut target = bibliography.clone();
  if !labels.is_empty() {
    document.set_attribute(&mut target, "labels", &labels.join(" "))?;
  }
  let freed = previous.get_attribute_ns("id", XML_NS);
  document.remove_node(previous);
  // …and its id: the continuation took its number (`begin_bibliography_clean`), its entries `bib.bibN`, but met the
  // id still taken when it was built (`biba`, Document.pm:1447-1456 recordID's modifyID); freed (:1460 unRecordID),
  // it is the survivor's, the one bibliography's `bib` (OXIDIZED_DESIGN_DIVERGENCES #653).
  if let Some(freed) = freed
    && let Some(own) = target.get_attribute_ns("id", XML_NS)
    && own != freed
    && document
      .findnodes("descendant::ltx:bibitem[@xml:id]", Some(&target))
      .first()
      .and_then(|entry| entry.get_attribute_ns("id", XML_NS))
      .is_some_and(|id| id.starts_with(&s!("{freed}.")))
  {
    document.unrecord_id(&own);
    let _ = target.remove_attribute_ns("id", XML_NS);
    document.set_attribute(&mut target, "xml:id", &freed)?;
  }
  Ok(())
}

/// The phantom entries (OXIDIZED_DESIGN #155, the `ltx:bibitem` note above) anywhere inside `root` — wherever the
/// `\par` that made one put it: in a list, in a float or table inside the bibliography (astro-ph0102416), in an entry's
/// tag (0904.3343, physics0603210), in an entry whose block it opened inside (2301.05139: 20, 0801.2060, 1007.2457),
/// after the bibliography (2403.05651, swept at the document's close). Last first, so an entry holding only phantoms is
/// one when it is reached.
fn prune_phantom_entries(document: &mut Document, root: &Node) {
  let entries = document.findnodes("descendant::ltx:bibitem", Some(root));
  for entry in entries.into_iter().rev() {
    if is_phantom_entry(document, &entry) {
      document.remove_node(entry);
    }
  }
}

/// A phantom entry: no key, no label of the author's (`\bibitem[Baker(2001)]{}` prints its label: `_labelled`, kernel
/// and natbib), and blocks (none at all, too) holding no text, only font runs (a `{\small` group before the first
/// `\bibitem`, 0801.2060) and page breaks — keyless entries are the norm in an aas `\references`, and one holding a
/// figure panel (astro-ph0102416: 12 of Figure 2's, astro-ph9902080 `heating.ps`), a formula, a rule, a reference, a
/// citation, a line break or an error mark is content.
fn is_phantom_entry(document: &mut Document, entry: &Node) -> bool {
  if entry
    .get_attribute("key")
    .is_some_and(|k| !k.trim().is_empty())
    || entry.has_attribute("_labelled")
    // anything but its tags and blocks (a float the `\par` opened it in, malformed) is not the phantom's to remove
    || !entry.get_child_elements().iter().all(|c| {
      with(model::get_node_qname(c), |q| q == "ltx:tags" || q == "ltx:bibblock")
    })
  {
    return false;
  }
  let blocks: Vec<Node> = entry
    .get_child_elements()
    .into_iter()
    .filter(|c| document::get_node_qname(c) == pin!("ltx:bibblock"))
    .collect();
  blocks.iter().all(|block| {
    block.get_content().trim().is_empty()
      && document
        .findnodes("descendant::*", Some(block))
        .iter()
        .all(|n| {
          with(model::get_node_qname(n), |q| {
            q == "ltx:text" || q == "ltx:pagination"
          })
        })
  })
}

/// A list auto-opened in the bibliography only for a phantom entry — the keyless `\save@bibitem{}` that a blank line or
/// `\\` after a float makes (`\par@in@bibliography`), pruned (`prune_phantom_entries`) — is left with
/// nothing, and goes with it (astro-ph0307431 left 21 empty lists, 0803.0740 14), when the bibliography closes.
/// The list the bibliography itself opens (`\thebibliography`'s `<ltx:biblist>`) is not auto-opened and stays, empty or
/// not.
fn remove_emptied_lists(document: &mut Document, root: &Node) {
  for list in document.findnodes("descendant::ltx:biblist", Some(root)) {
    if is_emptied_list(&list) {
      document.remove_node(list);
    }
  }
}

/// An auto-opened `ltx:biblist` with no entry left (`remove_emptied_lists`).
fn is_emptied_list(list: &Node) -> bool {
  with(model::get_node_qname(list), |q| q == "ltx:biblist")
    && list.has_attribute("_autoopened")
    && list.get_child_elements().is_empty()
}

/// A bibliography a `\bibitem` opened outside any bibliography (`\lx@mung@bibliography`, kernel and natbib alike) that no
/// unit heads (`absorb_bibliography_heading`) is headed as the PDF heads it, not with LaTeXML's `\refname` (user ruling
/// 2026-10-10; Perl always titles it `\refname`): the author's heading line just before it (`heading_line_run`: "{\bf
/// References}", 2003.07967, 2306.17386, 0806.0831; "Figure Captions", hep-ph9804413, hep-ph9804388) becomes its title and
/// leaves the body, else it has no visible title and no table-of-contents entry (quant-ph9706037, 1708.06608; pdflatex
/// lists nothing for it). The `ltx:bibliography` stays either way, and the lists its `\bibitem`s were taken from,
/// emptied, go (their labels to it).
fn head_implicit_bibliography(document: &mut Document, bibliography: &Node) -> Result<()> {
  // the paragraph before it in document order, past the lists the `\bibitem`s were taken from (emptied) and into the
  // unit it closed (the heading line ends a `\section`: 2003.07967, 2306.17386)
  let mut emptied = Vec::new();
  let mut previous = bibliography.get_prev_element_sibling();
  while let Some(node) = previous.clone() {
    if is_empty_block(&node) {
      previous = node.get_prev_element_sibling();
      emptied.push(node);
    } else if is_heading_unit(&node) {
      previous = node.get_last_element_child();
    } else {
      break;
    }
  }
  // …and the emptied lists ending that paragraph (`Text.\begin{enumerate}\bibitem…`)
  if let Some(paragraph) = &previous
    && with(model::get_node_qname(paragraph), |q| q == "ltx:para")
  {
    let mut last = paragraph.get_last_element_child();
    while let Some(node) = last
      && is_empty_block(&node)
    {
      last = node.get_prev_element_sibling();
      emptied.push(node);
    }
  }
  let own_title = bibliography
    .get_child_elements()
    .into_iter()
    .find(|c| with(model::get_node_qname(c), |q| q == "ltx:title"));
  let heading = previous
    .and_then(|paragraph| heading_line_run(document, &paragraph).map(|run| (paragraph, run)));
  let mut labels: Vec<String> = Vec::new();
  for block in emptied {
    collect_labels(&block, &mut labels);
    document.remove_node(block);
  }
  if let Some((paragraph, run)) = heading
    && let Some(mut title) = own_title
  {
    for own in bibliography
      .get_child_elements()
      .into_iter()
      .filter(|c| is_heading_part(c) && *c != title)
    {
      document.remove_node(own);
    }
    for old in title.get_child_nodes() {
      document.remove_node(old);
    }
    // the author's heading now, not the bibliography's own (`drop_own_bibliography_heading`)
    let _ = title.remove_attribute("_own_title");
    document.append_clone(&mut title, run.get_child_nodes())?;
    collect_labels(&paragraph, &mut labels);
    document.remove_node(paragraph);
  } else {
    drop_bibliography_heading(document, bibliography)?;
  }
  labels.extend(bibliography.get_attribute("labels"));
  if !labels.is_empty() {
    let mut target = bibliography.clone();
    document.set_attribute(&mut target, "labels", &labels.join(" "))?;
  }
  Ok(())
}

/// A bibliography the PDF prints no heading over: its heading parts (tags, title, toctitle) go, and so does its entry in
/// the TOC (`inlist="toc"`).
fn drop_bibliography_heading(document: &mut Document, bibliography: &Node) -> Result<()> {
  for own in bibliography
    .get_child_elements()
    .into_iter()
    .filter(is_heading_part)
  {
    document.remove_node(own);
  }
  let mut target = bibliography.clone();
  let lists: Vec<String> = target
    .get_attribute("inlist")
    .unwrap_or_default()
    .split_whitespace()
    .filter(|l| *l != "toc")
    .map(str::to_string)
    .collect();
  if lists.is_empty() {
    let _ = target.remove_attribute("inlist");
  } else {
    document.set_attribute(&mut target, "inlist", &lists.join(" "))?;
  }
  Ok(())
}

/// A continuation's own title (`_own_title`, `\thebibliography`), when it is still its title: it goes, with the TOC entry
/// it made. A title that came from elsewhere (an author's heading) stays.
fn drop_own_bibliography_heading(document: &mut Document, bibliography: &Node) -> Result<()> {
  let own = bibliography.get_child_elements().into_iter().any(|child| {
    with(model::get_node_qname(&child), |q| q == "ltx:title")
      && child.get_attribute("_own_title").is_some()
  });
  if own {
    drop_bibliography_heading(document, bibliography)?;
  }
  Ok(())
}

/// The heading line of the ruling (user 2026-10-10; "short heading-like styled line" as the coordinator read it): a
/// paragraph whose only content is one run styled apart from the paragraph's text (`heading_styled`) of at most four
/// words — ending in a full stop only as one word ("References.", not `{\bf This work was supported by NSF.}`) — with
/// no math, citation or reference inside (only nested text runs), beside nothing but emptied lists; the run, if so. By
/// structure alone, never by its words.
fn heading_line_run(document: &Document, paragraph: &Node) -> Option<Node> {
  if !with(model::get_node_qname(paragraph), |q| q == "ltx:para") {
    return None;
  }
  let mut lines = paragraph
    .get_child_elements()
    .into_iter()
    .filter(|c| !is_empty_block(c));
  let line = lines.next()?;
  if lines.next().is_some() || !with(model::get_node_qname(&line), |q| q == "ltx:p") {
    return None;
  }
  let mut run = None;
  for child in line.get_child_nodes() {
    match child.get_type() {
      Some(NodeType::ElementNode) => {
        if run.replace(child).is_some() {
          return None;
        }
      },
      Some(NodeType::TextNode) if !child.get_content().trim().is_empty() => return None,
      _ => {},
    }
  }
  let run = run?;
  let text = run.get_content();
  let words = text.split_whitespace().count();
  (with(model::get_node_qname(&run), |q| q == "ltx:text")
    && (1..=4).contains(&words)
    && (words == 1 || !text.trim_end().ends_with('.'))
    && heading_styled(document, &run, paragraph)
    && only_text_runs(&run))
  .then_some(run)
}

/// Whether `run` is set apart from its paragraph's text as a heading is: bold, small capitals, `\large` or larger, or
/// underlined — not italic (an `\emph`). Fonts are resolved at the document's finalization, after this close: the
/// nodes' own.
fn heading_styled(document: &Document, run: &Node, paragraph: &Node) -> bool {
  let font = document.get_node_font(run);
  let body = document.get_node_font(paragraph);
  let is = |value: Option<&Cow<'_, str>>, style: &str| value.is_some_and(|v| v.contains(style));
  let bold = is(font.get_series(), "bold") && !is(body.get_series(), "bold");
  let small_caps = is(font.get_shape(), "smallcaps") && !is(body.get_shape(), "smallcaps");
  // `\large` is 1.2 times the 10pt body (size10.clo), `\Large` 1.44
  let larger =
    matches!((font.get_size(), body.get_size()), (Some(run), Some(text)) if run >= text * 1.15);
  let underlined = run
    .get_attribute("class")
    .is_some_and(|class| class.split_whitespace().any(|c| c == "ltx_underline"));
  bold || small_caps || larger || underlined
}

/// Whether every element inside `node` is a text run (`ltx:text`): no math, citation, reference or other markup.
fn only_text_runs(node: &Node) -> bool {
  node
    .get_child_elements()
    .iter()
    .all(|c| with(model::get_node_qname(c), |q| q == "ltx:text") && only_text_runs(c))
}

/// Is `node` of the front matter a unit's model takes before its heading (`FrontMatter.class`,
/// LaTeXML-structure.rnc:672-674)?
fn is_front_matter(node: &Node) -> bool {
  with(model::get_node_qname(node), |q| {
    matches!(
      q,
      "ltx:subtitle"
        | "ltx:date"
        | "ltx:abstract"
        | "ltx:acknowledgements"
        | "ltx:keywords"
        | "ltx:classification"
        | "ltx:pubnote"
    )
  })
}

/// The `labels` of `node` and the elements inside it.
fn collect_labels(node: &Node, labels: &mut Vec<String>) {
  labels.extend(node.get_attribute("labels"));
  for child in node.get_child_elements() {
    collect_labels(&child, labels);
  }
}

/// Whether the bibliography being digested holds nothing yet: back from where the heading is to the bibliography's
/// `\thebibliography`, through the lists enclosing it (`with_box_list_levels`: a beamer frame's own), no entry
/// (`\lx@bibitem`, natbib's `\lx@NAT@lbibitem`: their key) and no text — at most the empty entries the recovery opened
/// for a blank line or the heading's own `\par` (`\par@in@bibliography`), which are pruned. Not found, it holds
/// something.
fn holds_nothing_yet() -> bool {
  with_box_list_levels(|levels| {
    for item in levels.iter().flat_map(|level| level.iter().rev()) {
      if let DigestedData::Whatsit(ws) = item.data()
        && ws.borrow().get_definition().get_cs().to_string() == "\\thebibliography"
      {
        return true;
      }
      if !carries_nothing(item) {
        return false;
      }
    }
    false
  })
}

/// Digested material that puts nothing in the document: white space, an empty box, a whatsit with no argument and no
/// body (the frontmatter fallback a heading digests before its `\@startsection@hook`, an entry opened with nothing in it)
/// or one that makes no box — the kernel's `alignmentSkippable` whatsits: `\label` (latex_constructs.pool.ltxml:3782-3792,
/// whose labels the pruned bibliography hands on, `prune_interrupted_bibliography`), a mark, a tabbing marker. A
/// `\label{refs}` before the heading keeps the one bibliography's `bib`/`bib.bibN` (OXIDIZED_DESIGN_DIVERGENCES #653).
fn carries_nothing(item: &Digested) -> bool {
  match item.data() {
    DigestedData::TBox(tbox) => {
      let tbox = tbox.borrow();
      tbox.get_property_bool("alignmentSkippable") || tbox.to_string().trim().is_empty()
    },
    DigestedData::List(list) => list
      .borrow()
      .unlist_ref()
      .iter()
      .all(|inner| carries_nothing(inner)),
    DigestedData::Whatsit(ws) => {
      let ws = ws.borrow();
      ws.get_property_bool("alignmentSkippable")
        || ws.is_empty().unwrap_or(false)
        || (ws
          .get_args()
          .iter()
          .all(|arg| arg.as_ref().is_none_or(|a| a.is_empty().unwrap_or(false)))
          && ws.get_body().ok().flatten().is_none())
    },
    DigestedData::Comment(_) => true,
    _ => false,
  }
}

/// The deferred-print list of the entry `key` (latexml_core `defer_print`): TeX's `\b@key`, the label `\@lbibitem`
/// writes to the .aux (latex.ltx:17746-17749 `\bibcite`) for a `\cite` to print.
fn cite_list(key: &str) -> String { s!("b@{key}") }

/// A citation prints its keys' labels (latex.ltx:17761-17771 `\@citex` reads each `\b@key`): the forms of each entry's
/// tags a reference prints, deferred by `\lx@bibitem` until then, are typeset (`resolve_deferred_prints`), and an entry
/// that comes later typesets them at once. Every citation reaches this through `\@@bibref` (the kernel's `\cite`,
/// natbib's, biblatex's, the `.bbl` styles'); `\nocite` prints nothing and does not (`\lx@mark@nocite`).
/// OXIDIZED_DESIGN_DIVERGENCES #750.
fn cite_prints_keys(bibrefs: &str) -> Result<()> {
  for key in bibrefs.split(',').filter(|key| !key.is_empty()) {
    resolve_deferred_prints(Some(&cite_list(key)))?;
  }
  Ok(())
}

/// The tokens of a trailing `\label[type]{key}` a site moves into a caption (`\@caption@postlabel`, sect09.rs;
/// caption's `\@captionof@postlabel`; algorithm2e's procedure caption), which read the optional type themselves:
/// `<label>{<key>}`, after `\lx@label@type{<type>}` when a type was given (cleveref.sty `\label[type]`,
/// KNOWN_PERL_ERRORS #998). An absent or blank type adds no token to the caption's text, which other code re-reads
/// and scans.
pub fn typed_label(label: Token, ty: Option<Tokens>, key: Tokens) -> Vec<Token> {
  let mut tokens = label_type(ty);
  tokens.extend([label, T_BEGIN!()]);
  tokens.extend(key.unlist());
  tokens.push(T_END!());
  tokens
}

/// `\lx@label@type{<type>}` for a `\label[type]` a site reads itself (`typed_label`; longtable's label, which names
/// the table it ends), or nothing for an absent or blank type.
pub fn label_type(ty: Option<Tokens>) -> Vec<Token> {
  let mut tokens = Vec::new();
  if let Some(ty) = ty.filter(|ty| {
    ty.unlist_ref()
      .iter()
      .any(|t| t.get_catcode() != Catcode::SPACE)
  }) {
    tokens.extend([T_CS!("\\lx@label@type"), T_BEGIN!()]);
    tokens.extend(ty.unlist());
    tokens.push(T_END!());
  }
  tokens
}
