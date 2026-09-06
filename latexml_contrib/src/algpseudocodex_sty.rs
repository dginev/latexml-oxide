//! algpseudocodex.sty — LaTeXML binding for algpseudocodex package.
//!
//! algpseudocodex builds on `algorithmicx` / `algpseudocode` and provides pseudocode
//! typesetting with customizable indentation guides, single-line and multiline comments,
//! and boxed code blocks.
//!
//! In raw TeX, `algpseudocodex` relies on two-pass TikZ overlay coordinates
//! and complex `\settowidth`/`\tabto`/`varwidth` calculations for comments,
//! which cause comments to split across two lines (line number followed by
//! an empty line) and boxed code blocks to vanish into degenerate SVGs.
//!
//! This binding (beyond-Perl surpass; see docs/parity/OXIDIZED_DESIGN_DIVERGENCES.md):
//! 1. Loads `algpseudocode` (which loads `algorithmicx`).
//! 2. Bypasses the two-pass TikZ / varwidth / tabto comment machinery with
//!    a clean single-line right-flushed `\Comment` (using `<ltx:text class='ltx_algpx_comment' cssstyle='float:right'>`),
//!    and a clean full-width `\LComment`.
//! 3. Honors `italicComments`, `rightComments`, `commentColor`, `beginComment`,
//!    `endComment`, `beginLComment`, `endLComment`, `noEnd`.
//! 4. Replaces TikZ overlay boxes with in-flow styled wrappers (`ltx:text`)
//!    capturing `draw=<color>`, `dashed`/`dotted`, `thick` options from
//!    `\BeginBox`, `\EndBox`, and `\BoxedString`.
//! 5. Declares dummy counters and stubs for internal registers (`algpx@codeBoxCount`, etc.).

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
  RequirePackage!("algpseudocode");

  // Package options and conditionals
  RawTeX!(
    r#"
    \newif\ifalgpx@italicComments \algpx@italicCommentstrue
    \newif\ifalgpx@rightComments  \algpx@rightCommentstrue
    \newif\ifalgpx@indLines       \algpx@indLinestrue
    \newif\ifalgpx@spaceRequire   \algpx@spaceRequiretrue
    \newif\ifalgpx@noEnd          \algpx@noEndfalse

    \def\algpx@commentColor{black}
    \def\algpx@beginComment{$\triangleright$~}
    \def\algpx@endComment{}
    \def\algpx@beginLComment{$\triangleright$~}
    \def\algpx@endLComment{~$\triangleleft$}
    \def\algpxDefaultBox{}

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

  DeclareOption!("italicComments", "\\algpx@italicCommentstrue");
  DeclareOption!("rightComments", "\\algpx@rightCommentstrue");
  DeclareOption!("indLines", "\\algpx@indLinestrue");
  DeclareOption!("spaceRequire", "\\algpx@spaceRequiretrue");
  DeclareOption!("noEnd", "\\algpx@noEndtrue");

  ProcessOptions!(keysets => ["algpseudocodex"]);

  RawTeX!(
    r#"
    \ifalgpx@noEnd
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

    \algrenewcomment[1]{%
      \ifalgpx@rightComments
        \lx@algpx@comment{\algpx@commentString{#1}}%
      \else
        \algpx@commentString{#1}%
      \fi
    }

    \algdef{SL}[LCOMMENT]{LComment}{0}[1]{%
      \algpx@commentFormat{\algpx@beginLComment#1\algpx@endLComment}%
    }
  "#
  );

  // In-flow boxes:
  // \BeginBox[opts] stores pending box options.
  // The next \item opens `<ltx:text class='ltx_framed ...' cssstyle='...'>`.
  // \EndBox closes the `<ltx:text>`.
  DefConstructor!("\\BeginBox []", sub [_document, args, _props] {
    let raw = args
      .first()
      .and_then(|a| a.as_ref())
      .map(|d| d.to_string())
      .unwrap_or_else(|| "algpxDefaultBox".to_string());
    let raw_str = if raw.is_empty() { "algpxDefaultBox" } else { &raw };
    assign_value("algpx@pending_box", Stored::String(pin(raw_str)), Some(Scope::Global));
  });

  DefConstructor!("\\algpx@check@box", sub [document, _args, _props] {
    let pending = lookup_value("algpx@pending_box");
    let open = lookup_value("algpx@open_box");

    let box_spec = match (pending, open) {
      (Some(Stored::String(p)), _) if !to_string(p).is_empty() => {
        let p_str = to_string(p);
        assign_value("algpx@open_box", Stored::String(p), Some(Scope::Global));
        assign_value("algpx@pending_box", Stored::String(pin("")), Some(Scope::Global));
        Some(p_str)
      }
      (_, Some(Stored::String(o))) if !to_string(o).is_empty() => {
        Some(to_string(o))
      }
      _ => None,
    };

    if let Some(spec) = box_spec {
      let (classes, cssstyle) = parse_box_options(&spec);
      let mut attrs = FxHashMap::default();
      attrs.insert("class".into(), format!("ltx_framed {classes}"));
      attrs.insert("cssstyle".into(), cssstyle);
      document.open_element("ltx:text", Some(attrs), None)?;
      assign_value("algpx@box_is_open", Stored::Bool(true), Some(Scope::Global));
    }
  });

  DefConstructor!("\\EndBox", sub [document, _args, _props] {
    let is_open = lookup_value("algpx@box_is_open").is_some_and(|s| matches!(s, Stored::Bool(true)));
    if is_open {
      document.close_element("ltx:text")?;
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
    \def\algpx@startCodeCommand{}
    \def\algpx@endCodeCommand{\@ifnextchar[{\algpx@endCodeCommand@opt}{\algpx@endCodeCommand@noopt}}
    \def\algpx@endCodeCommand@opt[#1]{}
    \def\algpx@endCodeCommand@noopt{}
    \def\algpx@startIndent{}
    \def\algpx@endIndent{}
    \def\algpx@startEndBlockCommand{}
    \def\algpx@checkPageBreak{}
    \def\algpx@drawIndentLine#1#2{}
    \@ifundefined{tikzmark}{\def\tikzmark#1{}}{}
  "#
  );
});
