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
    listings_sty::{listings_read_raw_file, lst_process_display, lst_run_body_via_input},
    tcolorbox_sty::{
      tcb_counter_step_code, tcb_listing_startend, tcb_resolve_listing_mode, tcb_xparse_listing,
    },
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
  // box. What matters for the logical output is the code BODY, captured verbatim
  // and CLOSED at \end{name}, and the box's title (`tcb_store_pending_title`, the
  // listing's title: OD #768). The raw library's body capture once ran past its
  // \end{name} and swallowed following content; since the virtual-file round trip
  // (`\tcbverbatimwrite` → `listings_read_raw_file`) tcolorbox's own definition
  // (tcblistingscore.code.tex:301-322, `\newenvironment{name}…{\tcblisting{#6,
  // options@for=name}}{\endtcblisting}`) closes too, but it draws the box as a
  // picture whose `label=` lands on the nearest labelled ancestor — a section, else
  // <document> (raw tcolorbox; SYNC silent-loss row "tcolorbox box label") — and the xparse family
  // loses its begin-line reading (77m measurement, OD #768). So: delegate to
  // listings' \lstnewenvironment (same name/[N][default] shape; the leading
  // [init-options] and trailing tcb options feed `tcb_listing_startend`).
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
  // The options are a macro ARGUMENT in TeX (`\NewDocumentCommand`, L379),
  // not a definition body, so they are read as `{}` and packed here (see
  // `\lstnewenvironment`, listings_sty.rs).
  DefPrimitive!("\\newtcbinputlisting []{}[Number][] {}", sub[(init, cmd, n, default, opts)] {
    let opts = opts.pack_parameters()?;
    let cmd_tok = cmd
      .unlist_ref()
      .iter()
      .find(|t| t.code == Catcode::CS)
      .copied();
    let Some(cmd_tok) = cmd_tok else {
      return Ok(Vec::new());
    };
    // The init options go through tcolorbox's own init processor under the command's name without
    // its backslash (tcblistingscore.code.tex:374-377 `\__tcobox_set_backslash_removed:Nn`, then
    // :359 `\__tcobox_process_newtcolorbox:nn`), as for a listing environment
    // (`tcb_listing_startend`): `use counter=example` gives each use `\thetcbcounter` (the
    // `\examplefile` template of 1912.06298, 2405.01480 …: `title={Example~\thetcbcounter: #2}`
    // was "undefined:\thetcbcounter" once the title was typeset).
    let box_name = cmd_tok.with_str(|s| s.trim_start_matches('\\').to_string());
    let init_src = init.map(|t| t.to_string()).unwrap_or_default();
    if !init_src.trim().is_empty() {
      let _ = digest(mouth::tokenize_internal(TeXString::assembled(format!(
        "\\tcb@proc@options@init{{{init_src}}}{{{box_name}}}"
      ))));
    }
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
        // (each use steps the box's counter, as `\tcb@proc@counter@autoanduse`'s `code=` does,
        // tcolorbox.sty:2311)
        if let Some(counter) = counter.filter(|_| {
          lookup_meaning(&T_CS!(format!("\\tcb@cnt@{box_name}"))).is_none()
        }) {
          digest(Tokenize!(TeXString::assembled(format!(
            "\\refstepcounter{{{counter}}}\\def\\thetcbcounter{{\\csname the{counter}\\endcsname}}"
          ))))?;
        } else {
          digest(Tokenize!(TeXString::assembled(tcb_counter_step_code(&box_name))))?;
        }
        let Some(file) = file else {
          return Ok(Tokens!());
        };
        let file = do_expand(Tokenize!(TeXString::assembled(file)))
          .map(|t| t.to_string())
          .unwrap_or_default();
        let file = file.trim();
        // Perl reads every listing file through listingsReadRawFile
        // (`FindFile(…, noltxml => 1)`, listings.sty.ltxml:322-334): the session's
        // virtual store first (a round-tripped `\jobname.<n>.listing`), then
        // kpathsea and the source directory. A read relative to the working
        // directory missed both a tex-tree file (commalists-tools-doc
        // `\DemoCodeFile{commalists-tools.sty}`) and one beside the document
        // (csvsimple-legacy `\xmllisting{namesort}`), and the listing came out
        // empty with no diagnostic.
        // (read before the box's options are resolved below, so a failed read parks no title,
        // phantom or mode for an unrelated later listing)
        let text = listings_read_raw_file(file)?.unwrap_or_default();
        // The box's options resolved as the box resolves them (`tcb_resolve_listing_mode`, as for a
        // listing environment): its title (typeset as the listing's — 1912.06298 and the other
        // `\examplefile` papers: `title={Example~\thetcbcounter: #2}`), its phantom code (a per-use
        // `label=` / `hypertarget=`, run at the listing's first line: `\examplefile[label={ex:header},
        // …]` was a dangling `\ref`) and its mode, which `\tcbinputlisting` (tcblistingscore.code.tex
        // :337-344 → `\tcb@inputlisting`, :163-177 inside the box, :59-160 outside) follows: the default
        // `listing and text` typesets the file again after the listing (`\tcbuselistingtext` =
        // `\input{\kvtcb@listingfile}`, :25), `text and listing` before it, `text only` alone
        // (dropped with 0 errors before). Guards `perfect_kernel_batch77m::{
        // newtcbinputlisting_keeps_its_title, newtcbinputlisting_keeps_its_label,
        // newtcbinputlisting_follows_its_mode}`.
        tcb_resolve_listing_mode(opts_subst)?;
        let execute = lookup_bool("LISTINGS_EXECUTE_BODY");
        AssignValue!("LISTINGS_EXECUTE_BODY" => false, Scope::Global);
        let layout = lookup_string("tcb_pending_layout");
        // (the text ends its paragraph, as the box it is typeset in does: tcolorbox.sty:360-366
        // `before`/`after` `\par`)
        let run_text = if execute {
          let mut run = lst_run_body_via_input(&box_name, &text)?.unlist();
          run.push(T_CS!("\\par"));
          run
        } else {
          Vec::new()
        };
        if layout == "O" {
          // No listing: the box holds its title bar and its text. With no listing to caption, the
          // title is a paragraph before the text; the phantom code has no element of the box to name
          // (a `label` there would name the enclosing section) and is dropped, as a `text only`
          // listing environment drops it (`\lxtcbdropphantom`).
          let title = match lookup_value("tcb_pending_title") {
            Some(Stored::Tokens(title)) => title,
            _ => Tokens!(),
          };
          AssignValue!("tcb_pending_phantom" => Stored::None, Scope::Global);
          AssignValue!("tcb_pending_title" => Stored::None, Scope::Global);
          AssignValue!("tcb_pending_remove_caption" => false, Scope::Global);
          let mut out = vec![T_CS!("\\begingroup")];
          if !title.is_empty() {
            out.extend([T_CS!("\\par"), T_CS!("\\noindent")]);
            out.extend(title.unlist());
            out.push(T_CS!("\\par"));
          }
          out.extend(run_text);
          out.push(T_CS!("\\endgroup"));
          return Ok(Tokens::new(out));
        }
        bgroup();
        // tcolorbox reads the file back `nolol` (tcblistings.code.tex:45, and `\lst@MakeCaption`
        // gobbled, :31-33 `listing remove caption`): no list-of-listings entry, so the file name
        // is no float's caption (guard `perfect_kernel_batch77m::newtcbinputlisting_keeps_its_title`).
        assign_value("LST@nolol", Stored::String(pin("true")), None);
        let display = lst_process_display(Some(Tokens::new(ExplodeText!(file))), &text);
        if run_text.is_empty() {
          return Ok(Tokens::new(display));
        }
        // the text inside the box's group, as `\tcbinputlisting`'s `\begingroup…\endgroup` holds it
        let mut out = Vec::with_capacity(display.len() + run_text.len());
        if layout == "OL" {
          out.extend(run_text);
          out.extend(display);
        } else {
          out.extend(display);
          let closer = out.pop();
          out.extend(run_text);
          out.extend(closer);
        }
        Ok(Tokens::new(out))
      },
    )));
    def_macro(cmd_tok, params, expansion, None)?;
  }, locked => true);
  DefMacro!("\\renewtcbinputlisting", "\\newtcbinputlisting", locked => true);
});
