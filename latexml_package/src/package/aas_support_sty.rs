use crate::engine::latex_constructs::{after_float, before_float_ex};
use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: aas_support.sty.ltxml — support macros for AAS styles

  // Package dependencies — Perl L28-39
  RequirePackage!("aas_macros");
  RequirePackage!("url");
  RequirePackage!("longtable");
  RequirePackage!("xcolor");
  RequirePackage!("hyperref");
  RequirePackage!("array");
  RequirePackage!("lineno");
  RequirePackage!("amssymb");
  RequirePackage!("epsf");
  RequirePackage!("ulem");

  // 2.1.3 Editorial Information
  DefMacro!("\\received{}", "\\@add@frontmatter{ltx:date}[role=received,name=Received]{#1}");
  DefMacro!("\\revised{}", "\\@add@frontmatter{ltx:date}[role=revised,name=Revised]{#1}");
  DefMacro!("\\accepted{}", "\\@add@frontmatter{ltx:date}[role=accepted,name=Accepted]{#1}");
  DefMacro!("\\journalid{}{}", "");
  DefMacro!("\\articleid{}{}", "");
  DefMacro!("\\paperid{}", "");
  DefMacro!("\\msid{}", "");
  DefMacro!("\\added{}", "");
  DefMacro!("\\replaced{}", "");
  DefMacro!("\\deleted{}", "");
  DefMacro!("\\explain{}", "");
  DefMacro!("\\edit{}{}", "");
  DefMacro!("\\ccc{}", "");
  DefMacro!("\\cpright{}{}", "\\@add@frontmatter{ltx:note}[role=copyright]{\\copyright #2: #1}");
  DefMacro!("\\journal{}", "");
  DefMacro!("\\volume{}", "");
  DefMacro!("\\issue{}", "");
  DefMacro!("\\SGMLbi{}", "#1");
  DefMacro!("\\SGMLbsc{}", "#1");
  DefMacro!("\\SGMLclc{}", "#1");
  DefMacro!("\\SGMLentity{}", "#1");
  DefMacro!("\\SGML{}", "");

  // 2.1.4 Short Comment
  DefMacro!("\\slugcomment{}", "\\@add@frontmatter{ltx:note}[role=slugcomment]{#1}");

  // 2.1.5 Running Heads
  DefMacro!("\\shorttitle{}", "\\@add@frontmatter{ltx:toctitle}{#1}");
  DefMacro!("\\shortauthors{}", "");
  DefMacro!("\\correspondingauthor{}", "\\lx@contact{correspondent}{#1}");
  DefMacro!("\\lefthead{}", "");
  DefMacro!("\\righthead{}", "");

  // 2.3 Title and Author Information
  AssignMapping!("DOCUMENT_CLASSES", "ltx_authors_multiline" => true);

  DefConstructor!("\\@@personname[]{}", "<ltx:personname>#2</ltx:personname>",
    mode => "restricted_horizontal", enter_horizontal => true);

  DefMacro!("\\author[]{}", "\\@add@frontmatter{ltx:creator}[role=author]{\\@@personname[#1]{#2}}");

  DefConstructor!("\\@@@affiliation{}", "^ <ltx:contact role='affiliation'>#1</ltx:contact>");
  DefMacro!("\\affiliation{}", "\\@add@to@frontmatter{ltx:creator}{\\@@@affiliation{#1}}");
  DefMacro!("\\affil", "\\affiliation");
  DefConstructor!("\\@@@altaffil{}", "^ <ltx:contact role='affiliation'>#1</ltx:contact>");
  DefMacro!("\\altaffiliation{}", "\\@add@to@frontmatter{ltx:creator}{\\@@@altaffil{#1}}");
  DefConstructor!("\\@@@authoraddr{}", "^ <ltx:contact role='address'>#1</ltx:contact>");
  DefMacro!("\\authoraddr{}", "\\@add@to@frontmatter{ltx:creator}{\\@@@authoraddr{#1}}");

  DefConstructor!("\\@@@email{}", "^ <ltx:contact role='email'>#1</ltx:contact>");
  DefMacro!("\\email{}", "\\@add@to@frontmatter{ltx:creator}{\\@@@email{#1}}");

  DefPrimitive!("\\and", None);
  DefMacro!("\\authoremail", "\\email");

  // Perl aas_support.sty.ltxml L119:
  //   AddToMacro(T_CS('\@startsection@hook'),
  //              TokenizeInternal('\let\email\@@email'));
  // When a section starts, locally Let \email = \@@email so that
  // \email{user@example} inside a section body renders as an inline
  // mailto link (via \@@email) rather than being pushed to the
  // frontmatter creator list. Pure additive parity port — no test
  // exercises \email inside a section so no golden risk.
  AddToMacro!("\\@startsection@hook", "\\let\\email\\@@email");

  // Affiliation marks — Perl L126-132
  DefMacro!("\\altaffilmark{}", "\\@altaffilmark{#1}");
  DefConstructor!("\\@altaffilmark{}", "<ltx:note role='affiliationmark' mark='#1'/>",
    enter_horizontal => true);
  DefConstructor!("\\altaffiltext{}{}", "<ltx:note role='affiliationtext' mark='#1'>#2</ltx:note>");

  DefMacro!("\\software{}", "\\@add@frontmatter{ltx:note}[role=software]{#1}");
  DefMacro!("\\submitjournal{}", "\\@add@frontmatter{ltx:note}[role=journal]{#1}");

  // DOI — Perl L137-138
  DefConstructor!("\\doi{}", "<ltx:ref href='https://doi.org/#1'>#1</ltx:ref>",
    enter_horizontal => true);

  // Collaboration — Perl L139-141
  DefConstructor!("\\@@@collaborator{}", "<ltx:note role='collaborator'>#1</ltx:note>");
  DefMacro!("\\collaboration{}{}", "\\@add@to@frontmatter{ltx:creator}{\\@@@collaborator{#2}}");
  DefMacro!("\\nocollaboration{}", "");

  // 2.5 Keywords
  DefMacro!("\\keywords{}", "\\@add@frontmatter{ltx:keywords}{#1}");
  Let!("\\subjectheadings", "\\keywords");

  // 2.6 Comments to Editors
  DefMacro!("\\notetoeditor{}", "");
  NewCounter!("editornote");
  DefMacro!("\\theeditornote", "E\\arabic{editornote}");

  // 2.8 Figure and Table Placement
  DefMacro!("\\placetable{}", "");
  DefMacro!("\\placefigure{}", "");
  DefMacro!("\\placeplate{}", "");
  NewCounter!("plate");
  DefMacro!("\\platename", "Plate");
  DefMacro!("\\platewidth{Dimension}", "");
  DefMacro!("\\platenum{}", "\\def\\theplate{#1}");
  DefMacro!("\\gridline{}", "");

  // Plate environments — Perl aas_support.sty.ltxml L179-201.
  // Each variant calls beforeFloat (sets \@captype, rebinds \\ → \lx@newline,
  // assigns \hsize) and afterFloat (closes the float scope, sets the
  // float number / id). The starred variant additionally passes
  // `double => 1` so \hsize gets \textwidth instead of \columnwidth
  // (two-column-spanning plate). Without these hooks, the Rust port
  // emits an empty <ltx:float> shell that loses caption/number metadata
  // and uses single-column box geometry even in the * variant.
  // Template additionally needs `inlist='#inlist' ?#1(placement='#1')`
  // to match the floats produced by \newfloat-style envs (acmart, rotating).
  DefEnvironment!("{plate}[]",
    "<ltx:float xml:id='#id' inlist='#inlist' ?#1(placement='#1') class='ltx_float_plate'>#tags#body</ltx:float>",
    before_digest => { before_float_ex("plate", None, false); },
    after_digest => sub[whatsit] { after_float(whatsit); },
    mode => "internal_vertical"
  );
  DefEnvironment!("{plate*}[]",
    "<ltx:float xml:id='#id' inlist='#inlist' ?#1(placement='#1') class='ltx_float_plate'>#tags#body</ltx:float>",
    before_digest => { before_float_ex("plate", None, true); },
    after_digest => sub[whatsit] { after_float(whatsit); },
    mode => "internal_vertical"
  );

  // Fig macros — Perl L205-221. The smart `\fig` peeks the token after
  // the first Semiverbatim arg: if it's `{` (T_BEGIN), it's a 3-arg
  // figure-with-caption (`\fig{label}{width}{caption}`); otherwise it's
  // a single-arg ref-like usage (`\fig{label}` → `\ref{label}`). This
  // dispatch is needed for papers like astro-ph/0003209 + astro-ph0503342
  // that redefine `\fig` as a one-arg `\ref` shorthand inside captions
  // /footnotes — without this peek, `\fig{F:image}` always opens an
  // `<ltx:figure>` element and can land inside `<ltx:note>`.
  DefMacro!("\\aas@fig Semiverbatim {Dimension}{}",
    "\\begin{figure}\\caption{#3}\\includegraphics[width=#2]{#1}\\end{figure}");
  DefMacro!("\\fig Semiverbatim Token", sub[(arg, test)] {
    // Push back the args in correct order so the dispatched CS reads them.
    // Push order is reversed for stack semantics: last unread is first read.
    gullet::unread_one(test);
    gullet::unread_one(T_END!());
    gullet::unread_vec(arg.clone().unlist());
    gullet::unread_one(T_BEGIN!());
    if test.get_catcode() == Catcode::BEGIN {
      Ok(Tokens!(T_CS!("\\aas@fig")))
    } else {
      // see arXiv:astro-ph/0003209 for an example use as \ref while
      // also loading aas_support.sty.ltxml
      Ok(Tokens!(T_CS!("\\ref")))
    }
  });
  Let!("\\leftfig", "\\fig");
  Let!("\\rightfig", "\\fig");
  Let!("\\boxedfig", "\\fig");
  DefMacro!("\\rotatefig{Number} Semiverbatim {Dimension}{}",
    "\\begin{figure}\\caption{#4}\\includegraphics[width=#3,angle=#1]{#2}\\end{figure}");

  // 2.9 Acknowledgements
  Tag!("ltx:acknowledgements", auto_close => true);
  DefConstructor!("\\acknowledgements", "<ltx:acknowledgements>");
  Let!("\\acknowledgments", "\\acknowledgements");
  // AASTeX 6.3+ shortcut: `\ack{...}` (with mandatory arg, distinct from
  // ptephy's argument-less form).
  DefMacro!("\\ack{}", "\\begin{acknowledgements}#1\\end{acknowledgements}");

  // 2.10 Facilities
  DefConstructor!("\\facility{}", "<ltx:text class='ltx_ast_facility'>#1</ltx:text>",
    enter_horizontal => true);
  DefMacro!("\\facilities{}", "\\@add@frontmatter{ltx:note}[role=facilities]{#1}");

  // 2.11 Appendices — Perl aas_support.sty.ltxml L247-249
  DefMacro!("\\appendix", "\\@appendix");
  // `\@appendix` starts section-numbered appendices, then re-scopes the
  // equation counter to reset within each appendix section. Perl uses
  // `scope => 'global'` on the `\theequation` redefinition so the new
  // numbering format outlives the current group — in Rust we pass
  // Some(Scope::Global) to def_macro for the same effect. The appendix
  // numbering is `\thesection\arabic{equation}` (no separator) — matches
  // AAS journal style, distinct from Rust's `\eqsecnum` macro L369 which
  // uses a dash separator.
  DefPrimitive!("\\@appendix", {
    start_appendices("section");
    // Perl L248 passes `idprefix => 'E'` so appendix equations get
    // xml:ids like `S1.E2`. Without the prefix, Rust falls back to the
    // default (empty) and collides with body-equation ids once the
    // document reaches its second appendix.
    new_counter(
      "equation",
      "section",
      Some(NewCounterOptions { idprefix: "E", ..Default::default() }),
    )?;
    def_macro(
      T_CS!("\\theequation"),
      None,
      mouth::tokenize_internal("\\thesection\\arabic{equation}"),
      Some(ExpandableOptions { scope: Some(Scope::Global), ..Default::default() }),
    )?;
  });

  // 2.12 Equations
  DefMacro!("\\mathletters", "\\lx@equationgroup@subnumbering@begin");
  DefMacro!("\\endmathletters", "\\lx@equationgroup@subnumbering@end");

  // 2.12 Equations — Perl L261 (proper tag setter, not empty stub)
  DefMacro!("\\eqnum{}",
    "\\lx@equation@settag{\\edef\\theequation{#1}\\lx@make@tags{equation}}");

  // 2.13 Citations — Perl L264-293
  DefMacro!("\\markcite{}", "");
  RequirePackage!("natbib");

  // Perl aas_support.sty.ltxml:283-291:
  //   DefConstructor('\references',
  //     "<ltx:bibliography xml:id='#id' ... ><ltx:title>#title</ltx:title><ltx:biblist>",
  //     afterDigest => sub { beginBibliography($_[1]); });
  //   DefConstructor('\endreferences', sub { maybeCloseElement biblist/bibliography; });
  //
  // Without `afterDigest => beginBibliography`, Rust's \bibitem fires
  // unguarded: the open `<ltx:biblist>` child-admission rules don't take
  // effect (beginBibliography installs them), so `\bibitem` ends up
  // absorbed by whatever the current element is — `<ltx:section>`,
  // `<ltx:para>`, `<ltx:text>`, `<ltx:XMath>` in the 4 failing 10k-sandbox
  // papers (astro-ph9711070, cond-mat0109365, nucl-ex9706010,
  // nucl-th0010030) → "malformed:ltx:bibitem isn't allowed in <ltx:X>".
  //
  // Matching revtex4_support_sty.rs:146-159's pattern for its own
  // `\references` (which already calls begin_bibliography). The Perl
  // attribute set (bibstyle/citestyle/sort/title) is richer than what
  // the Rust template currently emits — that's a separate enhancement;
  // landing the afterDigest hook alone is what closes the 4-paper
  // malformed:ltx:bibitem cluster.
  DefConstructor!(
    "\\references",
    "<ltx:bibliography xml:id='#id'><ltx:biblist>",
    after_digest => sub[whatsit] {
      crate::engine::latex_constructs::begin_bibliography(whatsit)?;
    }
  );
  DefConstructor!(
    "\\endreferences",
    sub[document, _whatsit, _props] {
      document.maybe_close_element("ltx:biblist")?;
      document.maybe_close_element("ltx:bibliography")?;
    }
  );
  Let!("\\reference", "\\bibitem");

  RequirePackage!("graphicx");

  // 2.14 Electronic Art
  DefMacro!("\\figurenum{}", "\\def\\thefigure{#1}");
  DefMacro!("\\epsscale{}", "");
  DefMacro!("\\plotone Semiverbatim", "\\includegraphics[width=\\textwidth]{#1}");
  DefMacro!("\\plottwo Semiverbatim Semiverbatim",
    "\\hbox{\\includegraphics[width=\\textwidth]{#1}\\includegraphics[width=\\textwidth]{#2}}");
  DefMacro!("\\plotfiddle Semiverbatim {}{}{}{}{}{}",
    "\\includegraphics[width=#4pt,height=#5pt]{#1}");

  // 2.14.2 Figure Captions
  // Perl: `DefMacro('\figcaption OptionalSemiverbatim', sub { ... })`.
  // The optional arg is `OptionalSemiverbatim` — catcodes are neutralized
  // so a literal `_` in `\figcaption[X_Y.ps]{...}` (paper-local filename
  // hint for List-of-Figures) doesn't trigger the math-mode subscript
  // catcode. Driver: arXiv:astro-ph/9808081 has 5× `\figcaption[X_Y.ps]`.
  // \figcaption checks if inside a figure environment.
  // If yes → \caption; if no → \@figcaption (wraps in figure env).
  DefMacro!("\\@figcaption {}", "\\begin{figure}#1\\end{figure}");
  DefMacro!("\\figcaption OptionalSemiverbatim", sub[(opt_arg)] {
    let env = state::lookup_string_from_sym(pin!("current_environment"));
    if env.contains("figure") {
      // Inside figure: act as \caption
      if let Some(opt) = opt_arg {
        Ok(Tokens!(T_CS!("\\caption"), T_OTHER!("["), opt, T_OTHER!("]")))
      } else {
        Ok(Tokens!(T_CS!("\\caption")))
      }
    } else {
      // Outside figure: wrap in \@figcaption
      Ok(Tokens!(T_CS!("\\@figcaption")))
    }
  });

  // 2.15 Tables
  RequirePackage!("deluxetable");
  Let!("\\planotable", "\\deluxetable");
  Let!("\\endplanotable", "\\enddeluxetable");

  // Perl: aas_support.sty.ltxml L380-383
  Let!("\\splitdeluxetable", "\\deluxetable");
  Let!("\\endsplitdeluxetable", "\\enddeluxetable");
  state::let_i(&T_CS!("\\splitdeluxetable*"), &T_CS!("\\deluxetable*"), None);
  state::let_i(&T_CS!("\\endsplitdeluxetable*"), &T_CS!("\\enddeluxetable*"), None);

  // aastex631.cls L4780-4781:
  //   \newif\ifstartlongtable
  //   \def\startlongtable{\vskip1sp\global\startlongtabletrue}
  // We treat as a no-op marker — our deluxetable / longtable handling
  // doesn't need the conditional flag. Driver: 2209.01632 (aastex631)
  // emitted "\startlongtable not defined" + alignment-tab cascade.
  DefMacro!("\\startlongtable", "");

  // Perl L373: Let('\savedollar' => T_MATH). The hidden 'h' column type
  // used by aas deluxetable tokenizes literal `$` from the template, so
  // the package stashes an active math-shift token into `\savedollar`
  // for later re-insertion. Port via state::let_i with T_MATH!().
  state::let_i(&T_CS!("\\savedollar"), &T_MATH!(), None);

  // Decimal table conditionals — Perl L338-345
  DefConditional!("\\ifcolnumberson");
  DefConditional!("\\ifdeluxedecimals");
  DefMacro!("\\deluxedecimals", "\\global\\deluxedecimalstrue");
  RawTeX!("\\global\\deluxedecimalsfalse");
  Let!("\\decimals", "\\deluxedecimals");
  DefMacro!("\\colnumbers", "");
  DefMacro!("\\deluxedecimalcolnumbers", "\\deluxedecimalstrue\\colnumbersontrue");
  Let!("\\decimalcolnumbers", "\\deluxedecimalcolnumbers");

  // Hidden column environment — Perl L374
  DefEnvironment!("{eatone}", "");

  // Perl aas_support.sty.ltxml L373-389: hidden-column types `h` and `B`.
  // Both wrap contents in \eatone (swallowed), producing a zero-width
  // sentinel cell. Perl L385-389 adds `B` with a TODO to "break table
  // eventually" — we match Perl's current behavior (identical to `h`).
  // The more complex `D` and `d` decimal-alignment column types (Perl
  // L349-356) use SplitTokens token-shuffling for dot alignment — the
  // helper itself (`base_utilities::split_tokens` + XUntil parameter
  // type) is now available, but porting is still deferred until a
  // concrete aastex paper with `D`/`d` columns surfaces as a
  // conversion gap, so the snapshot-regression risk is measurable.
  DefColumnType!("h", {
    with_current_build_template(|template_opt| {
      template_opt.unwrap().add_column(latexml_core::alignment::cell::Cell {
        before: Some(Tokens!(T_BEGIN!(), T_CS!("\\eatone"))),
        after:  Some(Tokens!(T_CS!("\\endeatone"), T_END!())),
        ..latexml_core::alignment::cell::Cell::default()
      })
    });
  });
  DefColumnType!("B", {
    with_current_build_template(|template_opt| {
      template_opt.unwrap().add_column(latexml_core::alignment::cell::Cell {
        before: Some(Tokens!(T_BEGIN!(), T_CS!("\\eatone"))),
        after:  Some(Tokens!(T_CS!("\\endeatone"), T_END!())),
        ..latexml_core::alignment::cell::Cell::default()
      })
    });
  });

  // aastex631.cls L2357-2359: \newcolumntype{C}/{L}/{R} are "math-shift
  // resistant" centered/left/right column types — they save the active
  // `$` (`\savedollar`) and `\let$\relax` so cell content like `$x$` is
  // treated as text rather than math-mode. Our Rust binding for
  // aastex.cls.ltxml ports `aas_support` but never raw-loads the actual
  // .cls file, so these `\newcolumntype` definitions never run.
  // Driver: 2209.01632 — `\begin{deluxetable*}{ccC}` triggered "Extra
  // alignment tab '&'" cascades because column type `C` was unrecognized
  // by `read_alignment_template`. Behavior is approximated as plain
  // c/l/r (the savedollar dance is unnecessary for our text-mode cells).
  DefColumnType!("C", {
    with_current_build_template(|template_opt| {
      template_opt.unwrap().add_column(latexml_core::alignment::cell::Cell {
        before: Some(Tokens!(T_CS!("\\hfil"))),
        after:  Some(Tokens!(T_CS!("\\hfil"))),
        ..latexml_core::alignment::cell::Cell::default()
      })
    });
  });
  DefColumnType!("L", {
    with_current_build_template(|template_opt| {
      template_opt.unwrap().add_column(latexml_core::alignment::cell::Cell {
        after: Some(Tokens!(T_CS!("\\hfil"))),
        ..latexml_core::alignment::cell::Cell::default()
      })
    });
  });
  DefColumnType!("R", {
    with_current_build_template(|template_opt| {
      template_opt.unwrap().add_column(latexml_core::alignment::cell::Cell {
        before: Some(Tokens!(T_CS!("\\hfil"))),
        ..latexml_core::alignment::cell::Cell::default()
      })
    });
  });

  DefMacro!("\\phn", "\\phantom{0}");
  DefMacro!("\\phd", "\\phantom{.}");
  DefMacro!("\\phs", "\\phantom{+}");
  DefMacro!("\\phm{}", "\\phantom{string}");

  DefEnvironment!("{interactive}{}{}", "#body");
  DefEnvironment!("{longrotatetable}", "#body");

  // 2.17.1 Celestial Objects and Data Sets
  DefConstructor!("\\objectname OptionalSemiverbatim {}",
    "<ltx:text class='ltx_ast_objectname'>#2 (catalog #1)</ltx:text>",
    enter_horizontal => true);
  Let!("\\object", "\\objectname");
  DefConstructor!("\\dataset OptionalSemiverbatim {}",
    "<ltx:text class='ltx_ast_dataset'>#2 (catalog #1)</ltx:text>",
    enter_horizontal => true);

  // 2.17.2 Ionic Species
  DefMacro!("\\ion{}{}", "{#1~\\expandafter\\uppercase\\expandafter{\\romannumeral #2}}");

  DefPrimitive!("\\sbond", "\u{2212}");
  DefPrimitive!("\\dbond", "=");
  DefPrimitive!("\\tbond", "\u{2261}");

  // 2.17.3 Fractions — Perl L435-442: \case uses a semantic text@frac constructor
  DefMacro!("\\case{}{}", "\\ensuremath{\\text@frac{#1}{#2}}");
  DefConstructor!("\\text@frac ScriptStyle ScriptStyle",
    "<ltx:XMApp><ltx:XMTok meaning='divide' role='FRACOP' mathstyle='text'/><ltx:XMArg>#1</ltx:XMArg><ltx:XMArg>#2</ltx:XMArg></ltx:XMApp>");
  Let!("\\slantfrac", "\\case");

  // 2.17.4 Astronomical Symbols
  DefPrimitive!("\\micron", "\u{00B5}m");
  DefMacro!("\\Sun", "\\sun");
  DefMacro!("\\Sol", "\\sun");
  DefPrimitive!("\\sun", "\u{2609}");
  DefPrimitive!("\\Mercury", "\u{263F}");
  DefPrimitive!("\\Venus", "\u{2640}");
  DefMacro!("\\Earth", "\\earth");
  DefMacro!("\\Terra", "\\earth");
  DefPrimitive!("\\earth", "\u{2295}");
  DefPrimitive!("\\Mars", "\u{2642}");
  DefPrimitive!("\\Jupiter", "\u{2643}");
  DefPrimitive!("\\Saturn", "\u{2644}");
  DefPrimitive!("\\Uranus", "\u{2645}");
  DefPrimitive!("\\Neptune", "\u{2646}");
  DefPrimitive!("\\Pluto", "\u{2647}");
  DefPrimitive!("\\Moon", "\u{263D}");
  DefMacro!("\\Luna", "\\Moon");
  DefPrimitive!("\\Aries", "\u{2648}");
  DefMacro!("\\VEq", "\\Aries");
  DefPrimitive!("\\Taurus", "\u{2649}");
  DefPrimitive!("\\Gemini", "\u{264A}");
  DefPrimitive!("\\Cancer", "\u{264B}");
  DefPrimitive!("\\Leo", "\u{264C}");
  DefPrimitive!("\\Virgo", "\u{264D}");
  DefPrimitive!("\\Libra", "\u{264E}");
  DefMacro!("\\AEq", "\\Libra");
  DefPrimitive!("\\Scorpius", "\u{264F}");
  DefPrimitive!("\\Sagittarius", "\u{2650}");
  DefPrimitive!("\\Capricornus", "\u{2651}");
  DefPrimitive!("\\Aquarius", "\u{2652}");
  DefPrimitive!("\\Pisces", "\u{2653}");

  DefPrimitive!("\\diameter", "\u{2300}");
  DefPrimitive!("\\sq", "\u{25A1}");

  DefPrimitive!("\\arcdeg", "\u{00B0}");
  Let!("\\degr", "\\arcdeg");
  DefPrimitive!("\\arcmin", "\u{2032}");
  DefPrimitive!("\\arcsec", "\u{2033}");
  DefMacro!("\\nodata", " ~$\\cdots$~ ");

  // Perl L491-498: \aas@@fstack constructor — formats astronomical unit
  // superscripts. Perl computes scriptpos dynamically as
  // "mid" . $stomach->getScriptLevel — the trailing digit signals
  // SUPERSCRIPTOP nesting depth (0 at top level, 1 inside a script,
  // etc.). Rust previously hard-coded 'mid1', breaking nested usage.
  // Also pickled `font => { shape => 'upright' }` (Perl L498) — the
  // raised symbol is upright by convention regardless of the
  // surrounding italic math font.
  // Perl 98f6e5de (2025-08-12) added `sizer => '#2'` so the sizer is the
  // symbol body (e.g. `d` in `\fd`), not the whole reversion — otherwise
  // nested fstack expressions miscompute layout width.
  DefConstructor!("\\aas@@fstack Undigested {}",
    "<ltx:XMApp role='POSTFIX'>\
       <ltx:XMTok role='SUPERSCRIPTOP' scriptpos='#scriptpos'/>\
       <ltx:XMTok>.</ltx:XMTok>\
       <ltx:XMWrap>#2</ltx:XMWrap>\
     </ltx:XMApp>",
    bounded => true,
    font => { shape => "upright" },
    reversion => "#1",
    sizer => "#2",
    properties => sub[_args] {
      Ok(stored_map!("scriptpos" => s!("mid{}", stomach::get_script_level())))
    }
  );

  // Perl aas_support.sty.ltxml L499: \aas@fstack{sym} — user-facing wrapper
  // around \aas@@fstack that enforces math mode via \ensuremath. This is the
  // CS other aastex-family bindings invoke when composing astronomical-unit
  // stacks; the Rust port had only the internal \aas@@fstack DefConstructor,
  // so direct consumers of \aas@fstack hit undefined-CS.
  DefMacro!("\\aas@fstack{}", "\\ensuremath{\\aas@@fstack{#1}}");

  DefMacro!("\\fd", "\\ensuremath{\\@fd}");
  DefMacro!("\\fh", "\\ensuremath{\\@fh}");
  DefMacro!("\\fm", "\\ensuremath{\\@fm}");
  DefMacro!("\\fs", "\\ensuremath{\\@fs}");
  DefMacro!("\\fdg", "\\ensuremath{\\@fdg}");
  DefMacro!("\\farcm", "\\ensuremath{\\@farcm}");
  DefMacro!("\\farcs", "\\ensuremath{\\@farcs}");
  DefMacro!("\\fp", "\\ensuremath{\\@fp}");

  // Perl L510-517: DefMath for internal \@f* macros — astronomical unit symbols
  DefMath!("\\@fd", "\\aas@@fstack{\\fd}{d}", role => "ID", meaning => "day", alias => "\\fd");
  DefMath!("\\@fh", "\\aas@@fstack{\\fh}{h}", role => "ID", meaning => "hour", alias => "\\fh");
  DefMath!("\\@fm", "\\aas@@fstack{\\fm}{m}", role => "ID", meaning => "minute", alias => "\\fm");
  DefMath!("\\@fs", "\\aas@@fstack{\\fs}{s}", role => "ID", meaning => "second", alias => "\\fs");
  DefMath!("\\@fdg", "\\aas@@fstack{\\fdg}{\\circ}", role => "ID", meaning => "degree", alias => "\\fdg");
  DefMath!("\\@farcm", "\\aas@@fstack{\\farcm}{\\prime}", role => "ID", meaning => "arcminute", alias => "\\farcm");
  DefMath!("\\@farcs", "\\aas@@fstack{\\farcs}{\\prime\\prime}", role => "ID", meaning => "arcsecond", alias => "\\farcs");
  DefMath!("\\@fp", "\\aas@@fstack{\\fp}{p}");

  DefMacro!("\\onehalf", "\\ifmmode\\case{1}{2}\\else\\text@onehalf\\fi");
  DefPrimitive!("\\text@onehalf", "\u{00BD}");
  DefMacro!("\\onethird", "\\ifmmode\\case{1}{3}\\else\\text@onethird\\fi");
  DefPrimitive!("\\text@onethird", "\u{2153}");
  DefMacro!("\\twothirds", "\\ifmmode\\case{2}{3}\\else\\text@twothirds\\fi");
  DefPrimitive!("\\text@twothirds", "\u{2154}");
  DefMacro!("\\onequarter", "\\ifmmode\\case{1}{4}\\else\\text@onequarter\\fi");
  DefPrimitive!("\\text@onequarter", "\u{00BC}");
  DefMacro!("\\threequarters", "\\ifmmode\\case{3}{4}\\else\\text@threequarters\\fi");
  DefPrimitive!("\\text@threequarters", "\u{00BE}");

  // Photometric bands — Perl aas_support.sty.ltxml L529-533. Each takes
  // `bounded => 1, font => { shape => 'italic' }` so the italicization
  // applies only to the band glyph and not to surrounding text — without
  // bounded, an `\ubvr` mid-paragraph would italicize all subsequent text
  // until the next font reset. Match Perl on both flags.
  DefPrimitive!("\\ubvr", "UBVR", bounded => true, font => { shape => "italic" });
  DefPrimitive!("\\ub", "U\u{2000}B", bounded => true, font => { shape => "italic" });
  DefPrimitive!("\\bv", "B\u{2000}V", bounded => true, font => { shape => "italic" });
  DefPrimitive!("\\vr", "V\u{2000}R", bounded => true, font => { shape => "italic" });
  DefPrimitive!("\\ur", "U\u{2000}R", bounded => true, font => { shape => "italic" });

  // amssymb aliases
  RequirePackage!("latexsym");
  RequirePackage!("amssymb");

  Let!("\\la", "\\lesssim");
  Let!("\\ga", "\\gtrsim");

  // Nominal conversion constants — Perl L545-560
  DefMacro!("\\nomSolarEffTemp", "\\leavevmode\\hbox{\\boldmath$\\mathcal{T}^{\\rm N}_{\\mathrm{eff}\\odot}$}");
  DefMacro!("\\nomTerrEqRadius", "\\leavevmode\\hbox{\\boldmath$\\mathcal{R}^{\\rm N}_{E\\mathrm e}$}");
  DefMacro!("\\nomTerrPolarRadius", "\\leavevmode\\hbox{\\boldmath$\\mathcal{R}^{\\rm N}_{E\\mathrm p}$}");
  DefMacro!("\\nomJovianEqRadius", "\\leavevmode\\hbox{\\boldmath$\\mathcal{R}^{\\rm N}_{J\\mathrm e}$}");
  DefMacro!("\\nomJovianPolarRadius", "\\leavevmode\\hbox{\\boldmath$\\mathcal{R}^{\\rm N}_{J\\mathrm p}$}");
  DefMacro!("\\nomTerrMass", "\\leavevmode\\hbox{\\boldmath$(\\mathcal{GM})^{\\rm N}_{\\mathrm E}$}");
  DefMacro!("\\nomJovianMass", "\\leavevmode\\hbox{\\boldmath$(\\mathcal{GM})^{\\rm N}_{\\mathrm J}$}");
  DefMacro!("\\Qnom", "\\leavevmode\\hbox{\\boldmath$\\mathcal{Q}^{\\rm N}_{\\odot}$}");
  Let!("\\Qn", "\\Qnom");
  DefMacro!("\\nom{}", "\\leavevmode\\hbox{\\boldmath$\\mathcal{#1}^{\\rm N}_{\\odot}$}");
  DefMacro!("\\Eenom{}", "\\leavevmode\\hbox{\\boldmath$\\mathcal{#1}^{\\rm N}_{Ee}$}");
  DefMacro!("\\Epnom{}", "\\leavevmode\\hbox{\\boldmath$\\mathcal{#1}^{\\rm N}_{Ep}$}");
  DefMacro!("\\Jenom{}", "\\leavevmode\\hbox{\\boldmath$\\mathcal{#1}^{\\rm N}_{Je}$}");
  DefMacro!("\\Jpnom{}", "\\leavevmode\\hbox{\\boldmath$\\mathcal{#1}^{\\rm N}_{Jp}$}");

  // 2.17.5 Hypertext — Perl L563-577
  // Perl L565: RequirePackage('url') — re-required here alongside the
  // hypertext definitions so `\url{}` is guaranteed loaded before
  // `\anchor`/`\@@email`/etc. AAS-macros that route URL content. The
  // package loader no-ops a re-require, so this is a faithful
  // transcription, not a repeated load.
  RequirePackage!("url");
  DefConstructor!("\\anchor Semiverbatim Semiverbatim", "<ltx:ref href='#1'>#2</ltx:ref>",
    enter_horizontal => true);
  DefConstructor!("\\@@email Semiverbatim", "<ltx:ref href='mailto:#1'>#1</ltx:ref>",
    enter_horizontal => true);

  // Misc
  DefMacro!("\\eqsecnum",
    "\\@addtoreset{equation}{section}\\def\\theequation{\\arabic{section}-\\arabic{equation}}");

  DefMacro!("\\singlespace", "");
  DefMacro!("\\doublespace", "");
  DefMacro!("\\tighten", "");
  DefMacro!("\\tightenlines", "");
  DefMacro!("\\nohyphenation", "");
  DefMacro!("\\offhyphenation", "");
  DefMacro!("\\ptlandscape", "");
  DefMacro!("\\refpar", "");
  DefMacro!("\\traceoutput", "");
  DefMacro!("\\tracingplain", "");

  DefMacro!("\\noprint {}", "");
  DefMacro!("\\figsetstart", "{\\bf Fig. Set}");
  DefMacro!("\\figsetend", "");
  DefMacro!("\\figsetgrpstart", "");
  DefMacro!("\\figsetgrpend", "");
  DefMacro!("\\figsetnum {}", "{\\bf #1.}");
  DefMacro!("\\figsettitle {}", "{\\bf #1}");
  DefMacro!("\\figsetgrpnum {}", "");
  DefMacro!("\\figsetgrptitle {}", "");
  DefMacro!("\\figsetplot {}", "");
  DefMacro!("\\figsetgrpnote {}", "");
});
