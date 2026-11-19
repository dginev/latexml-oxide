use latexml_package::prelude::*;

LoadDefinitions!({
  // An archival conversion never stamps the conversion's day. Perl L23-25 makes `\today` print nothing:
  //   AtBeginDocument(sub {
  //     DefMacroI('\today', undef, '\relax', locked => 1, scope => 'global');
  //   });
  // Beyond Perl (user ruling 2026-10-10): the job's clock is the paper's own date instead — the newest modification
  // time among the TeX source files of its bundle (`latexml::source_date`) — so LaTeX's and babel's `\today` print
  // the date arXiv's PDF shows, and a class parsing `\today` gets a date: ptapap.cls:322-325 splits `\edef`'d `\today`
  // with `\def\next#1#2#3#4\relax`, which on `\relax` ran away to "Paragraph ended before \next was complete" (run
  // 336, 1801.05985); 1811.05851's `\item[] \today` prints "November 14, 2018". Only the bundle's date counts, never
  // `SOURCE_DATE_EPOCH` (exported by every Nix shell and Debian build, often as 1980-01-01): an undated bundle keeps
  // Perl's `\relax` whatever that variable says, which then only seeds tex_job's registers. `\pdfcreationdate` reads
  // the same registers. The clock is set before the format loads (`RequirePackage!("latexml")` below): the l3kernel
  // copies it into `\c_sys_year_int` &c. at the job's start (`\g__sys_everyjob_tl`), which must agree with `\today`.
  // (Perl's begin-document `\relax` leaves the preamble reading the conversion's day: its ptapap papers print the
  // conversion's month.)
  let archival_date =
    source_date_epoch().is_some_and(latexml_engine::tex_job::assign_date_registers);

  // Perl: PassOptions('latexml', 'sty', ...) + RequirePackage('latexml')
  // Mirror Perl ar5iv.sty.ltxml: pass `rawstyles` (INCLUDE_STYLES => true,
  // kpsewhich enabled, system-wide texmf reachable). Earlier the Rust
  // port passed `localrawstyles` (kpsewhich suppressed, only paper-local
  // SEARCHPATHS) per past user direction (commit 9869267eb), but that
  // diverged from the Perl ar5iv profile and caused real parity gaps:
  // papers using system-installed-but-unbound .sty packages
  // (colonequals, comment, gnuplot, …) loaded fine in Perl but errored
  // in Rust with `\<missing-cs> undefined`. Switch back to `rawstyles`
  // for Perl-baseline parity (cf. feedback_sandbox_perl_baseline.md).
  // Perl ar5iv.sty.ltxml ships `pushbacklimit=599999, iflimit=3999999`
  // but the Rust port hits both ceilings on real ar5iv-profile papers
  // (witness arXiv:2605.16752v1). Empirical bisect (2026-05-22) on that
  // witness pinned the actual minima at: pushback ≈ 630000, iflimit
  // ≈ 8000000.
  //
  // ROOT-CAUSED 2026-06-30: the iflimit gap to Perl is NOT a bug to
  // "tighten away" — it is *deliberate, more-comprehensive runaway
  // counting*. Rust defines `\ifx`/`\ifcsname` (and the other low-level
  // TeX conditionals) as real `DefConditional`s that increment the global
  // `if_count` runaway guard; Perl does NOT count `\ifx`/`\ifcsname` toward
  // its `if_count` at all. On pgfkeys-driven tikz/pgfplots input (both
  // engines raw-load the real `pgfkeys.code.tex` — Perl's native pgfkeys
  // override is `__END__`-disabled), the gap is enormous: a controlled
  // 2-plot pgfplots figure that BOTH engines render identically (≈86 vs 87
  // graphic nodes) counts **148,078** conditionals in Rust vs **<200** in
  // Perl (≈740×), dominated by `\ifx` (63%) + `\ifcsname` (15%) inside the
  // pgfkeys key-dispatch. Counting these is *correct* — it is exactly what
  // the guard is for (a `\ifx`/`\ifcsname` runaway is invisible to Perl's
  // counter). So the right response is to raise the limit, not to count
  // less. Real *finite* heavy docs (multi-figure pgfplots papers) measure
  // ≈10–15M conditionals and complete in ≈24–43 s; a genuine runaway is
  // was then caught well before the worker's per-document timeout (at 16M:
  // ≈350k cond/s ⇒ ≈46 s ≪ the 180 s `--timeout`; the dispatcher lease is
  // 240 s) and by the RSS fuse. **iflimit raised
  // 8M → 16M (4× Perl's 3,999,999)** to recover coverage on the ≈17-paper
  // `\tikz@dashphase` Timeout cluster that Perl cannot convert at all
  // (Perl chokes on these papers' expl3 first). Pre-approved 2026-06-30.
  // **Raised 16M → 48M (62w)**: pgfplots/mhchem papers measured (release
  // build) 19M (2609.10563, 48 s), 21M (2609.16075, 45 s) and 39M (2609.07725,
  // 78 s) and complete without the limit, as do 2605.27177 (56 s) and 2605.04377
  // (98 s). Any runaway still stops at the 180 s timeout; the limit only ends a
  // faster one sooner, with the `IfLimit` label: one between ≈89k and ≈267k
  // cond/s (16M and 48M over 180 s) now ends at the timeout instead — ≈140
  // IfLimit papers per full pass, so the fleet time is negligible. 2609.30783 (a
  // memory runaway in lirseg) now ends at the box-list memory budget
  // (`Fatal:Stomach:MemoryBudget`, which rides the memory cap; was `IfLimit`).
  //
  // pushbacklimit RAISED 650000 → 5000000 (the binary's own default,
  // latexml_oxide.rs, and TeX's `main_memory`, one word per token,
  // texmf.cnf:820): a macro expansion copies its whole body into the flat
  // pushback (gullet.rs `unread_expansion`), where TeX pushes a pointer to the
  // token list (tex.web §323 `begin_token_list`), so a finite long list trips a
  // limit TeX has no counterpart for. pgf expands a whole soft path at once
  // (pgfsyssoftpath.code.tex:66-75, 94-98, 122-131): a 9000-sample smooth plot
  // is ≈650K tokens (tkz-grapheur's plots have 11,000). Perl with ar5iv stops on
  // that plot too (`Fatal:timeout:pushback_limit 599999`, Gullet.pm:300-306),
  // but not on `\edef\a{\a\a}` ×17, since its `readXToken`/`readBalanced`
  // never check the limit. The limits are pragmatic heuristics that let a large
  // manuscript convert (user ruling 2026-09-26); the cost is that a pushback
  // runaway stops later (a pgfmath-per-iteration loop: 2.3 s → 14.9 s), still
  // bounded by this limit and the timeout.
  //
  // tokenlimit RECALIBRATED 2026-06-10 (PR #249 review P1-2): the gullet
  // read checkpoints now count in all three reader loops (was: read_token
  // only), so the old 249999999 — calibrated under the old accounting —
  // silently shrank by the multi-counting factor. Measured heaviest
  // known-good ar5iv-profile paper under the new accounting: math0402448 at
  // 80.2M (`Info:gullet:progress`). 999999999 keeps the canvas profile's
  // generous backstop posture (runaways are cut much earlier by the cycle
  // guards / pushbacklimit / byte budget; the tokenlimit only bounds
  // aperiodic grind).
  // `rawclasses` (latexml.sty's option; passing it here is Rust-only, OXIDIZED_DESIGN_DIVERGENCES #444): a class
  // covered by no binding is interpreted raw, as TeX would, instead of falling to OmniBus's guesses — a class shipped
  // with the paper (62k, then `localrawclasses`: webofc 1301.7514, pasj00 0704.3654, raa 0802.3215, cms-tdr
  // 1010.5994; run 329: ~35% of the erroring papers ran on OmniBus over a class they ship) and, since 67v, a class in
  // TeX Live (paper, ptptex, nature, jpsj2, apa6, jacow, smfart, artikel1, …: 3,362 run-353 papers fell to OmniBus;
  // raw, error-free 1,485 → 3,079). Every binding, exact or alternate, still comes first (`load_class`).
  //
  // RUST-ONLY (OXIDIZED_DESIGN_DIVERGENCES #884): the class caption fonts knob (caption_sty.rs `CLASS_CAPTION_FONTS`, off
  // by default) as ar5iv.sty's own option, handed to latexml.sty with the options below, as a LaTeX package would
  // (`\DeclareOption{classcaptionfonts}{\PassOptionsToPackage{\CurrentOption}{latexml}}`):
  // `--preload=[classcaptionfonts]ar5iv.sty` sets the knob and keeps ar5iv's options. A second preload of latexml.sty
  // cannot: a preload's options go to `opt@<name>.<ext>` only (core_interface.rs, the preload loop), so the repeat load
  // has no new options to apply (content.rs `apply_repeat_options`) — `[classcaptionfonts]latexml.sty` after ar5iv.sty
  // sets no knob, and before it latexml.sty takes its options alone and ar5iv's are lost (`rawclasses`: a class read
  // raw under ar5iv falls to OmniBus, `Warning:missing_file`). Perl alike (Core.pm:333-335, Package.pm:2357,
  // :2586-2590); docs/SYNC_STATUS.md. Guard `perfect_kernel_batch78z::ar5iv_classcaptionfonts_option_sets_the_knob`.
  // So are latexml.sty's output-mode options (`pdfoutput`, `nopdfoutput`; OXIDIZED_DESIGN_DIVERGENCES #1046).
  for option in [
    "classcaptionfonts",
    "noclasscaptionfonts",
    "pdfoutput",
    "nopdfoutput",
  ] {
    DeclareOption!(option, {
      pass_options("latexml", "sty", vec![option.to_string()])?;
    });
  }
  ProcessOptions!();
  pass_options("latexml", "sty", vec![
    s!("ids"),
    s!("rawstyles"),
    s!("rawclasses"),
    s!("bibconfig=bbl,bib"),
    s!("nobreakuntex"),
    s!("magnify=1.2"),
    s!("zoomout=1.2"),
    s!("tokenlimit=999999999"),
    s!("iflimit=48000000"),
    s!("absorblimit=1299999"),
    s!("pushbacklimit=5000000"),
  ])?;
  RequirePackage!("latexml");

  // arXiv's build loads hyperref into every paper (a 00README `nohypertex` opts out), so a paper that uses `\url` or
  // `\href` without loading url.sty or hyperref builds there, while pdflatex alone stops at "Undefined control
  // sequence" (~1,300 papers in run 349: cond-mat0407333, cs0409028, hep-th0411273; 2101.07380's PDF says "LaTeX with
  // hyperref" though its source never loads it). Provide-only (user ruling 2026-10-14): at `\begin{document}` the one of
  // the two still undefined is defined as hyperref's, with url.sty's default `\UrlFont`/`\UrlLeft`/`\UrlRight` where
  // missing; a paper that loads either package, or defines the command itself in its preamble, keeps its own.
  // hyperref loads url.sty, so url.sty follows, provide-only too: its `\path`, `\urlstyle`, `\urldef`, `\Url` for the
  // names still undefined. An elsarticle-num `.bbl` defines `\path` only when `\href` is undefined
  // (`\expandafter\ifx\csname href\endcsname\relax \def\href#1#2{#2} \def\path#1{#1}\fi`); with `\href` supplied,
  // its `{\path{doi:…}}` entries were undefined (2605.07373, 2605.08166, 2605.12003).
  // OXIDIZED_DESIGN #588; guard `perfect_kernel_batch67a`.
  DefPrimitive!("\\lx@arxiv@provide@hyper@urls", {
    // a preamble use left an Error's stub, which is no definition of the paper's (the reviewer's S1)
    let undefined = |cs: &str| {
      let token = T_CS!(cs);
      if is_error_stub(&token) {
        remove_error_stub(&token);
      }
      !IsDefined!(&token)
    };
    let href = undefined("\\href");
    let url = undefined("\\url");
    if href || url {
      if !IsDefined!(&T_CS!("\\UrlFont")) {
        DefMacro!("\\UrlFont", "\\lx@fontencoding{ASCII}\\ttfamily");
      }
      if !IsDefined!(&T_CS!("\\UrlLeft")) {
        Let!("\\UrlLeft", "\\@empty");
      }
      if !IsDefined!(&T_CS!("\\UrlRight")) {
        Let!("\\UrlRight", "\\@empty");
      }
      hyperref_sty::define_hyper_urls(href, url)?;
      let _provide_only = definition::origin::ProvideOnlyGuard::new();
      // A provided `\href` answers a package's "is hyperref loaded?" (`\@ifundefined{href}`), so hyperref's no-op
      // anchor layer it then calls comes with it (a shipped algorithm2e.sty 3.6's `\nl`, cs0602002).
      if href {
        definition::origin::with_origin(
          definition::origin::DefinitionOrigin::Binding,
          hyperref_sty::define_hyper_anchors,
        )?;
      }
      RequirePackage!("url");
    }
  });
  at_begin_document(Tokens!(T_CS!("\\lx@arxiv@provide@hyper@urls")))?;

  // The rest of arXiv's hyperref a paper relies on unloaded is provided on first use, provide-only (DG ruling
  // 2026-11-19): nothing is defined until an undefined name of a family is met, preamble or body, so `\ifdefined`,
  // `\@ifundefined` and a paper's own `\newcommand` see what pdflatex without the package sees
  // (`kernel_autoload::name_first_use_provider`). The families: hyperref's `\texorpdfstring`, which printed both its
  // arguments (hep-th0702068's section titles); hyperref's user commands (math0202134 and hep-th0308186 `\hypersetup`,
  // 0807.4242 `\hyperlink`/`\hypertarget`, 0803.2070 `\autoref`); and the atbegshi hyperref 6.77a-7.01m loaded
  // (lpor2014.cls:2987 in 1503.07687, A&A and mnras preambles 1601.07542, 1605.02911; 2309.06611, 1607.05658).
  // OXIDIZED_DESIGN #588.
  for (provider, define, names) in [
    (
      "\\lx@arxiv@provide@texorpdfstring",
      hyperref_sty::define_texorpdfstring as fn() -> Result<()>,
      &["\\texorpdfstring"][..],
    ),
    (
      "\\lx@arxiv@provide@atbegshi",
      define_atbegshi_hooks,
      ATBEGSHI_COMMANDS,
    ),
    (
      "\\lx@arxiv@provide@hyperref",
      define_hyperref_user_commands,
      HYPERREF_USER_COMMANDS,
    ),
  ] {
    def_macro(
      T_CS!(provider),
      None,
      ExpansionBody::Closure(Rc::new(move |_args| {
        // arXiv adds hyperref to LaTeX papers only: a plain TeX paper's `\hyperref` stays undefined (hep-th0404136)
        if !lookup_bool("LaTeX.pool_loaded") {
          return Ok(Tokens!());
        }
        // global, as a package loaded in the preamble: the first use may be inside a group (a section title)
        let globaldefs = lookup_value("\\globaldefs").unwrap_or(Stored::Number(Number!(0)));
        assign_value(
          "\\globaldefs",
          Stored::Number(Number!(1)),
          Some(Scope::InPlace),
        );
        // inside a provide-only load (a fallback class's own first use) its guard is kept
        let _provide_only =
          (!definition::origin::provide_only()).then(definition::origin::ProvideOnlyGuard::new);
        let defined =
          definition::origin::with_origin(definition::origin::DefinitionOrigin::Binding, define);
        assign_value("\\globaldefs", globaldefs, Some(Scope::InPlace));
        defined?;
        Ok(Tokens!())
      })),
      None,
    )?;
    for name in names {
      binding::kernel_autoload::name_first_use_provider(&T_CS!(*name), T_CS!(provider));
    }
  }

  // Practical maximum for warnings
  AssignValue!("MAX_WARNINGS" => 10000i64, Scope::Global);

  // No date in the source (stdin, a single-file submission's undated entry): Perl's `\relax`.
  // We bind at load time with `locked => true, Scope::Global` instead of
  // wrapping in `\AtBeginDocument{\def\today{\relax}}` (which loses both
  // flags — `\def` is plain-TeX, with no LaTeXML lock). The lock rejects a
  // preamble package's later (re)definition of \today, matching the intent of
  // Perl's AtBeginDocument hook (defer until all packages have loaded). It
  // does NOT make timing irrelevant: binding loads run unlocked
  // (content.rs, state.rs), so the LaTeX format's own `\today`
  // (latex_constructs sect03.rs) would replace this one if it loaded later.
  // It loads earlier today, inside the latexml.sty preload (its body's
  // `\AddToHook` autoloads LaTeX.pool); a change that defers the format load
  // to `\documentclass` must move this to begin-document, as Perl does
  // (repro loader/latexml_preload_keeps_plain.tex).
  if !archival_date {
    DefMacro!("\\today", "\\relax", locked => true, scope => Some(Scope::Global));
  }

  // Perl L30-35: drop all non-remote <ltx:resource> nodes (keep only `http*`
  // src so the archival run doesn't embed default local CSS / JS).
  //   DefRewrite(xpath => 'descendant-or-self::ltx:resource', replace => sub {
  //     my ($self, $node) = @_;
  //     my $src = $node->getAttribute('src') || '';
  //     return if $src !~ /^http/;      # non-remote → silently drop
  //     $self->getNode->appendChild($node); });   # remote → re-attach
  // Beyond Perl: a style sheet given inline, with no file (`\lxRequireResource[type=text/css,content={…}]{}`), is
  // kept too: it embeds nothing local, and is how a paper styles its own HTML, the arXiv source having no room for a
  // CSS file (2606.12996's rules for its nested tables and appendix TOC entries); so is a class option's own rules
  // (article's `openbib`, `.ltx_bibblock{display:block;}`). Inline scripts stay dropped.
  DefRewrite!(xpath => "descendant-or-self::ltx:resource",
  replace => sub[document, nodes] {
    let node = nodes.pop().unwrap();
    let src = node.get_attribute("src").unwrap_or_default();
    let inline_css = src.is_empty()
      && node.get_attribute("type").as_deref() == Some("text/css")
      && !node.get_content().trim().is_empty();
    if src.starts_with("http") || inline_css {
      document.get_node_mut().add_child(node)?;
    }
  });

  // NOTE: Perl additionally monkey-patches LaTeXML::Post::MathML::outerWrapper
  // to set intent=':literal' on the top-level math element. That is a
  // post-processing hook (not a compile-time binding), tracked separately —
  // we do not emulate it here. See Perl source L45-73 for context.
});

/// hyperref's user commands provided on first use (with `\nameref`, from the nameref hyperref loads).
const HYPERREF_USER_COMMANDS: &[&str] = &[
  "\\hypersetup",
  "\\phantomsection",
  "\\autoref",
  "\\hyperref",
  "\\hypertarget",
  "\\hyperlink",
  "\\nameref",
  "\\pdfbookmark",
  "\\currentpdfbookmark",
  "\\subpdfbookmark",
  "\\belowpdfbookmark",
];

/// The binding's definitions of [`HYPERREF_USER_COMMANDS`], hyperref itself not loaded.
fn define_hyperref_user_commands() -> Result<()> {
  hyperref_sty::define_hypersetup()?;
  hyperref_sty::define_hyper_links()?;
  hyperref_sty::define_hyper_targets()?;
  hyperref_sty::define_autoref()?;
  hyperref_sty::define_autoref_names()?;
  hyperref_sty::define_pdfbookmarks()?;
  nameref_sty::define_nameref()
}

/// atbegshi's user commands (atbegshi-ltx.sty 2021/01/10 v1.0c :52-71), provided on first use.
const ATBEGSHI_COMMANDS: &[&str] = &[
  "\\AtBeginShipout",
  "\\AtBeginShipoutNext",
  "\\AtBeginShipoutFirst",
  "\\AtBeginShipoutAddToBox",
  "\\AtBeginShipoutAddToBoxForeground",
  "\\AtBeginShipoutUpperLeft",
  "\\AtBeginShipoutUpperLeftForeground",
  "\\AtBeginShipoutInit",
  "\\AtBeginShipoutDiscard",
  "\\AtBeginShipoutBox",
  "\\AtBeginShipoutOriginalShipout",
  "\\ShipoutBoxHeight",
  "\\ShipoutBoxDepth",
  "\\ShipoutBoxWidth",
];

/// atbegshi's user commands with the shipout they hook into absent. What pdflatex prints on a page — a page's
/// upper-left picture (`\AtBeginShipoutUpperLeft…`), its background or foreground box (`\AtBeginShipoutAddToBox…`), the
/// first page's material (`\AtBeginShipoutFirst`), directly or inside the material of `\AtBeginShipout` and `…Next` — is
/// the paper's page notice (2309.06611's IEEE copyright, 2403.15239's conference line, 2302.00362's "Preprint of the
/// paper which appeared in: …") and is kept once as a publication note; the rest of the hook material is dropped
/// unrun. `\AtBeginShipoutBox` is the kernel's `\ShipoutBox`, `\AtBeginShipoutInit` and `\AtBeginShipoutDiscard` do
/// nothing, the shipout box's dimensions are 0pt.
fn define_atbegshi_hooks() -> Result<()> {
  DefMacro!("\\AtBeginShipoutUpperLeft{}", "\\lx@arxiv@shipout@note{#1}");
  DefMacro!(
    "\\AtBeginShipoutUpperLeftForeground{}",
    "\\lx@arxiv@shipout@note{#1}"
  );
  DefMacro!("\\AtBeginShipoutAddToBox{}", "\\lx@arxiv@shipout@note{#1}");
  DefMacro!(
    "\\AtBeginShipoutAddToBoxForeground{}",
    "\\lx@arxiv@shipout@note{#1}"
  );
  DefMacro!("\\AtBeginShipout{}", sub[(material)] { Ok(Tokens::new(shipout_notes(&material).0)) });
  DefMacro!("\\AtBeginShipoutNext{}", sub[(material)] { Ok(Tokens::new(shipout_notes(&material).0)) });
  // (the first page's material is printed itself, beside the notes it holds)
  DefMacro!("\\AtBeginShipoutFirst{}", sub[(material)] {
    let (mut notes, rest) = shipout_notes(&material);
    notes.push(T_CS!("\\lx@arxiv@shipout@note"));
    notes.push(T_BEGIN!());
    notes.extend(rest);
    notes.push(T_END!());
    Ok(Tokens::new(notes))
  });
  // The note carries the words, not the page geometry: the material is set in a group where `\put(x,y){…}` keeps only
  // its `{…}` (atbegshi's `picture(0,0)` is not set up), and material with a picture (a `tikzpicture` overlay, a
  // `picture`) gives its text alone. Material printing no text (a rule, an image, 1612.04433's tikz highlighter) and
  // material already noted (a hook registered again, or once per page) make no note.
  DefPrimitive!("\\lx@arxiv@shipout@note{}", sub[(material)] {
    let seen = s!("lx@arxiv@shipout@note:{}", material.to_string());
    if lookup_bool(&seen) {
      return Ok(Vec::new());
    }
    AssignValue!(&seen => true, Scope::Global);
    let mut body = TokenizeInternal!(r"\hbox{\unitlength=1pt\relax\let\put\lx@arxiv@shipout@put").unlist();
    body.extend(material.unlist_ref().iter().cloned());
    body.push(T_END!());
    let stuff = digest(Tokens::new(body))?;
    let mut text = String::new();
    ink_text_printed(&stuff, &mut text);
    let words = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if !words.chars().any(char::is_alphanumeric) {
      return Ok(Vec::new());
    }
    let source = material.to_string();
    let content = if ["\\tikz", "\\pgfpicture", "picture}"].iter().any(|p| source.contains(p)) {
      ExplodeText!(words)
    } else {
      let n = lookup_int("lx_arxiv_shipout_notes") + 1;
      assign_value("lx_arxiv_shipout_notes", Stored::Int(n), Some(Scope::Global));
      assign_value(&s!("lx_arxiv_shipout_note_{n}"), Stored::Digested(stuff), Some(Scope::Global));
      mouth::tokenize_internal(TeXString::assembled(s!("\\lx@arxiv@shipout@box{{{n}}}"))).unlist()
    };
    let mut note = TokenizeInternal!(r"\lx@add@pubnote[role=note,name={}]{").unlist();
    note.extend(content);
    note.push(T_END!());
    unread(Tokens::new(note));
    Ok(Vec::new())
  });
  DefMacro!(
    "\\lx@arxiv@shipout@put SkipSpaces Match:( Until:, Until:) {}",
    "#4"
  );
  // (the `\hbox` `\lx@arxiv@shipout@note` digested, its list given where the note is digested)
  DefPrimitive!("\\lx@arxiv@shipout@box{}", sub[(n)] {
    let Some(Stored::Digested(stuff)) = lookup_value(&s!("lx_arxiv_shipout_note_{}", n.to_string().trim())) else {
      return Ok(Vec::new());
    };
    let items = stuff.unlist();
    Ok(match items.as_slice() {
      [hbox] => match hbox.data() {
        DigestedData::Whatsit(w) => w.borrow().get_arg(2).map(Digested::unlist).unwrap_or_default(),
        _ => items,
      },
      _ => items,
    })
  });
  Let!("\\AtBeginShipoutInit", "\\@empty");
  Let!("\\AtBeginShipoutDiscard", "\\@empty");
  if !IsDefined!(&T_CS!("\\AtBeginShipoutBox")) {
    if IsDefined!(&T_CS!("\\ShipoutBox")) {
      Let!("\\AtBeginShipoutBox", "\\ShipoutBox");
    } else {
      raw_tex("\\newbox\\AtBeginShipoutBox")?;
    }
  }
  if IsDefined!(&T_CS!("\\shipout")) {
    Let!("\\AtBeginShipoutOriginalShipout", "\\shipout");
  }
  DefMacro!("\\ShipoutBoxHeight", "0pt");
  DefMacro!("\\ShipoutBoxDepth", "0pt");
  DefMacro!("\\ShipoutBoxWidth", "0pt");
  Ok(())
}

/// A shipout hook's material split: the page notes it holds — a `\lx@arxiv@shipout@note{…}` for the argument of each
/// `\AtBeginShipoutUpperLeft…` / `\AtBeginShipoutAddToBox…` in it at any depth, an `\AtBeginShipoutFirst{…}` for each of
/// those — and the rest of it, every atbegshi hook taken out with its argument (so nothing is noted twice).
fn shipout_notes(material: &Tokens) -> (Vec<Token>, Vec<Token>) {
  let noted = [
    T_CS!("\\AtBeginShipoutUpperLeft"),
    T_CS!("\\AtBeginShipoutUpperLeftForeground"),
    T_CS!("\\AtBeginShipoutAddToBox"),
    T_CS!("\\AtBeginShipoutAddToBoxForeground"),
  ];
  let first = T_CS!("\\AtBeginShipoutFirst");
  let hooks = [T_CS!("\\AtBeginShipout"), T_CS!("\\AtBeginShipoutNext")];
  let tokens = material.unlist_ref();
  // (the end of the argument read from `i`: a balanced group or one token, spaces before it skipped)
  let argument = |mut i: usize| -> (usize, usize) {
    while i < tokens.len() && tokens[i].get_catcode() == Catcode::SPACE {
      i += 1;
    }
    let start = i;
    if i < tokens.len() && tokens[i].get_catcode() == Catcode::BEGIN {
      let mut level = 0;
      while i < tokens.len() {
        match tokens[i].get_catcode() {
          Catcode::BEGIN => level += 1,
          Catcode::END => level -= 1,
          _ => {},
        }
        i += 1;
        if level == 0 {
          break;
        }
      }
    } else if i < tokens.len() {
      i += 1;
    }
    (start, i)
  };
  let (mut notes, mut rest) = (Vec::new(), Vec::new());
  // (the hooks' arguments still open in `rest`: what they hold is dropped from it, their notes kept)
  let mut dropped_until = 0;
  let mut i = 0;
  while i < tokens.len() {
    let token = &tokens[i];
    if noted.contains(token) || *token == first {
      let (start, end) = argument(i + 1);
      notes.push(if *token == first {
        first
      } else {
        T_CS!("\\lx@arxiv@shipout@note")
      });
      notes.extend_from_slice(&tokens[start..end]);
      i = end;
      continue;
    }
    if hooks.contains(token) {
      dropped_until = dropped_until.max(argument(i + 1).1);
    } else if i >= dropped_until {
      rest.push(*token);
    }
    i += 1;
  }
  (notes, rest)
}
