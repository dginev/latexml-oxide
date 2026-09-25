//! Surpass #232: `\begin{document}` runs a user-redefined `\document` (as
//! LaTeX's `\begingroup\document`) and `\end{document}` a user-redefined
//! `\enddocument` (then `\endgroup`); the binding's aliases keep the
//! document constructors. The two corpus shapes: ltnews/l3news
//! `\renewenvironment{document}` around the `\input` of each issue (an
//! issue's `\end{document}` ended the whole newsletter in both engines) and
//! tools-overview's `\def\enddocument{<tail>\TO@enddocument}`. Whole
//! `<document>` elements are pinned: every paragraph, in order.

#[test]
fn renewed_document_environment_wraps_an_input_issue() {
  let driver = "\\documentclass{article}\n\\begin{document}\nDRIVER-START\n\\begingroup\n\\renewcommand*{\\documentclass}[2][]{}\n\\renewenvironment{document}{ISSUE-BEGIN}{ISSUE-END}\n\\input{issue1}\n\\endgroup\nDRIVER-AFTER-ISSUE\n\\end{document}\n";
  let issue = "\\documentclass{article}\n\\begin{document}\nISSUE-ONE-BODY\n\\end{document}\n";
  let (stderr, xml) =
    super::perfect_kernel_batch46::convert_files(driver, &[("issue1.tex", issue)]);
  assert_eq!(super::error_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "document",
    &[],
    r##"<document xmlns="http://dlmf.nist.gov/LaTeXML"><resource src="LaTeXML.css" type="text/css"/><resource src="ltx-article.css" type="text/css"/><para xml:id="p1"><p>DRIVER-START ISSUE-BEGIN ISSUE-ONE-BODY ISSUE-END</p></para><para xml:id="p2"><p>DRIVER-AFTER-ISSUE</p></para></document>"##,
  );
}

/// The absorb-a-subfile idiom (exam-n.cls:1348 `\includequestion`:
/// `\begingroup \let\document\@empty \let\enddocument\endinput
/// \input{…} \endgroup`): the question's `\begin{document}` is a renewed
/// `document` (a group opens), its `\end{document}` runs `\endinput` — a
/// closure, not a user macro — and must still close that group (sweep 82:
/// `\lx@finalize@document Attempt to end mode internal_vertical`, the
/// end side re-deriving the begin's decision from `\enddocument`'s shape).
#[test]
fn included_subfile_document_closes_its_group() {
  let driver = "\\documentclass{article}\n\\makeatletter\n\\def\\dummy@dc{\\@ifnextchar[\\dummy@@dc{\\dummy@@dc[]}}\n\\def\\dummy@@dc[#1]#2{}\n\\begin{document}\nMain before.\n\\begingroup\n  \\let\\documentclass\\dummy@dc\n  \\let\\document\\@empty\n  \\let\\enddocument\\endinput\n  \\input{qq}\n\\endgroup\nMain after.\n\\end{document}\n";
  let question = "\\documentclass{article}\n\\begin{document}\nQuestion body.\n\\end{document}\n";
  let (stderr, xml) = super::perfect_kernel_batch46::convert_files(driver, &[("qq.tex", question)]);
  assert_eq!(super::error_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "document",
    &[],
    r##"<document xmlns="http://dlmf.nist.gov/LaTeXML"><resource src="LaTeXML.css" type="text/css"/><resource src="ltx-article.css" type="text/css"/><para xml:id="p1"><p>Main before. Question body.</p></para><para xml:id="p2"><p>Main after.</p></para></document>"##,
  );
}

/// A package wrapping `\document` around the original and leaving
/// `\enddocument` alone (dblfnote.sty:210-211 `\let\dfn@document\document
/// \def\document{\dfn@document …}`; etoolbox's `\AtEndPreamble` likewise):
/// the begin opens its group, the wrapper runs the original constructor
/// inside it, and `\end{document}` finalizes as on the constructor path —
/// in TeX the original `\enddocument` never returns, so `\end`'s
/// `\endgroup` is never reached; an explicit one met the constructor's
/// mode frame ("\endgroup Attempt to close a group that switched to mode
/// internal_vertical", seven manuals in sweep 83: yafoot-man, guitartabs,
/// zanabazr, recorder-fingering, …).
#[test]
fn package_wrapped_document_finalizes_without_closing_the_begin_group() {
  let tex = "\\documentclass{article}\n\\makeatletter\n\\let\\dfn@document\\document\n\\def\\document{\\dfn@document\\def\\wrapped{WRAPPED}}\n\\makeatother\n\\begin{document}\nBody \\wrapped.\n\\end{document}\n";
  let (stderr, xml) = super::convert(tex, true);
  assert_eq!(super::error_count(&stderr), 0, "{stderr}");
  assert!(!stderr.contains("open groups"), "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "document",
    &[],
    r##"<document xmlns="http://dlmf.nist.gov/LaTeXML"><resource src="LaTeXML.css" type="text/css"/><resource src="ltx-article.css" type="text/css"/><para xml:id="p1"><p>Body WRAPPED.</p></para></document>"##,
  );
}

/// A renewed `document` is a normal environment, grouped: l3news.tex:109
/// wraps `\addtocontents` per issue (`\let\saved@addtocontents
/// \addtocontents` then `\renewcommand`); without the group the wrapper
/// leaked into the next `\input` issue and the third issue's `\let` captured
/// its own wrapper — a self-recursive macro, `PushbackLimit`. Three issues
/// through the same wrap must all arrive.
#[test]
fn renewed_document_environment_scopes_each_issue() {
  let driver = "\\documentclass{article}\n\\makeatletter\n\\newcommand{\\logline}[1]{}\n\\begin{document}\nDRIVER-START\n\\begingroup\n\\renewcommand*{\\documentclass}[2][]{}\n\\renewenvironment{document}{\\let\\saved@logline\\logline\\renewcommand*{\\logline}[1]{\\saved@logline{##1}}}{}\n\\input{issue1}\\input{issue2}\\input{issue3}\n\\endgroup\nDRIVER-END\n\\end{document}\n";
  let issue = |n: u32| {
    format!(
      "\\documentclass{{article}}\n\\begin{{document}}\n\\section{{Issue {n}}}\n\\logline{{x}}ISSUE-{n}-BODY\n\\end{{document}}\n"
    )
  };
  let files = [
    ("issue1.tex", issue(1)),
    ("issue2.tex", issue(2)),
    ("issue3.tex", issue(3)),
  ];
  let refs: Vec<(&str, &str)> = files.iter().map(|(n, t)| (*n, t.as_str())).collect();
  let (stderr, xml) = super::perfect_kernel_batch46::convert_files(driver, &refs);
  assert_eq!(super::error_count(&stderr), 0, "{stderr}");
  assert!(!stderr.contains("Fatal:"), "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "document",
    &[],
    r##"<document xmlns="http://dlmf.nist.gov/LaTeXML"><resource src="LaTeXML.css" type="text/css"/><resource src="ltx-article.css" type="text/css"/><para xml:id="p1"><p>DRIVER-START</p></para><section inlist="toc" xml:id="S1"><tags><tag>1</tag><tag role="refnum">1</tag><tag role="typerefnum">§1</tag></tags><title><tag close=" ">1</tag>Issue 1</title><para xml:id="S1.p1"><p>ISSUE-1-BODY</p></para></section><section inlist="toc" xml:id="S2"><tags><tag>2</tag><tag role="refnum">2</tag><tag role="typerefnum">§2</tag></tags><title><tag close=" ">2</tag>Issue 2</title><para xml:id="S2.p1"><p>ISSUE-2-BODY</p></para></section><section inlist="toc" xml:id="S3"><tags><tag>3</tag><tag role="refnum">3</tag><tag role="typerefnum">§3</tag></tags><title><tag close=" ">3</tag>Issue 3</title><para xml:id="S3.p1"><p>ISSUE-3-BODY</p></para><para xml:id="S3.p2"><p>DRIVER-END</p></para></section></document>"##,
  );
}

#[test]
fn redefined_enddocument_runs_its_tail_before_the_end() {
  let tex = "\\documentclass{article}\n\\makeatletter\n\\let\\TO@enddocument\\enddocument\n\\def\\enddocument{TAIL-BEFORE-END\\par\\TO@enddocument}\n\\makeatother\n\\begin{document}\nBODY\n\\end{document}\n";
  let (stderr, xml) = super::convert(tex, true);
  assert_eq!(super::error_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "document",
    &[],
    r##"<document xmlns="http://dlmf.nist.gov/LaTeXML"><resource src="LaTeXML.css" type="text/css"/><resource src="ltx-article.css" type="text/css"/><para xml:id="p1"><p>BODY TAIL-BEFORE-END</p></para></document>"##,
  );
}
