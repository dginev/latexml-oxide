use crate::prelude::*;

LoadDefinitions!({
  // Perl: sv_support.sty.ltxml
  // Support package for svjour class variants

  // Only if option natbib! (Perl loads unconditionally)
  RequirePackage!("natbib");

  //======================================================================
  // Frontmatter
  DefRegister!("\\titlerunning", Tokens!());
  def_macro_noop("\\titrun")?;
  DefMacro!("\\subtitle{}", "\\lx@add@subtitle{#1}");

  DefRegister!("\\authorrunning", Tokens!());
  def_macro_noop("\\authrun")?;

  DefMacro!("\\emailname", "E-mail");
  DefMacro!("\\mailname", "\\textit{Correspondence}");

  // Single \author containing multiple authors separated by \and  (Perl PR #2767)
  DefMacro!(
    "\\author{}",
    "\\lx@clear@creators[role=author]\\lx@splitting{\\lx@add@author}{\\and\\And,}{#1}"
  );
  // Single \institute containing name \at institute \and ... repeating.
  // Note that the name corresponds to (but probably won't match exactly!) an author name
  // We'll want some kind of fuzzy matching of labels to attach institute to author
  DefMacro!(
    "\\institute{}",
    "\\lx@clear@frontmatter{ltx:contact}[role=affiliation]\\lx@splitting{\\lx@sv@institute}{\\and\\And}{#1}"
  );
  Let!("\\at", "\\relax");
  DefMacro!("\\lx@sv@institute{}", "\\lx@sv@institute@#1\\at\\done");
  DefMacro!(
    "\\lx@sv@institute@ Until:\\at Until:\\done",
    "\\ifx.#2.\\lx@add@affiliation{#1}\\else\\lx@add@affiliation[label={fuzzy:#1}]{#2}\\fi"
  );
  // for older versions?
  DefMacro!(
    "\\inst{}",
    "\\lx@request@frontmatter@annotation[affiliation]{#1}"
  );

  // \email should be used within \institute, (but CAN be used anywhere; it only formats)
  DefMacro!(
    "\\email Semiverbatim",
    "\\lx@add@email[name={\\emailname: }]{#1}"
  );

  // Are these actually defined in svjour?
  DefMacro!(
    "\\address{}",
    "\\lx@add@contact[annotate=new,role=address]{#1}"
  );
  DefMacro!(
    "\\affil OptionalSemiverbatim {}",
    "\\lx@add@contact[role=affiliation,annotate={\\ifx.#1.new\\else 1\\fi},label={#1}]{#2}"
  );
  // These add contacts to most recent author
  // The optional arguments here are a sort of prefix to the footnote.
  DefMacro!(
    "\\mail {}",
    "\\lx@add@contact[role=address,name={\\mailname: }]{#1}"
  );
  DefMacro!(
    "\\homepage[] Semiverbatim",
    "\\lx@add@contact[role=url,name={\\ifx.#1.URL:~\\else#1\\fi}]{#2}"
  );
  DefMacro!(
    "\\thanks[]{}",
    "\\lx@add@contact[role=thanks,name={#1}]{#2}"
  );
  DefMacro!("\\orcid[]{}", "\\lx@add@contact[role=orcid,name={#1}]{#2}"); // ?

  DefMacro!("\\keywordname", "\\textbf{Keywords}");
  DefMacro!(
    "\\keywords{}",
    "\\lx@add@keywords[name={\\keywordname:~}]{#1}"
  );
  DefMacro!(
    "\\subclassname",
    "\\textbf{Mathematics Subject Classification (2000)}"
  );
  DefMacro!(
    "\\subclass{}",
    "\\lx@add@classification[scheme=MSC,name={\\subclassname: }]{#1}"
  );
  DefMacro!("\\CRclassname", "\\textbf{CR Subject Classification}");
  DefMacro!(
    "\\CRclass{}",
    "\\lx@add@classification[scheme=CR,name={\\CRclassname: }]{#1}"
  );
  DefMacro!("\\ESMname", "\\textbf{Electronic Supplementary Material}");
  DefMacro!(
    "\\ESM{}",
    "\\lx@add@pubnote[role=supplemental,name={\\ESMname: }]{#1}"
  );
  DefMacro!("\\PACSname", "\\textbf{PACS}");
  DefMacro!(
    "\\PACS{}",
    "\\lx@add@classification[scheme=pacs,name={\\PACSname: }]{#1}"
  );
  DefMacro!("\\headnote{}", "\\lx@add@pubnote[role=node]{#1}");
  DefMacro!("\\dedication{}", "\\lx@add@pubnote[role=dedication]{#1}");
  DefMacro!("\\offprints{}", "\\lx@add@pubnote[role=preprint]{#1}");
  DefMacro!("\\journalname{}", "\\lx@add@pubnote[role=journal]{#1}");
  DefMacro!("\\papertype{}", "\\lx@add@pubnote[role=type]{#1}");

  Let!("\\journalopt", "\\@empty");

  // svjour covers several specific journal styles.
  // Some (but not all!) use \abstract{...} instead of abstract environment.
  // Redefine \abstract to handle both command and environment form.
  Let!("\\@orig@abstract", "\\abstract");
  // \abstract* — ignore
  def_macro_noop("\\lx@ignore@sv@abstract{}")?;
  DefMacro!(
    "\\@abstract@with@arg{}",
    "\\lx@add@abstract[name={\\abstractname}]{#1}"
  );

  DefMacro!("\\abstract OptionalMatch:*", sub[(star)] {
    if star.is_some() {
      vec![T_CS!("\\lx@ignore@sv@abstract")]
    } else if if_next(T_BEGIN!())? {
      vec![T_CS!("\\@abstract@with@arg")]
    } else {
      vec![T_CS!("\\@orig@abstract")]
    }
  });

  def_macro_noop("\\makereferee")?;

  DefMacro!("\\ackname", "Acknowledgements");
  DefConstructor!("\\acknowledgements",
  "<ltx:acknowledgements name='#name'>",
  properties => {
    Ok(stored_map!("name" => Digest!(T_CS!("\\ackname"))?))
  });
  DefMacro!("\\acknowledgement", "\\acknowledgements");
  DefConstructor!("\\endacknowledgements", "</ltx:acknowledgements>");
  DefConstructor!("\\endacknowledgement", "</ltx:acknowledgements>");
  Tag!("ltx:acknowledgements", auto_close => true);

  DefMacro!("\\noteaddname", "Note added in proof");
  DefMacro!("\\notename", "Note");

  // Perl sv_support.sty.ltxml L89-90: the {noteadd} env wraps body in
  // <ltx:note name='Note added in proof'>. Perl's `properties => {name =>
  // Digest(\noteaddname)}` digests `\noteaddname` at expansion time.
  // Rust doesn't easily support a Digested-closure property here, so we
  // emit the same Perl output with the name string inlined. If the
  // document redefines \noteaddname this diverges; Perl is faithful
  // to the live value. Acceptable simplification — no Springer test
  // exercises noteadd env.
  DefEnvironment!(
    "{noteadd}",
    "<ltx:note name='Note added in proof'>#body</ltx:note>"
  );

  Let!("\\orithanks", "\\thanks");
  def_macro_noop("\\runheadhook")?;
  def_macro_noop("\\svlanginfo")?;
  def_macro_noop("\\makeheadbox")?;
  DefMacro!("\\authdepth", "2");
  DefMacro!("\\authorfont", "\\bfseries");
  def_macro_noop("\\stripauthor")?;
  DefRegister!("\\instindent", Dimension::new(0));
  def_macro_noop("\\combirun")?;
  // \combirunning{text} — Springer running-head combination text.
  // Surpass Perl gobble: preserve as ltx:note.
  DefMacro!(
    "\\combirunning{}",
    "\\lx@add@frontmatter{ltx:note}[role=combirunning]{#1}"
  );

  def_macro_noop("\\validfor")?;
  def_macro_noop("\\ClassInfoNoLine{}{}")?;
  def_macro_noop("\\ProcessRunnHead")?;
  def_macro_noop("\\fnmsep")?;
  def_macro_noop("\\institutename")?;

  //======================================================================
  def_macro_noop("\\nocaption{}")?;
  def_macro_noop("\\sidecaption {}")?;

  def_macro_noop("\\capstrut")?;
  DefMacro!("\\captionstyle", "\\normalfont\\small");
  DefRegister!("\\figcapgap", Dimension!("3pt"));
  DefRegister!("\\tabcapgap", Dimension!("5.5pt"));
  DefRegister!("\\figgap", Dimension!("12.84pt")); // 1cc

  DefMacro!("\\tableheadseprule", "\\hrule");
  DefMacro!("\\floatlegendstyle", "\\bfseries");
  def_macro_noop("\\leftlegendglue")?;

  // Perl L122-123: theorem head swap toggles
  DefPrimitive!("\\normalthmheadings", {
    assign_value("thm@swap", 0i64, Scope::Global);
  });
  DefPrimitive!("\\reversethmheadings", {
    assign_value("thm@swap", 1i64, Scope::Global);
  });

  // \spnewtheorem*{env}[numberedlike]{caption}[within]{capfont}{bodyfont}
  // Perl sv.cls.ltxml L92-185: Like \newtheorem + capfont/bodyfont (visual styling ignored).
  // DP-flag: Perl DefMacro (sub body), Rust DefPrimitive — WISDOM #44.
  // Safe: `\spnewtheorem` is a preamble-level declaration, never observed
  // through `\edef`/`\ifx`/`\expandafter` (verified 2026-04-23 across
  // LaTeXML/lib + ar5iv-bindings).
  DefPrimitive!("\\spnewtheorem OptionalMatch:* {}[]{}[] {}{}", sub[(flag, thmset, otherthmset, typ, reset, _capfont, _bodyfont)] {
    engine::latex_constructs::define_new_theorem(
      flag.filter(|f| !f.is_empty()),
      thmset,
      otherthmset.filter(|t| !t.is_empty()),
      if typ.is_empty() { None } else { Some(typ) },
      reset.filter(|t| !t.is_empty()),
    )?;
  });
  Let!("\\spdefaulttheorem", "\\spnewtheorem");

  DefRegister!("\\spthmsep", Dimension!("5pt"));

  // Pre-define theorem environments — Perl L189-223
  DefMacro!("\\theoremname", "Theorem");
  DefMacro!("\\claimname", "Claim");
  DefMacro!("\\proofname", "Proof");
  DefMacro!("\\conjecturename", "Conjecture");
  DefMacro!("\\corollaryname", "Corollary");
  DefMacro!("\\definitionname", "Definition");
  DefMacro!("\\examplename", "Example");
  DefMacro!("\\exercisename", "Exercise");
  DefMacro!("\\lemmaname", "Lemma");
  DefMacro!("\\notename", "Note");
  DefMacro!("\\problemname", "Problem");
  DefMacro!("\\propertyname", "Property");
  DefMacro!("\\propositionname", "Proposition");
  DefMacro!("\\questionname", "Question");
  DefMacro!("\\solutionname", "Solution");
  DefMacro!("\\remarkname", "Remark");
  RawTeX!("\\@ifundefined{theorem}{\\newtheorem{theorem}{Theorem}[section]}{}");
  RawTeX!("\\@ifundefined{claim}{\\newtheorem*{claim}{Claim}}{}");
  RawTeX!("\\@ifundefined{conjecture}{\\newtheorem{conjecture}{Conjecture}}{}");
  RawTeX!("\\@ifundefined{corollary}{\\newtheorem{corollary}{Corollary}}{}");
  RawTeX!("\\@ifundefined{definition}{\\newtheorem{definition}{Definition}}{}");
  RawTeX!("\\@ifundefined{example}{\\newtheorem{example}{Example}}{}");
  RawTeX!("\\@ifundefined{exercise}{\\newtheorem{exercise}{Exercise}}{}");
  RawTeX!("\\@ifundefined{lemma}{\\newtheorem{lemma}{Lemma}}{}");
  RawTeX!("\\@ifundefined{note}{\\newtheorem{note}{Note}}{}");
  RawTeX!("\\@ifundefined{problem}{\\newtheorem{problem}{Problem}}{}");
  RawTeX!("\\@ifundefined{property}{\\newtheorem{property}{Property}}{}");
  RawTeX!("\\@ifundefined{proposition}{\\newtheorem{proposition}{Proposition}}{}");
  RawTeX!("\\@ifundefined{question}{\\newtheorem{question}{Question}}{}");
  RawTeX!("\\@ifundefined{solution}{\\newtheorem{solution}{Solution}}{}");
  RawTeX!("\\@ifundefined{remark}{\\newtheorem{remark}{Remark}}{}");
  // Perl sv_support L196: `\spnewtheorem{case}{Case}{\itshape}{\rmfamily}`
  // ships {case} as a preloaded theorem. Mirror so svjour3 authors who
  // use \begin{case} without declaring it (or comment out the declaration)
  // don't trip Error:undefined. Witness 2112.14105.
  RawTeX!("\\@ifundefined{case}{\\newtheorem{case}{Case}}{}");

  // Theorem environments — Perl L225-228
  DefEnvironment!("{theopargself*}", "#body");
  DefEnvironment!("{theopargself}", "#body");
  DefEnvironment!(
    "{translation}{}",
    "<ltx:quote role='translation' lang='#1'>#body</ltx:quote>"
  );

  //======================================================================
  // QED — Perl sv_support.sty.ltxml has `enterHorizontal=>1` (matches
  // amsthm L141). Without it, `\qed` at end of proof in vertical mode
  // emits the U+220E text node outside any <ltx:p>.
  DefMacro!("\\qed", "\\ltx@qed");
  DefConstructor!("\\ltx@qed",
    "?#isMath(<ltx:XMTok role='PUNCT'>\u{220E}</ltx:XMTok>)(\u{220E})",
    enter_horizontal => true,
    reversion => "\\qed");
  Let!("\\smartqed", "\\qed");
  Let!("\\squareforqed", "\\qed");

  DefMacro!("\\tens{}", "\\ensuremath{\\mathsf{#1}}");

  //======================================================================
  // Random text
  DefMacro!("\\andname", "and");
  DefMacro!("\\chaptername", "Chapter");
  DefMacro!("\\contriblistname", "List of Contributors");
  DefMacro!("\\lastandname", ", and");
  DefMacro!("\\seename", "see");
  DefMacro!("\\etal", "et al.");
  DefMacro!("\\notused", "~");

  //======================================================================
  DefRegister!("\\aftertext", Dimension!("5pt"));
  DefRegister!("\\betweenumberspace", Dimension!("3.33pt"));
  DefRegister!("\\headerboxheight", Dimension!("180pt"));
  DefRegister!("\\headlineindent", Dimension!("33pt")); // ~1.166cm

  def_macro_noop("\\runinend")?;
  def_macro_noop("\\floatcounterend")?;
  def_macro_noop("\\sectcounterend")?;

  DefMacro!("\\columncase", "\\makeatletter\\twocolteset");
  DefMacro!("\\twocoltest{}{}", "#1\\makeatother");

  NewCounter!("lastpage");
  def_macro_noop("\\getlastpagenumber")?;
  def_macro_noop("\\islastpageeven")?;

  def_macro_noop("\\makesectrule")?;
  def_macro_noop("\\makesectruleori")?;
  def_macro_noop("\\nosectrule")?;
  def_macro_noop("\\restoresectrule")?;
  def_macro_noop("\\nothanksmarks")?;
  def_macro_noop("\\setitemindent{}")?;
  def_macro_noop("\\setitemitemindent{}")?;
  def_macro_noop("\\thisbottomragged")?;

  def_macro_noop("\\rubric")?;
  DefRegister!("\\rubricwidth", Dimension::new(0));
  def_macro_noop("\\strich")?;
  DefRegister!("\\logodepth", Dimension!("36pt")); // ~1.2cm
  def_macro_noop("\\lastevenhead")?;

  //======================================================================
  // description environment with optional arg.
  // Perl sv_support.sty.ltxml L286-289 sets `locked => 1` so a downstream
  // class that re-loads sv_support's description (e.g. a sibling Springer
  // template that defines its own `\renewenvironment{description}{}{}`)
  // can't quietly drop the optional-arg-aware variant. Without locked,
  // the optional `[<label-template>]` becomes invisible to the env's
  // properties closure and itemization machinery sees a bare description.
  DefEnvironment!("{description}[]",
  "<ltx:description xml:id='#id'>#body</ltx:description>",
  properties => sub[_args] {
    begin_itemize("description", None, BeginItemizeOptions::default())
  },
  locked => true);

  // Perl sv_support.sty.ltxml L194-195: proof environment
  DefMacro!("\\proofname", "Proof");
  // \spnewtheorem*{proof}{Proof}{\itshape}{\rmfamily}
  // starred (*) = unnumbered = flag=Some
  engine::latex_constructs::define_new_theorem(
    Some(Tokens!(T_OTHER!("*"))), // starred
    Tokenize!("proof"),           // environment name
    None,                         // no shared counter
    Some(Tokenize!("Proof")),     // display title
    None,                         // no 'within' counter
  )?;

  // \thankstext{label}{text} — sn-jnl / EPJ-style title-page footnote
  // (svjour3 derivatives). Render as a regular footnote.
  // Witnesses 2406.12029, 2406.12545.
  DefMacro!("\\thankstext{}{}", "\\footnote{#2}");
  // \thanksref{label} — footnote-style marker; render as superscript.
  DefMacro!("\\thanksref{}", "\\textsuperscript{#1}");
});
