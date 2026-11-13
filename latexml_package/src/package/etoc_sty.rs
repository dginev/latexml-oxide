use crate::prelude::*;

/// The sectional units a local table of contents lists the children of (etoc.sty:813-850 `\Etoc@localtableofcontents`).
const UNITS: &str = "ancestor-or-self::*[self::ltx:part or self::ltx:chapter or self::ltx:section or \
                     self::ltx:subsection or self::ltx:subsubsection or self::ltx:paragraph or \
                     self::ltx:subparagraph or self::ltx:appendix][@xml:id][1]";

#[rustfmt::skip]
LoadDefinitions!({
  // No Perl binding. etoc typesets its tables from the `.toc` file it writes; the raw package runs clean, and its
  // `\tableofcontents`/`\localtableofcontents` (etoc.sty:896-919) become the kernel's TOC element, filled in by
  // post-processing. `\etocsetnexttocdepth` (:2054) is etoc's own, read here as `tocdepth`; at `\Etoc@minf` or less a
  // table prints nothing (:957-960, `\invisibletableofcontents`). Issue #760.
  InputDefinitions!("etoc", noltxml => true, extension => Some(Cow::Borrowed("sty")));
  RawTeX!(r"\def\etoctableofcontents{\Etoc@tocdepthset\@ifstar\lx@etoc@toc\lx@etoc@toc}
\def\lx@etoc@toc{\ifnum\c@tocdepth>\Etoc@minf\ifEtoc@classstyle\else\lx@etoc@style@lost\fi
  \lx@tableofcontents\fi\Etoc@tocdepthreset}
\let\tableofcontents\etoctableofcontents
\def\localtableofcontents{\Etoc@tocdepthset\@ifstar\lx@etoc@localtoc\lx@etoc@localtoc}
\def\lx@etoc@localtoc{\ifnum\c@tocdepth>\Etoc@minf\ifEtoc@classstyle\else\ifEtoc@etocstyle\else
  \lx@etoc@style@lost\fi\fi\par\lx@etoc@localtoc@element{\contentsname}\fi\Etoc@tocdepthreset}");
  // The table is titled `\contentsname`; a user style's <before> (`\etocsettocstyle`, typeset unless a local table
  // keeps etoc's style: etoc.sty:946-955) is not typeset, its heading lost: counted until it is (RED
  // `etoc_settocstyle_heading_red.tex`).
  DefPrimitive!("\\lx@etoc@style@lost", {
    let before = match lookup_definition(&T_CS!("\\Etoc@tableofcontents@user@before"))? {
      Some(defn) => match defn.get_expansion() {
        Some(ExpansionBody::Tokens(body)) => body.to_string(),
        _ => String::new(),
      },
      None => String::new(),
    };
    if !before.trim().is_empty() {
      Warn!("unexpected", "\\etocsettocstyle",
        s!("The table of contents' style is not typeset; its heading is lost: {}", before.trim()));
    }
  });

  // The local table lists the enclosing unit's children: its `scope` is that unit's id (the schema's "otherwise an
  // xml:id", crossref.rs `fill_in_tocs`), the whole document's outside any unit.
  DefConstructor!("\\lx@etoc@localtoc@element{}", sub[document, args, props] {
      let node = document.get_node().clone();
      let scope = document.findnodes(UNITS, Some(&node)).into_iter().next()
        .and_then(|unit| unit.get_attribute_ns("id", XML_NS))
        .unwrap_or_else(|| "current".to_string());
      let attrs = string_map!("lists" => "toc".to_string(), "scope" => scope,
        "select" => prop_string!(props, "select"));
      document.open_element("ltx:TOC", Some(attrs), None)?;
      if let Some(title) = args[0].as_ref() {
        document.insert_element("ltx:title", vec![title], None)?;
      }
      document.close_element("ltx:TOC")?;
    },
    properties => sub[_args] { Ok(stored_map!("select" => toc_select()?)) }
  );
});
