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
    let (start, end) = tcb_listing_startend(init.as_ref(), &opts);
    Ok(Tokenize!(TeXString::assembled(format!(
      "\\lstnewenvironment{{{}}}[{}][{}]{{{}}}{{{}}}",
      name.to_string().trim(),
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
    use std::io::Write;
    if let Ok(mut fh) = std::fs::File::create(format!("{}.tcbtemp", jobname.trim())) {
      let _ = write!(fh, "{text}");
    }
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
  // fix above). Approximate the signature by its argument COUNT (O/o/m/d
  // specifiers) and delegate to \lstnewenvironment like the plain form.
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
/// `\lstnewenvironment{name}[n][]{start}{end}` using the specifier COUNT.
fn tcb_xparse_listing(
  name: Tokens,
  sig: Tokens,
  init: Option<&Tokens>,
  opts: &Tokens,
) -> Result<Tokens> {
  let sig_str = sig.to_string();
  let nargs = sig_str
    .chars()
    .filter(|c| matches!(c, 'O' | 'o' | 'm' | 'd' | 'D'))
    .count();
  let name_str = name.to_string();
  let (start, end) = tcb_listing_startend(init, opts);
  Ok(Tokenize!(TeXString::assembled(format!(
    "\\lstnewenvironment{{{}}}[{}][]{{{}}}{{{}}}",
    name_str.trim(),
    nargs,
    start,
    end
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
fn tcb_listing_startend(init: Option<&Tokens>, opts: &Tokens) -> (String, String) {
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
        start.insert_str(
          0,
          &format!("\\refstepcounter{{{val}}}\\def\\thetcbcounter{{\\csname the{val}\\endcsname}}"),
        );
      },
      "listing file" if !val.is_empty() => {
        start.push_str(&format!("\\lst@BeginAlsoWriteFile{{{val}}}"));
        end.push_str("\\lst@EndWriteFile");
      },
      _ => {},
    }
  }
  (start, end)
}
