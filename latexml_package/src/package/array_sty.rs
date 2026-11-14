use std::collections::VecDeque;

use crate::prelude::*;

/// A `w{align}{width}` / `W{align}{width}` column (array.sty:463-474): the cell in the alignment's own mode, its
/// width and alignment on the cell.
fn add_width_column(align: &str, width: Dimension) {
  let alignment = match align {
    "l" => Align::Left,
    "r" => Align::Right,
    _ => Align::Center,
  };
  with_building_template(|template| {
    template.add_column(Cell {
      align: Some(alignment),
      width: Some(width),
      ..Cell::default()
    });
  });
}

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: array.sty.ltxml

  // Perl L18: DefRegister('\extrarowheight' => Dimension('0pt'));
  DefRegister!("\\extrarowheight" => Dimension::new(0));

  // Not sure how to effect these
  DefMacro!("\\firsthline", "\\hline");
  DefMacro!("\\lasthline", "\\hline");

  DefColumnType!(">{}",  sub[(before)] {
    with_building_template(|template| {
      template.add_before_column(VecDeque::from(before.unlist()));
    });
  });
  DefColumnType!("<{}", sub[(after)] {
    with_building_template(|template| {
      template.add_after_column(after.unlist());
    });
  });

  // Same as p{} but vertically centered. Build the cell as the Perl-faithful
  // `\lx@tabular@p m {width} { … }` VBox (an `<ltx:inline-block>` digested as
  // VBoxContents), matching the global `p{}` column (tex_tables.rs) — so width
  // lands on the `<inline-block>` (not the `<td>`), block content nests directly,
  // and `\hsize`-aware wrapping sizes it (box-model fix). The vattach letter `m`
  // → `translate_attachment` → middle. (Was a plain `\vtop{}` → width-on-`<td>` +
  // `vattach`/width drift vs Perl — cluster-B residual C/D.)
  DefColumnType!("m{}", sub[(width)] {
    let mut before = vec![T_CS!("\\lx@tabular@p"), T_LETTER!("m"), T_BEGIN!()];
    before.extend(width.unlist());
    before.push(T_END!());
    before.push(T_BEGIN!());
    with_building_template(|template| {
      template.add_column(Cell {
        before: Some(Tokens::new(before)),
        after: Some(Tokens!(T_END!())),
        align: Some(Align::Justify),
        vattach: Some(String::from("middle")),
        ..Cell::default()
      });
    });
  });
  // Same as p{} but vertically bottom-aligned — `\lx@tabular@p b {width} { … }`.
  DefColumnType!("b{}", sub[(width)] {
    let mut before = vec![T_CS!("\\lx@tabular@p"), T_LETTER!("b"), T_BEGIN!()];
    before.extend(width.unlist());
    before.push(T_END!());
    before.push(T_BEGIN!());
    with_building_template(|template| {
      template.add_column(Cell {
        before: Some(Tokens::new(before)),
        after: Some(Tokens!(T_END!())),
        align: Some(Align::Justify),
        vattach: Some(String::from("bottom")),
        ..Cell::default()
      });
    });
  });

  // Like @{}, but should NOT suppress intercolumn space
  DefColumnType!("!{}", sub[(filler)] {
    with_building_template(|template| {
      template.add_between_column(filler.unlist());
    });
  });

  // w and W columns: specified alignment and width. array.sty:463-474 sets the cell as a `c` column does, in the
  // array's own mode (`\d@llarbegin`: `$` in an array, a group in a tabular), into `\ar@cellbox`, then
  // `\makebox[#2][#1]` (`W` lets `\hss` be `\hfil` first, so wide content overflows to the right): one line, never a
  // paragraph. Perl array.sty.ltxml:50-63 wraps the cell in `\vtop{…}`, internal vertical mode, so a math array's
  // `w` cell left math and its scripts were "can only appear in math mode" (KNOWN_PERL_ERRORS #660; 2407.14297
  // `bNiceArray{w{c}{0.75cm}…}`). The cell keeps the alignment's mode; its width and alignment ride on the cell.
  // Guard: `perfect_kernel_batch66l::array_w_column_keeps_math_mode`.
  DefColumnType!("w{}{Dimension}", sub[(align_arg, width)] {
    add_width_column(&align_arg.to_string(), width);
  });
  DefColumnType!("W{}{Dimension}", sub[(align_arg, width)] {
    add_width_column(&align_arg.to_string(), width);
  });

  // \newcolumntype — define new column types
  // Perl L68-71:
  //   DefPrimitive('\newcolumntype{}[Number][]{}', sub {
  //     my ($stomach, $char, $nargs, $opt, $replacement) = @_;
  //     DefMacroI(T_CS('\NC@rewrite@' . ToString($char)), convertLaTeXArgs($nargs, $opt), $replacement);
  //     return AddToPreamble(T_CS('\newcolumntype'), $char, $nargs, $opt, $replacement); });
  DefPrimitive!("\\newcolumntype{}[Number][]{}", sub[(ch, nargs, opt, replacement)] {
    let ch_str = ch.to_string();
    let nargs_val = nargs.value_of() as usize;
    let opt_clone = opt.clone();
    let replacement_toks: Tokens = replacement.revert().into();
    // Define \NC@rewrite@<char> as a macro with the replacement
    let cs_name = s!("\\NC@rewrite@{ch_str}");
    let cs_args = convert_latex_args(nargs_val, opt_clone)?;
    def_macro(T_CS!(cs_name), cs_args, Some(ExpansionBody::from(replacement_toks.clone())), None)?;
    // AddToPreamble: record as <?latexml preamble="\newcolumntype{C}[nargs][opt]{...}"?>
    // Perl L71 passes $nargs and $opt through verbatim — we must preserve
    // both so a rebuilt definition like `\newcolumntype{C}[1][c]{...}` round-trips.
    let mut pi_tokens = vec![T_CS!("\\lx@add@Preamble@PI")];
    pi_tokens.push(T_BEGIN!()); // start of Undigested arg
    pi_tokens.push(T_CS!("\\newcolumntype"));
    pi_tokens.push(T_BEGIN!());
    pi_tokens.extend(ExplodeText!(ch_str));
    pi_tokens.push(T_END!());
    if nargs_val > 0 {
      pi_tokens.push(T_OTHER!("["));
      pi_tokens.extend(ExplodeText!(s!("{nargs_val}")));
      pi_tokens.push(T_OTHER!("]"));
      // Perl L71: $opt is forwarded too. When present it's the default value
      // of the first argument — emit as `[<opt>]` so the preamble round-trips.
      if let Some(opt_tks) = opt {
        pi_tokens.push(T_OTHER!("["));
        pi_tokens.extend(opt_tks.unlist());
        pi_tokens.push(T_OTHER!("]"));
      }
    }
    pi_tokens.push(T_BEGIN!());
    pi_tokens.extend(replacement_toks.unlist());
    pi_tokens.push(T_END!());
    pi_tokens.push(T_END!()); // end of Undigested arg
    unread(Tokens::new(pi_tokens));
    // The gullet will read \lx@add@Preamble@PI{...} and the stomach will digest it
  });

  DefMacro!("\\arraybackslash", r"\let\\\tabularnewline");

  // array.sty's registers and preamble internals, which raw-loaded packages and classes build their
  // own alignments with (our tables do not run array's preamble builder): :121-122 `\col@sep` (the
  // column separation `\array`/`\tabular` set from `\arraycolsep`/`\tabcolsep`, :281/:302) and
  // `\@acol` (adds it to the preamble); :370 `\NC@list`, array's list of `\newcolumntype` letters
  // (`\NC@do<letter>`, which raw code reads and extends; ours keeps it empty: our column types do not
  // run array's `\NC@find` rewriting, :336-362); :430-431 `\extratabsurround` (2pt, the space
  // `\firsthline`/`\lasthline` add). Callers: eqnarray.sty:38 `{equationarray}`, ltxutil.sty:1122-1244
  // (revtex's alignment code, raw-loaded by revtex-derived classes), dgruyter_NEW.sty:1480 `\@array`.
  // Run 340: `\extratabsurround` 88, `\col@sep` 56, `\NC@list` 11, `\@acol` 8 first errors
  // (witnesses 2508.08881, 2103.15196, 2103.08263, 2407.04867, 2004.09405, 2401.02879, 2006.15794,
  // 2006.04593). Those callers then reach the rest of array's preamble builder, below.
  RawTeX!(r"\newdimen\col@sep
\def\@acol{\@addtopreamble{\hskip\col@sep}}
\newtoks\NC@list
\newlength{\extratabsurround}\setlength{\extratabsurround}{2pt}");
  // array.sty's preamble builder (v2.6n, 2025/09/25), as the package defines it, for raw code that
  // builds its own alignment preamble with it: :54 `\@addtopreamble` (global, where the kernel's is a
  // local `\edef`), :56-81 `\@testpach`, :94-200 the token registers,
  // `\insert@column`, `\@addamp` and the column classes, :266 `\do@row@strut`, :279-280
  // `\d@llarbegin`/`\d@llarend`, :382-423 `\save@decl`, `\@mkpream`, `\@classviii`, `\@classv`. Our
  // `{tabular}`/`{array}` read their templates themselves and never run it; the kernel's `\@mkpream`
  // (from the format) calls `\@classiv`/`\@classz`, which latex.ltx binds only inside `\@tabular` and
  // `\array` (:16550/:16561), so a raw call outside one was "undefined `\@classiv`". polytable.sty
  // (lhs2TeX's code blocks) builds a one-column preamble (:557, :588 `\@mkpream{@{}>{\hspre}l<{\hspost}@{}}`)
  // and splits it at `\@sharp` into the material before and after each cell (:591-607): every code
  // block was "Illegal character in array arg" and polytable's "Illegal preamble (no columns)", the
  // lines lost and a Fatal at the end of the file (2307.11194, 1705.09704; pdflatex 0 errors). Not
  // here: array's `\newcolumntype` rewriting (`\NC@find`, :336-367: `\NC@list` stays empty, so a raw
  // `\@mkpream` sees only the built-in column letters), `\array`/`\@tabular`/`\@array`/`\multicolumn`
  // (ours), the luamml sockets. KNOWN_PERL_ERRORS #1152; guard
  // `perfect_kernel_batch79p::polytable_code_block_keeps_its_lines`.
  RawTeX!(r"\def\@addtopreamble#1{\xdef\@preamble{\@preamble #1}}
\def\@testpach{\@chclass
 \ifnum \@lastchclass=6 \@ne \@chnum \@ne \else
  \ifnum \@lastchclass=7 5 \else
   \ifnum \@lastchclass=8 \tw@ \else
    \ifnum \@lastchclass=9 \thr@@
   \else \z@
   \ifnum \@lastchclass = 10 \else
   \edef\@nextchar{\expandafter\string\@nextchar}%
   \@chnum
   \if \@nextchar c\z@ \else
    \if \@nextchar l\@ne \else
     \if \@nextchar r\tw@ \else
   \z@ \@chclass
   \if\@nextchar |\@ne \else
    \if \@nextchar !6 \else
     \if \@nextchar @7 \else
      \if \@nextchar <8 \else
       \if \@nextchar >9 \else
  10
  \@chnum
  \if \@nextchar m\thr@@ \else
   \if \@nextchar p4 \else
    \if \@nextchar b5 \else
   \z@ \@chclass \z@ \@preamerr \z@ \fi \fi \fi \fi
   \fi \fi  \fi  \fi  \fi  \fi  \fi \fi \fi \fi \fi \fi}
\ExplSyntaxOn
\def\prepnext@tok{\advance \count@ \@ne
   \toks\count@{}}
\def\insert@column{%
  \UseTaggingSocket{tbl/cell/begin}%
   \@protected@firstofone { \the@toks \the \@tempcnta \ignorespaces }
   \@sharp \textonly@unskip
   \the@toks \the \count@ \relax
   \UseTaggingSocket{tbl/cell/end}%
}
\protected\def\textonly@unskip{\ifhmode\unskip\fi}
\protected\long\def\@protected@firstofone#1{#1}
\def\insert@pcolumn{%
   \UseTaggingSocket{tbl/pcell/begin}%
   \the@toks \the \@tempcnta \relax
   \ignorespaces \@sharp \unskip
   \the@toks \the \count@ \relax
   \UseTaggingSocket{tbl/pcell/end}%
}
\def\@addamp {
  \if@firstamp
    \@firstampfalse
  \else
    \edef\@preamble{\@preamble &
      \noexpand\tbl_update_cell_data: }
  \fi
}
\def\@classx{%
  \ifcase \@lastchclass
  \@acolampacol \or
  \@addamp \@acol \or
  \@acolampacol \or
  \or
  \@acol \@firstampfalse \or
  \@addamp
  \fi}
\def\@classz{\@classx
   \@tempcnta \count@
   \prepnext@tok
   \@addtopreamble{\ifcase \@chnum
      \hfil
      \hskip1sp%
      \d@llarbegin
      \insert@column
      \d@llarend
      \do@row@strut \hfil \or
      \hskip1sp\d@llarbegin\insert@column\d@llarend
      \do@row@strut \hfil \or
      \hfil\hskip1sp\d@llarbegin\insert@column\d@llarend
      \do@row@strut \or
    \setbox\ar@mcellbox\vbox
    \@startpbox{\@nextchar}\insert@pcolumn \@endpbox
    \ar@align@mcell
    \do@row@strut \or
   \vtop \@startpbox{\@nextchar}\insert@pcolumn \@endpbox\do@row@strut \or
   \vbox \@startpbox{\@nextchar}\insert@pcolumn \@endpbox\do@row@strut
  \fi}\prepnext@tok}
\newbox\ar@mcellbox
\def\ar@align@mcell{%
  \ifdim \ht\ar@mcellbox > \ht\strutbox
    \begingroup
     \dimen@\ht\ar@mcellbox
     \advance\dimen@-\ht\@arstrutbox
     \advance\dimen@\baselineskip
     \lower.5\dimen@\box\ar@mcellbox
    \endgroup
  \else
    \box\ar@mcellbox
  \fi}
\def\@arrayrule{\@addtopreamble \vline}
\def\@classvii{\ifnum \@lastchclass = \thr@@
   \@preamerr \thr@@ \fi}
\def\@classvi{\ifcase \@lastchclass
      \@acol \or
      \@addtopreamble{\hskip \doublerulesep}\or
      \@acol \or
      \@classvii
      \fi}
\def\@classii{\advance \count@ \m@ne
   \save@decl\prepnext@tok}
\def\@classi{\@classvi
   \ifcase \@chnum \@arrayrule \or
      \@classv \fi}
\def\@startpbox#1{\bgroup
  \color@begingroup
  \setlength\hsize{#1}\@arrayparboxrestore
   \everypar{%
      \vrule \@height \ht\@arstrutbox \@width \z@
      \everypar{}}%
   }
\let\@startpbox@action\@startpbox
\def\@endpbox{\@finalstrut\@arstrutbox \par \color@endgroup \egroup\hfil}
\let\do@row@strut\relax
\let\d@llarbegin\begingroup
\let\d@llarend\endgroup
\def\save@decl{\toks \count@ = \expandafter\expandafter\expandafter
                  {\expandafter\@nextchar\the\toks\count@}}
\ExplSyntaxOff
\def\@mkpream#1{\gdef\@preamble{}\@lastchclass 4 \@firstamptrue
   \let\@sharp\relax
   \def\@startpbox##1{\unexpanded\expandafter{\expandafter
                      \@startpbox@action\expandafter{##1}}}%
   \def\@startpbox@action##1{\unexpanded{\@startpbox@action{##1}}}%
   \let\@endpbox\relax
   \let\do@row@strut\relax
   \let\ar@align@mcell\relax
   \@temptokena{#1}\@tempswatrue
   \@whilesw\if@tempswa\fi{\@tempswafalse\the\NC@list}%
   \count@\m@ne
   \let\the@toks\relax
   \prepnext@tok
   \expandafter \@tfor \expandafter \@nextchar
    \expandafter :\expandafter =\the\@temptokena \do
   {\@testpach
   \ifcase \@chclass \@classz \or \@classi \or \@classii
     \or \save@decl \or \or \@classv \or \@classvi
     \or \@classvii \or \@classviii
     \or \@classx
     \or \@classx \fi
   \@lastchclass\@chclass}%
   \ifcase\@lastchclass
   \@acol \or
   \or
   \@acol \or
   \@preamerr \thr@@ \or
   \@preamerr \tw@ \@addtopreamble\@sharp \or
   \or
   \else  \@preamerr \@ne \fi
   \def\the@toks{\the\toks}}
\ExplSyntaxOn
\let\@classix\relax
\def\@classviii{\ifnum \@lastchclass >\z@\ifnum\@lastchclass=\tw@\else
      \@preamerr 4\@chclass 6 \@classvi \fi\fi}
\def\@classv{\save@decl
   \expandafter\NC@ecs\@nextchar\extracolsep{}\extracolsep\@@__tbl
   \@addtopreamble{\d@llarbegin\the@toks\the\count@\relax\d@llarend}%
   \prepnext@tok}
\def\NC@ecs#1\extracolsep#2#3\extracolsep#4\@@__tbl{\def\@tempa{#2}%
  \ifx\@tempa\@empty\else\toks\count@={#1\tabskip#2\relax#3}\fi}
\ExplSyntaxOff");
});
