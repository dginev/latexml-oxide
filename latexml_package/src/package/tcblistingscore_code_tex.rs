//! tcblistingscore.code.tex — tcolorbox `listings@core` library (loaded by the
//! `listings`, `listingsutf8` and `minted` libraries: tcblistings.code.tex:25,
//! tcbminted.code.tex:30).
//!
//! The raw library loads as-is; only the listing-box DEFINITION family is then
//! replaced (each entry documents why). Installing our definitions AFTER the raw
//! load is what a real LaTeX run's order is: tcblistingscore.code.tex:318-378
//! declares the family with `\NewDocumentCommand`, whose ltcmd
//! `command-already-defined` check is an `\errmessage` (counted since batch
//! 56g) — pre-defining them in tcolorbox_sty.rs raised seven of those on every
//! `\tcbuselibrary{listings}`. Witnesses: codebox-doc-en, postit, keytheorems-doc.
//! Guard: `perfect_kernel_batch56::tcbuselibrary_listings_installs_the_family_once`.
use latexml_core::keyval::split_keyval_source;

use crate::{
  package::{
    listings_sty::lst_process_display,
    tcolorbox_sty::{tcb_listing_startend, tcb_xparse_listing},
  },
  prelude::*,
};

#[rustfmt::skip]
LoadDefinitions!({
  InputDefinitions!(
    "tcblistingscore.code",
    extension => Some(Cow::Borrowed("tex")),
    noltxml => true
  );

  // \newtcblisting[init-options]{name}[N][default]{tcb-options} — a code-listing
  // box. Its box styling is purely visual; what matters for the logical output is
  // the code BODY, which must be captured verbatim and CLOSED at \end{name}. The
  // raw library's body capture does not integrate with LaTeXML's verbatim reader,
  // so the listing runs past its \end{name} and swallows following content.
  // Delegate to listings' \lstnewenvironment (same name/[N][default] shape; the
  // leading [init-options] and trailing tcb options feed `tcb_listing_startend`).
  // The signature MUST include the leading `[init-options]` optional (real
  // tcolorbox `\NewDocumentCommand \__tcobox_new_tcolorbox:w { m +O{} m o +o +m }`).
  DefMacro!("\\newtcblisting[]{}[][]{}", sub[(init, name, n, default, opts)] {
    let env_name = name.to_string().trim().to_string();
    let (start, end) = tcb_listing_startend(&env_name, init.as_ref(), &opts);
    // `[N]` alone = N MANDATORY arguments; only a present `[default]` makes the
    // first one optional (tcblistingscore.code.tex:318-323 → `\newenvironment`
    // shape). Emitting an empty `[]` default turned `\newtcblisting{DemoCode}[1]`
    // into an optional-argument environment, so `\begin{DemoCode}{listing only}`
    // read `#1` as empty and executed the displayed preamble code (sweep 39:
    // calculatoritems, mathador, randintlist, tikz-decofonts, tikzbrickfigurines,
    // OutilsGeomTikz, commalists-tools ×2, tikz2d-fr, tutodoc ×2 regressed).
    // Guard: `perfect_kernel_batch56::newtcblisting_mandatory_argument_stays_mandatory`.
    let mut signature = String::new();
    if let Some(n) = n {
      signature.push_str(&format!("[{}]", n.to_string().trim()));
      if let Some(default) = default {
        signature.push_str(&format!("[{default}]"));
      }
    }
    Ok(Tokenize!(TeXString::assembled(format!(
      "\\lstnewenvironment{{{env_name}}}{signature}{{{start}}}{{{end}}}"
    ))))
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
});
