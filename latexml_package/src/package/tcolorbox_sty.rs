//! tcolorbox.sty — colored and framed text boxes
//! Perl: tcolorbox.sty.ltxml
use crate::{
  package::listings_sty::{listings_read_raw_lines, lst_process_display},
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
  DefMacro!("\\newtcblisting[]{}[][]{}", sub[(init, name, n, default, opts)] {
    let env_name = name.to_string().trim().to_string();
    let (start, end) = tcb_listing_startend(&env_name, init.as_ref(), &opts);
    Ok(Tokenize!(TeXString::assembled(format!(
      "\\lstnewenvironment{{{}}}[{}][{}]{{{}}}{{{}}}",
      env_name,
      n.map(|t| t.to_string()).unwrap_or_default(),
      default.map(|t| t.to_string()).unwrap_or_default(),
      start,
      end
    ))))
  }, locked => true);

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
    unread(Tokenize!(TeXString::assembled(text.clone())));
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
    unread(Tokenize!(TeXString::assembled(text.clone())));
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

  // \NewTCBListing / \DeclareTCBListing / \RenewTCBListing / \ProvideTCBListing
  // [init-options]{name}{xparse-sig}{options}: the xparse-signature flavor of
  // \newtcblisting (tcbxparse library; tcblistingscore.code.tex:329-355 —
  // `\__tcobox_new_TCBListing:w { m +O{} >{\TrimSpaces} m +m +m }`, so the
  // user shape carries a LEADING optional, same as the plain `\newtcblisting`
  // fix above). Map the signature to a \lstnewenvironment shape (mandatory
  // count + one leading optional, see `tcb_xparse_listing`) and delegate like
  // the plain form.
  // Witnesses: keytheorems-doc L165 `\NewTCBListing{keythmscode}{ !O{} }{…}`
  // (31 uses, no leading optional); atableau.tex L655
  // `\NewTCBListing[use counter=example, …]{example}{ O{} s m }{…#1…#3…}`
  // (leading optional — without absorbing it the three `{}` args grab `[`,
  // `u`, `s` and the options body digests raw: misdefined:# storm).
  // Known residual: the `s` star specifier is not expressible via
  // \lstnewenvironment, so a starred `\begin{example}*` call mis-grabs `*`.
  DefMacro!("\\NewTCBListing[]{}{}{}", sub[(init, name, sig, opts)] {
    tcb_xparse_listing(name, sig, init.as_ref(), &opts)
  });
  DefMacro!("\\DeclareTCBListing[]{}{}{}", sub[(init, name, sig, opts)] {
    tcb_xparse_listing(name, sig, init.as_ref(), &opts)
  });
  DefMacro!("\\RenewTCBListing[]{}{}{}", sub[(init, name, sig, opts)] {
    tcb_xparse_listing(name, sig, init.as_ref(), &opts)
  });
  DefMacro!("\\ProvideTCBListing[]{}{}{}", sub[(init, name, sig, opts)] {
    tcb_xparse_listing(name, sig, init.as_ref(), &opts)
  });

  // \newtcbinputlisting[init]{\cmd}[n][default]{options} — defines \cmd to
  // INPUT a listing file into a listing box at each use
  // (tcblistingscore.code.tex L355-378: \cmd = \tcbinputlisting{options}).
  // Semantic core honored: `use counter from=<env>` steps the counter
  // recorded for <env> (see tcb_listing_startend) and exposes
  // \thetcbcounter; `listing file=F` — with the call's #-arguments
  // substituted and expanded — is read (VFS first, then disk) and displayed
  // as a listing. Witness: incgraph-doc.sty L128 `\newtcbinputlisting[use
  // counter from=texexptitled]{\inputexamplelisting}[3][]{…listing
  // file={#2}…}`.
  DefPrimitive!("\\newtcbinputlisting []{}[Number][] DefPlain", sub[(_init, cmd, n, default, opts)] {
    let cmd_tok = cmd
      .unlist_ref()
      .iter()
      .find(|t| t.code == Catcode::CS)
      .copied();
    let Some(cmd_tok) = cmd_tok else {
      return Ok(Vec::new());
    };
    let n: usize = n.value_of() as usize;
    let mut param_spec = String::new();
    if let Some(ref d) = default {
      param_spec.push_str(&format!("[Default:{d}]"));
      for _ in 1..n {
        param_spec.push_str("{}");
      }
    } else {
      for _ in 0..n {
        param_spec.push_str("{}");
      }
    }
    let params = if param_spec.is_empty() {
      None
    } else {
      parse_parameters(&param_spec, &cmd_tok, true)?
    };
    // Same shape as \lstinputlisting (listings_sty.rs): a MACRO whose
    // expansion opens the listing group (`bgroup()`) and yields the display
    // tokens; the display's own trailer closes it. A primitive that unread
    // the display, or a bare expansion without the bgroup, tripped
    // "close a group that switched to mode internal_vertical".
    let expansion: Option<ExpansionBody> = Some(ExpansionBody::Closure(Rc::new(
      move |args: Vec<ArgWrap>| {
        use latexml_core::keyval::split_keyval_source;
        let sub_args: Vec<Option<Cow<Tokens>>> = args
          .iter()
          .map(|a| match a {
            ArgWrap::None => None,
            ArgWrap::Tokens(t) => Some(Cow::Borrowed(t)),
            ArgWrap::Token(t) => Some(Cow::Owned(Tokens::new(vec![*t]))),
            other => Some(Cow::Owned(Tokens::new(ExplodeText!(other.to_string())))),
          })
          .collect();
        let opts_subst = opts.substitute_parameters(&sub_args);
        let mut counter: Option<String> = None;
        let mut file: Option<String> = None;
        for (key, val) in split_keyval_source(&opts_subst.to_string()) {
          let val = val.trim().trim_matches(['{', '}']).trim();
          match key.trim() {
            "use counter from" if !val.is_empty() => {
              if let Some(Stored::String(sym)) =
                lookup_value(&format!("tcb_env_counter_{val}"))
              {
                counter = Some(with(sym, |c| c.to_string()));
              }
            },
            "listing file" if !val.is_empty() => file = Some(val.to_string()),
            _ => {},
          }
        }
        if let Some(counter) = counter {
          digest(Tokenize!(TeXString::assembled(format!(
            "\\refstepcounter{{{counter}}}\\def\\thetcbcounter{{\\csname the{counter}\\endcsname}}"
          ))))?;
        }
        let Some(file) = file else {
          return Ok(Tokens!());
        };
        let file = do_expand(Tokenize!(TeXString::assembled(file)))
          .map(|t| t.to_string())
          .unwrap_or_default();
        let file = file.trim();
        let text = vfs_read(file)
          .or_else(|| std::fs::read_to_string(file).ok())
          .unwrap_or_default();
        bgroup();
        Ok(Tokens::new(lst_process_display(
          Some(Tokens::new(ExplodeText!(file))),
          &text,
        )))
      },
    )));
    def_macro(cmd_tok, params, expansion, None)?;
  }, locked => true);
  DefMacro!("\\renewtcbinputlisting", "\\newtcbinputlisting", locked => true);

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
fn tcb_xparse_listing(
  name: Tokens,
  sig: Tokens,
  init: Option<&Tokens>,
  opts: &Tokens,
) -> Result<Tokens> {
  let sig_str = sig.to_string();
  let specs: Vec<char> = sig_str
    .chars()
    .filter(|c| matches!(c, 'O' | 'o' | 'm' | 'd' | 'D' | 'r' | 'R'))
    .collect();
  let mandatory = specs
    .iter()
    .filter(|c| matches!(c, 'm' | 'r' | 'R'))
    .count();
  let leading_optional = specs
    .first()
    .is_some_and(|c| matches!(c, 'O' | 'o' | 'd' | 'D'));
  let name_str = name.to_string().trim().to_string();
  let (start, end) = tcb_listing_startend(&name_str, init, opts);
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

/// Distill the per-use side effects a tcb listing environment owes from its
/// `[init-options]` + `{options}` — the pieces of the tcolorbox option
/// machinery with document-visible consequences. Currently honored:
/// `use counter=N` (each use steps N and exposes `\thetcbcounter`,
/// tcbcounter.code.tex) and `listing file=F` (each use records the raw body
/// to F for `\input`-back, tcblistingscore `listing file`; F is expanded at
/// USE time so `\jobname.\thetcbcounter.listing` names follow the counter —
/// witness incgraph.tex L857 `\inputlisting{\n}` reading 12 such files).
/// Everything else remains presentation-only and is dropped.
fn tcb_listing_startend(env_name: &str, init: Option<&Tokens>, opts: &Tokens) -> (String, String) {
  use latexml_core::keyval::split_keyval_source;
  let mut start = String::new();
  let mut end = String::new();
  let source = format!(
    "{},{}",
    init.map(|t| t.to_string()).unwrap_or_default(),
    opts
  );
  for (key, val) in split_keyval_source(&source) {
    let val = val.trim().trim_matches(['{', '}']).trim();
    match key.trim() {
      "use counter" if !val.is_empty() => {
        // Record which LaTeX counter this env drives so
        // `use counter from=<env>` (\newtcbinputlisting) can share it.
        assign_value(
          &format!("tcb_env_counter_{env_name}"),
          Stored::String(pin(val)),
          Some(Scope::Global),
        );
        start.insert_str(
          0,
          &format!("\\refstepcounter{{{val}}}\\def\\thetcbcounter{{\\csname the{val}\\endcsname}}"),
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
