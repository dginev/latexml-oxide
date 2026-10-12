//! `l-aa.sty` (also shipped as `laa.sty`, the same file, and as `l-aa-ps.sty`, the same with PostScript fonts: copy in
//! astro-ph9606053, used by astro-ph9505054): Springer-Verlag's LaTeX 2.09 style for Astronomy &
//! Astrophysics, "L-AA vers. 3.0" (1992-1996), loaded as `\documentstyle{l-aa}`. Neither TeX Live nor most papers
//! using it ship it, so the document fell to OmniBus and stopped at `\thesaurus` (843 run-336 papers: astro-ph9804097,
//! astro-ph9405007, astro-ph0211472). It is the predecessor of `aa.cls`, which kept its document syntax (`\institute`
//! with `\inst`, `\offprints`, `\mail`, `\date`, `{abstract}`, `\keywords`, `{acknowledgements}`, the astronomical
//! symbols, plain TeX's `\cases` and `\pmatrix` in displays: astro-ph9908044), so this is the `aa` class binding with
//! what L-AA itself adds; a binding for the class, which `\documentstyle` prefers
//! (latex_constructs.pool.ltxml:105-108).
//! Rust-only: Perl has no `l-aa` binding. A paper shipping the file gets this binding too, as a binding outranks a raw
//! file (astro-ph9601070, astro-ph9601165).
use latexml_package::prelude::*;

LoadDefinitions!({
  DeclareOption!(None, {
    Digest!("\\PassOptionsToClass{\\CurrentOption}{aa}")?;
  });
  ProcessOptions!();
  LoadClass!("aa");
  // l-aa.sty:453-454 `\def\thesaurus#1{\gdef\@thesaurus{#1}}`: the A&A thesaurus codes, printed in the title's head box
  // as "Your thesaurus codes are: …" (:502-516 `\makeheadbox`) — the paper's subject classification, which the `aa`
  // binding (aa.cls, whose head box has none) drops. Read as the `\def` reads it, not `\long`: a `\par` in the argument
  // ends the call, "Paragraph ended before \thesaurus was complete" (tex.web §396), where a `%`-commented closing brace
  // (`… 02.19.1) % Shock waves }`, astro-ph9405007) otherwise made the rest of the document the codes.
  DefMacro!(
    "\\lx@l@aa@thesaurus{}",
    "\\lx@add@classification[scheme=thesaurus,name={Thesaurus codes:~}]{#1}"
  );
  DefMacro!("\\thesaurus", {
    let cs = T_CS!("\\thesaurus");
    let matching = set_scanner_status_checking_par(ScannerStatus::Matching, Some(cs), true);
    let codes = read_arg(ExpansionLevel::Off)?;
    match matching.end_taking_runaway_argument() {
      None => Ok(Invocation!(T_CS!("\\lx@l@aa@thesaurus"), vec![Some(codes)])),
      Some(runaway) => {
        abandon_runaway_call(cs, false, true, runaway)?;
        Ok(Tokens!())
      },
    }
  });
  // l-aa.sty:24-26: `\picplace{<height>}` frames the empty space a pasted-in figure fills; the `noframe` option makes
  // it the bare `\vspace` (:20).
  DefMacro!("\\picplace{}", "\\vspace{#1}");
  // l-aa.sty:381, :392 the float captions' labels, "Fig. 1." and "Table 1."
  // and the caption follows its label after a space, no colon (:371-377 `{\bf #1} #2`)
  RawTeX!("\\def\\fnum@figure{Fig.\\ \\thefigure.}\\def\\fnum@table{Table \\thetable.}");
  DefMacro!(
    "\\format@title@figure{}",
    "\\lx@tag[][ ]{\\lx@fnum@@{figure}}#1"
  );
  DefMacro!(
    "\\format@title@table{}",
    "\\lx@tag[][ ]{\\lx@fnum@@{table}}#1"
  );
  // l-aa.sty:648 `\def\keywords{\par\vspace{12pt}\noindent{\bf Key words: }}` takes no argument: the keywords are
  // the text after it, to the paragraph's end (`\keywords ISM: molecules --- …`, astro-ph0001079), or of a
  // `{keywords}` environment (astro-ph9609156, astro-ph9803302), or a group (`\keywords{supernova remnants --
  // pulsars}`, astro-ph9405007), the aa binding's argument (aa_support.sty.ltxml:96), which read the first letter of
  // the other two forms. Labelled as l-aa labels them, "Key words:".
  DefMacro!(
    "\\lx@l@aa@keywords{}",
    "\\lx@add@keywords[name={Key words:~}]{#1}"
  );
  DefMacro!("\\keywords", {
    skip_spaces()?;
    if if_next(T_BEGIN!())? {
      Ok(Tokens!(T_CS!("\\lx@l@aa@keywords")))
    } else {
      keywords_text(false)
    }
  });
  DefMacro!(T_CS!("\\begin{keywords}"), None, { keywords_text(true) });
  DefMacro!(T_CS!("\\end{keywords}"), None, "");
  // l-aa.sty:405-427 `\institute` and `\institutename`: the institutes numbered by position, `\and` between them,
  // which the authors' `\inst{n}` name (inst_support). An `\institute` whose entries carry their own `$^n$` marks
  // instead (`\institute{$^1$ Ioffe Institute … \\ $^2$ University of Crete …}`, atom-ph9605001, astro-ph9609131) is
  // an affiliation list labelled by them: as one numbered entry it went whole to the author of `\inst{1}`. One
  // institute (no `\and`), which l-aa sets unnumbered (:419-424), is every author's (astro-ph9405007, astro-ph9603094),
  // a mark on it too when the names carry none (astro-ph9609156).
  Let!("\\lx@l@aa@institute", "\\institute");
  // Such papers mark their names alike (`A.Y.\,Potekhin$^{1,2,3}$ \and G.G.\,Pavlov$^{4,1}$`, atom-ph9605001): those
  // marks request the affiliations so labelled, which the `\inst` author list (inst_support) does not read.
  Let!("\\lx@l@aa@author", "\\author");
  DefMacro!("\\author[]{}", sub[(short, names)] {
    let names: Tokens = names;
    if names.unlist_ref().iter().any(|t| {
      t.get_catcode() == Catcode::SUPER || *t == T_CS!("\\textsuperscript") || *t == T_CS!("\\inst")
    }) {
      AssignValue!("lx@l@aa@marked@names" => true, Some(Scope::Global));
    }
    Ok(if names.unlist_ref().iter().any(|t| t.get_catcode() == Catcode::SUPER || *t == T_CS!("\\textsuperscript")) {
      Invocation!(T_CS!("\\lx@add@authors@list"), vec![Some(names)])
    } else {
      Invocation!(T_CS!("\\lx@l@aa@author"), vec![short, Some(names)])
    })
  }, locked => true);
  DefMacro!("\\institute{}", sub[(list)] {
    let list: Tokens = list;
    let several = list.unlist_ref().iter().any(|t| *t == T_CS!("\\and") || *t == T_CS!("\\And"));
    let marked = leads_with_superscript_mark(list.unlist_ref());
    Ok(if marked && lookup_bool("lx@l@aa@marked@names") || marked && several {
      Invocation!(T_CS!("\\lx@add@affiliations"), vec![None, Some(list)])
    } else if !several {
      Invocation!(T_CS!("\\lx@add@affiliation"), vec![Some(TokenizeInternal!("annotate=all")), Some(list)])
    } else {
      Invocation!(T_CS!("\\lx@l@aa@institute"), vec![Some(list)])
    })
  });
});

/// Whether `list` begins with a superscript mark: `$^1$`, `$^{1}$`, `^1` or `\textsuperscript{1}` (an affiliation
/// list labelled by its marks: also ptephy's `\affil`).
pub(crate) fn leads_with_superscript_mark(list: &[Token]) -> bool {
  let mut tokens = list.iter().filter(|t| t.get_catcode() != Catcode::SPACE);
  match tokens.next() {
    Some(t) if t.get_catcode() == Catcode::MATH => tokens
      .next()
      .is_some_and(|t| t.get_catcode() == Catcode::SUPER),
    Some(t) => t.get_catcode() == Catcode::SUPER || *t == T_CS!("\\textsuperscript"),
    None => false,
  }
}

/// The keywords `\keywords` starts, to the paragraph's end: a `\par`, the `\end` of the environment it stands in
/// (`{abstract}`), a heading or `\begin`; with `in_environment` those of a `{keywords}` environment, to its
/// `\end{keywords}`, the paragraphs in it one list. What ends them is put back.
fn keywords_text(in_environment: bool) -> Result<Tokens> {
  let mut body: Vec<Token> = Vec::new();
  while let Some(token) = read_token()? {
    if token.get_catcode() == Catcode::BEGIN {
      body.push(token);
      body.extend(read_balanced(ExpansionLevel::Off, false, false)?.unlist());
      body.push(T_END!());
    } else if !in_environment
      && [
        "\\begin",
        "\\section",
        "\\subsection",
        "\\subsubsection",
        "\\maketitle",
      ]
      .iter()
      .any(|cs| token == T_CS!(*cs))
    {
      unread_vec(vec![token]);
      break;
    } else if token == T_CS!("\\par") {
      if !in_environment {
        unread_vec(vec![token]);
        break;
      }
      body.push(T_SPACE!());
    } else if token == T_CS!("\\end") {
      let env = read_arg(ExpansionLevel::Off)?;
      let name = env.to_string();
      if name == "keywords" || !in_environment {
        let mut end = vec![token, T_BEGIN!()];
        end.extend(env.unlist());
        end.push(T_END!());
        unread_vec(end);
        break;
      }
      body.push(token);
      body.push(T_BEGIN!());
      body.extend(env.unlist());
      body.push(T_END!());
    } else {
      body.push(token);
    }
  }
  while body
    .last()
    .is_some_and(|t| t.get_catcode() == Catcode::SPACE)
  {
    body.pop();
  }
  let lead = body
    .iter()
    .take_while(|t| t.get_catcode() == Catcode::SPACE)
    .count();
  body.drain(..lead);
  Ok(Invocation!(T_CS!("\\lx@l@aa@keywords"), vec![Some(
    Tokens::new(body)
  )]))
}
