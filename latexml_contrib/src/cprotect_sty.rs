//! `cprotect.sty` — verbatim material inside macro arguments.
//!
//! There is **no upstream Perl binding** for this package (Perl LaTeXML 0.8.8
//! reports `Warning:missing_file:cprotect` and then `Error:undefined:\cprotect`
//! on the issue #347 witness), so this is a beyond-Perl contribution and lives
//! in `latexml_contrib`.
//!
//! ## What the package does, and how the port differs
//!
//! `cprotect.sty` (Bruno Le Floch, TL `tex/latex/cprotect/cprotect.sty`) works
//! around a pure-TeX limitation: a braced macro argument is tokenized *before*
//! the macro runs, so a `\verb` inside it never gets to change catcodes and the
//! `\`, `%`, `#`, … of its body have already been mis-tokenized. Its machinery
//! (`\CPT@readContent` / `\makeallother` / `\CPT@Write`) reads the argument with
//! every catcode forced to 12, **writes it out to `\jobname-N.cpt`**, and hands
//! the command `\protect\input <that file>` instead — the re-`\input` is what
//! re-tokenizes the text so `\verb` can finally do its job.
//!
//! LaTeXML has a mouth-level equivalent of "write it out and read it back"
//! that needs no disk round-trip: eTeX's `\scantokens` (`latexml_engine::etex`,
//! Perl `eTeX.pool.ltxml` L251) opens a fresh `Mouth` over a string. So the
//! port keeps cprotect's *reading* half verbatim and replaces its *file* half:
//!
//! ```text
//! \cprotect\section{<raw>}   ==>   \section{\scantokens{<raw>}}
//! ```
//!
//! `<raw>` is read with the specials neutralized (the `\makeallother` stand-in),
//! except that `{`/`}` keep catcodes 1/2 so `read_balanced` does the brace
//! counting that `\CPT@gobbleOneB` does by hand.
//!
//! Not ported: `\ReadVerbatimUntil` (the package-author-level generic verbatim
//! reader) and the `gobbling-escape=`/`gobbling-letter=` options — the latter
//! only tune the `^^E`/`^^L` sentinels of the file round-trip we do not perform.
//!
//! ## One deliberate deviation from `cprotect.sty`
//!
//! A **multi-letter signature keeps going after a single-token argument.** In
//! the package, `\CPT@read@mone` (the `m` branch for an argument that is not a
//! `{`-group) is `\CPT@cs\CPT@next` — it emits the command and stops, dropping
//! the rest of the signature. So `\cprotect[mm]\section*{\verb|\x|}` protects
//! nothing and errors in pdflatex (`! LaTeX Error: \verb illegal in argument`,
//! verified TL2025), the same as the plain `\cprotect\section*{…}` that the
//! default `om` signature cannot express either. Here the loop continues, so
//! `[mm]` is the working spelling for a starred sectioning command. This only
//! ever *adds* a case the package errors on — every signature it handles
//! correctly is handled identically. Plain `\cprotect\section*{…}` still
//! degrades exactly as the package does (the `*` is the `m` argument, the group
//! reaches `\section*` unprotected), so a document that works with pdflatex
//! converts the same way here.

use latexml_package::prelude::*;

/// cprotect's `\makeallother` stand-in: the characters whose catcode must be
/// neutralized while the protected argument is read, so that the text survives
/// to the `\scantokens` re-read intact. `{`/`}` are deliberately NOT in the
/// list — keeping them as BEGIN/END lets `read_balanced` find the end of the
/// argument, which is exactly what cprotect emulates by counting its own
/// `\CPT@other@bgroup`/`\CPT@other@egroup` in `\CPT@gobbleOneB`.
///
/// The space is included (cprotect makes it catcode 12 too): a run of spaces
/// inside `\verb|a  b|` must not collapse before the re-read.
///
/// So is `\r`, the `\endlinechar` the mouth appends to every line. At its usual
/// catcode 5 an end-of-line collapses to one space token and the argument comes
/// back as a single line — which silently flattens the package's *other*
/// headline case, `\cprotect\footnote{\begin{verbatim}…\end{verbatim}}`, into
/// one run-on line. As catcode 12 it survives into the `\scantokens` string,
/// where `Mouth::split_lines` breaks on it again. cprotect gets the same effect
/// from `\newlinechar`\``\^^M` in `\CPT@Write`.
static CPROTECT_OTHER: [char; 10] = ['\\', '%', '#', '$', '&', '^', '_', '~', ' ', '\r'];

/// Open a group in which [`CPROTECT_OTHER`] is neutralized; the caller pops it.
fn begin_protected_read() {
  push_frame();
  for special in CPROTECT_OTHER {
    assign_catcode(special, Catcode::OTHER, Some(Scope::Local));
  }
}

/// Read one balanced group whose opening `{` has already been consumed, with
/// [`CPROTECT_OTHER`] neutralized. Mirrors `\CPT@readContent` under
/// `\makeallother`.
fn read_protected_group() -> Result<Tokens> {
  begin_protected_read();
  let raw = read_balanced(ExpansionLevel::Off, false, false);
  pop_frame()?;
  raw
}

/// The `[...]`-delimited twin of [`read_protected_group`] (`\CPT@read@d` with
/// `#1#2` = `[`/`]`): peek for `open`, and when it is there read up to the
/// MATCHING `close` with [`CPROTECT_OTHER`] neutralized. Returns `None` (having
/// consumed nothing) when the optional argument is absent — `\CPT@read@d@none`.
///
/// The depth count is not decoration: `read_until` would stop at the first
/// `close`, whereas cprotect explicitly balances its delimiter pair in
/// `\CPT@gobbleOneB`, so `\cprotect\cmd[a[b]c]{…}` keeps its whole optional.
fn read_optional_protected(open: Token, close: Token) -> Result<Option<Tokens>> {
  let Some(token) = read_non_space()? else {
    return Ok(None);
  };
  if token != open {
    unread_one(token);
    return Ok(None);
  }
  begin_protected_read();
  // Scan inside a closure so the catcode frame is popped on the error path too.
  let scan = (|| -> Result<(Vec<Token>, bool)> {
    let mut raw: Vec<Token> = Vec::new();
    let mut depth = 1_usize;
    while let Some(token) = read_token()? {
      if token == close {
        depth -= 1;
        if depth == 0 {
          return Ok((raw, true));
        }
      } else if token == open {
        depth += 1;
      }
      raw.push(token);
    }
    Ok((raw, false))
  })();
  pop_frame()?;
  let (raw, closed) = scan?;
  if !closed {
    Error!(
      "expected",
      "delimiter",
      s!("Runaway \\cprotect optional argument, looking for {close}")
    );
  }
  Ok(Some(Tokens::new(raw)))
}

/// Wrap already-read raw tokens as `{\scantokens{<raw>%}}` — the port's stand-in
/// for cprotect's `{\protect\input \jobname-N.cpt\relax}` (`\CPT@read@mbeg`).
///
/// The trailing `%` is cprotect's `\CPT@postText` in LaTeXML dress. Re-reading
/// text through a mouth (`\input` there, `\scantokens` here) appends the
/// end-of-line, i.e. a spurious space at the end of every protected argument;
/// cprotect eats it by ending the written file with the `^^E^^L` control word
/// (a CS name absorbs the following space, and the CS itself is `\def`'d
/// empty). A comment character does the same job with the machinery we have.
fn scantokens_group(raw: Tokens) -> Vec<Token> {
  let mut out = vec![T_BEGIN!(), T_CS!("\\scantokens"), T_BEGIN!()];
  out.extend(raw.unlist());
  out.push(T_OTHER!("%"));
  out.push(T_END!());
  out.push(T_END!());
  out
}

LoadDefinitions!({
  // `\outer\long\def\cprotect{\icprotect}` + `\newcommand{\icprotect}[2][om]`
  // (cprotect.sty, `\icprotect` / `\CPT@read@args`): an OPTIONAL argument
  // signature, then the command token itself, then that command's own arguments
  // read per the signature.
  //
  // The optional is read here rather than declared as a `[om]` parameter so the
  // whole read stays in one place: the signature decides how many further
  // arguments to consume and each of them needs the neutralized-catcode read.
  DefMacro!("\\cprotect", {
    let argsig = read_optional(None)?
      .map(|sig| sig.to_string())
      // `\icprotect`'s `[om]` default: one optional, then one mandatory.
      .unwrap_or_else(|| "om".to_string());
    let Some(command) = read_non_space()? else {
      return Ok(Tokens!());
    };
    let mut out = vec![command];
    for arg_kind in argsig.chars() {
      match arg_kind {
        // `\CPT@read@o` = `\CPT@read@d[]`: protect a `[...]` when present,
        // otherwise consume nothing.
        'o' => {
          if let Some(raw) = read_optional_protected(T_OTHER!("["), T_OTHER!("]"))? {
            out.push(T_OTHER!("["));
            out.extend(scantokens_group(raw));
            out.push(T_OTHER!("]"));
          }
        },
        // `\CPT@read@m`: a `{...}` group is read raw and re-scanned
        // (`\CPT@read@mbeg`); a single token is passed straight through
        // (`\CPT@read@mone`) — there is nothing to protect in one token.
        'm' => match read_non_space()? {
          Some(token) if token.get_catcode() == Catcode::BEGIN => {
            out.extend(scantokens_group(read_protected_group()?));
          },
          Some(token) => out.push(token),
          None => break,
        },
        // cprotect also knows `d`, a user-chosen delimiter pair whose two
        // delimiters `\CPT@read@d` picks up from the input; an unknown letter is
        // a package error there. We are more forgiving in both cases: an
        // unhandled signature letter consumes nothing, so the command still
        // reaches the digester and reads that argument itself — unprotected,
        // but no worse than without `\cprotect`.
        _ => {},
      }
    }
    Ok(Tokens::new(out))
  });

  // `\newcommand{\cMakeRobust}[1]{...}` (cprotect.sty): stash the old meaning
  // under `\CPT@old@<name>` and redefine `<name>` to cprotect itself.
  DefPrimitive!("\\cMakeRobust DefToken", sub[(target)] {
    let name = target.to_string();
    let saved = T_CS!(&format!("\\CPT@old@{}", name.trim_start_matches('\\')));
    let_i(&saved, &target, None);
    // The interesting targets are precisely the ones the engine LOCKS —
    // `\section`/`\subsection`/… are `DefMacro!(…, locked => true)` so a
    // document's own `\renewcommand\section` cannot destroy the semantic
    // structure. `\cMakeRobust` is not that kind of redefinition: it keeps the
    // original meaning (under `\CPT@old@…`) and only wraps it, so bypassing the
    // lock here is the same "an addition, never a replacement of the body's
    // intent" case that `\g@addto@macro` unlocks for (Perl `Package.pm` L2527,
    // `local $UNLOCKED = 1`). The `:locked` flag itself is untouched, so a later
    // `\renewcommand\section` from the source is still ignored.
    let _unlock = local_state_unlocked_guard(true);
    def_macro(target, None, Tokens!(T_CS!("\\cprotect"), saved), None)?;
    Ok(())
  });

  // `\def\cprotEnv\begin{\CPTbegin}` + `\CPTbegin{#1}` re-emitting
  // `\begin{#1} \protect\input <file> \end{#1}`. In LaTeXML an environment body
  // is digested straight from the mouth — `\verb` inside `\begin{center}…` is
  // already read with live catcodes — so the write-and-read-back round trip is
  // a no-op. Expanding `\cprotEnv` to nothing leaves the following
  // `\begin{env}` in place, which is that same net effect.
  DefMacro!(T_CS!("\\cprotEnv"), None, Tokens!());
  Let!("\\CPTbegin", "\\begin");
});
