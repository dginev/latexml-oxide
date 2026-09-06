//! algpseudocodex.sty — LaTeXML binding for algpseudocodex package.
//!
//! algpseudocodex builds on `algorithmicx` / `algpseudocode` and provides pseudocode
//! typesetting with customizable indentation guides, single-line and multiline comments,
//! and boxed code blocks.
//!
//! On `perfect_kernel`, raw loading already renders `\Comment` on one line (`float:right`)
//! with 0 Fatal; the binding provides full semantic support for `\LComment`, `\BeginBox`,
//! `\EndBox`, and `\BoxedString` (which raise undefined control sequence errors under raw loading)
//! and implements clean in-flow box wrappers.
//!
//! This binding (beyond-Perl surpass; see docs/parity/OXIDIZED_DESIGN_DIVERGENCES.md #214):
//! 1. Loads `algpseudocode` (which loads `algorithmicx`).
//! 2. Defensively guards against the path where old `algorithmic.sty` was loaded first
//!    and `algorithmicx` bailed (witness: 2410.03000).
//! 3. Honors `italicComments`, `rightComments`, `commentColor` (default `gray`),
//!    `beginComment`, `endComment`, `beginLComment`, `endLComment`, and `noEnd` (default `true`).
//!    TikZ visual-only options (`indLines`, `spaceRequire`) are parsed and safely dropped.
//! 4. Preserves `\Statex` in-line break semantics (`<break/>` within the open line box)
//!    matching raw TeX varwidth-in-open-box execution.
//! 5. Replaces TikZ overlay boxes with in-flow styled wrappers (`ltx:text`)
//!    capturing `draw=<color>`, `dashed`/`dotted`, `thick` options from
//!    `\BeginBox`, `\EndBox`, and `\BoxedString`. Multi-line boxes open only on the
//!    pending->open transition to prevent double-opening.
//! 6. Declares dummy counters and stubs for internal registers (`algpx@codeBoxCount`, etc.).

use latexml_package::prelude::*;
use rustc_hash::FxHashMap;

fn parse_box_options(raw: &str) -> (String, String) {
  let mut classes = vec!["ltx_algpx_boxed".to_string()];
  let mut color = "black".to_string();
  let mut style = "solid".to_string();
  let mut width = "1px".to_string();

  for part in raw.split(',') {
    let part = part.trim();
    if part.is_empty() || part == "algpxDefaultBox" {
      continue;
    }
    if let Some((k, v)) = part.split_once('=') {
      let k = k.trim();
      let v = v.trim();
      if k == "draw" {
        color = v.to_string();
        classes.push(format!("ltx_border_{v}"));
      }
    } else {
      match part {
        "dashed" => {
          style = "dashed".to_string();
          classes.push("ltx_dashed".to_string());
        },
        "dashdotted" => {
          style = "dashed".to_string();
          classes.push("ltx_dashdotted".to_string());
        },
        "dotted" => {
          style = "dotted".to_string();
          classes.push("ltx_dotted".to_string());
        },
        "thick" => {
          width = "2px".to_string();
          classes.push("ltx_thick".to_string());
        },
        "very thick" => {
          width = "3px".to_string();
          classes.push("ltx_very_thick".to_string());
        },
        "thin" => {
          width = "0.5px".to_string();
          classes.push("ltx_thin".to_string());
        },
        other => {
          if !other.is_empty() {
            classes.push(format!("ltx_{other}"));
          }
        },
      }
    }
  }

  let css = format!("border: {width} {style} {color};");
  (classes.join(" "), css)
}

LoadDefinitions!({
  // If old algorithmic.sty was loaded first, algorithmicx bails and algpseudocode
  // is ineffective. Bail cleanly here as well with defensive stubs (witness class 2410.03000).
  if lookup_meaning(&T_CS!("\\c@ALC@line")).is_some()
    || (lookup_meaning(&T_CS!("\\algorithmic")).is_some()
      && lookup_meaning(&T_CS!("\\algrenewcomment")).is_none())
  {
    Warn!(
      "unexpected",
      "\\algorithmic",
      "Another package has already defined \\algorithmic, will not load algpseudocodex.sty"
    );
    def_macro_noop("\\LComment [] {}")?;
    def_macro_noop("\\BeginBox []")?;
    def_macro_noop("\\EndBox")?;
    def_macro_noop("\\BoxedString [] {}")?;
    return Ok(());
  }

  RequirePackage!("algpseudocode");
  RequirePackage!("etoolbox");

  // Package options and conditionals
  // Defaults from algpseudocodex.sty:43-52:
  // noEnd is true, indLines is true, spaceRequire is true,
  // italicComments is true, rightComments is true, commentColor is gray.
  RawTeX!(
    r#"
    \newif\ifalgpx@italicComments \algpx@italicCommentstrue
    \newif\ifalgpx@rightComments  \algpx@rightCommentstrue
    \newif\ifalgpx@indLines       \algpx@indLinestrue
    \newif\ifalgpx@spaceRequire   \algpx@spaceRequiretrue
    \newif\ifalgpx@noEnd          \algpx@noEndtrue

    \def\algpx@commentColor{gray}
    \def\algpx@beginComment{$\triangleright$~}
    \def\algpx@endComment{}
    \def\algpx@beginLComment{$\triangleright$~}
    \def\algpx@endLComment{~$\triangleleft$}
    \def\algpxDefaultBox{}

  "#
  );

  DefKeyVal!("algpseudocodex", "italicComments", "");
  DefKeyVal!("algpseudocodex", "rightComments", "");
  DefKeyVal!("algpseudocodex", "indLines", "");
  DefKeyVal!("algpseudocodex", "spaceRequire", "");
  DefKeyVal!("algpseudocodex", "noEnd", "");
  DefKeyVal!("algpseudocodex", "commentColor", "");
  DefKeyVal!("algpseudocodex", "beginComment", "");
  DefKeyVal!("algpseudocodex", "endComment", "");
  DefKeyVal!("algpseudocodex", "beginLComment", "");
  DefKeyVal!("algpseudocodex", "endLComment", "");
  // The registrations above define the `\KV@algpseudocodex@<key>` macros
  // (keyval_qname), which `ProcessOptions` digests with the option's value —
  // so the REAL handlers (algpseudocodex.sty:43-52's key bodies) must come
  // AFTER them, or every explicit package option is silently a no-op.
  RawTeX!(
    r#"
    \def\KV@algpseudocodex@italicComments#1{\csname algpx@italicComments#1\endcsname}
    \def\KV@algpseudocodex@rightComments#1{\csname algpx@rightComments#1\endcsname}
    \def\KV@algpseudocodex@indLines#1{\csname algpx@indLines#1\endcsname}
    \def\KV@algpseudocodex@spaceRequire#1{\csname algpx@spaceRequire#1\endcsname}
    \def\KV@algpseudocodex@noEnd#1{\csname algpx@noEnd#1\endcsname}
    \def\KV@algpseudocodex@commentColor#1{\def\algpx@commentColor{#1}}
    \def\KV@algpseudocodex@beginComment#1{\def\algpx@beginComment{#1}}
    \def\KV@algpseudocodex@endComment#1{\def\algpx@endComment{#1}}
    \def\KV@algpseudocodex@beginLComment#1{\def\algpx@beginLComment{#1}}
    \def\KV@algpseudocodex@endLComment#1{\def\algpx@endLComment{#1}}
  "#
  );

  DeclareOption!("italicComments", "\\algpx@italicCommentstrue");
  DeclareOption!("rightComments", "\\algpx@rightCommentstrue");
  DeclareOption!("indLines", "\\algpx@indLinestrue");
  DeclareOption!("spaceRequire", "\\algpx@spaceRequiretrue");
  DeclareOption!("noEnd", "\\algpx@noEndtrue");

  ProcessOptions!(keysets => ["algpseudocodex"]);

  RawTeX!(
    r#"
    \ifalgpx@noEnd
      \ifdefined\algtext
        \algtext*{EndWhile}%
        \algtext*{EndFor}%
        \algtext*{EndForAll}%
        \algtext*{EndLoop}%
        \algtext*{EndRepeat}%
        \algtext*{EndIf}%
        \algtext*{EndProcedure}%
        \algtext*{EndFunction}%
        \algtext*{EndStructure}%
        \algtext*{EndClass}%
        \algtext*{EndProperties}%
        \algtext*{EndMethods}%
      \fi
    \fi
  "#
  );

  // Clean comment rendering:
  // Right-flushed comments wrap in `<ltx:text class='ltx_algpx_comment' cssstyle='float:right'>`.
  DefConstructor!(
    "\\lx@algpx@comment{}",
    "<ltx:text class='ltx_algpx_comment' cssstyle='float:right'>#1</ltx:text>"
  );

  RawTeX!(
    r#"
    \newcommand{\algpx@commentFormat}[1]{%
      \@ifundefined{textcolor}{%
        \ifalgpx@italicComments\textit{#1}\else#1\fi
      }{%
        \expandafter\ifx\csname algpx@commentColor\endcsname\@empty
          \ifalgpx@italicComments\textit{#1}\else#1\fi
        \else
          \ifalgpx@italicComments\textit{\textcolor{\algpx@commentColor}{#1}}\else\textcolor{\algpx@commentColor}{#1}\fi
        \fi
      }%
    }
    \newcommand{\algpx@commentString}[1]{%
      \algpx@commentFormat{\algpx@beginComment#1\algpx@endComment}%
    }

    \ifdefined\algrenewcomment
      \algrenewcomment[1]{%
        \ifalgpx@rightComments
          \lx@algpx@comment{\algpx@commentString{#1}}%
        \else
          \algpx@commentString{#1}%
        \fi
      }
    \else
      \providecommand{\Comment}[1]{%
        \ifalgpx@rightComments
          \lx@algpx@comment{\algpx@commentString{#1}}%
        \else
          \algpx@commentString{#1}%
        \fi
      }
    \fi

    \ifdefined\algdef
      \algdef{SL}[LCOMMENT]{LComment}{0}[1]{%
        \algpx@commentFormat{\algpx@beginLComment#1\algpx@endLComment}%
      }
    \else
      \providecommand{\LComment}[1]{%
        \algpx@commentFormat{\algpx@beginLComment#1\algpx@endLComment}%
      }
    \fi
  "#
  );

  // Code command line box:
  // In raw algpseudocodex.sty:185, \algpx@startCodeCommand opens a varwidth box
  // and \algpx@endCodeCommand closes it. \Statex has no hook, so its text sits
  // inside the open box, causing \lx@algorithmicx@@item to insert <ltx:break/>
  // rather than a nested <ltx:listingline> (preserved by statex_continues_the_open_line_box).
  // Marking with _noautoclose="1" prevents maybe_close_element from prematurely
  // auto-closing the line box when \Statex runs.
  DefConstructor!("\\algpx@startCodeCommand", sub [document] {
    if lookup_value("algpx@code_open").is_some_and(|s| matches!(s, Stored::Bool(true))) {
      let _ = document.maybe_close_element("ltx:text");
      assign_value("algpx@code_open", Stored::Bool(false), Some(Scope::Global));
    }
    let mut attrs = FxHashMap::default();
    attrs.insert("class".into(), "ltx_algpx_code".into());
    attrs.insert("_noautoclose".into(), "1".into());
    document.open_element("ltx:text", Some(attrs), None)?;
    assign_value("algpx@code_open", Stored::Bool(true), Some(Scope::Global));
  });

  DefConstructor!("\\algpx@endCodeCommand []", sub [document] {
    if lookup_value("algpx@code_open").is_some_and(|s| matches!(s, Stored::Bool(true))) {
      let _ = document.maybe_close_element("ltx:text");
      assign_value("algpx@code_open", Stored::Bool(false), Some(Scope::Global));
    }
  });

  // In-flow boxes:
  // \BeginBox[opts] stores pending box options.
  // The next \item opens `<ltx:text class='ltx_framed ...' cssstyle='...'>`.
  // Multi-line boxes only open on the pending->open transition to prevent double-opening.
  // \EndBox closes the `<ltx:text>`.
  DefConstructor!("\\BeginBox []", sub [_document, args] {
    let raw = args
      .first()
      .and_then(|a| a.as_ref())
      .map(|d| d.to_string())
      .unwrap_or_else(|| "algpxDefaultBox".to_string());
    let raw_str = if raw.is_empty() { "algpxDefaultBox" } else { &raw };
    assign_value("algpx@pending_box", Stored::String(pin(raw_str)), Some(Scope::Global));
  });

  DefConstructor!("\\algpx@check@box", sub [document] {
    let pending = lookup_value("algpx@pending_box");
    if let Some(Stored::String(p)) = pending && !to_string(p).is_empty() {
      let spec = to_string(p);
      assign_value("algpx@open_box", Stored::String(p), Some(Scope::Global));
      assign_value("algpx@pending_box", Stored::String(pin("")), Some(Scope::Global));
      let (classes, cssstyle) = parse_box_options(&spec);
      let mut attrs = FxHashMap::default();
      attrs.insert("class".into(), format!("ltx_framed {classes}"));
      attrs.insert("cssstyle".into(), cssstyle);
      document.open_element("ltx:text", Some(attrs), None)?;
      assign_value("algpx@box_is_open", Stored::Bool(true), Some(Scope::Global));
    }
  });

  DefConstructor!("\\EndBox", sub [document] {
    // If an inner code line is currently open, close it before closing the box.
    if lookup_value("algpx@code_open").is_some_and(|s| matches!(s, Stored::Bool(true))) {
      let _ = document.maybe_close_element("ltx:text");
      assign_value("algpx@code_open", Stored::Bool(false), Some(Scope::Global));
    }
    let is_open = lookup_value("algpx@box_is_open").is_some_and(|s| matches!(s, Stored::Bool(true)));
    if is_open {
      let _ = document.maybe_close_element("ltx:text");
      assign_value("algpx@box_is_open", Stored::Bool(false), Some(Scope::Global));
    }
    assign_value("algpx@open_box", Stored::String(pin("")), Some(Scope::Global));
    assign_value("algpx@pending_box", Stored::String(pin("")), Some(Scope::Global));
  });

  DefConstructor!(
    "\\BoxedString [] {}",
    "<ltx:text class='ltx_framed #classes' cssstyle='#cssstyle'>#2</ltx:text>",
    properties => sub [args] {
      let raw = args
        .first()
        .and_then(|a| a.as_ref())
        .map(|d| d.to_string())
        .unwrap_or_else(|| "algpxDefaultBox".to_string());
      let (classes, cssstyle) = parse_box_options(&raw);
      Ok(stored_map!("classes" => classes, "cssstyle" => cssstyle))
    }
  );

  // Hook \lx@algorithmicx@item to trigger box checking at the start of each line
  DefMacro!(
    "\\lx@algorithmicx@item[]",
    "\\@ifnextchar\\nointerlineskip{}{\\lx@algpx@item}"
  );
  DefMacro!(
    "\\lx@algpx@item",
    "\\lx@algorithmicx@@item\\algpx@check@box"
  );

  // Hook start and end code commands to State and control blocks
  RawTeX!(
    r#"
    \ifdefined\pretocmd
      \pretocmd{\State}{\algpx@endCodeCommand}{}{}
      \pretocmd{\While}{\algpx@endCodeCommand}{}{}
      \pretocmd{\For}{\algpx@endCodeCommand}{}{}
      \pretocmd{\ForAll}{\algpx@endCodeCommand}{}{}
      \pretocmd{\Loop}{\algpx@endCodeCommand}{}{}
      \pretocmd{\Repeat}{\algpx@endCodeCommand}{}{}
      \pretocmd{\Until}{\algpx@endCodeCommand}{}{}
      \pretocmd{\If}{\algpx@endCodeCommand}{}{}
      \pretocmd{\ElsIf}{\algpx@endCodeCommand}{}{}
      \pretocmd{\Else}{\algpx@endCodeCommand}{}{}
      \pretocmd{\Procedure}{\algpx@endCodeCommand}{}{}
      \pretocmd{\Function}{\algpx@endCodeCommand}{}{}
      \pretocmd{\Structure}{\algpx@endCodeCommand}{}{}
      \pretocmd{\Class}{\algpx@endCodeCommand}{}{}
      \pretocmd{\Properties}{\algpx@endCodeCommand}{}{}
      \pretocmd{\Methods}{\algpx@endCodeCommand}{}{}
      \pretocmd{\EndWhile}{\algpx@endCodeCommand}{}{}
      \pretocmd{\EndFor}{\algpx@endCodeCommand}{}{}
      \pretocmd{\EndLoop}{\algpx@endCodeCommand}{}{}
      \pretocmd{\EndIf}{\algpx@endCodeCommand}{}{}
      \pretocmd{\EndProcedure}{\algpx@endCodeCommand}{}{}
      \pretocmd{\EndFunction}{\algpx@endCodeCommand}{}{}
      \pretocmd{\EndStructure}{\algpx@endCodeCommand}{}{}
      \pretocmd{\EndClass}{\algpx@endCodeCommand}{}{}
      \pretocmd{\EndProperties}{\algpx@endCodeCommand}{}{}
      \pretocmd{\EndMethods}{\algpx@endCodeCommand}{}{}
      \pretocmd{\LComment}{\algpx@endCodeCommand}{}{}
      \pretocmd{\endalgorithmic}{\algpx@endCodeCommand}{}{}
    \fi
    \let\lx@algpx@orig@endalgorithmic\endalgorithmic
    \def\endalgorithmic{\algpx@endCodeCommand\lx@algpx@orig@endalgorithmic}
    \ifdefined\AtEndEnvironment
      \AtEndEnvironment{algorithmic}{\algpx@endCodeCommand}
    \fi
    \ifdefined\apptocmd
      \apptocmd{\State}{\algpx@startCodeCommand}{}{}
    \fi
  "#
  );

  // Dummy counters, lengths, and stubs for internal algpseudocodex macros
  RawTeX!(
    r#"
    \newcounter{algpx@codeBoxCount}
    \newcounter{algpx@nestedBoxedStringCount}
    \newcounter{algpx@nestedBoxedStringMaxCount}
    \newcounter{algpx@startedBoxesCount}
    \newcounter{algpx@endedBoxesCount}
    \newcounter{algpx@pageCount}
    \newcounter{algpx@tmpCount}

    \newlength{\algpx@codeBoxInnerSep}
    \newlength{\algpx@codeBoxOuterSep}
    \newlength{\algpx@codeBoxSep}
    \newlength{\algpx@tmpLen}
    \newlength{\algpx@indShiftX}

    \def\algpx@drawCodeBox#1#2#3#4#5{}
    \def\algpx@setCodeBoxNorth#1{}
    \def\algpx@setCodeBoxSouth#1{}
    \def\algpx@setCodeBoxEast{}
    \def\algpx@setCodeBoxWest{}
    \def\algpx@addBoxSpacing#1#2#3#4{}
    \def\algpx@setBoxesToStoredMax{}
    \def\algpx@startIndent{}
    \def\algpx@endIndent{}
    \def\algpx@startEndBlockCommand{}
    \def\algpx@checkPageBreak{}
    \def\algpx@drawIndentLine#1#2{}
    \@ifundefined{tikzmark}{\def\tikzmark#1{}}{}
  "#
  );
});
