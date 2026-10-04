//! crossreftools.sty: every extractor reads a label's data from `\r@<label>`, which a second LaTeX run defines from the
//! `.aux` (hyperref's five fields: reference, page, name, anchor, unused; crossreftools.sty:293-345), and
//! `\crtlistoflabels` inputs the `.lla` file the labels wrote (`\crtaddlabeltotoc`, :433-441; `\@crt@listofl@bels`,
//! :407-421). Neither round trip happens here, so the extractors printed `[UNDEFINED]` or nothing and the list was
//! empty. The binding is that second pass, per label: `\crtaddlabeltotoc`, which the package's `\label` runs right
//! after the label (:638-656), defines `\r@<label>` from the values at the label — a backward reference resolves, a
//! forward one stays undefined, as in a first run — and records the label's `.lla` line (`\numberline{\crtrefnumber
//! {<label>}}<label>`) as a reference to the label. The list prints the recorded lines when it is built, after the
//! whole document is read (a streamed conversion builds it with the lines read so far: residual). Links go to the
//! label (`\hyperref[<label>]`), not to the hyperref anchor name in the data, which LaTeXML does not make. With
//! cleveref, whose binding sets `\label` after the raw begin-document hooks, the package's wrapper is put back around
//! it (cleveref loaded after crossreftools keeps cleveref's `\label` unwrapped: residual). Witness
//! crossreftools/crossreftools_driver. Guards `perfect_kernel_batch61::{crossreftools_label_data_and_list,
//! crossreftools_without_cleveref}`.
use latexml_package::prelude::*;

LoadDefinitions!({
  InputDefinitions!("crossreftools", noltxml => true, extension => Some(Cow::Borrowed("sty")), handleoptions => true);
  RawTeX!(
    r"\def\crtaddlabeltotoc#1{\lx@crt@newlabel{#1}%
  \ifcrtfinal\else\lx@crt@lla{#1}{\lx@crt@entry{#1}{\crtrefnumber{#1}}}\fi}
\def\lx@crt@newlabel#1{\begingroup
  \ifdefined\@currentlabelname\else\let\@currentlabelname\@empty\fi
  \ifdefined\@currentHref\else\let\@currentHref\@empty\fi
  \ifx\@currentHref\@empty\protected@edef\@currentHref{\lx@crt@counter.\@currentlabel}\fi
  \protected@xdef\lx@crt@data{{\@currentlabel}{\thepage}{\@currentlabelname}{\@currentHref}{}}%
  \endgroup\expandafter\global\expandafter\let\csname r@#1\endcsname\lx@crt@data}
\def\crthypercref#1{\if@crt@hyperrefloaded\hyperref[#1]{\crtcref{#1}}\else\crtcref{#1}\fi}
\def\crthyperCref#1{\if@crt@hyperrefloaded\hyperref[#1]{\crtCref{#1}}\else\crtCref{#1}\fi}
\def\@crtlnameref@unstarred#1{\crtifdefinedlabel{#1}{%
  \if@crt@hyperrefloaded\hyperref[#1]{\@crtlnameref@starred{#1}}\else\@crtlnameref@starred{#1}\fi}{}}
\def\@crtunameref@unstarred#1{\crtifdefinedlabel{#1}{%
  \if@crt@hyperrefloaded\hyperref[#1]{\@crtunameref@starred{#1}}\else\@crtunameref@starred{#1}\fi}{}}
\def\crt@nameref@unstarred#1{\crtifdefinedlabel{#1}{%
  \if@crt@hyperrefloaded\hyperref[#1]{\crtrefname{#1}}\else\crtrefname{#1}\fi}{}}"
  );
  // The package wraps whatever `\label` is at `\begin{document}` (:638-656); a binding's own begin-document `\label`
  // (cleveref_sty.rs) is installed after the raw hooks ran, so the wrapper is put back around it, the order LaTeX's
  // hooks give.
  at_begin_document(TokenizeInternal!(
    r"\ifx\label\lx@cleverref@label
  \let\crt@l@bels@fe\label\def\label{\@ifnextchar[{\l@belwithopt@rg}{\l@belwithoutopt@rg}}\fi"
  ))?;
  // hyperref's anchor names a label's counter before its first `.` (`chapter.1`; `\crtrefcounter`,
  // crossreftools.sty:332-343, splits it there); LaTeXML makes none, so the data carries the counter the label
  // follows.
  DefMacro!("\\lx@crt@counter", sub[_args] {
    let counter = lookup_string("current_counter");
    ExplodeText!(if counter.is_empty() { "Doc-Start".to_string() } else { counter })
  });
  // A label's `.lla` line, digested where the label is (as text), kept for the list; a label set twice (material
  // digested again) is listed once.
  DefPrimitive!("\\lx@crt@lla Semiverbatim {}", sub[(label, line)] {
    let seen = s!("crt_lla_seen:{}", label.to_string());
    if lookup_bool(&seen) {
      return Ok(Vec::new());
    }
    assign_value(&seen, true, Some(Scope::Global));
    begin_mode("text")?;
    let digested = digest(line);
    end_mode("text")?;
    push_value("crt_lla", digested?)?;
  });
  // The line's number, then the label's name as written (its `_` or `^` are the name's characters).
  DefConstructor!("\\lx@crt@entry Semiverbatim {}",
  "<ltx:tocentry><ltx:ref labelref='#label'>#2 #1</ltx:ref></ltx:tocentry>",
  properties => sub[args] {
    let label = args[0].as_ref().map(|a| a.to_string()).unwrap_or_default();
    Ok(stored_map!("label" => clean_label(&label, None).into_owned()))
  });
  DefConstructor!("\\@crt@listofl@bels", sub[document, _args, _props] {
    // The `.lla` file is read once (crossreftools.sty:408-421: `\@input`, then `\openout` truncates it).
    let lines: Vec<Digested> = match remove_value("crt_lla") {
      Some(Stored::VecDequeStored(lines)) => {
        lines.into_iter().filter_map(|line| line.into()).collect()
      },
      _ => Vec::new(),
    };
    let mut attrs: HashMap<String, String> = HashMap::default();
    attrs.insert("class".into(), "ltx_listoflabels".into());
    document.open_element("ltx:TOC", Some(attrs), None)?;
    document.open_element("ltx:toclist", None, None)?;
    for line in &lines {
      document.absorb(line, None)?;
    }
    document.close_element("ltx:toclist")?;
    document.close_element("ltx:TOC")?;
  });
});
