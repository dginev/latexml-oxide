use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: emulateapj.cls.ltxml — Seems to be equivalent to aastex.
  // Perl `LoadClass('aastex', withoptions => 1)` forwards the current class
  // options; Rust's `load_class_with_options` (latexml_core::binding::content)
  // reads `class_options` from state and passes them through. Previous
  // `load_class("aastex", Vec::new(), ...)` silently dropped the user's
  // `\documentclass[...]{emulateapj}` options before they reached aastex.
  LoadClass!("aastex", withoptions => true);
  RequireResource!("ltx-apj.css");
  RequirePackage!("emulateapj");
  // emulateapj.cls is a revtex4 class of its own (:165-167 `\LoadClass…{\@revtex@cls}`), not an AASTeX version: it has
  // no AASTeX 6 `\fig` family or `\gridline` (which the aastex binding defines for a 6+ class only, aastex_cls.rs), so a
  // paper's own `\fig` is its own (astro-ph/0503342 `\newcommand{\fig}`; KNOWN_PERL_ERRORS #557). The 5.x
  // `\subsubsubsection`/`\supportfrom` stay, the family's union (user 2026-10-10).
  for command in ["\\fig", "\\leftfig", "\\rightfig", "\\boxedfig", "\\rotatefig", "\\gridline"] {
    Let!(T_CS!(command), "\\@undefined");
  }
  // …nor AASTeX 5.x's `{plate}` (emulateapj.cls defines none; aastex_cls.rs `undefine_plate`).
  aastex_cls::undefine_plate();
  // …and unlocked: emulateapj_sty.rs locks the `\fig` it found defined, which a 6+ `aastex.cls` the paper ships leaves
  // (aastex_cls.rs), and a lock on an undefined `\fig` would refuse the paper's own (64f review r3).
  assign_value("\\fig:locked", Stored::None, Some(Scope::Global));
  // emulateapj's deluxetable is a longtable opened at the environment's `\begin` (emulateapj.cls:1430-1460, `{table*}`
  // and `{longtable*}` :1464-1495), not AASTeX's table whose data `\startdata#1\enddata` holds (aastex701.cls:11384,
  // deluxetable_sty.rs): `\tablehead` (:1329-1345) sets its heading rows in the table, the rows after it are rows with
  // or without a `\startdata`, which ends the row in progress and makes the rows since the head the (continuation)
  // heading (:1324 `\kill…\endhead`; 1605.03270, 2002.02307), and `\enddata` only ends the last row (:1497
  // `\def\enddata{\\[\lastrowheight@]…}`), so rows after it are rows too and a second one is harmless. Here the data
  // opens at `\tablehead` or `\startdata`, whichever comes first, and closes at the environment's end (with the rule
  // `\tablehead`'s `\endfoot` sets, :1344-1345), so the `\tablecaption`/`\tablewidth` set before them (stored, :1294,
  // :1359) stay out of it as before. Perl's AASTeX table (deluxetable.sty.ltxml:35-43)
  // left a head's `\\ \cline`, the rows of a table without `\startdata`, and the rows after `\enddata` out of the
  // alignment (1411.3719, 1909.11243, 1002.1848, 1902.00588; 1506.07185, 1705.08932: a table ended without `\enddata`;
  // KNOWN_PERL_ERRORS #665, #874). Guard: `perfect_kernel_batch68y::emulateapj_deluxetable_is_open_from_its_head`.
  // emulateapj's mark is text, `\tablenotemark{#1}` (:1424-1427) and `\footnotetext[#1]` outside a table (:1319), not
  // AASTeX's math (aas_support_sty.rs); in the table the notes are kept for its end (:1424-1427 `\DT@p@ftn`), as the
  // `\tablecomments`/`\tablerefs` lists are (:1436-1437, deluxetable_sty.rs), so a row holding only them (after the
  // last `\enddata`) is an empty row, dropped. A `\label` there is the table's (`\@currentlabel`, the caption's), kept
  // for the end too, so it makes no row either.
  DefMacro!("\\tablenotetext{}{}", "\\lx@deluxe@tablenotetext{#1}{#2}");
  DefMacro!("\\lx@eapj@tablenotetext{}{}", r"\g@addto@macro\tblnote@list{\lx@deluxe@tablenotetext{#1}{#2}}");
  DefMacro!("\\lx@eapj@label{}",
    r"\iflx@deluxetable@data\expandafter\@firstoftwo\else\expandafter\@secondoftwo\fi
{\g@addto@macro\lx@eapj@labels{\lx@eapj@saved@label{#1}}}{\lx@eapj@saved@label{#1}}");
  // The class's `\tablehead`, `\startdata` and `\enddata` are its own, global (:1324, :1329, :1497): a document's
  // redefinition stands (1102.2440's two-argument copy of `\tablehead`), and in a `{longtable}` of the document's own
  // they are its rows too (1810.10130: `\tablehead` is longtable code, :1329-1345), the head ending the row the
  // `\tablecaption` and `\tabletypesize` before it are in (the class's `\kill`).
  RequirePackage!("longtable"); // emulateapj.cls:192 (REVTeX's support has loaded it already)
  RawTeX!(r"\newif\iflx@eapj@deluxetable");
  DefMacro!("\\lx@eapj@begin@deluxetable",
    r"\lx@eapj@deluxetabletrue\global\lx@deluxetable@datafalse\lx@eapj@head@forget
\let\tablenotetext\lx@eapj@tablenotetext\let\lx@eapj@saved@label\label\let\label\lx@eapj@label\gdef\lx@eapj@labels{}");
  DefMacro!("\\lx@eapj@open@data",
    r"\iflx@deluxetable@data\expandafter\@gobble\else\expandafter\@firstofone\fi
{\global\lx@deluxetable@datatrue\bgroup\@deluxetable@bindings\@@deluxetabular\lx@begin@alignment\hline\hline}");
  DefMacro!("\\lx@eapj@head{}", r"\lx@alignment@begin@heading#1\\\hline\lx@alignment@end@heading");
  // The rows the head ends with (`\noalign`, between rows), and the rows after them that a `\startdata` makes the
  // heading: the `\kill…\endhead` of :1324 (see `\startdata` below).
  DefPrimitive!("\\lx@eapj@head@rows", {
    if let Some(alignment) = lookup_alignment()
      && let Some(data) = alignment.alignment_cell() {
        let rows = data.borrow().rows().len();
        assign_value("lx@eapj@head@rows", Stored::Int(rows as i64), Some(Scope::Global));
      }
  });
  DefPrimitive!("\\lx@eapj@head@forget", {
    assign_value("lx@eapj@head@rows", Stored::None, Some(Scope::Global));
  });
  DefPrimitive!("\\lx@eapj@head@extend", {
    if let Some(Stored::Int(head)) = lookup_value("lx@eapj@head@rows") {
      assign_value("lx@eapj@head@rows", Stored::None, Some(Scope::Global));
      if let Some(alignment) = lookup_alignment()
        && let Some(data) = alignment.alignment_cell() {
          let mut al = data.borrow_mut();
          // The rows since the head, without the last: the pseudo-row of this `\noalign`, the current row. They go
          // back in place, so the row indices stay.
          let mut rows = Vec::new();
          while al.rows().len() > head as usize && let Some(row) = al.remove_row() {
            rows.push(row);
          }
          rows.reverse();
          let last = rows.len().saturating_sub(1);
          for row in rows.iter_mut().take(last) {
            // A row with content is a heading row; an empty one (the `\\` after the head) and a rule between them
            // (`\cline`) are on the first page in no TeX table (a continuation head is not), so they are emptied
            // into pseudo-rows without rules, which the alignment's normalization drops.
            let empty = row.is_pseudo()
              || row.get_columns().iter().all(|col| col.boxes.as_ref().is_none_or(Digested::is_skippable));
            for col in row.get_columns_mut() {
              if empty {
                col.border.clear();
                col.boxes = None;
              } else {
                col.thead_in_column = true;
              }
            }
            if empty {
              row.set_pseudo();
            }
          }
          // With heading rows promoted, the head's own rule (`#1\\\hline`, a pseudo-row ending the head) goes under
          // the last of them, above the first data row, where TeX's first page has it (the promoted rows are not on
          // it; 1605.03270 Table 1, OD #663): it moves to the last row, this `\noalign`'s, whose rules the
          // normalization gives the next row.
          let promoted = rows[..last].iter().any(|row| !row.is_pseudo());
          if promoted && let Some(current) = rows.last_mut() {
            let mut rules = Vec::new();
            while al.rows().back().is_some_and(|row| row.is_pseudo()) && let Some(mut rule) = al.remove_row() {
              for (col, target) in rule.get_columns_mut().iter_mut().zip(current.get_columns_mut().iter_mut()) {
                target.border.extend(col.border.chars().filter(|c| matches!(c, 't' | 'T')));
                col.border.retain(|c| !matches!(c, 't' | 'T'));
              }
              rules.push(rule);
            }
            rules.reverse();
            al.append_rows(rules);
          }
          al.append_rows(rows);
        }
    }
  });
  DefMacro!("\\tablehead{}",
    r"\iflx@eapj@deluxetable\expandafter\@firstoftwo\else\expandafter\@secondoftwo\fi
{\lx@eapj@open@data\lx@eapj@head{#1}\noalign{\lx@eapj@head@rows}}
{\ifx\kill\lx@longtable@kill\expandafter\@firstoftwo\else\expandafter\@secondoftwo\fi
  {\crcr\lx@eapj@head{#1}}{\def\@deluxetable@header{\lx@eapj@head{#1}}}}");
  // `\startdata` (:1324) is `\kill\caption{…}\endhead\hline\endfoot` in every table (`\tablehead`'s
  // `\@table@not@headedfalse` is set in the cell its own `\kill` ends, so the switch is back to true): it ends the row
  // in progress, and the rows since the head become the table's (continuation) heading. Here it opens the data of a
  // table without a head, and after a head ends the row in progress — kept, it is the author's second head line
  // (1605.03270 `\tablehead{…}\\ \colhead{} & \colhead{Name} & …` with no `\\`; TeX measures it and prints it on no
  // page) — and makes the rows since the head heading rows, without the empty ones (2002.02307). Guard:
  // `perfect_kernel_batch68y::emulateapj_startdata_ends_the_head`.
  DefMacro!("\\startdata", r"\iflx@eapj@deluxetable\expandafter\lx@eapj@startdata\fi");
  DefMacro!("\\lx@eapj@startdata",
    r"\iflx@deluxetable@data\expandafter\@firstoftwo\else\expandafter\@secondoftwo\fi
{\crcr\noalign{\lx@eapj@head@extend}}{\lx@eapj@open@data}");
  // `\tablecaption` (:1294) is the caption of the table it is in, a `{longtable}`'s too (outside any float stored for the
  // next deluxetable, deluxetable_sty.rs `\lx@deluxe@tablecaption`; 1010.4329): the `\caption` in effect there,
  // not the float's that deluxetable_sty.rs `\let` it be. The class stores it (`\gdef\@tablecaption`) for the head's
  // `\caption` to set; one given in the data (after `\tablehead`, before `\startdata`) is kept for the table's end, as
  // the notes and labels are, so the row it is in stays empty (e2: `\tablehead{…}` `\tablecaption{…\label{t1}}`).
  DefMacro!("\\tablecaption",
    r"\iflx@eapj@deluxetable\iflx@deluxetable@data\expandafter\expandafter\expandafter\lx@eapj@data@caption
\else\expandafter\expandafter\expandafter\caption\fi\else\expandafter\lx@deluxe@tablecaption\fi");
  DefMacro!("\\lx@eapj@data@caption{}", r"\g@addto@macro\lx@eapj@labels{\caption{#1}}");
  DefMacro!("\\enddata", r"\iflx@deluxetable@data\crcr\fi");
  DefMacro!("\\lx@eapj@end@deluxetable",
    r"\iflx@deluxetable@data\expandafter\@firstofone\else\expandafter\@gobble\fi
{\crcr\hline\global\lx@deluxetable@datafalse\lx@end@alignment\@end@deluxetabular\egroup}\lx@eapj@labels
\spew@tblnotes");
  DefMacro!("\\deluxetable{}",
    r"\set@deluxetable@template{#1}\def\@deluxetable@header{}\lx@eapj@begin@deluxetable\begin{table}\lx@deluxe@stored@caption");
  DefMacro!(T_CS!("\\deluxetable*"), "{}",
    r"\set@deluxetable@template{#1}\def\@deluxetable@header{}\lx@eapj@begin@deluxetable\begin{table*}\lx@deluxe@stored@caption");
  DefMacro!("\\enddeluxetable", r"\lx@eapj@end@deluxetable\end{table}");
  DefMacro!(T_CS!("\\enddeluxetable*"), None, r"\lx@eapj@end@deluxetable\end{table*}");
  // emulateapj loads REVTeX 4.1 when it is installed (emulateapj.cls:76-78, :151-167; copies since 11/10/2009, 1601.00007's
  // too), whose `\email`/`\homepage` set the address with `\href`, in the current font (revtex4-1.cls:1959-1961,
  // 2383-2391): roman, as pdflatex prints them (CMR8, also under `[revtex4]`), not AASTeX 5.x's `\tt` or REVTeX 4.0's
  // `\url` that the aastex binding it loads installs. The `_` is kept (user ruling 2026-10-11).
  DefMacro!("\\email[]{}", "\\lx@add@email{\\lx@address@textfont{#2}}");
  DefMacro!("\\homepage[] Semiverbatim", "\\lx@add@url[name={#1}]{\\lx@address@textfont{#2}}");
});
