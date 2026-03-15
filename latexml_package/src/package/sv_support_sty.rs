use crate::prelude::*;
LoadDefinitions!({
  // Perl: sv_support.sty.ltxml
  // Support package for svjour class variants

  // Only if option natbib! (Perl loads unconditionally)
  RequirePackage!("natbib");
  RequirePackage!("inst_support");

  //======================================================================
  // Frontmatter
  DefRegister!("\\titlerunning", Tokens!());
  DefMacro!("\\titrun", "");
  DefMacro!("\\subtitle{}", "\\@add@frontmatter{ltx:subtitle}{#1}");

  DefRegister!("\\authorrunning", Tokens!());
  DefMacro!("\\authrun", "");

  DefMacro!("\\emailname", "E-mail");
  DefConstructor!("\\@@@email{}", "^ <ltx:contact role='email' name='#name'>#1</ltx:contact>",
    properties => {
      Ok(stored_map!("name" => Digest!(T_CS!("\\emailname"))?))
    });
  DefMacro!("\\email Semiverbatim", "\\@add@to@frontmatter{ltx:creator}{\\@@@email{#1}}");
  DefMacro!("\\mailname", "\\textit{Correspondence}");
  DefConstructor!("\\@@@mail{}", "^ <ltx:contact role='address' name='#name'>#1</ltx:contact>",
    properties => {
      Ok(stored_map!("name" => Digest!(T_CS!("\\mailname"))?))
    });
  DefMacro!("\\mail{}", "\\@add@to@frontmatter{ltx:creator}{\\@@@mail{#1}}");

  DefMacro!("\\keywordname", "\\textbf{Keywords}");
  DefMacro!("\\keywords{}", "\\@add@frontmatter{ltx:keywords}[name={\\keywordname}]{#1}");
  DefMacro!("\\subclassname", "\\textbf{Mathematics Subject Classification (2000)}");
  DefMacro!("\\subclass{}", "\\@add@frontmatter{ltx:classification}[scheme=MSC,name={\\subclassname}]{#1}");
  DefMacro!("\\CRclassname", "\\textbf{CR Subject Classification}");
  DefMacro!("\\CRclass{}", "\\@add@frontmatter{ltx:classification}[scheme=CR,name={\\CRclassname}]{#1}");
  DefMacro!("\\ESMname", "\\textbf{Electronic Supplementary Material}");
  DefMacro!("\\ESM{}", "\\@add@frontmatter{ltx:note}[role=supplemental,name={\\ESMname}]{#1}");
  DefMacro!("\\PACSname", "\\textbf{PACS}");
  DefMacro!("\\PACS{}", "\\@add@frontmatter{ltx:classification}[scheme=pacs,name={\\PACSname}]{#1}");
  DefMacro!("\\headnote{}", "\\@add@frontmatter{ltx:note}{#1}");
  DefMacro!("\\dedication{}", "\\@add@frontmatter{ltx:note}[role=dedicatory]{#1}");
  DefMacro!("\\offprints{}", "\\@add@frontmatter{ltx:note}[role=offprints]{#1}");
  DefMacro!("\\journalname{}", "\\@add@frontmatter{ltx:note}[role=journal]{#1}");
  DefMacro!("\\papertype{}", "\\@add@frontmatter{ltx:classification}[scheme=papertype]{#1}");

  Let!("\\journalopt", "\\@empty");

  // svjour covers several specific journal styles.
  // Some (but not all!) use \abstract{...} instead of abstract environment.
  // Redefine \abstract to handle both command and environment form.
  Let!("\\@orig@abstract", "\\abstract");
  // \abstract* — ignore
  DefMacro!("\\lx@ignore@sv@abstract{}", "");
  DefMacro!("\\@abstract@with@arg{}", "\\@add@frontmatter{ltx:abstract}[name={\\abstractname}]{#1}");

  DefMacro!("\\abstract OptionalMatch:*", sub[(star)] {
    if star.is_some() {
      vec![T_CS!("\\lx@ignore@sv@abstract")]
    } else if gullet::if_next(T_BEGIN!())? {
      vec![T_CS!("\\@abstract@with@arg")]
    } else {
      vec![T_CS!("\\@orig@abstract")]
    }
  });

  DefMacro!("\\makereferee", "");

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

  Let!("\\orithanks", "\\thanks");
  DefMacro!("\\runheadhook", "");
  DefMacro!("\\svlanginfo", "");
  DefMacro!("\\makeheadbox", "");
  DefMacro!("\\authdepth", "2");
  DefMacro!("\\authorfont", "\\bfseries");
  DefMacro!("\\stripauthor", "");
  DefRegister!("\\instindent", Dimension::new(0));
  DefMacro!("\\combirun", "");
  DefMacro!("\\combirunning{}", "");

  DefMacro!("\\validfor", "");
  DefMacro!("\\ClassInfoNoLine{}{}", "");
  DefMacro!("\\ProcessRunnHead", "");
  DefMacro!("\\fnmsep", "");
  DefMacro!("\\institutename", "");

  //======================================================================
  DefMacro!("\\nocaption{}", "");
  DefMacro!("\\sidecaption {}", "");

  DefMacro!("\\capstrut", "");
  DefMacro!("\\captionstyle", "\\normalfont\\small");
  DefRegister!("\\figcapgap", Dimension!("3pt"));
  DefRegister!("\\tabcapgap", Dimension!("5.5pt"));
  DefRegister!("\\figgap", Dimension!("12.84pt")); // 1cc

  DefMacro!("\\tableheadseprule", "\\hrule");
  DefMacro!("\\floatlegendstyle", "\\bfseries");
  DefMacro!("\\leftlegendglue", "");

  // TODO: \spnewtheorem — complex theorem definition (porting deferred)
  DefMacro!("\\spnewtheorem OptionalMatch:* {}[]{}[] {}{}", "");
  Let!("\\spdefaulttheorem", "\\spnewtheorem");

  DefRegister!("\\spthmsep", Dimension!("5pt"));

  //======================================================================
  DefMacro!("\\qed", "\\ltx@qed");
  DefConstructor!("\\ltx@qed",
    "?#isMath(<ltx:XMTok role='PUNCT'>\u{220E}</ltx:XMTok>)(\u{220E})",
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

  DefMacro!("\\runinend", "");
  DefMacro!("\\floatcounterend", "");
  DefMacro!("\\sectcounterend", "");

  DefMacro!("\\columncase", "\\makeatletter\\twocolteset");
  DefMacro!("\\twocoltest{}{}", "#1\\makeatother");

  NewCounter!("lastpage");
  DefMacro!("\\getlastpagenumber", "");
  DefMacro!("\\islastpageeven", "");

  DefMacro!("\\makesectrule", "");
  DefMacro!("\\makesectruleori", "");
  DefMacro!("\\nosectrule", "");
  DefMacro!("\\restoresectrule", "");
  DefMacro!("\\nothanksmarks", "");
  DefMacro!("\\setitemindent{}", "");
  DefMacro!("\\setitemitemindent{}", "");
  DefMacro!("\\thisbottomragged", "");

  DefMacro!("\\rubric", "");
  DefRegister!("\\rubricwidth", Dimension::new(0));
  DefMacro!("\\strich", "");
  DefRegister!("\\logodepth", Dimension!("36pt")); // ~1.2cm
  DefMacro!("\\lastevenhead", "");

  //======================================================================
  // description environment with optional arg
  DefEnvironment!("{description}[]",
    "<ltx:description xml:id='#id'>#body</ltx:description>",
    properties => sub[_args] {
      begin_itemize("description", None, BeginItemizeOptions::default())
    });
});
