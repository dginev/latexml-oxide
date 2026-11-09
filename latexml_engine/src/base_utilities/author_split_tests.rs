use latexml_core::state::{State, StateOptions, set_state};

// (the field readers' units, moved from `author_block/` to `defs/frontmatter_fields/` by Z18a)
use super::defs::frontmatter_fields::{definitions_at_author, math_braces_as_printed};
use super::*;

fn setup() {
  // T_CS!/T_BEGIN! etc. and tokenize_internal need a thread-local State for
  // string interning.
  set_state(State::new(StateOptions::default()));
}
fn tk(s: &'static str) -> Tokens { mouth::tokenize_internal(s) }

/// The replayed statements of a title code `body`, as strings (`definitions_at_author`).
fn replayed(body: &'static str) -> Vec<String> {
  definitions_at_author(tk(body).unlist_ref())
    .expect("reads \\@author")
    .into_iter()
    .map(|(_, statement)| Tokens::new(statement).to_string())
    .collect()
}

#[test]
fn title_code_definitions_open_at_author() {
  setup();
  // a spaced `\newcommand [1] {…}` is read whole; `\let`; the last of two `\def`s comes last; a group closed before
  // `\@author` is not replayed
  assert_eq!(
    replayed(
      r"{\let\addr\itshape\newcommand\note [1] {[#1]}\def\dup{a}{\def\dup{b}\begingroup\def\gone{G}\endgroup\@author}}"
    ),
    [
      r"\let\addr\itshape",
      r"\newcommand\note[1] {[#1]}",
      r"\def\dup{a}",
      r"\def\dup{b}"
    ]
  );
}

#[test]
fn title_code_definitions_global_local_and_skipped() {
  setup();
  // a `\gdef` (and `\global\def`) made in a closed group is kept, replayed as a local `\def`; `\edef`,
  // `\renewcommand` and a definition closing a group it did not open (2601.13359) are not replayed
  assert_eq!(
    replayed(
      r"{\gdef\g{1}\global\def\h{2}}{\edef\e{3}\renewcommand\r{4}\def\affiliations{\egroup\par\bgroup}\let\x\egroup\def\sep{\end{tabular}\begin{tabular}{c}}\def\ok{\begin{center}c\end{center}}\@author}"
    ),
    [
      r"\def\g{1}",
      r"\def\h{2}",
      r"\def\ok{\begin{center}c\end{center}}"
    ]
  );
  assert!(definitions_at_author(tk(r"{\def\name{N}}").unlist_ref()).is_none());
}

#[test]
fn whole_line_cs_wrapper_detects_font_wrapper() {
  setup();
  let (cmd, inner) = whole_line_cs_wrapper(&tk("\\textbf{A, B}")).expect("is a wrapper");
  assert_eq!(cmd, T_CS!("\\textbf"));
  assert!(!inner.is_empty());
}

#[test]
fn whole_line_cs_wrapper_rejects_non_wrappers() {
  setup();
  // No leading control sequence.
  assert!(whole_line_cs_wrapper(&tk("A, B")).is_none());
  // Content trails the group -> not a WHOLE-line wrapper.
  assert!(whole_line_cs_wrapper(&tk("\\textbf{A}x")).is_none());
}

#[test]
fn split_author_line_splits_top_level_commas() {
  setup();
  assert_eq!(split_author_line(tk("Alice, Bob, Carol")).len(), 3);
}

#[test]
fn split_author_line_splits_literal_and() {
  setup();
  assert_eq!(split_author_line(tk("Alice and Bob")).len(), 2);
}

#[test]
fn split_author_line_unwraps_font_wrapper_per_author() {
  setup();
  // Beyond-Perl: brace-hidden commas would collapse to ONE creator in Perl;
  // we unwrap and re-apply the wrapper so each name is its own creator.
  let got = split_author_line(tk("\\textbf{Alice, Bob, Carol}"));
  assert_eq!(got.len(), 3);
  for piece in &got {
    let v = piece.unlist_ref();
    assert_eq!(v[0], T_CS!("\\textbf"));
    assert_eq!(v[1].get_catcode(), Catcode::BEGIN);
  }
}

#[test]
fn split_author_line_leaves_single_wrapped_name_intact() {
  setup();
  // Only unwrap when there is actually an inner list to split.
  assert_eq!(split_author_line(tk("\\textbf{Alice}")).len(), 1);
}

// --- html_feedback#6637 helpers (OXIDIZED_DESIGN #52) ---

#[test]
fn normalize_hspace_becomes_quad_separator() {
  setup();
  let out = normalize_hspace_separators(tk("A \\hspace{1cm} B \\hspace*{2em} C \\hfill D"));
  // Every horizontal-space macro became a \quad separator…
  assert_eq!(position_of(&out, &[T_CS!("\\quad")]), Some(3));
  assert!(position_of(&out, &[T_CS!("\\hspace")]).is_none());
  assert!(position_of(&out, &[T_CS!("\\hfill")]).is_none());
  // …and the length arguments did not leak as text.
  let s = out.to_string();
  assert!(
    !s.contains("1cm") && !s.contains("2em"),
    "length leaked: {s}"
  );
}

#[test]
fn footnote_symbol_operand_classification() {
  setup();
  // Note-marks (symbols) are recognised…
  for sym in ["*", "**", "\\dagger", "\\ddagger", "\\ast", "\\star", "\\S"] {
    assert!(
      is_footnote_symbol_operand(&tk(sym).unlist()),
      "{sym} should be a footnote symbol"
    );
  }
  // …affiliation marks (numeric/lettered) and non-symbol CS are NOT.
  for aff in ["1", "12", "a", "\\text", "\\inst"] {
    assert!(
      !is_footnote_symbol_operand(&tk(aff).unlist()),
      "{aff} must NOT be a footnote symbol"
    );
  }
  // An empty operand is not a mark.
  assert!(!is_footnote_symbol_operand(&[]));
}

#[test]
fn rewrite_symbol_superscripts_recovers_marks() {
  setup();
  // $^{*}$, ${}^{\dagger}$, \textsuperscript{*}, $^{**}$ → keepsup sentinel, and
  // the superscript token is gone (no longer an affiliation-marker trigger).
  for marked in [
    "X$^{*}$",
    "X${}^{\\dagger}$",
    "X\\textsuperscript{*}",
    "X$^{**}$",
  ] {
    let out = rewrite_symbol_superscripts(tk(marked));
    assert!(
      position_of(&out, &[T_CS!("\\lx@frontmatter@keepsup")]).is_some(),
      "{marked} did not rewrite to keepsup: {out}"
    );
    assert!(
      position_of(&out, &[T_SUPER!()]).is_none(),
      "{marked} left a bare superscript trigger: {out}"
    );
  }
}

#[test]
fn rewrite_symbol_superscripts_leaves_affiliation_and_math() {
  setup();
  // Numeric affiliation marks stay as superscript markers (untouched)…
  let aff = rewrite_symbol_superscripts(tk("X$^{1}$ \\and Y$^{1}$Institute"));
  assert!(
    position_of(&aff, &[T_SUPER!()]).is_some(),
    "numeric affiliation mark must be preserved: {aff}"
  );
  assert!(position_of(&aff, &[T_CS!("\\lx@frontmatter@keepsup")]).is_none());
  // …and real math (non-empty base) is not rewritten — including a SYMBOL
  // superscript on a base (`$a^{*}$`), which is genuine math, not a note-mark.
  for expr in ["$x^2$", "$a^{*}$"] {
    let math = rewrite_symbol_superscripts(tk(expr));
    assert!(
      position_of(&math, &[T_SUPER!()]).is_some(),
      "real math superscript lost in {expr}: {math}"
    );
    assert!(position_of(&math, &[T_CS!("\\lx@frontmatter@keepsup")]).is_none());
  }
}

#[test]
fn is_link_line_bare_trailing_href() {
  setup();
  // a bare `\href` ending the line, or one whose second argument is missing, reads no argument past the line (was a
  // slice past its end, 77p r4 re-review) and prints nothing, so is no link line (77p r9: 2601.02680's `\url{}`); a
  // whole link is a link line, a line of spacing alone none
  for (line, link) in [
    ("\\href", false),
    ("\\href ", false),
    ("\\href{https://x.org}", false),
    ("\\href{https://x.org}{x.org} \\url{}", true),
    ("\\small \\href{https://x.org}{x.org}", true),
    ("\\url{https://x.org} text", false),
    ("\\vspace{2mm}", false),
    // (in groups and sizes, after a label ending in a colon: 2602.22394, 2602.23802, 2602.09509, 77p r6)
    ("{\\footnotesize \\url{https://x.org}}", true),
    ("{\\tt{\\small{\\url{https://x.org}}}}", true),
    ("Demo Code:~\\url{https://x.org}", true),
    ("Demo Code \\url{https://x.org}", false),
    ("\\url{https://x.org} \\url{https://y.org}", false),
  ] {
    assert_eq!(is_link_line(&tk(line)), link, "{line}");
  }
}

#[test]
fn dotted_initials_are_no_web_address() {
  // (77p r11, the r7 re-review's finding 2: 2601.06085, 2606.03745)
  for (text, web) in [
    ("Edward A.G. Schuur", false),
    ("T.J.C. Bezerra", false),
    ("J.Smith", false),
    ("version 1.2", false),
    ("x.org", true),
    ("ai.meta.com", true),
    ("github.io/traj_evo_search", true),
    ("www.x", true),
    ("https://a.b", true),
    ("aable@x.edu", false),
  ] {
    assert_eq!(reads_as_web_address(text), web, "{text}");
  }
}

#[test]
fn is_url_line_not_a_mailbox() {
  setup();
  // a link line to a web page is a URL contact, one to a mailbox not, an e-mail line ([`mailto_line`]; 2601.00240's
  // `\textbf{Correspondence:} \href{mailto:…}{…}`, 77p r7, r8)
  for (line, url) in [
    ("Code:~\\url{https://x.org}", true),
    ("\\href{https://x.org}{x.org}", true),
    (
      "\\textbf{Correspondence:} \\href{mailto:a@x.org}{a@x.org}",
      false,
    ),
    ("Univ A", false),
    // (77p r9: a phrase, a colour around the link, an empty `\url{}` beside it; a linked institution or name is
    // none, nor a link to a bare address, 2606.26479, 2604.04847)
    ("\\href{https://x.org}{Project page}", true),
    ("{\\color{orange}\\textbf{\\url{https://x.org}}}", true),
    ("\\href{https://x.org}{x.org} \\url{}", true),
    (
      "\\href{https://launchsafe.com}{\\textbf{LaunchSafe Research}}",
      false,
    ),
    ("\\href{https://www.mit.edu}{MIT}", false),
    ("\\href{https://x.org}{Davide Grossi}", false),
    ("\\textbf{Correspondence:} \\href{b@x.org}{b@x.org}", false),
    // (77p r10: the link's wording and target decide, not the shape of its words — a link word is a page
    // (2602.22227, 2601.18810), a bare domain an institution (the r6 re-review's x10, w8), a handle's profile a page
    // (w21), a person's name that name (2603.22058), an icon a link)
    ("\\href{https://x.org/p}{Project Page}", true),
    (
      "\\href{https://x.substack.com/p/a}{\\textcolor{blue}{Substack Article}}",
      true,
    ),
    ("\\href{https://github.com/x/y}{GitHub Repository}", true),
    ("\\href{https://huggingface.co/x}{Hugging Face Demo}", true),
    ("\\href{https://www.google.com}{Google}", false),
    ("\\href{https://x.ai/}{xAI}", false),
    ("Contact: \\href{https://x.com/aable}{@aable}", true),
    (
      "\\textit{Author:} \\href{https://x.org/ms}{Masashi Sekine}",
      false,
    ),
    (
      "\\href{https://github.com/x}{\\includegraphics{gh.png}}",
      true,
    ),
  ] {
    assert_eq!(is_url_line(&tk(line)), url, "{line}");
  }
}

#[test]
fn bare_address_needs_a_local_part_and_a_dotted_domain() {
  // (77p r10, the r6 re-review's finding 6: a handle, a list, a web page are no one address)
  for (text, bare) in [
    ("binzhu@smu.edu.sg", true),
    ("@aable", false),
    ("aable@localhost", false),
    ("{a,b}@x.edu", false),
    ("x.com/aable@b.org", false),
  ] {
    assert_eq!(is_bare_address(text), bare, "{text}");
  }
  assert!(!links_a_mailbox("https://x.com/aable", "@aable"));
  assert!(links_a_mailbox("mailto:a@x.org", "a@x.org"));
}

#[test]
fn non_printing_arguments_are_no_letters() {
  setup();
  // every command of the one table ([`NON_PRINTING_ARGUMENTS`]) drops its non-printing arguments and keeps what it
  // prints (77p r10, the r6 re-review's finding 5: 2609.25669's `\fontfamily{pcr}`, w14's `\scalebox{0.9}`)
  for (line, printed) in [
    ("\\color[rgb]{0,0,1}aable", "aable"),
    ("\\textcolor{blue}{aable}", "aable"),
    ("\\colorbox{red}{aable}", "aable"),
    ("\\fcolorbox[rgb]{1,0,0}[gray]{0.5}{aable}", "aable"),
    ("\\pagecolor{white}aable", "aable"),
    ("\\colorlet{mine}[rgb]{red}aable", "aable"),
    ("\\definecolor{mine}{rgb}{1,0,0}aable", "aable"),
    ("\\fontfamily{pcr}\\selectfont aable", "aable"),
    ("\\fontseries{b}aable", "aable"),
    ("\\fontshape{it}aable", "aable"),
    ("\\fontencoding{T1}aable", "aable"),
    ("\\fontsize{9}{11}aable", "aable"),
    ("\\usefont{T1}{pcr}{m}{n}aable", "aable"),
    ("\\scalebox{0.9}[0.8]{aable}", "aable"),
    ("\\rotatebox[origin=c]{90}{aable}", "aable"),
    ("\\raisebox{-1pt}[0pt][0pt]{aable}", "aable"),
    ("\\resizebox*{2cm}{!}{aable}", "aable"),
    ("\\includegraphics*[scale=0.1]{icon.pdf}aable", "aable"),
    ("x\\hspace*{2pt}aable", "x aable"),
    ("x\\vspace{1ex}aable", "x aable"),
    // (77p r11, the r7 re-review's finding 11: probes p1, p2, p4, p5, p6)
    ("\\rule{0pt}{2ex}aable", "aable"),
    ("\\rule[-1pt]{1em}{1em}aable", "aable"),
    ("x\\phantom{xx}aable", "x aable"),
    ("x\\hphantom{xx}aable", "x aable"),
    ("\\vphantom{Xy}aable", "aable"),
    ("x\\kern2em aable", "x aable"),
    ("x\\kern-1.5pt aable", "x aable"),
    ("x\\hskip 2em plus 1fil aable", "x aable"),
    ("x\\hskip\\fill aable", "x aable"),
    ("x\\vskip 1ex minus 2pt aable", "xaable"),
    ("x\\kern2emaable", "x aable"),
  ] {
    let text: String = without_non_printing_arguments(tk(line).unlist_ref())
      .iter()
      .filter(|t| {
        matches!(
          t.get_catcode(),
          Catcode::LETTER | Catcode::OTHER | Catcode::SPACE
        )
      })
      .map(|t| t.to_string())
      .collect();
    assert_eq!(text, printed, "{line}");
  }
  assert_eq!(
    email_addresses(&tk(
      "\\texttt{\\fontfamily{pcr}\\selectfont xw109@illinois.edu, tunder@illinois.edu}"
    )),
    Some(vec![s!("xw109@illinois.edu"), s!("tunder@illinois.edu")])
  );
  // (a `\url`'s braces print: a brace list of local parts, 2609.00588; a `;` parts them, 2607.16926)
  assert_eq!(
    email_addresses(&tk("\\texttt{\\url{{co_gy,lyu}@x.ac.jp}}")),
    Some(vec![s!("co_gy@x.ac.jp"), s!("lyu@x.ac.jp")])
  );
  assert_eq!(
    email_addresses(&tk("{\\tt\\small \\{hjjang; dychoi\\}@vclab.kaist.ac.kr}")),
    Some(vec![
      s!("hjjang@vclab.kaist.ac.kr"),
      s!("dychoi@vclab.kaist.ac.kr")
    ])
  );
}

#[test]
fn empty_marks_and_compound_symbol_labels() {
  setup();
  // an empty mark is no mark (user ruling R11 2026-10-20, 2604.19440)
  assert_eq!(
    drop_empty_marks(tk("Ann Able \\\\ \\textsuperscript{}Univ A $^{ }$B")).to_string(),
    tk("Ann Able \\\\ Univ A B").to_string()
  );
  assert_eq!(
    drop_empty_marks(tk("Ann$^{1}$")).to_string(),
    tk("Ann$^{1}$").to_string()
  );
  // (a note's text is the note's, kept as written — `\thanks`, its optional argument too, `\footnote`: the XML's note
  // holds what the observer's rehearsal does, OD #916)
  assert_eq!(
    drop_empty_marks(tk(
      "Ann\\thanks{\\textsuperscript{}} \\textsuperscript{}Univ A\\thanks[2]{$^{}$x}\\footnote{\\textsuperscript{}}"
    ))
    .to_string(),
    tk("Ann\\thanks{\\textsuperscript{}} Univ A\\thanks[2]{$^{}$x}\\footnote{\\textsuperscript{}}").to_string()
  );
  // (in a block other marks make a marked one it stays: 2604.22129)
  let marked = "Ann$^{1}$~~~Bob\\textsuperscript{}";
  assert_eq!(
    drop_empty_marks(tk(marked)).to_string(),
    tk(marked).to_string()
  );
  // a compound symbol mark is its run whole and a label per symbol, a doubled star one (the r6 re-review's x1, w10,
  // w11; 2602.19190, 2606.15609)
  assert_eq!(clean_frontmatter_labels("\\dagger*", ""), [
    "\\dagger*",
    "\\dagger",
    "*"
  ]);
  assert_eq!(clean_frontmatter_labels("\\dagger *", ""), [
    "\\dagger_*",
    "\\dagger",
    "*"
  ]);
  assert_eq!(clean_frontmatter_labels("**", ""), ["**"]);
}

#[test]
fn mailto_line_is_a_labelled_email_line() {
  setup();
  // a link line to a mailbox is an e-mail line under the text before it, whatever it says, and its address the one
  // the link prints, its target's when it prints none (2601.00240, 2601.07974's placeholder target, 77p r8)
  for (line, label, address) in [
    (
      "\\small{ \\textbf{Correspondence:} \\href{mailto:email@domain}{a.b@x.org} }",
      Some("Correspondence:"),
      "a.b@x.org",
    ),
    ("\\href{mailto:a@x.org}{Write to us}", None, "a@x.org"),
    (
      "Contact:~\\href{mailto:a@x.org}{a@x.org}",
      Some("Contact:"),
      "a@x.org",
    ),
    // (77p r9: a link to a bare address without `mailto:`, the ACL template's, 2604.04847; a footnote symbol before
    // the label)
    (
      "\\textbf{Correspondence:} \\href{b@x.org}{b@x.org}",
      Some("Correspondence:"),
      "b@x.org",
    ),
    (
      "\\lx@frontmatter@keepsup{*}\\lx@frontmatter@symbolmark{*}Corresponding author: \\href{mailto:a@x.org}{a@x.org}",
      Some("Corresponding author:"),
      "a@x.org",
    ),
  ] {
    let (got_label, addresses) = labelled_emails(&tk(line)).unwrap().expect(line);
    assert_eq!(
      got_label.map(|label| label.to_string()).as_deref(),
      label,
      "{line}"
    );
    let texts: Vec<String> = addresses.iter().map(|a| address_text(&a.address)).collect();
    assert_eq!(texts, [address], "{line}");
  }
  for line in [
    "\\href",
    "Mail: \\href",
    "\\textbf{Code:} \\href{https://x.org}{x.org}",
    "Write \\href{mailto:a@x.org}{a@x.org}",
  ] {
    assert!(labelled_emails(&tk(line)).unwrap().is_none(), "{line}");
  }
}

#[test]
fn email_addresses_skip_colour_arguments() {
  setup();
  // a colour command's colour is no letter of an address (2604.19440's `\href{mailto:…}{\color{black}\texttt{…}}`,
  // the r5 re-review's c1-c6; 77p r9); what it prints stays
  for line in [
    "\\texttt{\\{\\color{black}aable, bbaker\\}@x.edu}",
    "\\textcolor{blue}{\\texttt{aable@x.edu, bbaker@x.edu}}",
    "\\textcolor[rgb]{0,0,1}{aable@x.edu, bbaker@x.edu}",
    "\\fcolorbox{red}{white}{aable@x.edu, bbaker@x.edu}",
    "\\texttt{\\{}\\href{mailto:aable@x.edu}{\\color{black}\\texttt{aable},} \\href{mailto:bbaker@x.edu}{\\color{black}\\texttt{bbaker}}\\texttt{\\}@x.edu}",
  ] {
    assert_eq!(
      email_addresses(&tk(line)),
      Some(vec![s!("aable@x.edu"), s!("bbaker@x.edu")]),
      "{line}"
    );
  }
  // (several linked addresses behind a label, the r5 re-review's u22)
  let (label, addresses) = labelled_emails(&tk(
    "Emails: \\href{mailto:a@x.org}{a@x.org}, \\href{mailto:b@y.org}{b@y.org}",
  ))
  .unwrap()
  .expect("a labelled list");
  assert_eq!(
    label.map(|label| label.to_string()).as_deref(),
    Some("Emails:")
  );
  let texts: Vec<String> = addresses.iter().map(|a| address_text(&a.address)).collect();
  assert_eq!(texts, ["a@x.org", "b@y.org"]);
}

#[test]
fn email_addresses_skip_optional_arguments() {
  setup();
  // a command's optional arguments are no address text (2602.10975's `\makebox[\textwidth][c]{\tt …}`, 77p r6)
  assert_eq!(
    email_addresses(&tk(
      "\\makebox[\\textwidth][c]{\\tt haiyang.wang@huawei.com ~ lue.fan@ia.ac.cn}"
    )),
    Some(vec![s!("haiyang.wang@huawei.com"), s!("lue.fan@ia.ac.cn")])
  );
}

#[test]
fn prints_nothing_reads_spacing_fonts_and_paragraphs() {
  setup();
  // (2609.05430's `\large{~~}`, 2606.00747's blank line, 2103.00887's `\tt\small`; 77p r6)
  for (line, nothing) in [
    ("\\large{~~}", true),
    ("\\vspace{2pt}", true),
    ("{\\small \\vspace{2pt}}", true),
    ("\\par", true),
    ("\\tt\\small ", true),
    ("\\large{Univ A}", false),
    ("\\url{https://x.org}", false),
  ] {
    assert_eq!(prints_nothing(&tk(line)), nothing, "{line}");
  }
}

#[test]
fn rewrite_symbol_superscripts_splits_a_glued_symbol() {
  setup();
  // `$^{1*}$` and `\textsuperscript{a\dagger}`: the mark stays a superscript marker, the symbol is shown…
  for (marked, mark) in [
    ("X$^{1*}$", "X$^{1}$"),
    ("X\\textsuperscript{a\\dagger}", "X\\textsuperscript{a}"),
  ] {
    let out = rewrite_symbol_superscripts(tk(marked)).to_string();
    assert!(
      out.starts_with(&tk(mark).to_string()) && out.contains("\\lx@frontmatter@keepsup"),
      "{marked}: {out}"
    );
  }
  // …and a list of marks or a number alone is no glued symbol.
  for whole in ["X$^{12}$", "X$^{a,b}$", "X$^{1,*}$"] {
    let out = rewrite_symbol_superscripts(tk(whole));
    assert!(
      position_of(&out, &[T_CS!("\\lx@frontmatter@keepsup")]).is_none(),
      "{whole}: {out}"
    );
  }
}

/// 78u: `classify_thanks` reads its funding words as words — "erc" in "Perception" (2509.05368), "nsf" in "transfer",
/// "fund" in "Fundamental" and "Recherche"'s "erc" are an institution's or a note's, not funding.
#[test]
fn classify_thanks_reads_funding_words_whole() {
  for note in [
    "Guangdong Laboratory of Machine Perception and Intelligent Computing",
    "Tata Institute of Fundamental Research, Mumbai",
    "Centre de Recherche en Informatique",
    "Nihon University, Tokyo",
    "Konrad-Zuse-Zentrum",
  ] {
    assert_eq!(classify_thanks(note), "note", "{note}");
  }
  for funding in [
    "Supported by the ERC Starting Grant 12345",
    "This work was funded by NSFC (No. 62276)",
    "Partially supported by NSF grant DMS-1234",
    "NIH R01 award",
    "ONR N00014",
    "DARPA program",
    "Research grants from Google",
    "Financiado por la Fundación BBVA",
    "SNSF Postdoctoral Fellow",
    "Chercheur qualifié FNRS",
    "NSERC Discovery",
  ] {
    assert_eq!(classify_thanks(funding), "funding", "{funding}");
  }
  assert_eq!(classify_thanks("Corresponding author"), "correspondence");
  assert_eq!(
    classify_thanks("These authors contributed equally"),
    "contribution"
  );
  // ("nsf" inside "transfer" is no funder; the work's circumstances, opening the note, are a contribution since 103c)
  assert_eq!(
    classify_thanks("Work done during an internship on transfer learning"),
    "contribution"
  );
  // (an equal share of advising is a phrase, its two words side by side: 2609.32870's "Equal advising." — and not a
  // thanks to an advisor or a board's name, review rev94a r2 S1)
  for share in [
    "Equal advising.",
    "Equal supervision",
    "These authors advised equally",
  ] {
    assert_eq!(classify_thanks(share), "contribution", "{share}");
  }
  assert_eq!(
    classify_thanks("I thank my advisor for equally generous support."),
    "funding"
  );
  assert_eq!(
    classify_thanks("Member of the Equal Opportunity Advisory Board."),
    "note"
  );
}

/// 103c: an author's part in the work is a contribution (census129 cluster 6) — a project role, a phrase of whole words
/// ("†Project lead", 2609.18514; "Joint first authors.", 2608.22772; "Core Contribution", 2408.15240), and the work's
/// circumstances where they open the note ("Work done during an internship", 2106.09776) — not a word of an
/// institution's name ("Project Leadership Institute", "Core Facility") nor circumstances after an institution
/// ("Facebook (work done while at JHU/APL)", 1702.02684); and a legend entry is legends, names and addresses alone,
/// no institution (`entry_legend_kind`: "DeepAuto.ai († : Equal advising)", 2609.34327, is mixed; a name in it is a
/// carrier's, not "Google DeepMind", review rev103c S2). The role readings an institution's name can hold
/// (`role_contribution`) count only where every piece of the text is a legend (review rev103c S1).
#[test]
fn classify_thanks_reads_project_roles_and_the_works_circumstances() {
  // (the role readings read the text's pieces, `reads_as_affiliation`: a State, as a conversion has)
  setup();
  for contribution in [
    "\u{2020}Project lead",
    "Project Leader.",
    "Joint first authors.",
    "Co-first authors",
    "Core Contribution",
    "core contributors",
    "Work done during an internship",
    "Work was done while at Google",
    "* Work done as an intern at Adobe",
    "these authors share senior authorship",
    // (a caller's TeX source, read as it prints)
    "$^{\\dagger}$Project leader.",
    "\\textsuperscript{$\\dagger$}Joint first authors.",
  ] {
    assert_eq!(
      classify_thanks(contribution),
      "contribution",
      "{contribution}"
    );
  }
  for note in [
    "Project Leadership Institute, Univ A",
    "Core Facility for Imaging",
    "Facebook (work done while at JHU/APL)",
    "Majority of work done at IIT Hyderabad",
    "First authors meeting",
  ] {
    assert_eq!(classify_thanks(note), "note", "{note}");
  }
  for role in [
    "Project lead",
    "Core Contributor Network, Paris",
    "Work done during an internship",
  ] {
    assert!(role_contribution(role), "{role}");
  }
  for share in [
    "Equal contribution",
    "These authors contributed equally",
    "Equal advising.",
    "Corresponding author",
  ] {
    assert!(!role_contribution(share), "{share}");
  }
  // (a role reads as a contribution only where every piece of the text is a legend — `classify_thanks`'s gate for every
  // reader of the kind, review rev103c S1; a role naming its author is the Kind step's, which knows its carriers)
  for affiliation in [
    "Project Lead Institute, Univ D",
    "Core Contributor Network, Paris",
    "Senior Author Program, Acme Corp",
    "Project lead: Yi Yang",
  ] {
    assert_eq!(classify_thanks(affiliation), "note", "{affiliation}");
  }
  assert_eq!(
    classify_thanks("Equal contribution, Univ A"),
    "contribution"
  );
  // (the entry's carriers, as their personnames print: a name in a legend is one of theirs, review rev103c S2)
  let carriers: Vec<String> = ["Ann Able1,*", "Xiaofan Xu", "Hua Zhu"]
    .map(String::from)
    .to_vec();
  assert_eq!(
    entry_legend_kind("Joint first authors.", &carriers),
    Some("contribution")
  );
  assert_eq!(
    entry_legend_kind("Corresponding author:", &carriers),
    Some("correspondence")
  );
  assert_eq!(
    entry_legend_kind("Corresponding author: Ann Able", &carriers),
    Some("correspondence")
  );
  assert_eq!(
    entry_legend_kind("Deceased 3 March 2021; formerly at Univ C", &carriers),
    Some("deceased")
  );
  assert_eq!(
    entry_legend_kind(
      "Equal contribution. | * Correspondence: chaokun@tsinghua.edu.cn",
      &carriers
    ),
    Some("correspondence")
  );
  assert_eq!(
    entry_legend_kind(
      "Corresponding authors. Email: yunhui@x.jp (Y.W.) nomura@x.jp (M.N.)",
      &carriers
    ),
    Some("correspondence")
  );
  assert_eq!(
    entry_legend_kind(
      "Corresponding author, email: loizides@ikf.uni-frankfurt.de",
      &carriers
    ),
    Some("correspondence")
  );
  assert_eq!(
    entry_legend_kind("corresponding author(s): Xiaofan Xu, Hua Zhu", &carriers),
    Some("correspondence")
  );
  for affiliation in [
    "corresponding author | SSE, CUHKSZ FNii, CUHKSZ",
    "DeepAuto.ai (\u{2020} : Equal advising)",
    "ByteDance *equal contribution",
    "GEELY. * means equal contribution",
    "Corresponding author | Proceedings of a workshop",
    "Corresponding author: Univ A, Town",
    "Department of Equal Contribution Studies, Univ C",
    "Project Lead Institute, Univ D",
    "Facebook (work done while at JHU/APL)",
    "Univ A, Town",
    "Present address: Univ X, Village",
    // (a company's name shaped as a person's, no carrier's: review rev103c S2)
    "Equal contribution, Google DeepMind",
    "Work done while at Google, Mountain View",
    "Corresponding author: Bob Baker",
  ] {
    assert_eq!(
      entry_legend_kind(affiliation, &carriers),
      None,
      "{affiliation}"
    );
  }
}

/// 100a: "Deceased", as a word and in any case, is an annotation on the author of the same kind as "Corresponding
/// author" (FIDELITY_CHARTER §4, user ruling 2026-11-02): the `deceased` kind, `ltx_thanks_deceased` on a thanks note,
/// a legend line its carriers' note (`legend_note_kind`, review rev99b2 q4e). Not inside another word ("Predeceased").
#[test]
fn classify_thanks_reads_deceased_as_a_word() {
  for deceased in [
    "Deceased",
    "deceased.",
    "(Deceased)",
    "DECEASED",
    "Deceased 12 March 2020",
  ] {
    assert_eq!(classify_thanks(deceased), "deceased", "{deceased}");
  }
  assert_eq!(classify_thanks("Predeceased by his wife"), "note");
  assert_eq!(
    classify_thanks("Corresponding author; deceased"),
    "correspondence"
  );
}

/// 100a r2: "Deceased" opens a note's text where the word comes first, behind its mark (`opens_with_deceased`); an
/// institution named with it inside does not (review rev100a S2).
#[test]
fn opens_with_deceased_reads_the_first_word() {
  assert!(opens_with_deceased("Deceased"));
  assert!(opens_with_deceased(
    "\u{2020}Deceased 3 March 2021; formerly at Univ C"
  ));
  assert!(opens_with_deceased(" (deceased)"));
  assert!(!opens_with_deceased(
    "Department of Deceased Estates, Univ C"
  ));
  assert!(!opens_with_deceased("Predeceased"));
}

/// 100a r2: an asterisk is one key however it is written or printed — `*`, `∗` (U+2217), `⁎`, `✱`, `＊`, `\ast`,
/// `\textasteriskcentered` — on both sides of a match, the name's request and the line's label (`one_asterisk`, review
/// rev100a B1; user ruling 2026-11-02: all asterisks are the same key); a glyph run keeps its parts.
#[test]
fn clean_frontmatter_labels_keys_an_asterisk_one_way() {
  for spelled in [
    "*",
    "\u{2217}",
    "\u{204E}",
    "\u{2731}",
    "\u{FF0A}",
    "\\ast",
    "\\textasteriskcentered",
    "{\\ast}",
  ] {
    assert_eq!(
      clean_frontmatter_labels(spelled, "affiliation"),
      ["affiliation:*"],
      "{spelled}"
    );
  }
  assert_eq!(clean_frontmatter_labels("1,\u{2217}", "affiliation"), [
    "affiliation:1",
    "affiliation:*"
  ]);
  assert_eq!(clean_frontmatter_labels("1\u{2217}", ""), ["1*"]);
  assert_eq!(clean_frontmatter_labels("1,\u{2731}", "affiliation"), [
    "affiliation:1",
    "affiliation:*"
  ]);
}

/// 78u: a note written inside a mark is taken out after it, the mark's labels left (2512.05464), and a mark holding
/// only a note goes (2212.00809).
#[test]
fn notes_out_of_marks_takes_the_note_after_the_mark() {
  setup();
  let out = |s: &'static str| notes_out_of_marks(tk(s)).to_string();
  let tk = |s: &'static str| tk(s).to_string();
  assert_eq!(
    out(r"Ann\textsuperscript{\rm 1,\rm 2\thanks{Work {done}.}}, Bob"),
    tk(r"Ann\textsuperscript{\rm 1,\rm 2}\thanks{Work {done}.}, Bob")
  );
  assert_eq!(
    out(r"Ann$^{\mathbf{1,\thanks{Equal.}}}$ and Cy$^{\thanks{Convener.}}$"),
    tk(r"Ann$^{\mathbf{1,}}\thanks{Equal.}$ and Cy$\thanks{Convener.}$")
  );
  assert_eq!(
    out(r"Ann$^{1\footnotemark[1]}$"),
    tk(r"Ann$^{1}\footnotemark[1]$")
  );
  // (a note after the mark, and a mark holding none, are left as they are)
  assert_eq!(
    out(r"Ann$^{1}$\thanks{X}, Bob\textsuperscript{2}"),
    tk(r"Ann$^{1}$\thanks{X}, Bob\textsuperscript{2}")
  );
}

#[test]
fn collaboration_and_team_words_stay_non_name_words() {
  // (a collaboration's or team's head word reads as no name wherever `non_name_word` is asked — the recognizer's
  // "no institution word before the head" test excepts exactly these; rev97cd D7)
  for word in COLLABORATION_WORDS.iter().chain(TEAM_WORDS) {
    assert!(NON_NAME_WORDS.contains(word), "{word}");
    assert!(non_name_word(word), "{word}");
  }
}

#[test]
fn a_line_frames_a_team_by_more_than_the() {
  // (a team or group word on a line of its own names a collaboration only framed by more than a bare "the" — `The
  // Robotics Group` under a name is its institution; a collaboration word by "the" too; rev97cd D2)
  assert_eq!(collaboration_line_framing("The Robotics Group"), None);
  assert_eq!(
    collaboration_line_framing("the Applied Mathematics Group"),
    None
  );
  assert_eq!(
    collaboration_line_framing("for the GAPS Team"),
    Some(Framing::Framed)
  );
  assert_eq!(
    collaboration_line_framing("(The Data Group)"),
    Some(Framing::Framed)
  );
  assert_eq!(
    collaboration_line_framing("WIDE Consortium"),
    Some(Framing::Unframed)
  );
  assert_eq!(
    collaboration_line_framing("the ATLAS Collaboration"),
    Some(Framing::The)
  );
  assert_eq!(
    collaboration_line_framing("Representing the BES Collaboration"),
    Some(Framing::Framed)
  );
  assert_eq!(
    collaboration_line_framing("On Behalf Of The ATLAS Collaboration"),
    Some(Framing::Framed)
  );
  // (a piece of a names line: "the" frames it, `…, the ASDEX Upgrade Team and the EUROfusion MST1 Team`, 1603.09150)
  assert!(reads_as_collaboration("the ASDEX Upgrade Team", false));
}

#[test]
fn a_contact_call_in_a_name_is_no_name_text() {
  setup();
  // (a heading's collaboration made in a name's content, after its words: no part of them — read, "WestXYZ" spelled no
  // address, 1612.09318's `awest@` went to no one; rev97cd r2)
  let headed = tk(
    r"\lx@author@withsup{C~West$^1$\lx@add@contact[role=collaboration,annotate=1]{XYZ Collaboration}}",
  );
  assert_eq!(visible_name_text(headed.unlist_ref()), "C West");
  assert_eq!(spelled_owners(&[NameKeys::of(&headed)], "cwest@x.edu"), [0]);
}

#[test]
fn a_name_spells_a_run_of_the_local_parts_words() {
  setup();
  // (census138 EMAILSPELL, brief 107f B, OD #970 "107f": a surname or given name spells a run of the local part's words
  // — inside one word, at its start or its end; never strictly inside another word, nor across a separator)
  let keys = [
    NameKeys::of(&tk("Shengyuan A. Yang")),
    NameKeys::of(&tk("Lay Kee Ang")),
  ];
  assert_eq!(spelled_owners(&keys, "ricky_ang@x.edu"), [1]);
  assert_eq!(spelled_owners(&keys, "ang2@x.edu"), [1]);
  assert_eq!(spelled_owners(&keys, "kee.ang@x.edu"), [1]);
  assert_eq!(spelled_owners(&keys, "shengyuan.yang@x.edu"), [0]);
  assert!(spelled_owners(&keys, "bangkok@x.edu").is_empty());
  assert!(spelled_owners(&keys, "zhangwei@x.edu").is_empty());
  // (between initials, two letters at most on either side: `kmfonsecar@`, 1202.4210; `vemarinc@`, 2311.18024)
  let keys = [
    NameKeys::of(&tk("K. M. Fonseca Romero")),
    NameKeys::of(&tk("Ann Able")),
  ];
  assert_eq!(spelled_owners(&keys, "kmfonsecar@x.co"), [0]);
  let keys = [
    NameKeys::of(&tk("Victor Marin")),
    NameKeys::of(&tk("Ann Able")),
  ];
  assert_eq!(spelled_owners(&keys, "vemarinc@x.co"), [0]);
  // (a three-letter word between letters is no name part: `yvanwy@` is no "van", 2202.02002; `byang@` no "yan", 2602.00582)
  let keys = [
    NameKeys::of(&tk("Anton van den Hengel")),
    NameKeys::of(&tk("Kangjia Yan")),
  ];
  assert!(spelled_owners(&keys, "yvanwy@x.com").is_empty());
  assert!(spelled_owners(&keys, "byang@x.cn").is_empty());
  // (between the name's own initials a shorter part does: `rjosem@`, 1809.06911; `mycheng@` is Mingyue Cheng's, whose
  // whole surname ends the word, not Enhong Chen's, weak, 2603.07528; a joined spelling ending a word spells only at five
  // letters, `pyli@` no "yli", 2105.07637)
  let keys = [
    NameKeys::of(&tk("Jose M. Rodriguez-Nogales")),
    NameKeys::of(&tk("Enhong Chen")),
    NameKeys::of(&tk("Mingyue Cheng")),
  ];
  assert_eq!(spelled_owners(&keys, "rjosem@x.es"), [0]);
  assert_eq!(spelled_owners(&keys, "mycheng@x.cn"), [2]);
  let keys = [
    NameKeys::of(&tk("Pengyang Li")),
    NameKeys::of(&tk("Yanan Li")),
  ];
  assert!(spelled_owners(&keys, "pyli@x.com").is_empty());
  // (a truncated surname's letters end or start a word as a whole one does: `mkaufman@` Kaufmann's; one strictly inside
  // a word spells no one)
  let keys = [NameKeys::of(&tk("Michael Kaufmann"))];
  assert_eq!(spelled_owners(&keys, "mkaufman@x.de"), [0]);
  assert_eq!(spelled_owners(&keys, "kaufma.m@x.de"), [0]);
  assert!(spelled_owners(&keys, "abckaufmay@x.de").is_empty());
  // (a joined spelling ending a word whole spells it, `sechenqi@` Qi Chen's "chenqi", 2003.00397; an apostrophe inside
  // a name's word joins it, `damicos@` Simone D'Amico's "damico", 2606.17317)
  let keys = [
    NameKeys::of(&tk("Qi Chen")),
    NameKeys::of(&tk("Mingkui Tan")),
  ];
  assert_eq!(spelled_owners(&keys, "sechenqi@x.cn"), [0]);
  let keys = [
    NameKeys::of(&tk("Simone D'Amico")),
    NameKeys::of(&tk("Ann Able")),
  ];
  assert_eq!(spelled_owners(&keys, "damicos@x.edu"), [0]);
  // (an escaped underscore parts the words: `address_text`)
  assert_eq!(
    address_owners(
      &[
        NameKeys::of(&tk("Lay Kee Ang")),
        NameKeys::of(&tk("Shengyuan A. Yang"))
      ],
      &tk(r"ricky\_ang@x.edu")
    ),
    [0]
  );
  // (a brace set in math is the brace it prints: `math_braces_as_printed`, OD #972)
  assert_eq!(
    math_braces_as_printed(&tk(r"$\{$a,b$\}$@x.org")).to_string(),
    tk(r"\{a,b\}@x.org").to_string()
  );
  assert_eq!(
    math_braces_as_printed(&tk(r"$\lbrace$a$\rbrace$@x.org")).to_string(),
    tk(r"\{a\}@x.org").to_string()
  );
  assert_eq!(
    math_braces_as_printed(&tk(r"$x$@y.org")).to_string(),
    tk(r"$x$@y.org").to_string()
  );
}

#[test]
fn a_weak_spelling_never_decides_between_names() {
  setup();
  // (brief 107f r2, the one boundary predicate: a joined spelling ending a word it does not open, or a key of four
  // letters or more strictly inside a word between letters that are none of the name's initials, owns an address only
  // no other name spells — review rev107f B1, 2510.24320)
  let keys = [
    NameKeys::of(&tk("Jixuan Huang")),
    NameKeys::of(&tk("Xuanjing Huang")),
  ];
  assert_eq!(spelled_owners(&keys, "xjhuang@x.cn"), [0, 1]);
  assert_eq!(spelled_owners(&keys, "jhuang@x.cn"), [0]);
  assert_eq!(spelled_owners(&keys, "x.jhuang@x.cn"), [0]);
  let keys = [
    NameKeys::of(&tk("Yi Wang")),
    NameKeys::of(&tk("Xiaoyu Wang")),
  ];
  assert_eq!(spelled_owners(&keys, "yiwang@x.cn"), [0]);
  assert_eq!(spelled_owners(&keys, "xywang@x.cn"), [0, 1]);
  // (uncontested, a weak spelling owns it: `sechenqi@` Qi Chen's "chenqi", 2003.00397; `vemarinc@` Víctor Marín's
  // "marin", 2311.18024; contested by a whole word, it never wins: `vemarinc@` beside a Vemarin)
  let keys = [NameKeys::of(&tk("Qi Chen")), NameKeys::of(&tk("Wei Chen"))];
  assert_eq!(spelled_owners(&keys, "sechenqi@x.cn"), [0]);
  // (all of them weak, as long: no tie-break by the surname, `kkwongar@` beside two Wongs, 2608.26715)
  let keys = [
    NameKeys::of(&tk("Hiu Tung Wong")),
    NameKeys::of(&tk("Wong Kam-Kwai")),
  ];
  assert_eq!(spelled_owners(&keys, "kkwongar@x.hk"), [0, 1]);
  let keys = [
    NameKeys::of(&tk("Victor Marin")),
    NameKeys::of(&tk("Ann Vemarinc")),
  ];
  assert_eq!(spelled_owners(&keys, "vemarinc@x.co"), [1]);
  // (the sides of a key inside a word are counted in characters, not bytes: two accented letters are two)
  let keys = [
    NameKeys::of(&tk("Victor Marin")),
    NameKeys::of(&tk("Ann Able")),
  ];
  assert_eq!(spelled_owners(&keys, "vémarinc@x.co"), [0]);
  assert!(spelled_owners(&keys, "véémarinc@x.co").is_empty());
  // (brief 107g: a four-letter surname, after two letters at most, whatever follows it: `tpengqy@` Q. Y. Peng's, 2004.06253;
  // `eetanggz@` Guozhi Tang's, 2106.10681; `ylwanguestc@` Yinglong Wang's, 2208.05114. After three letters never. A
  // hyphen-linked surname joined: `levtalor@` L. Tal-Or's, 2112.09029, review rev107g)
  let keys = [
    NameKeys::of(&tk("Q. Y. Peng")),
    NameKeys::of(&tk("Guozhi Tang")),
    NameKeys::of(&tk("Yinglong Wang")),
    NameKeys::of(&tk("L. Tal-Or")),
  ];
  assert_eq!(spelled_owners(&keys, "tpengqy@x.cn"), [0]);
  assert_eq!(spelled_owners(&keys, "eetanggz@x.cn"), [1]);
  assert_eq!(spelled_owners(&keys, "ylwanguestc@x.cn"), [2]);
  assert_eq!(spelled_owners(&keys, "levtalor@x.il"), [3]);
  assert_eq!(spelled_owners(&keys, "abcpeng7@x.il"), [0]);
  assert!(spelled_owners(&keys, "abcpengxy@x.il").is_empty());
}

#[test]
fn an_own_call_moves_only_to_a_strictly_better_full_spelling() {
  setup();
  // (FIDELITY_CHARTER §4 #420, brief 107g, rev107f2 finding 1: a tie keeps it — `yang91@` Yang Zhang's beside Hengyun Yang,
  // 1707.09730; `minhnnhat@` Nhat Ho's beside Anh-Minh Phan, 2511.18493; `ricky_ang@` Lay Kee Ang's, 2604.02636)
  let keeps = |holder: &'static str, owner: &'static str, address: &str| {
    own_call_keeps(
      &NameKeys::of(&tk(holder)),
      &NameKeys::of(&tk(owner)),
      address,
      false,
    )
  };
  assert!(keeps("Yang Zhang", "Hengyun Yang", "yang91@x.edu"));
  assert!(keeps("Nhat Ho", "Anh-Minh Phan", "minhnnhat@x.edu"));
  assert!(keeps("Lay Kee Ang", "Shengyuan A. Yang", "ricky_ang@x.edu"));
  assert!(keeps("Yang Zhang", "Hengyun Yang", "yzhang@x.edu"));
  // (a longer spelling moves it, whichever part of it the holder's key is: its tail, a truncation, its head — 0902.4311,
  // 0907.3301, 1008.3764, 2107.06981; review rev107f finding 3)
  assert!(!keeps("Mark Anderson", "Lee Sanderson", "sanderson@y.edu"));
  assert!(!keeps("Peter Andersen", "Maria Anderson", "anderson@y.edu"));
  assert!(!keeps("Lay Kee Ang", "Angela Rossi", "angela.rossi@y.edu"));
  assert!(!keeps("Jang Soo Kim", "Dongsu Kim", "dongsu.kim@x.kr"));
  assert!(!keeps("Gianni Pola", "Giordano Pola", "giordano.pola@x.it"));
  assert!(!keeps("Yi Liang", "Zhu-Xing Liang", "zx.liang55@x.cn"));
  assert!(!keeps(
    "Italy. email",
    "Filippo Neri",
    "filippo.neri.email@x.com"
  ));
  // (a weak spelling moves no own call — 2411.01273, 2609.11148 —, but for an address set after the last author, its
  // holder's name not standing there: 2004.06253)
  assert!(keeps("Husheng Yu", "Xiangmin Shen", "yhsheng@y.edu"));
  assert!(keeps("Chao Huang", "Wang Hong", "zhonggong.hc@y.edu"));
  let last = |holder: &'static str, owner: &'static str, address: &str| {
    own_call_keeps(
      &NameKeys::of(&tk(holder)),
      &NameKeys::of(&tk(owner)),
      address,
      true,
    )
  };
  assert!(!last("Y. J. Shang", "Q. Y. Peng", "tpengqy@y.edu"));
  assert!(last("Yang Zhang", "Hengyun Yang", "yang91@y.edu"));
  // (the holder's name standing there weakly: 2608.26715's `kkwongar@` on the last author, Wong Kam-Kwai)
  assert!(last("Wong Kam-Kwai", "Hiu Tung Wong", "kkwongar@y.edu"));
}

#[test]
fn a_name_spells_by_its_base_letters_and_its_transliteration() {
  setup();
  // (html_feedback #7233, OD #970: an accent or letter command written in the source, or the letter itself, spells by
  // its base letter and by its transliteration; as printed (`as_printed`, whom an e-mail moves to) by its
  // transliteration too, a spelling of the name as printed (user ruling 2026-11-05), never by a base-letter fold of a
  // letter it prints (FIDELITY_CHARTER §4 #420: `muller-group@` stays))
  for (name, local, base) in [
    (r#"Moritz Sch\"affler"#, "mo.schaeffler", "m.schaffler"),
    (r#"Moritz Sch\"{a}ffler"#, "mo.schaeffler", "m.schaffler"),
    ("Moritz Schäffler", "mo.schaeffler", "m.schaffler"),
    (r"Gus Wei{\ss}enborn", "g.weissenborn", "g.weisenborn"),
    ("Gus Weißenborn", "g.weissenborn", "g.weisenborn"),
    (r"Ida Hj\o rth", "i.hjoerth", "i.hjorth"),
    (r"Jon Fl\aa ten", "j.flaaten", "j.flaten"),
    (r"Jon Fl\r{a}ten", "j.flaaten", "j.flaten"),
    (r"Mads S\ae ther", "m.saether", "m.sather"),
  ] {
    let keys = [NameKeys::of(&tk(name))];
    assert_eq!(spelled_owners(&keys, &s!("{local}@x.org")), [0], "{name}");
    assert_eq!(spelled_owners(&keys, &s!("{base}@x.org")), [0], "{name}");
    let printed = [NameKeys::as_printed(&tk(name))];
    assert_eq!(
      spelled_owners(&printed, &s!("{local}@x.org")),
      [0],
      "{name}"
    );
    if !name.contains('\\') {
      assert!(
        spelled_owners(&printed, &s!("{base}@x.org")).is_empty(),
        "{name}"
      );
    }
  }
  // (no looser matching otherwise: another accent's letter has no transliteration, `\'e` is e alone, one spelling)
  assert_eq!(NameKeys::of(&tk(r"Ren\'e Ruiz")).spellings().count(), 1);
  assert_eq!(
    visible_name_text(tk(r#"Sch\"affler"#).unlist_ref()),
    "Schaffler"
  );
}

#[test]
fn email_addresses_write_out_unspaced_lists() {
  setup();
  // (batch 103a: a list of local parts written without spaces before one domain, inside a `;`- or `,`-parted line, is
  // each part at the domain — the frontmatter observer's reading too; 2112.00969)
  for line in [
    "ann@a.edu; dzyang,ashishbshah@fb.com",
    "ann@a.edu, dzyang,ashishbshah@fb.com",
    "\\texttt{ann@a.edu; dzyang,ashishbshah@fb.com}",
  ] {
    assert_eq!(
      email_addresses(&tk(line)),
      Some(vec![
        s!("ann@a.edu"),
        s!("dzyang@fb.com"),
        s!("ashishbshah@fb.com")
      ]),
      "{line}"
    );
  }
  // (a capitalised word or a digit run is no local part: no address invented, the line no list)
  assert_eq!(email_addresses(&tk("Berlin,Germany,foo@bar.de")), None);
  assert_eq!(email_addresses(&tk("5551234,info@x.org")), None);
  // (review rev103a B1, S1: a word naming a contact or an initial is no local part, nor does an initial end a list —
  // no `email@x.org`, `contact@a.edu`, `d@fb.com`, `smith@x.org` invented, the line no list)
  for line in [
    "email,ann@x.org, bob@x.org",
    "ann@a.edu; contact,bob@a.edu",
    "ann@a.edu; d,ashishbshah@fb.com",
    "smith,j@x.org, doe@x.org",
  ] {
    assert_eq!(email_addresses(&tk(line)), None, "{line}");
  }
  // (a spaced list is no list the reader parts)
  assert_eq!(
    email_addresses(&tk("ann@a.edu; dzyang, ashishbshah@fb.com")),
    None
  );
  // (int131fix: a run glued to a word that is no local part is no list — left whole, no part cut out of it; a contact's
  // word before it is a label, astro-ph0612088)
  assert_eq!(
    write_out_unspaced_lists("M.R.Pestalozzi,a.chrysostomou,j.l.collett@herts.ac.uk"),
    "M.R.Pestalozzi,a.chrysostomou,j.l.collett@herts.ac.uk"
  );
  assert_eq!(
    write_out_unspaced_lists("email,ann,bob@x.org"),
    "email,ann@x.org,bob@x.org"
  );
  assert_eq!(
    write_out_unspaced_lists("ann@a.edu,dzyang,ashishbshah@fb.com"),
    "ann@a.edu,dzyang@fb.com,ashishbshah@fb.com"
  );
}

#[test]
fn spelled_unspaced_list_writes_out_a_list_of_spelled_parts() {
  setup();
  // (int131fix, astro-ph0612088: every part spells one author, each a different one, capitals as printed)
  let names = ["M. R. Pestalozzi", "A. Chrysostomou", "J. L. Collett"];
  let keys: Vec<NameKeys> = names.iter().map(|&n| NameKeys::of(&tk(n))).collect();
  let owner = |part: &str| spelled_owner(&keys, &s!("{part}@x.org"));
  assert_eq!(
    spelled_unspaced_list(
      "M.R.Pestalozzi,a.chrysostomou,j.l.collett@herts.ac.uk",
      &owner
    ),
    Some(vec![
      s!("M.R.Pestalozzi@herts.ac.uk"),
      s!("a.chrysostomou@herts.ac.uk"),
      s!("j.l.collett@herts.ac.uk")
    ])
  );
  // (a part spelling no one, two parts of one author, a spaced list, one address: none)
  for text in [
    "M.R.Pestalozzi,info,j.l.collett@herts.ac.uk",
    "pestalozzi,m.pestalozzi@herts.ac.uk",
    "M.R.Pestalozzi, a.chrysostomou@herts.ac.uk",
    "j.l.collett@herts.ac.uk",
  ] {
    assert_eq!(spelled_unspaced_list(text, &owner), None, "{text}");
  }
}

/// The Kind step for an entry's address line reads a line's label as printed (`insert::address_line_cut`, OD #966): a
/// bracketed e-mail label is the label without its bracket (hep-ph0611390's `(email: braghin@…)` had left `(email` in
/// the entry; 1009.1145's `[email:…]`); words before an e-mail label stay (1111.6932); a contact's words alone before
/// the addresses of a legend line (an entry's only line, a mark opening it) stay as its legend, else they name the
/// e-mails; an address beside words with no label and a tagged run are no line of addresses.
#[test]
fn an_address_line_cut_reads_the_label_as_printed() {
  let none = |_: &str| false;
  let cut = |text: &str, only: bool| {
    address_line_cut(text, &none, only).map(|cut| (cut.start, cut.label, cut.addresses))
  };
  assert_eq!(
    cut("(email: braghin@if.usp.br) ", true),
    Some((0, Some("email".to_string()), vec![
      "braghin@if.usp.br".to_string()
    ]))
  );
  let legend = " To whom correspondence should be addressed; E-mail: ann.able@a.edu";
  assert_eq!(
    cut(legend, false),
    Some(
      (legend.find(';').unwrap(), Some("E-mail".to_string()), vec![
        "ann.able@a.edu".to_string()
      ])
    )
  );
  assert_eq!(
    cut(" Corresponding author: ann.able@a.edu", true),
    Some((" Corresponding author".len(), None, vec![
      "ann.able@a.edu".to_string()
    ]))
  );
  assert_eq!(
    cut(" Corresponding author: ann.able@a.edu", false),
    Some((0, Some("Corresponding author".to_string()), vec![
      "ann.able@a.edu".to_string()
    ]))
  );
  assert_eq!(cut("Univ A, Town, see data@a.org for data", false), None);
  assert_eq!(cut("ann.able@a.edu (Ann), bbaker@a.edu (Bob)", false), None);
  // (a cut that would leave the words before it an open bracket its tail closes declines — the line stays as printed:
  // `(Fax: +31-20/44-47999, E-mail: stefan@nat.vu.nl)`, physics0308076, bundle int134a fm_checks check 7)
  assert_eq!(
    cut("(Fax: +31-20/44-47999, E-mail: stefan@nat.vu.nl)", false),
    None
  );
  assert_eq!(cut("Univ A [Fax: 12, E-mail: a@x.org]", false), None);
  // (a balanced bracket before the label does not)
  assert!(cut("Univ A (Dept B), E-mail: a@x.org", false).is_some());
  // (107c: an unlabelled tail of addresses after a list's or a sentence's boundary — 2602.07392, 1903.04089,
  // 1204.4622 —, never one beside a word: "Univ A, Town, see data@a.org" stays prose)
  let tail = "Dept A, Univ A; aable@a.edu, bbaker@a.edu";
  assert_eq!(
    cut(tail, false),
    Some((tail.find(';').unwrap(), None, vec![
      "aable@a.edu".to_string(),
      "bbaker@a.edu".to_string()
    ]))
  );
  assert_eq!(
    cut("Univ A, ann@a.edu", false),
    Some(("Univ A".len(), None, vec!["ann@a.edu".to_string()]))
  );
  assert_eq!(
    cut("Street 1, Town. ann@a.edu", false),
    Some(("Street 1, Town.".len(), None, vec!["ann@a.edu".to_string()]))
  );
  assert_eq!(cut("Univ A, Town, see data@a.org", false), None);
  // (a word glued to its address, an initial's list, or contact words alone stay: review rev103a S1, B1)
  assert_eq!(cut("smith,j@x.org, doe@x.org", false), None);
  assert_eq!(cut("email,ann@x.org, bob@x.org", false), None);
  // (a word before the comma that may be the list's local part: no partial cut, 2003.08808; a brace's list is its own
  // boundary, 2309.06986)
  assert_eq!(
    cut("Ottawa, Canada, mmoza102, wslee@uottawa.ca", false),
    None
  );
  let braced = "Haifa, Israel {avico@, arielba@}technion.ac.il, ami@gmail.com.";
  assert_eq!(
    cut(braced, false),
    Some(("Haifa, Israel".len(), None, vec![
      "avico@technion.ac.il".to_string(),
      "arielba@technion.ac.il".to_string(),
      "ami@gmail.com".to_string()
    ]))
  );
  // (107c: contact words and "to", no colon, a label: 2606.31381; "write to" stays prose)
  assert_eq!(
    cut(" Corresponding to ann@a.edu", false),
    Some((0, Some("Corresponding to".to_string()), vec![
      "ann@a.edu".to_string()
    ]))
  );
  assert_eq!(cut("Univ A, please write to ann@a.edu", false), None);
  // (107c r2: a correspondence role phrase before the label, marks alone before it, is the label's own — the line whole,
  // the phrase its owner's note at the flush, 1711.00192 —; one after other words is not)
  assert_eq!(
    cut(" * Corresponding author, E-mail: qz@a.org ", false),
    Some((0, Some("Corresponding author, E-mail".to_string()), vec![
      "qz@a.org".to_string()
    ]))
  );
  let worded = "Univ A, Corresponding author, E-mail: qz@a.org";
  assert_eq!(
    cut(worded, false),
    Some((
      "Univ A, Corresponding author".len(),
      Some("E-mail".to_string()),
      vec!["qz@a.org".to_string()]
    ))
  );
  // (107c r2: a stop the cut strands after a list's separator goes with the addresses, 1503.09120 — review rev107c N2;
  // a stop ending the words stays, above)
  assert_eq!(
    cut("Valparaiso, IN 46383, USA; . wen.lu@valpo.edu", false),
    Some(("Valparaiso, IN 46383, USA".len(), None, vec![
      "wen.lu@valpo.edu".to_string()
    ]))
  );
}
