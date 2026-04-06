/// OmniBus.cls — fallback class for documents with unknown document classes.
/// Port of LaTeXML/Package/OmniBus.cls.ltxml (312 lines).
///
/// Defines common frontmatter commands, theorem environments, natbib autoloads,
/// and various compatibility macros encountered in real-world arxiv submissions.
use crate::prelude::*;

LoadDefinitions!({
  // Load article as base class — Perl L36
  LoadClass!("article");

  // Common packages — Perl L39-49
  RequirePackage!("inst_support");
  RequirePackage!("epsf");
  RequirePackage!("graphicx");
  RequirePackage!("aas_macros");

  // natbib autoloads: load natbib when citation commands are used
  for trigger in [
    "\\citet", "\\citep", "\\citealt", "\\citealp", "\\citenum",
    "\\citeauthor", "\\citefullauthor", "\\citeyear", "\\citeyearpar",
    "\\citeauthoryear", "\\setcitestyle", "\\bibpunct",
  ] {
    let cs = T_CS!(trigger);
    if !IsDefined!(&cs) {
      let cs_clone = cs;
      def_macro(cs, None,
        latexml_core::definition::ExpansionBody::Closure(Rc::new(move |_args| {
          require_package("natbib", RequireOptions::default())?;
          Ok(Tokens::new(vec![cs_clone]))
        })), None)?;
    }
  }

  // Frontmatter environments
  DefEnvironment!("{frontmatter}", "#body");
  DefEnvironment!("{mainmatter}", "#body");
  DefEnvironment!("{backmatter}", "#body");

  // Common frontmatter macros
  DefMacro!("\\shorttitle{}", "\\@add@frontmatter{ltx:toctitle}{#1}");
  DefMacro!("\\subtitle{}", "\\@add@frontmatter{ltx:subtitle}{#1}");
  DefMacro!("\\shortauthor{}", "");

  DefRegister!("\\titlerunning", Tokens!());
  DefRegister!("\\authorrunning", Tokens!());
  Let!("\\runningauthor", "\\authorrunning");
  Let!("\\runauthor", "\\authorrunning");

  DefMacro!("\\runningtitle{}", None, None);
  Let!("\\runninghead", "\\runningtitle");
  DefMacro!("\\shortauthors{}", None, None);
  DefMacro!("\\alignauthor", None, None);

  // Email
  DefConstructor!("\\@@@email{}{}", "^<ltx:contact role='#2'>#1</ltx:contact>");
  DefMacro!("\\email{}", "\\@add@to@frontmatter{ltx:creator}{\\@@@email{#1}{email}}");
  Let!("\\emailaddr", "\\email");
  DefMacro!("\\emailname", "E-mail");

  // Affiliations
  DefConstructor!("\\@@@affiliation{}", "^<ltx:contact role='affiliation'>#1</ltx:contact>");
  DefMacro!("\\affil{}", "\\@add@to@frontmatter{ltx:creator}{\\@@@affiliation{#1}}");
  DefMacro!("\\affiliation{}", "\\@add@to@frontmatter{ltx:creator}{\\@@@affiliation{#1}}");

  DefConstructor!("\\@@@address{}", "^<ltx:contact role='address'>#1</ltx:contact>");
  DefMacro!("\\address[]{}", "\\@add@to@frontmatter{ltx:creator}{\\@@@address{#2}}");
  Let!("\\affaddr", "\\address");

  // Name components (functionally no-ops for LaTeXML)
  DefMacro!("\\prefix{}", "#1");
  DefMacro!("\\suffix{}", "#1");
  DefMacro!("\\fnms{}", "#1");
  DefMacro!("\\snm{}", "#1");

  // Keywords
  DefMacro!("\\keywords{}", "\\@add@frontmatter{ltx:keywords}{#1}");
  DefMacro!("\\kword{}", "\\@add@frontmatter{ltx:keywords}{#1}");

  // Classification
  DefMacro!("\\classification{}", "\\@add@frontmatter{ltx:classification}{#1}");
  DefMacro!("\\pacs{}", "\\@add@frontmatter{ltx:classification}[scheme=pacs]{#1}", locked => true);

  // Dates and metadata
  DefMacro!("\\editors{}", "\\@add@frontmatter{ltx:note}[role=editors]{#1}");
  DefMacro!("\\received{}", "\\@add@frontmatter{ltx:date}[role=received]{#1}");
  DefMacro!("\\revised{}", "\\@add@frontmatter{ltx:date}[role=revised]{#1}");
  DefMacro!("\\accepted{}", "\\@add@frontmatter{ltx:date}[role=accepted]{#1}");
  DefMacro!("\\pubyear{}", "\\@add@frontmatter{ltx:date}[role=publication]{#1}");
  DefMacro!("\\copyrightyear{}", "\\@add@frontmatter{ltx:date}[role=copyright]{#1}");
  DefMacro!("\\preprint{}", "\\@add@frontmatter{ltx:note}[role=preprint]{#1}");
  DefMacro!("\\dedicated{}", "\\@add@frontmatter{ltx:note}[role=dedicated]{#1}");
  DefMacro!("\\articletype{}", "\\@add@frontmatter{ltx:note}[role=articletype]{#1}");
  DefMacro!("\\journal{}", "\\@add@frontmatter{ltx:note}[role=journal]{#1}");
  DefMacro!("\\jname{}", "\\@add@frontmatter{ltx:note}[role=journal]{#1}");
  DefMacro!("\\volume{}", "\\@add@frontmatter{ltx:note}[role=volume]{#1}");
  DefMacro!("\\titlenote{}", "\\@add@frontmatter{ltx:note}[role=titlenote]{#1}");

  // Abstract aliases
  Let!("\\abstracts", "\\abstract");
  Let!("\\abst", "\\abstract");

  // Acknowledgements
  DefConstructor!("\\acknowledgments", "<ltx:acknowledgements name='#name'>",
    properties => {
      Ok(stored_map!("name" => stomach::digest(T_CS!("\\acknowledgmentsname"))?))
    }
  );
  DefConstructor!("\\endacknowledgments", "</ltx:acknowledgements>");
  Tag!("ltx:acknowledgements", auto_close => true);
  DefMacro!("\\acknowledgmentsname", "Acknowledgements");
  Let!("\\acknowledgements", "\\acknowledgments");
  Let!("\\endacknowledgements", "\\endacknowledgments");
  Let!("\\theacknowledgments", "\\acknowledgments");
  Let!("\\endtheacknowledgments", "\\endacknowledgments");

  // Common utility macros
  DefMacro!("\\comment{}", "");
  DefMacro!("\\etal", "\\textit{et al.}");
  DefMacro!("\\firstsection", "");

  // Section aliases from 1990s arXiv
  DefMacro!("\\Section", "\\@startsection{section}{1}{}{}{}{}", locked => true);
  DefMacro!("\\Subsection", "\\@startsection{subsection}{2}{}{}{}{}", locked => true);
  DefMacro!("\\Subsubsection", "\\@startsection{subsubsection}{3}{}{}{}{}", locked => true);
  DefMacro!("\\Paragraph", "\\@startsection{paragraph}{4}{}{}{}{}", locked => true);
  DefMacro!("\\Subparagraph", "\\@startsection{subparagraph}{5}{}{}{}{}", locked => true);

  // Author block environment
  DefEnvironment!("{aug}", "#body");

  // Affiliation marks — Perl L148-157
  DefMacro!("\\altaffilmark{}", "\\@altaffilmark{#1}");
  DefConstructor!("\\@altaffilmark{}", "<ltx:note role='affiliationmark' mark='#1'/>",
    enter_horizontal => true);
  Let!("\\affilnum", "\\@altaffilmark");
  DefConstructor!("\\altaffiltext{}{}", "<ltx:note role='affiliationtext' mark='#1'>#2</ltx:note>");
  DefRegister!("\\affilskip" => Dimension::new(0));

  // More metadata — Perl L234-256
  DefMacro!("\\communicated{}", "\\@add@frontmatter{ltx:date}[role=communicated]{#1}");
  DefMacro!("\\presented{}", "\\@add@frontmatter{ltx:date}[role=presented]{#1}");
  DefMacro!("\\issue{}", "\\@add@frontmatter{ltx:note}[role=issue]{#1}");
  DefMacro!("\\terms{}", "\\@add@frontmatter{ltx:note}[role=terms]{#1}");
  DefMacro!("\\conferenceinfo{}{}", "\\@add@frontmatter{ltx:note}[role=conference]{#1 #2}");
  DefMacro!("\\category{}{}{}[]", "\\@add@frontmatter{ltx:classification}[scheme=category]{#1 #2 #3}");
  DefMacro!("\\resumen{}", "\\@add@frontmatter{ltx:abstract}{#1}");
  Let!("\\CopyrightYear", "\\copyrightyear");
  DefRegister!("\\confinfo" => Tokens!());
  DefRegister!("\\acmcopyr" => Tokens!());
  DefRegister!("\\copyrightetc" => Tokens!());
  Let!("\\crdata", "\\acmcopyr");

  // DOI — Perl L204
  DefConstructor!("\\lx@doi{}", "<ltx:ref href='https://doi.org/#1'>#1</ltx:ref>",
    enter_horizontal => true);

  // References — Perl L274-284
  DefConstructor!("\\references", "<ltx:bibliography xml:id='#id'><ltx:biblist>");
  DefConstructor!("\\endreferences", "</ltx:biblist></ltx:bibliography>");
  Let!("\\reference", "\\bibitem");

  // Misc compatibility
  DefMacro!("\\thanksref{}", "");
  DefMacro!("\\numberofauthors{}", "");
  DefMacro!("\\printead{}", "");
  DefMacro!("\\firstpage{}", "");
  DefMacro!("\\lastpage{}", "");
  DefMacro!("\\corref{}", "");
  DefMacro!("\\listofauthors{}", "");
  DefMacro!("\\indexauthor{}", "");
  DefMacro!("\\preface", "");
  DefMacro!("\\thankstext", "");
  Let!("\\fulladdresses", "\\address");
  Let!("\\smonth", "\\month");
  Let!("\\syear", "\\year");
  DefMacro!("\\ion{}{}", "{#1 \\textsc{#2}}");
  DefMacro!("\\kwd[]{}", "\\@add@frontmatter{ltx:keywords}{#2, }");

  // Speaker macro — Perl L98
  DefMacro!("\\speaker{}", "\\@add@frontmatter{ltx:creator}[role=speaker]{\\@personname{#1}}");

  // EAD macro — Perl L95
  DefMacro!("\\ead{}[]", "\\@add@to@frontmatter{ltx:creator}{\\@@@email{#1}{#2}}");
});
