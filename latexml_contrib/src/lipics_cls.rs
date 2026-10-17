//! Stub for LIPIcs class (Dagstuhl Leibniz International Proceedings).
use latexml_core::definition::origin::{DefinitionOrigin, OriginGuard};
use latexml_package::prelude::*;

LoadDefinitions!({
  LoadClass!("OmniBus");
  // lipics-v2021.cls:514-1098 loads these packages, in this order. The binding stands in for the class, so they load
  // here or not at all: `\newcolumntype` (array, 2609.13401), `{subfigure}` (subcaption, 2609.10114), the `{CCSXML}`
  // block the class excludes (comment, :861-862; its `_`s were math errors, 2609.13485), `\multirow`, `tabularx`,
  // `threeparttable`, `listings`, soul with the class's `\textsolittle` (:540-541)… Font, PDF and page-layout packages
  // (microtype, lmodern, fontawesome5, hyperxmp, totpages, colorprofiles) are left out, and so are the class's MnSymbol
  // block (:534-538) and its options to amsmath (`tbtags,fleqn`).
  // Not xcolor at class time: the class loads `color` and takes xcolor at the document's start only when the paper
  // has not (:543-546, below), so a paper's own `\usepackage[table]{xcolor}` decides its options (colortbl's
  // `\rowcolor`, array `m{}` columns). (Perl has no lipics binding: OmniBus with `maybeRequireDependencies`,
  // Package.pm:2732, 2776-2794, which regex-scans the shipped class and so loads xcolor at class time.)
  RequirePackage!("fontenc", options => vec![s!("T1")]);
  RequirePackage!("textcomp");
  RequirePackage!("eucal", options => vec![s!("mathscr")]);
  RequirePackage!("amssymb");
  RequirePackage!("soul");
  RawTeX!(
    r"\sodef\textsolittle{}{.12em}{.5em\@plus.08em\@minus.06em}{.4em\@plus.275em\@minus.183em}"
  );
  RequirePackage!("color");
  // :542-553: at the document's start, xcolor if the paper has not loaded it (so a paper's own `[table]` stands) and the
  // class's named colours, which papers use (`lipicsYellow`, 2609.13401).
  RawTeX!(
    r"\AtBeginDocument{\@ifpackageloaded{xcolor}{}{\RequirePackage{xcolor}}\definecolor{darkgray}{rgb}{0.31,0.31,0.33}\definecolor[named]{lipicsGray}{rgb}{0.31,0.31,0.33}\definecolor[named]{lipicsBulletGray}{rgb}{0.60,0.60,0.61}\definecolor[named]{lipicsLineGray}{rgb}{0.51,0.50,0.52}\definecolor[named]{lipicsLightGray}{rgb}{0.85,0.85,0.86}\definecolor[named]{lipicsYellow}{rgb}{0.99,0.78,0.07}}"
  );
  RequirePackage!("babel");
  RequirePackage!("amsmath");
  RequirePackage!("enumerate");
  RequirePackage!("graphicx");
  RequirePackage!("array");
  RequirePackage!("multirow");
  RequirePackage!("tabularx");
  RequirePackage!("threeparttable", options => vec![s!("online")]);
  RequirePackage!("listings");
  RequirePackage!("lineno", options => vec![s!("left"), s!("mathlines")]);
  RequirePackage!("hyperref");
  // :707-709, the caption setup the class gives its figures: caption's binding formats the captions from them
  // ("Figure 1" and a space, the label sans-serif bold, the caption small; caption_sty.rs `caption_format_tokens`).
  RequirePackage!("caption", options => vec![
    s!("labelsep=space"),
    s!("singlelinecheck=false"),
    String::from("font={up,small}"),
    String::from("labelfont={sf,bf}"),
    s!("listof=false"),
  ]);
  RequirePackage!("rotating", options => vec![s!("figuresright")]);
  RequirePackage!("subcaption");
  RequirePackage!("xstring");
  RequirePackage!("comment");
  RawTeX!(r"\excludecomment{CCSXML}");
  RequirePackage!("amsthm");
  // :188-193: the class's options, `\DeclareOption{thm-restate}{\let\usethmrestate\relax}` and the like, read from the
  // class's own option list (`\opt@<class>.cls`, as a wrapper's `\LoadClass[thm-restate]{lipics-v2021}` passes it) or
  // the document's.
  let class_options: Vec<String> = [
    "\\opt@lipics-v2024.cls",
    "\\opt@lipics-v2021.cls",
    "\\opt@lipics-v2019.cls",
    "\\opt@lipics.cls",
    "\\@classoptionslist",
  ]
  .iter()
  .filter_map(|list| {
    lookup_definition(&T_CS!(list))
      .ok()
      .flatten()
      .and_then(|defn| match defn.get_expansion() {
        Some(ExpansionBody::Tokens(body)) => Some(body.to_string()),
        _ => None,
      })
  })
  .collect();
  let class_option = |name: &str| {
    class_options
      .iter()
      .any(|options| options.split(',').any(|option| option.trim() == name))
  };
  // :1015-1016: the `thm-restate` option (`\ifx\usethmrestate\relax\RequirePackage{thm-restate}\fi`) loads
  // thm-restate, providing the `restatable` environment (witness 2211.04601, `\documentclass[...,thm-restate]
  // {lipics-v2021}`). Only then: thm-restate loads thmtools, and a document's keytheorems refuses to load beside it
  // (2604.22493).
  let thm_restate = class_option("thm-restate");
  if thm_restate {
    RequirePackage!("thm-restate");
  }
  // lipics-v2021.cls:1113 `\RequirePackage[capitalise,noabbrev]{cleveref}` (guarded by
  // the `cleveref` class option). Without it `\cref`/`\Cref` come out Error:undefined — Perl (no lipics binding:
  // OmniBus's dependency scan of the shipped class) gets cleveref. Must load AFTER hyperref. Witness 2606.01187.
  RequirePackage!("cleveref", options => vec!["capitalise".to_string(), "noabbrev".to_string()]);

  // lipics-v2019.cls:660, lipics-v2021.cls:766-800 (and the 2018 copy, 1905.06668's: :636), the five-part author:
  // `\author{name}{affiliations}{email}{orcid}{funding}`. Its affiliations are separated by `\and` (`\def\and{\newline}`,
  // :758), the last one may be `\and \url{homepage}` (`\StrBehind{#2}{\and \url}`); the email the class links
  // (`\href{mailto:#3}`) and the ORCID (`\href{#4}{\orcidsymbol}`), both read as written (Semiverbatim: an `_` in them was
  // a math error); the funding goes to the class's Funding paragraph as "name: funding" (:798-803). Under the
  // `anonymous` option (:192) the class prints "Anonymous author" and "Anonymous affiliation" and nothing else of it.
  // The 2016 `lipics.cls` keeps LaTeX's `\author` (17 of the 92 shipped copies). Before, the name alone was the author
  // and the other four arguments were typeset in the body (2607.26691, 2603.11908, 2307.12098, 2608.03845).
  let five_part_author = [
    "\\opt@lipics-v2019.cls",
    "\\opt@lipics-v2021.cls",
    "\\opt@lipics-v2024.cls",
  ]
  .iter()
  .any(|list| has_meaning(&T_CS!(list)));
  if five_part_author {
    AssignValue!("lx_lipics_anonymous" => class_option("anonymous"), Some(Scope::Global));
    DefMacro!(
      "\\author{}{} Semiverbatim Semiverbatim {}",
      sub[(name, affiliations, email, orcid, funding)] {
        lipics_author(name, affiliations, email, orcid, funding)
      }
    );
  }

  // LIPIcs frontmatter — preserve author content as ltx:note
  // frontmatter entries with role markers.
  // lipics-v2021.cls L919: \newcommand{\relatedversiondetails}[3][]{...\textit{#2}:
  // \href{#3}{...}...} — a "Related Version" line (type + URL, with optional
  // linktext=/cite= keyval). Was undefined (Perl defines it). Preserve the core
  // (type + linked URL); the optional keyval is dropped. Witness 2311.17226.
  // …with its keys (`linktext=`, `cite=`, :910-911, :919-935) and its URL read as one (Semiverbatim): an `_` in it was a
  // math error (2603.11908).
  RawTeX!(
    r"\define@key{relatedversiondetails}{linktext}{\def\relatedversiondetails@linktext{#1}}
\define@key{relatedversiondetails}{cite}{\def\relatedversiondetails@cite{#1}}
\def\lx@lipics@relatedversiondetails#1#2#3{\begingroup\let\relatedversiondetails@linktext\@empty
\let\relatedversiondetails@cite\@empty\setkeys{relatedversiondetails}{#1}%
\ifx\relatedversiondetails@linktext\@empty\protected@edef\tmp{\noexpand\textit{#2}:\space{\noexpand\url{#3}}}\else
\protected@edef\tmp{\noexpand\textit{#2}:\space{\noexpand\href{#3}{\noexpand\texttt{\relatedversiondetails@linktext}}}}\fi
\ifx\relatedversiondetails@cite\@empty\else
\protected@edef\tmp{\tmp\noexpand\nobreakspace\noexpand\cite{\relatedversiondetails@cite}}\fi
\expandafter\lx@lipics@note\expandafter{\tmp}{related-version}\endgroup}
\def\lx@lipics@note#1#2{\lx@add@frontmatter{ltx:note}[role=#2]{#1}}"
  );
  DefMacro!(
    "\\relatedversiondetails[]{} Semiverbatim",
    "\\lx@lipics@relatedversiondetails{#1}{#2}{#3}"
  );
  DefMacro!(
    "\\Copyright{}",
    "\\@add@frontmatter{ltx:note}[role=copyright]{#1}"
  );
  def_macro_noop("\\CopyrightDetails")?;
  DefMacro!(
    "\\authorrunning{}",
    "\\@add@frontmatter{ltx:note}[role=runningauthor]{#1}"
  );
  DefMacro!(
    "\\titlerunning{}",
    "\\@add@frontmatter{ltx:note}[role=runningtitle]{#1}"
  );
  DefMacro!(
    "\\funding{}",
    "\\@add@frontmatter{ltx:note}[role=funding]{#1}"
  );
  DefMacro!(
    "\\fundingAgency{}",
    "\\@add@frontmatter{ltx:note}[role=funding-agency]{#1}"
  );
  DefMacro!(
    "\\authorcredit{}",
    "\\@add@frontmatter{ltx:note}[role=authorcredit]{#1}"
  );
  def_macro_noop("\\nolinenumbers")?;
  DefMacro!(
    "\\category{}",
    "\\@add@frontmatter{ltx:note}[role=category]{#1}"
  );
  DefMacro!(
    "\\related{}",
    "\\@add@frontmatter{ltx:note}[role=related]{#1}"
  );
  DefMacro!(
    "\\relatedversion{}",
    "\\@add@frontmatter{ltx:note}[role=relatedversion]{#1}"
  );
  DefMacro!(
    "\\supplement{}",
    "\\@add@frontmatter{ltx:note}[role=supplement]{#1}"
  );
  // :941-985 `\supplementdetails[keys]{type}{url}`: "type (subcategory): url" (or the `linktext=` linked to it), the
  // `cite=` citation, the Software Heritage archive link (`swhid=`, `swhlinktext=` defaulting to the id before its `;`),
  // its URL read as one (Semiverbatim): typeset as text, an `_` in it was a math error (2607.26691, 2307.12098).
  RawTeX!(
    r"\define@key{supplementdetails}{linktext}{\def\supplementdetails@linktext{#1}}
\define@key{supplementdetails}{cite}{\def\supplementdetails@cite{#1}}
\define@key{supplementdetails}{subcategory}{\def\supplementdetails@subcategory{#1}}
\define@key{supplementdetails}{swhlinktext}{\def\supplementdetails@swhlinktext{#1}}
\define@key{supplementdetails}{swhid}{\def\supplementdetails@swhid{#1}}
\define@key{supplementdetails}{swhdelimiter}{\def\supplementdetails@swhdelimiter{#1}}
\def\supplementdetails@swhdelimiter{\\ \hspace*{1.2em}}\let\supplementdetails@swhlinktext\@empty
\def\lx@lipics@swhname#1;#2\@nil{#1}
\def\lx@lipics@supplementdetails#1#2#3{\begingroup\let\supplementdetails@linktext\@empty
\let\supplementdetails@cite\@empty\let\supplementdetails@subcategory\@empty\let\supplementdetails@swhid\@empty
\setkeys{supplementdetails}{#1}%
\ifx\supplementdetails@subcategory\@empty\protected@edef\tmp{\noexpand\textit{#2}}\else
\protected@edef\tmp{\noexpand\textit{#2\noexpand\,\noexpand\,(\supplementdetails@subcategory)}}\fi
\ifx\supplementdetails@linktext\@empty\protected@edef\tmp{\tmp:\space{\noexpand\url{#3}}}\else
\protected@edef\tmp{\tmp:\space{\noexpand\href{#3}{\noexpand\texttt{\supplementdetails@linktext}}}}\fi
\ifx\supplementdetails@cite\@empty\else
\protected@edef\tmp{\tmp\noexpand\nobreakspace\noexpand\cite{\supplementdetails@cite}}\fi
\ifx\supplementdetails@swhid\@empty\else\ifx\supplementdetails@swhlinktext\@empty
\edef\supplementdetails@swhlinktext{\expandafter\lx@lipics@swhname\supplementdetails@swhid;\@nil}\fi
\protected@edef\tmp{\tmp\noexpand\supplementdetails@swhdelimiter{} archived at
\noexpand\href{https://archive.softwareheritage.org/\supplementdetails@swhid}{\noexpand\nolinkurl{\supplementdetails@swhlinktext}}}\fi
\expandafter\lx@lipics@note\expandafter{\tmp}{supplement}\endgroup}"
  );
  DefMacro!(
    "\\supplementdetails[]{} Semiverbatim",
    "\\lx@lipics@supplementdetails{#1}{#2}{#3}"
  );
  // \acknowledgements{text} — render as structural ltx:acknowledgements
  // (post-processors map to canonical role/styling).
  DefConstructor!(
    "\\acknowledgements{}",
    "<ltx:acknowledgements>#1</ltx:acknowledgements>"
  );
  DefMacro!(
    "\\ccsdesc[]{}",
    "\\@add@frontmatter{ltx:classification}[scheme=ccs]{#2}"
  );
  DefMacro!(
    "\\subjclass[]{}",
    "\\@add@frontmatter{ltx:classification}[scheme=AMS]{#2}"
  );
  DefMacro!(
    "\\keywords{}",
    "\\@add@frontmatter{ltx:classification}[scheme=keywords]{#1}"
  );
  DefMacro!("\\event{}", "\\@add@frontmatter{ltx:note}[role=event]{#1}");
  DefMacro!(
    "\\EventEditors{}",
    "\\@add@frontmatter{ltx:note}[role=editors]{#1}"
  );
  DefMacro!(
    "\\EventLongTitle{}",
    "\\@add@frontmatter{ltx:note}[role=event-title]{#1}"
  );
  DefMacro!(
    "\\EventShortTitle{}",
    "\\@add@frontmatter{ltx:note}[role=event-shorttitle]{#1}"
  );
  DefMacro!(
    "\\EventAcronym{}",
    "\\@add@frontmatter{ltx:note}[role=event-acronym]{#1}"
  );
  DefMacro!(
    "\\EventYear{}",
    "\\@add@frontmatter{ltx:note}[role=year]{#1}"
  );
  DefMacro!(
    "\\EventDate{}",
    "\\@add@frontmatter{ltx:note}[role=event-date]{#1}"
  );
  DefMacro!(
    "\\EventLocation{}",
    "\\@add@frontmatter{ltx:note}[role=event-location]{#1}"
  );
  // EventLogo wraps \includegraphics or visual content; preserve.
  DefMacro!(
    "\\EventLogo{}",
    "\\@add@frontmatter{ltx:note}[role=event-logo]{#1}"
  );
  DefMacro!(
    "\\SeriesVolume{}",
    "\\@add@frontmatter{ltx:note}[role=series-volume]{#1}"
  );
  DefMacro!(
    "\\ArticleNo{}",
    "\\@add@frontmatter{ltx:note}[role=articleno]{#1}"
  );
  // LIPIcs L739: \EventNoEds{N} sets editor count.
  def_macro_noop("\\EventNoEds{}")?;
  // LIPIcs L860: \hideLIPIcs sets \@hideLIPIcs to suppress the
  // article-number/page header. No-op in XML. Witness 2502.11299 +6.
  def_macro_noop("\\hideLIPIcs")?;
  // \headers{left}{right} — LIPIcs running-header alias used by
  // some templates. Round-34 surpass-Perl: preserve as ltx:note so
  // the author-typed text isn't dropped.
  DefMacro!(
    "\\headers{}{}",
    "\\@add@frontmatter{ltx:note}[role=runningheads]{#1 / #2}"
  );

  // lipics-v2021.cls:1119-1256: theorem-like environments (2609.11285's copy; the 2019+ versions add `{exercise}`,
  // `{note}` and the starred `{note*}`, `{remark*}`, `{claim*}`; run 340: `{remark*}` 38 first errors, 2508.09892).
  // `{theorem}` is numbered within sections under the `numberwithinsect` option (:1126-1128). Its siblings share its
  // counter (`[theorem]`), except under `autoref` (which loads aliascnt, :1119-1121) without `thm-restate`
  // (:1172-1236): there each sibling's counter is an aliascnt alias of `theorem`, with its `\<name>autorefname`, numbered
  // within sections under `numberwithinsect`, so a document can number by it (keytheorems' `sibling=corollary`,
  // 2604.22493: `No counter 'corollary'` six times).
  let autoref = class_option("autoref");
  let numberwithinsect = class_option("numberwithinsect");
  let alias_counters = autoref && !thm_restate;
  if autoref {
    RequirePackage!("aliascnt");
  }
  let mut theorems = String::from("\\newtheorem{theorem}{Theorem}");
  if numberwithinsect {
    theorems.push_str("\\numberwithin{theorem}{section}");
  }
  for (name, title) in [
    ("lemma", "Lemma"),
    ("corollary", "Corollary"),
    ("proposition", "Proposition"),
    ("definition", "Definition"),
    ("observation", "Observation"),
    ("remark", "Remark"),
    ("example", "Example"),
    ("claim", "Claim"),
    ("conjecture", "Conjecture"),
    ("exercise", "Exercise"),
    ("note", "Note"),
  ] {
    if name == "note" {
      theorems.push_str("\\theoremstyle{remark}");
    }
    if alias_counters {
      theorems.push_str(&format!(
        "\\newaliascnt{{{name}}}{{theorem}}\\newtheorem{{{name}}}[{name}]{{{title}}}\\aliascntresetthe{{{name}}}\
         \\newcommand{{\\{name}autorefname}}{{{title}}}"
      ));
      if numberwithinsect {
        theorems.push_str(&format!("\\numberwithin{{{name}}}{{section}}"));
      }
    } else {
      theorems.push_str(&format!("\\newtheorem{{{name}}}[theorem]{{{title}}}"));
    }
  }
  theorems.push_str(
    r"\newtheorem*{note*}{Note}\newtheorem*{remark*}{Remark}\newtheorem*{claim*}{Claim}\theoremstyle{plain}",
  );
  RawTeX!(&theorems);
  // :1041-1056 `{claimproof}`, a proof of a claim, and `\claimqedhere`: the class renews `\qedsymbol` to a grey ◁
  // for its own `\popQED` only; our `\qed`, as Perl's, prints ∎ whatever `\qedsymbol` is, so the claim's end (and
  // `\claimqedhere`) set `\qed` to the ◁ just there, a proof nested inside keeping ∎. :1057-1060 `\lipicsEnd` (the ⌟
  // ending an example or remark); :380-383 `\proofsubparagraph` (a run-in subparagraph heading in proofs); :572-577 the
  // `(i)`/`(a)`/`(1)` enumerates. Run 340 first errors: `{claimproof}` 348, `\lipicsEnd` 51, `\proofsubparagraph` 60,
  // `{romanenumerate}` 34, `{alphaenumerate}` 32 (witnesses 2508.16319, 2008.01422, 2103.15659, 1904.11668,
  // 2103.14599).
  RawTeX!(
    r"\def\lx@lipics@claimqed{\def\qed{\textcolor{lipicsGray}{\ensuremath{\vartriangleleft}}}}
\newenvironment{claimproof}[1][\proofname]{\begin{proof}[#1]}{\lx@lipics@claimqed\end{proof}}
\newcommand{\claimqedhere}{\begingroup\lx@lipics@claimqed\qedhere\endgroup}
\DeclareRobustCommand{\lipicsEnd}{\leavevmode\unskip\penalty9999 \hbox{}\nobreak\hfill\quad\hbox{$\lrcorner$}}
\newcommand{\proofsubparagraph}{\@startsection{subparagraph}{5}{\z@}%
  {3.25ex \@plus1ex \@minus .2ex}{-1em}{\sffamily\normalsize\bfseries}}
\def\romanenumerate{\enumerate[(i)]}\let\endromanenumerate\endenumerate
\def\alphaenumerate{\enumerate[(a)]}\let\endalphaenumerate\endenumerate
\def\bracketenumerate{\enumerate[(1)]}\let\endbracketenumerate\endenumerate"
  );
  // :1001-1005 `\flag[width]{image}`, an icon (an artifact badge) set in the margin: the image, inline.
  RawTeX!(r"\newcommand\flag[2][0.9cm]{\includegraphics[width=#1]{#2}}");
  // The front-matter setters of the 2016-2019 versions only (lipics.cls, lipics-v2019.cls; 17-19 of 92 sampled
  // copies), guesses a paper's own definition replaces: `\EventShortName`, `\serieslogo`, `\DOIPrefix` store what the
  // page furniture prints (run 340: `\serieslogo` 123, 2103.15651); `\DOI` the DOI the title block prints after the
  // abstract when set (1507.03682's copy :391-393, `\doiHeading\doi{\@DOI}`; the class itself sets it empty, :643),
  // once, as a pubnote at `\maketitle`; `\volumeinfo{editors}{n}{event}
  // {volume}{issue}{first page}` the event, the note `\event` makes.
  {
    let _guess = OriginGuard::new(DefinitionOrigin::Fallback);
    RawTeX!(
      r"\def\EventShortName#1{\gdef\@EventShortName{#1}}
\def\serieslogo#1{\gdef\@serieslogo{#1}}
\def\DOI#1{\gdef\@DOI{#1}}
\def\DOIPrefix#1{\gdef\@DOIPrefix{#1}}"
    );
    DefMacro!(
      "\\volumeinfo{}{}{}{}{}{}",
      "\\if\\relax\\detokenize{#3}\\relax\\else\\@add@frontmatter{ltx:note}[role=event]{#3}\\fi"
    );
  }
  RawTeX!(
    r"\def\lx@lipics@doi{\@ifundefined{@DOI}{}{\ifx\@DOI\@empty\else\lx@add@pubnote[role=doi]{\@DOI}\fi}}"
  );
  AddToMacro!("\\lx@maketitle@body", "\\lx@lipics@doi");
});

/// Whether `tokens` holds nothing but spaces (the class's `\if!#n!` test).
fn is_blank(tokens: &Tokens) -> bool {
  tokens
    .unlist_ref()
    .iter()
    .all(|t| t.get_catcode() == Catcode::SPACE)
}

/// `tokens` without its leading and trailing spaces.
fn trimmed(tokens: &[Token]) -> Vec<Token> {
  let start = tokens
    .iter()
    .position(|t| t.get_catcode() != Catcode::SPACE)
    .unwrap_or(tokens.len());
  let end = tokens
    .iter()
    .rposition(|t| t.get_catcode() != Catcode::SPACE)
    .map_or(start, |i| i + 1);
  tokens[start..end].to_vec()
}

/// The parts of `tokens` between its top-level `\and`s.
fn split_on_and(tokens: &Tokens) -> Vec<Vec<Token>> {
  let and = T_CS!("\\and");
  let mut parts = vec![Vec::new()];
  let mut depth = 0i32;
  for token in tokens.unlist_ref() {
    match token.get_catcode() {
      Catcode::BEGIN => depth += 1,
      Catcode::END => depth -= 1,
      _ => {},
    }
    if depth == 0 && *token == and {
      parts.push(Vec::new());
    } else if let Some(part) = parts.last_mut() {
      part.push(*token);
    }
  }
  parts
}

/// An ORCID iD (`0000-0002-1825-0097`, the last character a digit or `X`) or its orcid.org address, with or without
/// the scheme or `www.`; an argument holding a macro is taken as one (`\lx@add@orcid` keeps it as written).
fn is_orcid(tokens: &[Token]) -> bool {
  if tokens.iter().any(|t| t.get_catcode() == Catcode::CS) {
    return true;
  }
  let text = Tokens::new(tokens.to_vec())
    .to_string()
    .trim()
    .to_ascii_lowercase();
  let mut rest = text.as_str();
  for scheme in ["https://", "http://"] {
    if let Some(r) = rest.strip_prefix(scheme) {
      rest = r;
      break;
    }
  }
  rest = rest.strip_prefix("www.").unwrap_or(rest);
  if rest.starts_with("orcid.org/") {
    return true;
  }
  let groups: Vec<&str> = rest.split('-').collect();
  groups.len() == 4
    && groups.iter().enumerate().all(|(i, group)| {
      group.len() == 4
        && group
          .chars()
          .enumerate()
          .all(|(j, c)| c.is_ascii_digit() || (i == 3 && j == 3 && c == 'x'))
    })
}

/// The address an ORCID slot holds when it is an email address (`name@host`, or a `mailto:` link to one): one `@`
/// with text before it, a dotted host after it, no white space and no URL scheme.
fn email_address(tokens: &[Token]) -> Option<String> {
  if tokens.iter().any(|t| t.get_catcode() == Catcode::CS) {
    return None;
  }
  let text = Tokens::new(tokens.to_vec()).to_string();
  let text = text.trim();
  let address = text.strip_prefix("mailto:").unwrap_or(text);
  let (local, host) = address.split_once('@')?;
  (!local.is_empty()
    && host.contains('.')
    && !host.contains('@')
    && !address.contains(char::is_whitespace)
    && !local.contains("://"))
  .then(|| address.to_string())
}

/// `\cs{content}` as tokens.
fn invoke(cs: &str, content: Vec<Token>) -> Vec<Token> {
  let mut tokens = vec![T_CS!(cs), T_BEGIN!()];
  tokens.extend(content);
  tokens.push(T_END!());
  tokens
}

/// The five-part `\author` of lipics-v2019/v2021 (`\author` in `load_definitions`): the creator, its affiliations and
/// homepage, email, ORCID and funding note, through the frontmatter API.
fn lipics_author(
  name: Tokens,
  affiliations: Tokens,
  email: Tokens,
  orcid: Tokens,
  funding: Tokens,
) -> Result<Tokens> {
  let anonymous = lookup_bool("lx_lipics_anonymous");
  let mut out = Vec::new();
  if anonymous {
    out.extend(invoke("\\lx@add@creator", Explode!("Anonymous author")));
    if !is_blank(&affiliations) {
      out.extend(invoke(
        "\\lx@add@affiliation",
        Explode!("Anonymous affiliation"),
      ));
    }
    return Ok(Tokens::new(out));
  }
  out.extend(invoke("\\lx@add@creator", name.unlist_ref().to_vec()));
  for part in split_on_and(&affiliations) {
    let part = trimmed(&part);
    if part.is_empty() {
      continue;
    }
    // `\and \url{homepage}`: the class's homepage, linked from the author's name (:796)
    if part[0] == T_CS!("\\url") {
      let rest = trimmed(&part[1..]);
      let url = match (rest.first(), rest.last()) {
        (Some(first), Some(last))
          if first.get_catcode() == Catcode::BEGIN
            && last.get_catcode() == Catcode::END
            && rest.len() >= 2 =>
        {
          rest[1..rest.len() - 1].to_vec()
        },
        _ => rest,
      };
      out.extend(invoke("\\lx@add@url", url));
    } else {
      out.extend(invoke("\\lx@add@affiliation", part));
    }
  }
  let email = (!is_blank(&email)).then(|| trimmed(email.unlist_ref()));
  if let Some(email) = &email {
    out.extend(invoke("\\lx@add@email", email.clone()));
  }
  // the class links the slot as written (lipics-v2021.cls:797 `\if!#4!\else{\,\href{#4}{\orcidsymbol}}\fi`); what the
  // author put there is read by its type: an ORCID iD, bare or its orcid.org address, is the author's ORCID
  // (`\lx@add@orcid`); an email address (all five authors of 2607.06446, `{}{gerhard@uni-bremen.de}`) is the author's
  // email (`\lx@add@email`, once when the email slot holds it too), which as a link would be a relative one to nowhere;
  // any other address is the class's link to it
  if !is_blank(&orcid) {
    let orcid = trimmed(orcid.unlist_ref());
    if is_orcid(&orcid) {
      out.extend(invoke("\\lx@add@orcid", orcid));
    } else if let Some(address) = email_address(&orcid) {
      let same = email
        .as_ref()
        .is_some_and(|email| Tokens::new(email.clone()).to_string().trim() == address);
      if !same {
        // as written, or the address a `mailto:` link names
        let written = Tokens::new(orcid.clone()).to_string().trim() == address;
        let address = if written {
          orcid
        } else {
          mouth::tokenize_internal(TeXString::assembled(address)).unlist()
        };
        out.extend(invoke("\\lx@add@email", address));
      }
    } else {
      let mut link = vec![T_CS!("\\href"), T_BEGIN!()];
      link.extend(orcid);
      link.extend([T_END!(), T_BEGIN!(), T_CS!("\\lx@orcidlogo"), T_END!()]);
      out.extend(
        mouth::tokenize_internal(TeXString::assembled(
          r"\lx@annotate@frontmatter{ltx:creator}{ltx:contact}[role=orcid]".to_string(),
        ))
        .unlist(),
      );
      out.extend(invoke_group(link));
    }
  }
  if !is_blank(&funding) {
    let mut note = vec![
      T_CS!("\\textit"),
      T_BEGIN!(),
      T_BEGIN!(),
      T_CS!("\\let"),
      T_CS!("\\footnote"),
      T_CS!("\\@gobble"),
    ];
    note.extend(name.unlist_ref().iter().copied());
    note.extend([T_END!(), T_END!(), T_OTHER!(":"), T_CS!("\\space")]);
    note.extend(funding.unlist_ref().iter().copied());
    let mut tokens = vec![T_CS!("\\lx@add@frontmatter"), T_BEGIN!()];
    tokens.extend(Explode!("ltx:note"));
    tokens.push(T_END!());
    tokens.push(T_OTHER!("["));
    tokens.extend(Explode!("role=funding"));
    tokens.push(T_OTHER!("]"));
    tokens.extend(invoke_group(note));
    out.extend(tokens);
  }
  Ok(Tokens::new(out))
}

/// `{content}` as tokens.
fn invoke_group(content: Vec<Token>) -> Vec<Token> {
  let mut tokens = vec![T_BEGIN!()];
  tokens.extend(content);
  tokens.push(T_END!());
  tokens
}
