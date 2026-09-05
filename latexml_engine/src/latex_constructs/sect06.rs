//! `latex_constructs` section 6: C.6 Displayed Paragraphs
//!
//! Split verbatim from the single `LoadDefinitions!` body (2026-09-03); the
//! definitions run in the original order from `super::load_definitions`.

use super::*;

#[rustfmt::skip]
pub(crate) fn load() -> Result<()> {
  // ======================================================================
  // C.6 Displayed Paragraphs
  // ======================================================================

  // `mode => "internal_vertical"`: LaTeX's `{center}`/`{flushleft}`/
  // `{flushright}` open a trivlist (vertical structure); the body's
  // `\par` switches into horizontal for paragraph text but BOUND_MODE
  // stays vertical so display-math recognition (`tex_math.rs:447`
  // mirror of `TeX_Math.pool.ltxml:65`: `$$` only consumed when
  // BOUND_MODE ends with "vertical") works inside the body. Perl
  // `latex_constructs.pool.ltxml:1262-1264` doesn't set `mode =>`
  // — anticipates an upstream Perl PR fleshing out the `mode =>`
  // machinery. Without this, papers using `\begin{center}$$X$$
  // \end{center}` lose the display math, the second `$` reads as
  // closing the first inline, and array content lands inside
  // `<ltx:p>` triggering schema violations. Driver: astro-ph0203201
  // (table*+center+$$+array).
  DefEnvironment!("{center}", sub[document, _args, props] {
    document.maybe_close_element("ltx:p")?; // this starts a new vertical block
    // aligning will take care of \\\\ "rows"
    aligning_environment("center", "ltx_centering", document, props)?;
    Ok(())
  }, mode => "internal_vertical");
  // HOWEVER, define a plain \center to act like \centering (?)
  DefMacro!("\\center", "\\centering");
  def_macro_noop("\\endcenter")?;
  // Perl latex_constructs.pool.ltxml L1208-1213: flushleft → aligningEnvironment(
  // 'left', 'ltx_align_left'); flushright → ('right', 'ltx_align_right'). The
  // earlier Rust port passed "center" as the align value for BOTH (a copy-paste
  // from {center} above), so flushleft/flushright produced `align="center"`
  // instead of `align="left"`/`align="right"`.
  DefEnvironment!("{flushleft}", sub[document, _args, props] {
    document.maybe_close_element("ltx:p")?; // this starts a new vertical block
    aligning_environment("left", "ltx_align_left", document, props)?;
    Ok(())
  }, mode => "internal_vertical");
  DefEnvironment!("{flushright}", sub[document, _args, props] {
    document.maybe_close_element("ltx:p")?; // this starts a new vertical block
    aligning_environment("right", "ltx_align_right", document, props)?;
    Ok(())
  }, mode => "internal_vertical");
  // Perl latex_constructs.pool.ltxml L1316-1318: "Redefine these so they work
  // both as environments, and as single commands". The bare `\flushleft` /
  // `\flushright` commands (without matching `\end...`) are used as
  // declarations — they should NOT push a group frame + enter
  // restricted_horizontal, since that would leak mode when the enclosing
  // group (e.g. `table*`) closes.
  //
  // `\begin{flushleft}` / `\end{flushleft}` go through a separate environment
  // constructor and are unaffected by these Let aliases.
  //
  // Fixes sandbox papers 0705.2808 and 0707.4170 (mode mismatch at
  // `\end{table*}` when document uses `\flushleft` as a command inside the
  // float body).

  // # These add an operation to be carried out on the current node & following siblings, when the
  // current group ends. # These operators will add alignment (class) attributes to each "line" in
  // the current block. #DefPrimitiveI('\centering',   undef, sub {
  // UnshiftValue(beforeAfterGroup=>T_CS('\@add@centering')); }); # NOTE: THere's a problem here.
  // The current method seems to work right for these operators # appearing within the typical
  // environments.  HOWEVER, it doesn't work for a simple \bgroup or \begingroup!!! # (they don't
  // create a node! or even a whatsit!)
  // Perl: setupAligningContext saves [node, node.lastChild] to ALIGNING_NODE.
  // applyAligningContext then only applies class to children AFTER the saved lastChild.
  // `\centering`/`\raggedright`/`\raggedleft` are MACROS in latex.ltx
  // (`\def\centering{\let\\\@centercr …}`, :16419-16433) — expandable — over
  // constructor cores. That matters for expl3's V-type expansion:
  // `\__exp_eval_register:N` (expl3-code.tex:2507-2517) tells a register
  // from a macro with `\exp_after:wN\if_meaning:w\exp_not:N #1 #1`, so a
  // NON-expandable `\let\raggedsignature=\centering` (DIN.lco:130) was taken
  // for a register and `\the`'d — scrlttr2.cls:5095 `\tl_if_in:nVTF {…}
  // \raggedsignature` in `\closing`: "You can't use \raggedsignature after
  // \the" (bfh-ci letter, SFSesim, makelabels ×2, scrlttr2copy; Perl's
  // constructors fail identically, pool:1237-1240).
  DefMacro!("\\centering", "\\lx@do@centering");
  DefMacro!("\\raggedright", "\\lx@do@raggedright");
  DefMacro!("\\raggedleft", "\\lx@do@raggedleft");
  DefConstructor!("\\lx@do@centering", sub[doc,_args] {
    setup_aligning_context(doc);
  },
  before_digest => {
    unshift_value("beforeAfterGroup", vec![T_CS!("\\@add@centering")]);
  });
  // Perl: latex_constructs.pool.ltxml lines 1299-1302
  DefConstructor!("\\lx@do@raggedright", sub[doc,_args] {
    setup_aligning_context(doc);
  },
    before_digest => {
      unshift_value("beforeAfterGroup", vec![T_CS!("\\@add@raggedright")]);
    });
  DefConstructor!("\\lx@do@raggedleft", sub[doc,_args] {
    setup_aligning_context(doc);
  },
    before_digest => {
      unshift_value("beforeAfterGroup", vec![T_CS!("\\@add@raggedleft")]);
    });

  DefConstructor!("\\@add@centering", sub[document] {
    apply_aligning_context(document, "center", "ltx_centering")?;
  });
  // Note that \raggedright is essentially align left (undef align, just class)
  DefConstructor!("\\@add@raggedright", sub[document] {
    apply_aligning_context(document, "", "ltx_align_left")?;
  });
  DefConstructor!("\\@add@raggedleft", sub[document] {
    apply_aligning_context(document, "", "ltx_align_right")?;
  });
  DefConstructor!("\\@add@flushright", sub[document] {
    let node_opt = lookup_value("ALIGNING_NODE");
    if let Some(Stored::Node(node)) = node_opt {
      for mut child in node.get_child_elements() {
        set_align_or_class(document, &mut child, "right", "ltx_align_right")?;
      }
    }
  });
  DefConstructor!("\\@add@flushleft", sub[document] {
    let node_opt = lookup_value("ALIGNING_NODE");
    if let Some(Stored::Node(node)) = node_opt {
      for mut child in node.get_child_elements() {
        set_align_or_class(document, &mut child, "left", "ltx_align_left")?;
      }
    }
  });

  // Perl latex_constructs.pool.ltxml L1317-1318: Redefine so `\flushleft` /
  // `\flushright` work both as environments AND as single commands.
  // As a command (no matching `\end...`), the bare CS acts like
  // `\raggedright` / `\raggedleft` — a declaration that applies via
  // beforeAfterGroup rather than opening a restricted_horizontal group
  // frame. `\begin{flushleft}` / `\end{flushleft}` still go through the
  // environment constructors and are unaffected.
  Let!("\\flushright", "\\raggedleft");
  Let!("\\flushleft", "\\raggedright");
  // …and their `\end…` partners are no-ops: the declarations open no frame,
  // while the kernel `\endflushleft` (latex.ltx `\endtrivlist`) would end a
  // list that was never begun — comment.sty's doc `noverb` env
  // (comment.tex:12-18 `\flushleft`…`\endflushleft`) and bidicode.sty:195/198
  // `BDef` (tram-doc): "Attempt to end mode internal_vertical". Perl aliases
  // only the openers (pool:1257-1258). `\begin{flushleft}` keeps its own
  // environment constructor.
  Let!("\\endflushright", "\\relax");
  Let!("\\endflushleft", "\\relax");

  // Perl: Let('\@block@cr', '\lx@newline');  # Obsolete, but in case still used
  Let!("\\@block@cr", "\\lx@newline");
  DefEnvironment!("{quote}",
    "<ltx:quote>#body</ltx:quote>",
    mode => "internal_vertical");
  DefEnvironment!("{quotation}",
    "<ltx:quote>#body</ltx:quote>",
    mode => "internal_vertical");
  DefEnvironment!("{verse}",
    "<ltx:quote role='verse'>#body</ltx:quote>",
    mode => "internal_vertical");

  //======================================================================
  // C.6.2 List-Making environments
  //======================================================================
  Tag!("ltx:item",        auto_close => true, auto_open => true);
  Tag!("ltx:inline-item", auto_close => true, auto_open => true);

  // These are for the (not quite legit) case where \item appears outside
  // of an itemize, enumerate, etc, environment.
  // DefCon('\item[]',
  //   "<ltx:item>?&defined(#1)(<ltx:tags><ltx:tag>#1</ltx:tag></ltx:tags>)");
  // DefCon('\subitem[]',
  //   "<ltx:item>?&defined(#1)(<ltx:tags><ltx:tag>#1</ltx:tag></ltx:tags>)");
  // DefCon('\subsubitem[]',
  //   "<ltx:item>?&defined(#1)(<ltx:tags><ltx:tag>#1</ltx:tag></ltx:tags>)");

  // Or maybe best just to do \par ?
  DefMacro!("\\item[]", "\\par");
  DefMacro!("\\subitem[]", "\\par");
  DefMacro!("\\subsubitem[]", "\\par");

  AssignValue!("@itemlevel" => 0, Some(Scope::Global));
  AssignValue!("enumlevel"  => 0, Some(Scope::Global));
  AssignValue!("@desclevel" => 0, Some(Scope::Global));
  // protection against lower-level code...
  DefConditional!("\\if@noitemarg");
  DefMacro!("\\@item", "\\item"); // Hopefully no circles...
  def_macro_noop("\\@itemlabel")?; // Maybe needs to be same as \item will be using?

  // These counters are ONLY used for id's of ALL the various itemize, enumerate, etc elements
  // Only create the 1st level (so that binding style can start numbering 'within' appropriately)
  // Additional ones created by need.
  NewCounter!("@itemizei",   "section",      idprefix => "I");

  // Perl latex_constructs.pool L1450-1455 (CURRENT upstream — the old port
  // here implemented a pre-#2798 constructor that closed ltx:p/ltx:para at
  // construction time and never issued a real \par):
  //   DefMacro('\preitem@par', sub {
  //     return ((LookupValue('itemization_items') || 0) > 0
  //       ? (T_CS('\par'), Invocation(T_CS('\vskip'), T_CS('\itemsep')),
  //                        Invocation(T_CS('\vskip'), T_CS('\parsep')))
  //       : ()); });
  // Between items a REAL \par runs in the live context — repacking the
  // previous item's text into a width-carrying horizontal List (so the
  // sizer line-breaks it as a paragraph, not one long line) — followed by
  // the inter-item glue. Nothing before the FIRST item, which also covers
  // the 2004.07710 trivlist-in-itemize case the old constructor guarded:
  // with no \par, the freshly opened <ltx:itemize> wrapper stays open.
  // (\par itself closes ltx:p/ltx:para at construction.) Without this,
  // itemize inside a measured \vbox under-sized by ~2 lines per item and
  // tcolorbox frames clipped their content (2605.02240).
  DefMacro!("\\preitem@par", sub[_args] {
    if lookup_int("itemization_items") > 0 {
      Ok(Tokens!(
        T_CS!("\\par"),
        T_CS!("\\vskip"), T_CS!("\\itemsep"),
        T_CS!("\\vskip"), T_CS!("\\parsep")
      ))
    } else {
      Ok(Tokens::default())
    }
  });

  // Perl: latex_constructs.pool.ltxml L1560
  DefMacro!("\\@mklab{}", "\\hfil #1");

  // id, but NO refnum (et.al) attributes on itemize \\item ...
  // unless the optional tag argument was given!
  // We"ll make the <ltx:tag> from either the optional arg, or from \\labelitemi..
  DefMacro!("\\itemize@item", "\\preitem@par\\itemize@item@");
  DefConstructor!("\\itemize@item@ OptionalUndigested",
    "<ltx:item xml:id='#id' itemsep='#itemsep'>#tags",
    properties => sub[args] {
      let undigested = args[0].as_ref().map(|d| d.raw_tokens()).unwrap_or_default();
      ref_step_item_counter(undigested) });
  DefConstructor!("\\inline@itemize@item OptionalUndigested",
    "<ltx:inline-item xml:id='#id'>#tags",
    properties => sub[args] {
      let undigested = args[0].as_ref().map(|d| d.raw_tokens()).unwrap_or_default();
      ref_step_item_counter(undigested) });

  DefMacro!("\\enumerate@item", "\\preitem@par\\enumerate@item@");
  DefConstructor!("\\enumerate@item@ OptionalUndigested",
    "<ltx:item xml:id='#id' itemsep='#itemsep'>#tags",
    properties => sub[args] {
      let undigested = args[0].as_ref().map(|d| d.raw_tokens()).unwrap_or_default();
      ref_step_item_counter(undigested) });
  DefConstructor!("\\inline@enumerate@item OptionalUndigested",
    "<ltx:inline-item xml:id='#id'>#tags",
    properties => sub[args] {
      let undigested = args[0].as_ref().map(|d| d.raw_tokens()).unwrap_or_default();
      ref_step_item_counter(undigested) });

  DefMacro!("\\description@item", "\\preitem@par\\description@item@");
  DefConstructor!("\\description@item@ OptionalUndigested",
    "<ltx:item xml:id='#id' itemsep='#itemsep'>#tags",
    properties => sub[args] {
      let undigested = args[0].as_ref().map(|d| d.raw_tokens()).unwrap_or_default();
      ref_step_item_counter(undigested) });
  DefConstructor!("\\inline@description@item OptionalUndigested",
    "<ltx:inline-item xml:id='#id'>#tags",
    properties => sub[args] {
      let undigested = args[0].as_ref().map(|d| d.raw_tokens()).unwrap_or_default();
      ref_step_item_counter(undigested) });

  // NOTE: no before_digest_end \par on list environments — Perl has none.
  // An isolated Digest(\par) repacks an EMPTY temp box list and resets MODE
  // to the bound vertical mode, DEFUSING the env-end
  // leave_horizontal_internal repack; item text then stays as bare char
  // boxes and the vertical sizer stacks each one as a 12pt line (952pt for
  // a 16-word item — witness 2605.02240's 12000pt-tall tcolorbox frames).
  // endMode does the repacking, exactly like Perl (Stomach.pm endMode L553).
  // latex.ltx:15859 `\list` does `\let\makelabel\@mklab` and itemize /
  // enumerate (:16072/:16061) `\def\makelabel##1{…}` locally, so a document's
  // GLOBAL `\makelabel` (a 2-argument figure-label helper,
  // mathfont-user-guide.tex:85) never sees the item labels. LaTeXML's
  // `\fnum@@itemi` = `{\makelabel{\label@itemi}}` called the user's macro
  // (Perl too: "\textbullet should not appear between \csname and
  // \endcsname"). Guard: `perfect_kernel_batch54::global_makelabel_does_not_reach_list_items`.
  DefEnvironment!("{itemize}",
    "<ltx:itemize xml:id='#id'>#body</ltx:itemize>",
    before_digest => { def_macro_identity("\\makelabel{}")?; },
    properties => { BeginItemize!("itemize", "@item") },
    locked => true,
    mode => "internal_vertical"
  );
  DefEnvironment!("{enumerate}",
    "<ltx:enumerate xml:id='#id'>#body</ltx:enumerate>",
    before_digest => { def_macro_identity("\\makelabel{}")?; },
    properties => { BeginItemize!("enumerate", "enum") },
    locked => true,
    mode => "internal_vertical"
  );
  DefEnvironment!("{description}",
    "<ltx:description  xml:id='#id'>#body</ltx:description>",
    before_digest => { Let!("\\makelabel", "\\descriptionlabel"); },
    properties => { BeginItemize!("description", "@desc") },
    locked => true,
    mode => "internal_vertical"
  );

  def_macro_identity("\\makelabel{}")?;
  //----------------------------------------------------------------------
  // Basic itemize bits
  // Fake counter for itemize to give id's to ltx:item.
  NewCounter!("@itemi",   "", idwithin => "@itemizei", idprefix => "i");
  NewCounter!("@itemii",  "", idwithin => "@itemi",    idprefix => "i");
  NewCounter!("@itemiii", "", idwithin => "@itemii",   idprefix => "i");
  NewCounter!("@itemiv",  "", idwithin => "@itemiii",  idprefix => "i");
  NewCounter!("@itemv",   "", idwithin => "@itemiv",   idprefix => "i");
  NewCounter!("@itemvi",  "", idwithin => "@itemv",    idprefix => "i");
  // These are empty to make the "refnum" go away.
  def_macro_noop("\\the@itemi")?;
  def_macro_noop("\\the@itemii")?;
  def_macro_noop("\\the@itemiii")?;
  def_macro_noop("\\the@itemiv")?;
  def_macro_noop("\\the@itemv")?;
  def_macro_noop("\\the@itemvi")?;

  // Formatted item tags.
  // Really should be in the class file, but already was here.
  DefMacro!("\\labelitemi", "\\textbullet");
  DefMacro!("\\labelitemii", "\\normalfont\\bfseries \\textendash");
  DefMacro!("\\labelitemiii", "\\textasteriskcentered");
  DefMacro!("\\labelitemiv", "\\textperiodcentered");

  // Make the fake counters point to the real labels
  DefMacro!("\\label@itemi", "\\labelitemi");
  DefMacro!("\\label@itemii", "\\labelitemii");
  DefMacro!("\\label@itemiii", "\\labelitemiii");
  DefMacro!("\\label@itemiv", "\\labelitemiv");

  // These hookup latexml"s tagging to normal latex"s \labelitemi...
  DefMacro!("\\fnum@@itemi", r"{\makelabel{\label@itemi}}");
  DefMacro!("\\fnum@@itemii", r"{\makelabel{\label@itemii}}");
  DefMacro!("\\fnum@@itemiii", r"{\makelabel{\label@itemiii}}");
  DefMacro!("\\fnum@@itemiv", r"{\makelabel{\label@itemiv}}");

  DefMacro!("\\lx@poormans@ordinal{}", sub[(ctr)] {
    let mut ctr_str      = CounterValue!(&ctr.to_string()).value_of().to_string();
    let last_char = ctr_str.chars().last().unwrap_or('.');
    if last_char.is_ascii_digit() {
      ctr_str.push_str(PM_ORDINAL_SUFFICES[last_char.to_digit(10).unwrap() as usize]);
    }
    T_OTHER!(ctr_str)
  });
  DefMacro!("\\itemtyperefname", "item");
  DefMacro!("\\itemcontext", "\\space in \\@listcontext");
  def_macro_noop("\\itemcontext")?;
  // Probably would help to give a bit more context for the ii & higher?
  DefMacro!(
    "\\typerefnum@@itemi",
    "\\lx@poormans@ordinal{@itemi} \\itemtyperefname \\itemcontext"
  );
  DefMacro!(
    "\\typerefnum@@itemii",
    "\\lx@poormans@ordinal{@itemii} \\itemtyperefname \\itemcontext"
  );
  DefMacro!(
    "\\typerefnum@@itemiii",
    "\\lx@poormans@ordinal{@itemiii} \\itemtyperefname \\itemcontext"
  );
  DefMacro!(
    "\\typerefnum@@itemiv",
    "\\lx@poormans@ordinal{@itemiv} \\itemtyperefname \\itemcontext"
  );
  //----------------------------------------------------------------------
  // Basic enumeration bits

  // Class file should have
  //  NewCounter for enumi,...,
  //  define \labelenumi,... and probably \p@enumii...
  NewCounter!("enumi",   "", idwithin => "@itemizei", idprefix => "i");
  NewCounter!("enumii",  "", idwithin => "enumi",     idprefix => "i");
  NewCounter!("enumiii", "", idwithin => "enumii",    idprefix => "i");
  NewCounter!("enumiv",  "", idwithin => "enumiii",   idprefix => "i");
  NewCounter!("enumv",   "", idwithin => "enumiv",    idprefix => "i"); // A couple of extra
  NewCounter!("enumvi",  "", idwithin => "enumv",     idprefix => "i");

  // How the refnums look... (probably should be in class file, but already here)
  DefMacro!("\\p@enumii", "\\theenumi");
  DefMacro!("\\p@enumiii", "\\theenumi(\\theenumii)");
  DefMacro!("\\p@enumiv", "\\p@enumii\\theenumiii");

  // Formatting of item tags (probably should be in the class file, but already here)
  DefMacro!("\\labelenumi", "\\theenumi.");
  DefMacro!("\\labelenumii", "(\\theenumii)");
  DefMacro!("\\labelenumiii", "\\theenumiii.");
  DefMacro!("\\labelenumiv", "\\theenumiv.");

  // These hookup latexml"s tagging to normal latex"s \labelenummi...
  DefMacro!("\\fnum@enumi", "{\\makelabel{\\labelenumi}}");
  DefMacro!("\\fnum@enumii", "{\\makelabel{\\labelenumii}}");
  DefMacro!("\\fnum@enumiii", "{\\makelabel{\\labelenumiii}}");
  DefMacro!("\\fnum@enumiv", "{\\makelabel{\\labelenumiv}}");

  // These define the typerefnum form, for out-of-context \ref's
  DefMacro!("\\enumtyperefname", "item");
  DefMacro!(
    "\\typerefnum@enumi",
    "\\enumtyperefname~\\p@enumi\\theenumi \\itemcontext"
  );
  DefMacro!(
    "\\typerefnum@enumii",
    "\\enumtyperefname~\\p@enumii\\theenumii \\itemcontext"
  );
  DefMacro!(
    "\\typerefnum@enumiii",
    "\\enumtyperefname~\\p@enumiii\\theenumiii \\itemcontext"
  );
  DefMacro!(
    "\\typerefnum@enumiv",
    "\\enumtyperefname~\\p@enumiv\\theenumiv \\itemcontext"
  );

  //----------------------------------------------------------------------
  // Basic description list bits
  // Fake counter for itemize to give id"s to ltx:item.
  NewCounter!("@desci",   "", idwithin => "@itemizei", idprefix => "i");
  NewCounter!("@descii",  "", idwithin => "@desci",    idprefix => "i");
  NewCounter!("@desciii", "", idwithin => "@descii",   idprefix => "i");
  NewCounter!("@desciv",  "", idwithin => "@desciii",  idprefix => "i");
  NewCounter!("@descv",   "", idwithin => "@desciv",   idprefix => "i");
  NewCounter!("@descvi",  "", idwithin => "@descv",    idprefix => "i");
  // No refnum"s here, either
  def_macro_noop("\\the@desci")?;
  def_macro_noop("\\the@descii")?;
  def_macro_noop("\\the@desciii")?;
  def_macro_noop("\\the@desciv")?;
  def_macro_noop("\\the@descv")?;
  def_macro_noop("\\the@descvi")?;
  // These hookup latexml"s numbering to normal latex"s
  // Umm.... but they"re not normally used, since \item usually gets an argument!
  DefMacro!("\\descriptionlabel{}", "\\normalfont\\bfseries #1");
  DefMacro!("\\fnum@@desci", "{\\descriptionlabel{}}");
  DefMacro!("\\fnum@@descii", "{\\descriptionlabel{}}");
  DefMacro!("\\fnum@@desciii", "{\\descriptionlabel{}}");
  DefMacro!("\\fnum@@desciv", "{\\descriptionlabel{}}");

  DefMacro!("\\desctyperefname", "item");

  // Blech
  for lvl in &[
    "@itemi", "@itemii", "@itemiii", "@itemiv", "@itemv", "@itemvi",
  ] {
    DefMacro!(T_CS!(s!("\\{}name", lvl)), None, T_CS!("\\itemtyperefname"));
  }
  for lvl in &["enumi", "enumii", "enumiii", "enumiv"] {
    DefMacro!(T_CS!(s!("\\{}name", lvl)), None, T_CS!("\\enumtyperefname"));
  }
  for lvl in &[
    "@desci", "@descii", "@desciii", "@desciv", "@descv", "@descvi",
  ] {
    DefMacro!(T_CS!(s!("\\{}name", lvl)), None, T_CS!("\\desctyperefname"));
  }

  //======================================================================
  // C.6.3 The list and trivlist environments.
  //======================================================================
  // Generic lists are given a way to format the item label, and presumably
  // a counter.

  DefConditional!("\\if@nmbrlist");
  def_macro_noop("\\@listctr")?;
  DefPrimitive!("\\usecounter{}", sub[(counter)] {
    let counter = Expand!(counter).to_string();
    let counter_opt = if counter.is_empty() { None } else { Some(counter.as_str()) };
    begin_itemize("list", counter_opt, BeginItemizeOptions {
      nolevel: !counter.is_empty(),
      ..BeginItemizeOptions::default() })?;
  });

  // `\@listdepth` accounting mirrors latex.ltx:15852 (`\list` … `\global
  // \advance\@listdepth\@ne`) and :15913 (`\endlist` … `\global\advance
  // \@listdepth\m@ne`). The closer's decrement is load-bearing even though the
  // native list never consults the depth: a raw class that redefines `\list`
  // alone (memoir.cls:4580 is latex.ltx's `\list` verbatim, with the `>5 →
  // \@toodeep` check) keeps our `\endlist`, so without the decrement the depth
  // climbed monotonically and every list after the sixth raised "Too deeply
  // nested" (memman: 88 errors from `adjustwidth`, memoir.cls:11268). Perl
  // (latex_constructs.pool.ltxml:1644/1651) shares the leak. Guard:
  // `perfect_kernel_batch54::endlist_decrements_listdepth`.
  DefMacro!(
    r"\list{}{}",
    r"\global\advance\@listdepth\@ne\let\@listctr\@empty#2\ifx\@listctr\@empty\usecounter{}\fi\expandafter\def\csname fnum@\@listctr\endcsname{#1}\lx@list"
  );
  DefMacro!("\\endlist", r"\global\advance\@listdepth\m@ne\endlx@list");

  // Start an anonymous list (often misused)
  DefConstructor!("\\lx@list",
    "<ltx:itemize>",
    before_digest => {
      begin_mode("internal_vertical")?;
      // Name the frame, so `\endtrivlist` (latex.ltx:15913 `\endlist` =
      // `\endtrivlist`; a raw `\@trivlist` … `\endtrivlist` pair) can tell
      // the list's own mode frame from an enclosing one.
      assign_value("groupInitiator", Stored::Token(T_CS!("\\lx@list")), None);
    });
  // Close the anonymous list if we're still within one.
  //
  // Perl (latex_constructs.pool.ltxml:1647-1653): `\lx@list` =
  // beginMode('internal_vertical'), `\endlx@list` = endMode('internal_vertical')
  // — symmetric MODE frames, so `\endlist` also closes a list opened by a
  // `{enumerate}`/`{itemize}` environment's begin macro (`mode =>
  // "internal_vertical"` above): nih/denselists.sty:10,16 `\let\Onumerate
  // \enumerate`, `\newenvironment{Enumerate}{\Onumerate\Nospacing}{\endlist}`
  // (example-biosketch, polydemo; RUST-ONLY, Perl clean). A bgroup/egroup
  // pair keyed on groupInitiator (batches 51/54) could not pop that frame,
  // so `\end{Enumerate}`'s `\endgroup` cascaded. `end_mode` keeps Perl's
  // Stomach.pm:524-531 shape when the top frame is not the list's own: it
  // Errors "Attempt to end mode" and does NOT pop — which is what protected
  // memoir.cls:4580's raw `\list` (latex.ltx's, ending in `\@trivlist`, no
  // group) paired with our `\endlist` (memman 144→1001, biblatex-oxref ×4,
  // verbatimcopy, dlfltxb; sweep 28), now served by OXIDIZED_DESIGN #180.
  // Guards: `perfect_kernel_batch51::endlist_without_lx_list_frame`,
  // `perfect_kernel_batch54::endlist_decrements_listdepth`,
  // `perfect_kernel_batch56::endlist_closes_an_enumerate_opened_by_its_begin_macro`.
  DefConstructor!("\\endlx@list", sub[document] {
  document.maybe_close_element("ltx:itemize")?; },
  before_digest => { end_mode("internal_vertical")?; });

  DefConstructor!("\\list@item OptionalUndigested",
    "<ltx:item xml:id='#id' itemsep='#itemsep'>#tags",
    properties => sub[args] {
      let undigested = args[0].as_ref().map(|d| d.raw_tokens()).unwrap_or_default();
      ref_step_item_counter(undigested) }
  );

  // Perl latex_constructs.pool.ltxml L1720-1726:
  //   DefConstructor('\trivlist', "<ltx:itemize _autoclose='1'>", mode=>internal_vertical, …);
  //   DefConstructor('\endtrivlist', sub { maybeCloseElement('ltx:itemize') }, beforeDigest=>Digest('\par'));
  // The `\endtrivlist` is an *idempotent* closer — `maybeCloseElement` is a
  // no-op when the element is already closed. That matters when user code
  // calls `\endtrivlist` directly (e.g. arxiv 0908.0398's `\cqfd → …\endtrivlist`),
  // then `\end{proof}` closes the outer trivlist, then `\end{proof}`'s own
  // `\endproof → \endtrivlist` fires again. Perl swallows the double-close;
  // Rust's previous DefEnvironment emitted a strict env-frame closer that
  // errored on the second call.
  DefConstructor!("\\trivlist",
    "<ltx:itemize _autoclose='1'>",
    mode => "internal_vertical",
    properties => {
      begin_itemize("trivlist", None, BeginItemizeOptions::default())?
    }
  );
  DefConstructor!("\\endtrivlist",
    sub[document, _args, _props] {
      document.maybe_close_element("ltx:itemize")?;
    },
    before_digest => {
      Digest!("\\par")?;
      // A list opened through `\@trivlist` (OXIDIZED_DESIGN #180 below) owns
      // an `\lx@list` frame; `\endtrivlist` is its kernel closer
      // (latex.ltx:15913 `\endlist` → `\endtrivlist`; 0802.2207
      // `mathtrivlist` pairs `\@trivlist` with `\endtrivlist` directly).
      // Our own `\trivlist` opens no frame, so the pop is conditional; the
      // `\lx@list` frame is a MODE frame (Perl's beginMode), closed as one.
      if is_value_bound("groupInitiator", Some(0))
        && lookup_token("groupInitiator").as_ref() == Some(&T_CS!("\\lx@list"))
      {
        end_mode("internal_vertical")?;
      }
    }
  );

  // OXIDIZED_DESIGN #180 (PLANS P38): `\@trivlist` is the shared list opener.
  // latex.ltx:15848/15871/15903 — `\list` and `\trivlist` both end in
  // `\@trivlist`, and `\endlist` is `\endtrivlist`. Perl neutralizes it
  // (`DefMacro('\@trivlist', '\relax', locked => 1)`, pool:1732), so a raw
  // class or package that redefines `\list` alone (memoir.cls:4580 =
  // latex.ltx's `\list` verbatim; memoir's `adjustwidth` = `\begin{list}`,
  // used by digiconfigs/memman/memexsupp/MemoirChapStyles; autolist.sty:37-109
  // `\Sublist`) opened no list at all while OUR `\endlist`/`\endlx@list`
  // still expected the `\lx@list` frame — "Attempt to end mode
  // internal_vertical" on every such list (28 docs, sweep 30). Now
  // `\@trivlist` starts the itemization (unless the list's setup already
  // ran `\usecounter`, which binds `itemcounter` in this frame) and opens
  // `\lx@list`. The kernel body's `\@noitemerr` paths are not reproduced
  // (a bare `\@trivlist` before any `\item` is fine).
  DefPrimitive!("\\lx@trivlist@setup", {
    if !is_value_bound("itemcounter", Some(0)) {
      begin_itemize("list", None, BeginItemizeOptions::default())?;
    }
  });
  DefMacro!("\\@trivlist", "\\lx@trivlist@setup\\lx@list");
  DefMacro!("\\trivlist@item", "\\preitem@par\\trivlist@item@");
  DefConstructor!("\\trivlist@item@ OptionalUndigested",
    "<ltx:item xml:id='#id' itemsep='#itemsep'><ltx:tags><ltx:tag>#tag</ltx:tag></ltx:tags>",
    // At least an empty tag! ?
    properties => sub[args] {
      if let Some(ref arg) = args[0] {
        if let DigestedData::Postponed(tag_tokens) = arg.data() {
          let tag_expanded = Expand!(tag_tokens.clone());
          let tag = digest(tag_expanded)?;
          Ok(stored_map!("tag" => tag))
        } else {
          Ok(SymHashMap::default())
        }
      } else {
          Ok(SymHashMap::default())
      }
    }
  );

  // Perl latex_constructs.pool L1675-1679: these five carry REAL default
  // glue (LaTeX's list spacing); zeroing them under-measured every list
  // (begin_itemize's padtop/padbottom = \topsep+\parskip+\partopsep) and
  // clipped tcolorbox frames drawn from the estimates (2605.02240).
  DefRegister!("\\topsep"             => Glue!("8pt plus 2pt minus 4pt"));
  DefRegister!("\\partopsep"          => Glue!("2pt plus 1pt minus 1pt"));
  DefRegister!("\\lx@default@itemsep" => Glue!("4pt plus 2pt minus 1pt"));
  DefRegister!("\\itemsep"            => Glue!("4pt plus 2pt minus 1pt"));
  DefRegister!("\\parsep"             => Glue!("4pt plus 2pt minus 1pt"));
  DefRegister!("\\@topsep"            => Glue::new(0));
  DefRegister!("\\@topsepadd"         => Glue::new(0));
  DefRegister!("\\@outerparskip"      => Glue::new(0));
  DefRegister!("\\leftmargin"         => Dimension::new(0));
  DefRegister!("\\rightmargin"        => Dimension::new(0));
  DefRegister!("\\listparindent"      => Dimension::new(0));
  DefRegister!("\\itemindent"         => Dimension::new(0));
  DefRegister!("\\labelwidth"         => Dimension::new(0));
  DefRegister!("\\labelsep"           => Dimension::new(0));
  DefRegister!("\\@totalleftmargin"   => Dimension::new(0));
  DefRegister!("\\leftmargini"        => Dimension::new(0));
  DefRegister!("\\leftmarginii"       => Dimension::new(0));
  DefRegister!("\\leftmarginiii"      => Dimension::new(0));
  DefRegister!("\\leftmarginiv"       => Dimension::new(0));
  DefRegister!("\\leftmarginv"        => Dimension::new(0));
  DefRegister!("\\leftmarginvi"       => Dimension::new(0));
  DefRegister!("\\@listdepth"         => Number::new(0));
  DefRegister!("\\@itempenalty"       => Number::new(0));
  DefRegister!("\\@beginparpenalty"   => Number::new(0));
  DefRegister!("\\@endparpenalty"     => Number::new(0));
  DefRegister!("\\labelwidthi"        => Dimension::new(0));
  DefRegister!("\\labelwidthii"       => Dimension::new(0));
  DefRegister!("\\labelwidthiii"      => Dimension::new(0));
  DefRegister!("\\labelwidthiv"       => Dimension::new(0));
  DefRegister!("\\labelwidthv"        => Dimension::new(0));
  DefRegister!("\\labelwidthvi"       => Dimension::new(0));

  DefRegister!("\\@itemdepth" => Number::new(0));
  // \@maxlistdepth and the \@listi..vi family are not in Perl
  // latex_*.pool.ltxml — they live in `latex_constructs_rust_only.rs`
  // (loads last). Identical-body duplicates removed from here.

  //======================================================================
  // C.6.4 Verbatim
  //======================================================================
  // NOTE: how's the best way to get verbatim material through?
  // DefEnvironment!("{verbatim}", "<ltx:verbatim>#body</ltx:verbatim>");
  // DefEnvironment!("{verbatim*}", "<ltx:verbatim>#body</ltx:verbatim>");

  DefMacro!(
    "\\@verbatim",
    r"\par\aftergroup\lx@end@verbatim\lx@@verbatim"
  ); // Close enough?
  // Perl latex_constructs.pool.ltxml L1774-1782: enterHorizontal => 1 + beforeDigest.
  DefConstructor!("\\lx@@verbatim", "<ltx:verbatim font='#font'>",
  enter_horizontal => true,
  before_digest => {
    begin_semiverbatim(Some(&SEMIVERBATIM_CHARS));
    merge_font(fontmap!(family => "typewriter", series => "medium", shape => "upright"));
    assign_catcode(' ', Catcode::ACTIVE, None);  // Do NOT (necessarily) skip spaces after \verb!!!
    Let!(&T_ACTIVE!(' '), T_SPACE!());
  });
  DefConstructor!(r"\lx@end@verbatim", "</ltx:verbatim>",
    before_digest => { end_semiverbatim()?; });

  // verbatim is a bit of special case;
  // It looks like an environment, but it only ends with an explicit "\end{verbatim}" on it's own line.
  // So, we'll end up doing things more manually.
  // We're going to sidestep the Gullet for inputting,
  // and also the usual environment capture.
  DefConstructor!(T_CS!("\\begin{verbatim}"), None,
    "<ltx:verbatim font='#font'>#body</ltx:verbatim>",
    before_digest => { before_digest_verbatim() }
    after_digest => sub[whatsit] { after_digest_verbatim(false, whatsit)?; },
    before_construct => sub[document, _whatsit] {
      document.maybe_close_element("ltx:p")?; }
  );
  DefConstructor!(T_CS!("\\begin{verbatim*}"), None,
    "<ltx:verbatim font='#font'>#body</ltx:verbatim>",
    before_digest => { before_digest_verbatim() }
    after_digest => sub[whatsit] { after_digest_verbatim(true, whatsit)?; },
    before_construct => sub[document, _whatsit] {
      document.maybe_close_element("ltx:p")?; }
  );
  // The terminator is re-executed through the current `\end` macro
  // (`after_digest_verbatim`, latex.ltx:15438 `\@xverbatim`); these fused
  // ends make the kernel `\end` a no-op for it, since the constructor has
  // already closed its group and emitted `</ltx:verbatim>`.
  DefMacro!(T_CS!("\\end{verbatim}"), None, Tokens!());
  DefMacro!(T_CS!("\\end{verbatim*}"), None, Tokens!());

  // Perl latex_constructs.pool.ltxml L1847 — re-let `\nobreakspace`
  // to LaTeXML's `\lx@nobreakspace` (= NBSP `\u{00A0}`). Required HERE
  // (not just plain_base.rs) so the override survives `LoadFormat`'s
  // dump path: the dump captures latex.ltx's
  // `\nobreakspace → \protect\nobreakspace<sp>` chain which decays to a
  // regular space + `\leavevmode\nobreak\<sp>`. Without this Let, the
  // hyperref autoref wrapping `\sectionautorefname\nobreakspace\thesection`
  // produced `section 1` (regular space) instead of `section\u{00A0}1`.
  Let!("\\nobreakspace", "\\lx@nobreakspace");

  DefPrimitive!("\\@vobeyspaces", {
    AssignCatcode!(' ', Catcode::ACTIVE);
    Let!(&T_ACTIVE!(' '), T_CS!("\\nobreakspace"));
  });
  DefMacro!("\\@xobeysp", "\\nobreakspace");

  // WARNING: Need to be careful about what catcodes are active here
  // And clearly separate expansion from digestion
  DefMacro!("\\verb", {
    match read_verb_invocation()? {
      Some(inner) => {
        let mut result = vec![T_CS!("\\lx@hidden@bgroup")];
        result.extend(inner);
        result.push(T_CS!("\\lx@hidden@egroup"));
        Ok(Tokens::new(result))
      },
      None => Ok(Tokens!()),
    }
  });

  DefPrimitive!("\\lx@use@visiblespace", {
    // Do NOT (necessarily) skip spaces after \verb!!!
    assign_catcode(' ', Catcode::ACTIVE, None);
    // Visible space
    Let!(&T_ACTIVE!(' '), T_OTHER!("\u{2423}"));
  });

  // Arrange to digest the body in text mode, to keep (eg) "_" from turning to "\_"
  DefMacro!(
    "\\@internal@verb{}{}{}",
    r"\ifmmode\@internal@math@verb{#1}{#2}{#3}\else\@internal@text@verb{#1}{#2}{#3}\fi"
  );
  // `encoding => "ASCII"` (OXIDIZED_DESIGN #144, issue #723): `\verb`'s body is
  // literal catcode-12 text, so under T1 a `~`/`^` would decode to Bruce Miller's
  // accent glyphs U+02DC/U+02C6 (#2435). Verbatim wants them ASCII; the identity
  // `ASCII` fontmap keeps `~`/`^` literal while `typewriter` still styles the run.
  DefConstructor!("\\@internal@math@verb{} Undigested {}",
    "<ltx:XMTok font='#font'>#3</ltx:XMTok>",
    mode      => "text",
    enter_horizontal => true,
    font      => { family => "typewriter", series => "medium", shape => "upright", encoding => "ASCII" },
    reversion => "\\verb#1#2#3#2");
  DefConstructor!("\\@internal@text@verb{} Undigested {}",
    "<ltx:verbatim font='#font'>#3</ltx:verbatim>",
    font            => { family => "typewriter", series => "medium", shape => "upright", encoding => "ASCII" },
    enter_horizontal => true,
    before_construct => sub[doc,_whatsit] {
      if !document::can_contain(doc.get_element().as_ref().unwrap(), "#PCDATA") {
        doc.open_element("ltx:p", None, None)?;
      }
    },
    reversion => "\\verb#1#2#3#2");

  // Actually, latex sets catcode to 13 ... is this close enough?
  DefPrimitive!("\\obeycr", {
    AssignValue!("PRESERVE_NEWLINES", 1);
  });
  DefPrimitive!("\\restorecr", {
    AssignValue!("PRESERVE_NEWLINES", 0);
  });
  DefMacro!(T_CS!("\\normalsfcodes"), None, Tokens!());

  Ok(())
}
