//! tcolorbox.sty — colored and framed text boxes
//! Perl: tcolorbox.sty.ltxml
use latexml_core::keyval::split_keyval_source;

use crate::{
  package::listings_sty::{listings_read_raw_lines, lst_process_display, lst_run_body_via_input},
  prelude::*,
};

#[rustfmt::skip]
LoadDefinitions!({
  // used in tcbbreakable.code.tex assuming it was defined
  DefRegister!("\\doublecol@number" => Number::new(0));
  // Ensure only unbreakable mode is possible.
  // Perl: locked => 1 prevents raw TeX tcbbreakable.code.tex from overriding
  // with the real breakable implementation (uses output routines → infinite loop).
  DefMacro!("\\tcb@init@breakable", "\\tcb@init@unbreakable", locked => true);

  // Perl 93f875a6: pre-define \tcb@use@autoparskip before raw TeX loading,
  // as pgfkeys initialization may not complete and the \AtBeginDocument hook
  // at tcolorbox.sty:1142 would call it undefined.
  DefMacro!("\\tcb@use@autoparskip", "\\relax");

  RequirePackage!("expl3");
  RequirePackage!("xparse");

  InputDefinitions!("tcolorbox", noltxml => true, extension => Some(Cow::Borrowed("sty")));

  // Suppress tcolorbox's library version check. The Rust kpathsea binding
  // may resolve a different tcolorbox.sty version than the library files,
  // causing spurious "tcolorbox is not installed correctly" errors.
  // Make the check a no-op — the versions are always compatible in practice.
  DefMacro!("\\tcb@check@library@version", "", locked => true);

  // \newtcblisting[init-options]{name}[N][default]{tcb-options} (tcolorbox
  // `listings`/`minted` library) — a code-listing box. Its box styling is purely
  // visual; what matters for the logical output is the code BODY, which must be
  // captured verbatim and CLOSED at \end{name}. The raw library's body capture
  // does not integrate with LaTeXML's verbatim reader, so the listing runs past
  // its \end{name} and swallows following content (sections leak into
  // <ltx:verbatim>). Delegate to listings' \lstnewenvironment (same
  // name/[N][default] shape; the leading [init-options] and trailing tcb options
  // are dropped), whose verbatim reader terminates correctly. `locked` so a later
  // raw `\tcbuselibrary{listings}` can't clobber it.
  //
  // The signature MUST include the leading `[init-options]` optional (real
  // tcolorbox `\NewDocumentCommand \__tcobox_new_tcolorbox:w { m +O{} m o +o +m }`,
  // mirrored by tcblistings): omitting it made a call like
  // `\newtcblisting[auto counter,...]{promptbox}[2][]{...}` read `[auto counter,...]`
  // as the mandatory name, so `{promptbox}` was never defined and its verbatim body
  // tokenized as normal LaTeX — every `_`/`^` in the listing then raising
  // "Script can only appear in math mode" (Perl raw-loads the real macro and is
  // clean). Witness: 2606.00555 (leading init-options). Prior witnesses use no
  // leading optional and are unaffected: 2507.00833 (ar5iv #569/#570), 2402.13846 (#504).
  DefPrimitive!("\\lxtcblistingmode{}", sub[(opts)] {
    // Whether the body is EXECUTED is tcolorbox's own resolved state, not the
    // option text: tcblistingscore.code.tex:195-224 makes every mode a style
    // that `\let`s `\tcb@listing@process` / `\tcb@inputlisting` /
    // `\tcb@use@listing@other`, and the text part runs only when the process
    // reaches `\tcb@use@listing@other` = `\tcbuselistingtext` (:24-35, :429
    // default `listing and text`). A key-name scan of the options missed a
    // mode hidden in a user `.style` (tutodoc.cls:1208 `listing only` inside
    // `tutodoc-full-listing-style`, a Perl program then ran as LaTeX) and a
    // mode set by the ENCLOSING environment (codebox.sty:268 `\tcbset{listing
    // only}` before its `\DeclareTCBListing` box: `#include` lines reached the
    // stomach). So: set the per-use options through `\tcbset` in a group —
    // exactly what the box does — and read the three macros back. Without the
    // library (no `\tcb@listing@process`), keep the literal scan.
    // Guards: `perfect_kernel_batch56::{tcb_listing_mode_hidden_in_a_style_is_honoured,
    // tcb_listing_mode_set_by_the_enclosing_environment_is_honoured}`.
    const NO_TEXT_MODES: &[&str] = &[
      "listing only",
      "comment only",
      "listing and comment",
      "comment and listing",
      "comment side listing",
      "listing side comment",
      "comment above listing",
      "comment above* listing",
      "listing above comment",
      "listing above* comment",
      "comment outside listing",
      "listing outside comment",
    ];
    // `untex()`, not `to_string()`: the option tokens carry substituted
    // environment arguments (`before lower={…#3\par}` with `#3` =
    // `\dots ii (Brussels…)`, oxnotes-doc.tex:216/1652), and a separator-less
    // concatenation re-tokenizes `\dots ii` as `\dotsii`.
    // Guard: `perfect_kernel_batch56::tcb_listing_option_tokens_keep_cs_boundaries`.
    let text = opts.untex();
    if lookup_meaning(&T_CS!("\\tcb@listing@process")).is_some() {
      // The executed part is the box's LOWER part (tcblistingscore.code.tex:30-34
      // `\tcb@listing@listingAndOther` = listing, `\tcblower`, then the text),
      // so it runs inside the box's resolved `before lower*`/`after lower*`
      // wrapper: `tikz lower` (tcolorbox.sty:712) = `\begin{tikzpicture}[…]` …
      // `\end{tikzpicture}`. Captured here for `\lx@lstenv@body` (tikz2d-fr,
      // OutilsGeomTikz: `\draw`/`{scope}` undefined outside a picture).
      // Guard: `perfect_kernel_batch56::tcblisting_tikz_lower_wraps_the_executed_body`.
      let src = format!(
        "\\begingroup\\tcbset{{{text}}}\
         \\ifdefined\\kvtcb@before@lower\\global\\let\\lx@tcb@execbefore\\kvtcb@before@lower\
           \\else\\global\\let\\lx@tcb@execbefore\\@empty\\fi\
         \\ifdefined\\kvtcb@after@lower\\global\\let\\lx@tcb@execafter\\kvtcb@after@lower\
           \\else\\global\\let\\lx@tcb@execafter\\@empty\\fi\
         \\ifx\\tcb@use@listing@other\\tcbuselistingtext \
           \\ifx\\tcb@inputlisting\\tcb@inputlisting@inside \
             \\ifx\\tcb@listing@process\\tcb@listing@listing \\lxtcbexec0 \\else\\lxtcbexec1 \\fi\
           \\else\\lxtcbexec1 \\fi\
         \\else\\lxtcbexec0 \\fi\
         \\endgroup"
      );
      let _ = digest(mouth::tokenize_internal(TeXString::assembled(src)));
    } else {
      let execute = !split_keyval_source(&text)
        .iter()
        .any(|(key, _)| NO_TEXT_MODES.contains(&key.trim()));
      AssignValue!("LISTINGS_EXECUTE_BODY" => execute, Scope::Global);
    }
    Ok(Vec::new())
  });
  // Begin-line argument eaters for `tcb_xparse_listing`'s unmapped specifiers.
  // No `@` in these names: `tcb_xparse_listing` emits them through `Tokenize!`
  RawTeX!(r"\let\lxtcbifnext\@ifnextchar\def\lxtcbeatone#1{}\def\lxtcbeatangle<#1>{}\def\lxtcbeatparen(#1){}\def\lxtcbeatbracket[#1]{}\long\def\lxtcbifnextnospace#1#2#3{\def\lxtcbtempa{#2}\def\lxtcbtempb{#3}\def\lxtcbtemptarget{#1}\futurelet\lxtcblettoken\lxtcbifnchnospace}\def\lxtcbifnchnospace{\ifx\lxtcblettoken\lxtcbtemptarget\let\lxtcbtempc\lxtcbtempa\else\let\lxtcbtempc\lxtcbtempb\fi\lxtcbtempc}");
  DefMacro!("\\lx@tcb@execbefore", "");
  DefMacro!("\\lx@tcb@execafter", "");
  DefPrimitive!("\\lxtcbexec Number", sub[(n)] {
    AssignValue!("LISTINGS_EXECUTE_BODY" => n.value_of() != 0, Scope::Global);
    Ok(Vec::new())
  });
  // `\newtcblisting`, the `\NewTCBListing` family and `\newtcbinputlisting` are
  // installed by the `tcblistingscore.code.tex` binding (tcblistingscore_code_tex.rs)
  // AFTER the raw library loads — see there.

  // tcolorbox `documentation` library (tcbdocumentation.code.tex L242-255):
  // {dispExample} routes its body through `\tcbwritetemp` (verbatim write to
  // \jobname.tcbtemp, re-input as listing + executed). Our engine cannot
  // close that raw verbatim capture, so the environment leaked an OPEN
  // <ltx:verbatim> that swallowed the rest of the document (assoccnt manual:
  // 28 malformed-in-verbatim errors; perfect-kernel "malformed ltx:* in
  // verbatim" cluster). Reproduce the semantic directly, showexpl-style
  // (showexpl_sty.rs precedent): capture raw, emit the code listing, then
  // re-tokenize the body so it EXECUTES as the preview. `locked` so the raw
  // library load cannot restore the \tcbwritetemp path.
  DefPrimitive!(T_CS!("\\dispExample"), None, {
    bgroup();
    let text = listings_read_raw_lines("dispExample");
    unread(Tokenize!(TeXString::assembled("\\end{dispExample}".to_string())));
    // `\tcbusetemp` = `\input` of the temp file (tcolorbox.sty:2820). Guard:
    // `perfect_kernel_batch56::dispexample_body_runs_with_live_catcodes`.
    unread(lst_run_body_via_input("dispExample", &text)?);
    unread(Tokens::new(lst_process_display(None, &text)));
  }, locked => true);
  // The raw library redefines the END macros too (\enddispExample =
  // \endtcbwritetemp\endgroup…, tcbdocumentation.code.tex L244-251) — every
  // piece of the family must be locked or the raw load re-wires it back into
  // the write-temp machinery (witnessed as `t.tcbtemp` missing-file +
  // \endgroup mode errors).
  DefMacro!("\\enddispExample", "", locked => true);
  // {dispExample*}{options} — same, options are presentational.
  DefPrimitive!("\\lx@dispExampleStar{}", sub[(_opts)] {
    bgroup();
    let text = listings_read_raw_lines("dispExample*");
    unread(Tokenize!(TeXString::assembled("\\end{dispExample*}".to_string())));
    unread(lst_run_body_via_input("dispExampleStar", &text)?);
    unread(Tokens::new(lst_process_display(None, &text)));
  });
  DefMacro!(T_CS!("\\dispExample*"), None, "\\lx@dispExampleStar", locked => true);
  DefMacro!(T_CS!("\\enddispExample*"), None, "", locked => true);
  // {dispListing} — listing only, no preview.
  // Bare `\tcbwritetemp` … `\endtcbwritetemp` — tcolorbox's verbatim
  // record-to-\jobname.tcbtemp, invoked from OTHER environments' begin/end
  // hooks (keytheorems-doc L157-162 `{codepreamble}` = `\tcbset{mark
  // preamble}\tcbwritetemp` … `\endtcbwritetemp`). The real scanner reads
  // until the SURROUNDING environment's \end line; capture the same way,
  // store for `\tcbusetemplisting`, write the file for `\input`-back
  // consumers, and re-emit the `\end{env}` so the wrapper closes normally.
  DefPrimitive!(T_CS!("\\tcbwritetemp"), None, {
    let env = match lookup_value("current_environment") {
      Some(Stored::String(sym)) => with(sym, |s| s.to_string()),
      _ => String::from("tcbwritetemp"),
    };
    let text = listings_read_raw_lines(&env);
    assign_value("TCB@templisting", Stored::String(pin(&text)), Some(Scope::Global));
    let jobname = do_expand(Tokens!(T_CS!("\\jobname")))
      .map(|t| t.to_string())
      .unwrap_or_else(|_| String::from("texput"));
    // Virtual store only — the `\input`-back consumer resolves it via the
    // VFS (find_file consults it first); a disk write here would land in
    // the process CWD, not the destination directory (a stray
    // `<jobname>.tcbtemp` once leaked into the repo root from a test run).
    vfs_store(&format!("{}.tcbtemp", jobname.trim()), &text);
    unread(Tokenize!(TeXString::assembled(format!("\\end{{{env}}}"))));
  }, locked => true);
  DefMacro!(T_CS!("\\endtcbwritetemp"), None, "", locked => true);
  // `\begin{tcbverbatimwrite}{file}` (tcolorbox.sty:2726-2735): verbatim-write
  // the body to <file>. Raw, it ran through our `\verbatim@`, which — like
  // Perl's verbatim.sty.ltxml:76-104 — keeps the `\begin`-line remainder as
  // an EMPTY first line (real verbatim.sty:107-112 `\verbatim@start` swallows
  // that `^^M`), so csvsimple read a blank header ("File 'grade.csv' starts
  // with an empty line!" and a 900-line cascade once the reading examples
  // executed; csvsimple-legacy, SHARED, pdflatex clean). Capture like
  // `\tcbwritetemp` (the listings reader drops the remainder) and re-emit
  // `\end{env}` so the environment closes.
  // Guard: `perfect_kernel_batch56::tcbverbatimwrite_has_no_leading_blank_line`.
  DefPrimitive!("\\tcbverbatimwrite{}", sub[(file)] {
    let env = match lookup_value("current_environment") {
      Some(Stored::String(sym)) => with(sym, |s| s.to_string()),
      _ => String::from("tcbverbatimwrite"),
    };
    let text = listings_read_raw_lines(&env);
    // The raw `{tcblisting}` environment (tcblistingscore.code.tex:275-283)
    // passes `\kvtcb@listingfile` = `\jobname.listing`, which the later
    // `\tcbinputlisting@core` reads back EXPANDED: store under the expanded
    // name (sweep #40 regressed 25 manuals with `missing_file:<job>.listing`).
    // Guard: `perfect_kernel_batch56::raw_tcblisting_environment_round_trips_its_listing_file`.
    let name = Expand!(file).to_string().trim().to_string();
    vfs_store(&name, &text);
    unread(Tokenize!(TeXString::assembled(format!("\\end{{{env}}}"))));
    Ok(Vec::new())
  }, locked => true);
  DefMacro!(T_CS!("\\endtcbverbatimwrite"), None, "", locked => true);
  // Emit the recorded text as a listing (tcolorbox `\tcbusetemplisting`).
  DefPrimitive!(T_CS!("\\tcbusetemplisting"), None, {
    let text = match lookup_value("TCB@templisting") {
      Some(Stored::String(sym)) => with(sym, |s| s.to_string()),
      _ => String::new(),
    };
    if !text.is_empty() {
      unread(Tokens::new(lst_process_display(None, &text)));
    }
  }, locked => true);



  DefPrimitive!(T_CS!("\\dispListing"), None, {
    bgroup();
    let text = listings_read_raw_lines("dispListing");
    // Like \tcbwritetemp, dispListing RECORDS the example: consumers follow
    // it with `\tcbusetemp` (= `\input{\jobname.tcbtemp}`, raw tcolorbox) to
    // execute the source just displayed — witness incgraph.tex L722.
    assign_value("TCB@templisting", Stored::String(pin(&text)), Some(Scope::Global));
    let jobname = do_expand(Tokens!(T_CS!("\\jobname")))
      .map(|t| t.to_string())
      .unwrap_or_else(|_| String::from("texput"));
    vfs_store(&format!("{}.tcbtemp", jobname.trim()), &text);
    unread(Tokenize!(TeXString::assembled("\\end{dispListing}".to_string())));
    unread(Tokens::new(lst_process_display(None, &text)));
  }, locked => true);
  DefMacro!("\\enddispListing", "", locked => true);
});

/// Delegate an xparse-signature TCB listing declaration to
/// `\lstnewenvironment{name}[n][]{start}{end}`.
///
/// Only the MANDATORY specifiers (`m`/`r`/`R`) count as `\lstnewenvironment`
/// mandatory arguments, plus one optional slot when the signature LEADS with
/// an optional (`O`/`o`/`D`/`d`, after any `!`/`+` prefix). Counting every
/// specifier as mandatory turned neoschool.cls:5168
/// `\NewTCBListing{code}{ O{} m !O{} !O{..} !O{..} }` into
/// `\lstnewenvironment{code}[5][]`, so a bare `\begin{code}{latex}` grabbed
/// three body tokens including its own `\end`, the verbatim scan ran on to
/// the NEXT `\end{code}` and swallowed the `\begin{sidebyside}` in between —
/// tcolorbox's global `\c@tcblayer` (tcolorbox.sty:1411/1491 `\tcb@layer@inc`
/// only at the swallowed begin, `\tcb@layer@dec` at the surviving end) went
/// negative and every later box errored `every box on layer 0/-N` (251 of
/// neoschool's 273 errors; Perl 0). Trailing optionals are box styling
/// (tcbxparse absorbs them only when a `[` is present) and are dropped.
/// Guard: `perfect_kernel_batch53::tcb_listing_trailing_optionals_not_mandatory`.
pub(crate) fn tcb_xparse_listing(
  name: Tokens,
  sig: Tokens,
  init: Option<&Tokens>,
  opts: &Tokens,
) -> Result<Tokens> {
  let sig_str = sig.to_string();
  let specs_defaults = xparse_signature_specs_defaults(&sig_str);
  let specs: Vec<(char, String, bool)> = specs_defaults
    .iter()
    .map(|(c, d, _, bang)| (*c, d.clone(), *bang))
    .collect();
  let mandatory = specs
    .iter()
    .filter(|(c, _, _)| matches!(c, 'm' | 'r' | 'R'))
    .count();
  // The `[n][]` arity can express one LEADING bracket optional; every other
  // specifier the arity cannot express — `s`, `t<c>`, `d`/`D` with non-`[]`
  // delimiters, `G`/`g`, trailing `O`/`o`, or any `!`-modified optional
  // (which forbids skipping leading whitespace/newlines, unlike LaTeX's
  // `\@ifnextchar[` in `\lstnewenvironment`) — is absorbed from the begin
  // line by an eater in the start code (xparse reads them in the
  // same peek-then-consume way, ltcmd `\__cmd_grab_D:w`/`_G:w`/`_t:w`).
  // Left unconsumed, `\begin{tdoclatex}<opts>` / `\begin{example}*` /
  // `\begin{doccode}{opts}` (tutodoc.cls:1024, simplebnf-doc.tex:58,
  // istgame-doc.tex:129) leaked into the captured body and re-entered the
  // environment on `\input`-back (MemoryBudget fatal ×4, sweep #41).
  // Guard: `perfect_kernel_batch56::tcb_listing_unmapped_begin_line_args_are_absorbed`.
  let leading_optional = specs
    .first()
    .is_some_and(|(c, d, bang)| !bang && (matches!(c, 'O' | 'o') || (matches!(c, 'd' | 'D') && d == "[]")));
  let mut eaters = String::new();
  for (i, (c, d, bang)) in specs.iter().enumerate() {
    if i == 0 && leading_optional {
      continue;
    }
    match c {
      's' => eaters.push_str("\\lxtcbifnext*{\\lxtcbeatone}{}"),
      't' => eaters.push_str(&format!("\\lxtcbifnext{}{{\\lxtcbeatone}}{{}}", d)),
      'G' | 'g' => eaters.push_str("\\lxtcbifnext\\bgroup{\\lxtcbeatone}{}"),
      'O' | 'o' => {
        if *bang {
          eaters.push_str("\\lxtcbifnextnospace[{\\lxtcbeatbracket}{}");
        } else {
          eaters.push_str("\\lxtcbifnext[{\\lxtcbeatbracket}{}");
        }
      },
      'd' | 'D' => match d.as_str() {
        "<>" => eaters.push_str("\\lxtcbifnext<{\\lxtcbeatangle}{}"),
        "()" => eaters.push_str("\\lxtcbifnext({\\lxtcbeatparen}{}"),
        "[]" => {
          if *bang {
            eaters.push_str("\\lxtcbifnextnospace[{\\lxtcbeatbracket}{}");
          } else {
            eaters.push_str("\\lxtcbifnext[{\\lxtcbeatbracket}{}");
          }
        },
        _ => {},
      },
      _ => {},
    }
  }
  // The options body numbers its `#n` by POSITION in the full signature.
  // An absorbed specifier still owns its slot: its `#n` becomes the
  // specifier's default text (`D(){teal}` → `teal`, empty for `o`/`s`/`g`),
  // and the mandatory (and the mapped leading optional) slots are renumbered
  // to the `\lstnewenvironment` arity — `\NewTCBListing{egcite}{D(){ok} o m !o}
  // {colframe=#1,…}` (oxyear-doc.tex:216) otherwise fed the citation text to
  // `colframe` once the leading `D()` was eaten (sweep #42 regression).
  // Guard: `perfect_kernel_batch56::tcb_listing_absorbed_specifiers_keep_positional_numbers`.
  let mut slots: Vec<Option<String>> = Vec::new(); // Some(text) = substitute, None = keep `#k`
  let mut next = 0usize;
  let mut renumber: Vec<usize> = Vec::new();
  for (i, (c, _d, default, _bang)) in specs_defaults.iter().enumerate() {
    let mapped = (i == 0 && leading_optional) || matches!(c, 'm' | 'r' | 'R');
    if mapped {
      next += 1;
      renumber.push(next);
      slots.push(None);
    } else {
      renumber.push(0);
      slots.push(Some(default.clone()));
    }
  }
  let opts_renumbered = renumber_param_tokens(opts, &slots, &renumber);
  let name_str = name.to_string().trim().to_string();
  let (start, end) = tcb_listing_startend(&name_str, init, &opts_renumbered);
  let start = format!("{eaters}{start}");
  let arity = if leading_optional {
    format!("[{}][]", mandatory + 1)
  } else {
    format!("[{}]", mandatory)
  };
  Ok(Tokenize!(TeXString::assembled(format!(
    "\\lstnewenvironment{{{}}}{}{{{}}}{{{}}}",
    name_str, arity, start, end
  ))))
}

/// Rewrite the `#k` parameter references of a tcb options body per
/// [`tcb_xparse_listing`]'s slot map: a substituted slot becomes its default
/// text (tokenized at the current catcodes), a mapped slot its new number.
fn renumber_param_tokens(opts: &Tokens, slots: &[Option<String>], renumber: &[usize]) -> Tokens {
  let toks = opts.unlist_ref();
  let mut out: Vec<Token> = Vec::with_capacity(toks.len());
  let mut i = 0;
  while i < toks.len() {
    let t = &toks[i];
    if t.get_catcode() == Catcode::PARAM
      && let Some(next) = toks.get(i + 1)
      && let Some(k) = next.to_string().chars().next().and_then(|c| c.to_digit(10))
      && next.get_catcode() != Catcode::PARAM
    {
      let idx = (k as usize).saturating_sub(1);
      match slots.get(idx) {
        Some(Some(text)) => {
          out.extend(Tokenize!(TeXString::assembled(text.clone())).unlist());
        },
        Some(None) => {
          out.push(*t);
          out.push(T_OTHER!(&renumber[idx].to_string()));
        },
        None => {
          out.push(*t);
          out.push(*next);
        },
      }
      i += 2;
      continue;
    }
    out.push(*t);
    i += 1;
  }
  Tokens::new(out)
}

/// The specifiers of an xparse argument signature, in order: the letter, its
/// delimiter pair when it has one (`d<>`/`D<>{…}` → `"<>"`, `t*` → `"*"`) and its
/// `{default}` text (`O`, `D`, `G`; empty otherwise). `!`/`+` prefixes are skipped.
fn xparse_signature_specs_defaults(sig: &str) -> Vec<(char, String, String, bool)> {
  let chars: Vec<char> = sig.chars().collect();
  let mut out = Vec::new();
  let mut i = 0;
  let mut has_bang = false;
  let take_group = |i: &mut usize| -> String {
    let mut text = String::new();
    if *i < chars.len() && chars[*i] == '{' {
      let mut depth = 0;
      while *i < chars.len() {
        match chars[*i] {
          '{' => {
            depth += 1;
            if depth > 1 {
              text.push('{');
            }
          },
          '}' => {
            depth -= 1;
            if depth == 0 {
              *i += 1;
              break;
            }
            text.push('}');
          },
          c => text.push(c),
        }
        *i += 1;
      }
    }
    text
  };
  while i < chars.len() {
    let c = chars[i];
    i += 1;
    if c == '!' {
      has_bang = true;
      continue;
    }
    match c {
      'm' | 'o' | 's' | 'g' | 'v' | 'b' | 'l' | 'u' | 'e' | 'E' => {
        out.push((c, String::new(), String::new(), has_bang));
        has_bang = false;
      },
      'O' | 'G' => {
        while i < chars.len() && chars[i] == ' ' {
          i += 1;
        }
        let d = take_group(&mut i);
        out.push((c, String::new(), d, has_bang));
        has_bang = false;
      },
      't' => {
        while i < chars.len() && chars[i] == ' ' {
          i += 1;
        }
        let d = chars.get(i).map(|c| c.to_string()).unwrap_or_default();
        i += 1;
        out.push((c, d, String::new(), has_bang));
        has_bang = false;
      },
      'd' | 'D' | 'r' | 'R' => {
        while i < chars.len() && chars[i] == ' ' {
          i += 1;
        }
        let d: String = chars.iter().skip(i).take(2).collect();
        i += 2;
        let mut default = String::new();
        if matches!(c, 'D' | 'R') {
          while i < chars.len() && chars[i] == ' ' {
            i += 1;
          }
          default = take_group(&mut i);
        }
        out.push((c, d, default, has_bang));
        has_bang = false;
      },
      _ => {}, // `+`, `>`, spaces, unknown
    }
  }
  out
}

/// Distill the per-use side effects a tcb listing environment owes from its
/// `[init-options]` + `{options}` — the pieces of the tcolorbox option
/// machinery with document-visible consequences. Currently honored:
/// `use counter=N` (each use steps N and exposes `\thetcbcounter`,
/// tcbcounter.code.tex) and `listing file=F` (each use records the raw body
/// to F for `\input`-back, tcblistingscore `listing file`; F is expanded at
/// USE time so `\jobname.\thetcbcounter.listing` names follow the counter —
/// witness incgraph.tex L857 `\inputlisting{\n}` reading 12 such files).
/// Everything else remains presentation-only and is dropped.
pub(crate) fn tcb_listing_startend(
  env_name: &str,
  init: Option<&Tokens>,
  opts: &Tokens,
) -> (String, String) {
  let mut start = String::new();
  let mut end = String::new();
  let source = format!(
    "{},{}",
    init.map(|t| t.to_string()).unwrap_or_default(),
    opts
  );
  // The counter keys run through tcolorbox's OWN init processor
  // (tcolorbox.sty:2339 `\tcb@proc@options@init{init}{env}` → `auto counter`
  // / `use counter` / `use counter from` / `number within` …, :2254-2262 and
  // :2297-2333), so `\tcb@cnt@<env>` and `\thetcb@cnt@<env>` exist for a
  // later `\newtcolorbox[use counter from=<env>]` (tcolorbox manual's
  // preamble D: `texexptitledspec` from `texexptitled` — "Extra \endcsname"
  // + `\the\tcb@cnt@texexptitled` undefined when the native listing env
  // dropped its init). Each use then steps the recorded counter the way
  // `\tcb@new@colopt`'s `code=` does (:2311).
  if init.is_some_and(|t| !t.to_string().trim().is_empty()) {
    let init_src = init.map(|t| t.to_string()).unwrap_or_default();
    let _ = digest(mouth::tokenize_internal(TeXString::assembled(format!(
      "\\tcb@proc@options@init{{{init_src}}}{{{env_name}}}"
    ))));
    start.insert_str(
      0,
      &format!(
        "\\ifcsdef{{tcb@cnt@{env_name}}}{{\\letcs\\tcbcounter{{tcb@cnt@{env_name}}}\
         \\letcs\\thetcbcounter{{thetcb@cnt@{env_name}}}\\refstepcounter{{\\tcbcounter}}}}{{}}"
      ),
    );
  }
  // The listing MODE (tcblistingscore.code.tex:200-215: `listing and text`
  // default, `listing only`, `text only`, the `*comment*` forms) decides
  // whether the body is also executed; the per-use `#1` is only known at use
  // time, so the resolved option text is handed to `\lxtcblistingmode` in
  // the start code and read back by `\lx@lstenv@body`.
  // Only the box options: the `[init]` keys live in `/tcb/new/` and are not
  // `\tcbset`-able.
  start.push_str(&format!("\\lxtcblistingmode{{{opts}}}"));
  for (key, val) in split_keyval_source(&source) {
    let val = val.trim().trim_matches(['{', '}']).trim();
    match key.trim() {
      "use counter" | "auto counter" => {
        // Record which LaTeX counter this env drives so
        // `use counter from=<env>` (\newtcbinputlisting) can share it.
        let counter = if key.trim() == "auto counter" || val.is_empty() {
          format!("tcb@cnt@{env_name}")
        } else {
          val.to_string()
        };
        assign_value(
          &format!("tcb_env_counter_{env_name}"),
          Stored::String(pin(&counter)),
          Some(Scope::Global),
        );
      },
      "listing file" if !val.is_empty() => {
        start.push_str(&format!("\\lxlstbeginwritefile{{{val}}}"));
        end.push_str("\\lxlstendwritefile");
      },
      _ => {},
    }
  }
  (start, end)
}
