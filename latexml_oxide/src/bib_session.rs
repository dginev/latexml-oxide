//! The recursive BibTeX session behind post-processing's bibliography.
//!
//! Port of `LaTeXML::Post::MakeBibliography::convertBibliography`
//! (`MakeBibliography.pm` L171-242): a raw `.bib` is not parsed by a bespoke
//! reader, it is **converted** — by the same engine, through `BibTeX.pool`, with
//! the article's own class and packages preloaded so the fields mean what the
//! article meant.
//!
//! It lives here rather than in `latexml_post` because a conversion needs
//! `convert_document`, which needs this crate's model loader; `latexml_post`
//! depending on `latexml_oxide` would be a cycle. `latexml_post` declares the
//! hook, this module fills it in — see [`install`].

use latexml_core::{
  Core, CoreOptions,
  binding::content::{InputDefinitionOptions, input_definitions},
  common::{DigestionMode, error::*},
  s,
};
// The post-phase diagnostic macros: plain reporters, with none of the
// too-many-errors escalation the engine-side `Error!` carries. This module runs
// inside post-processing, and an inner-session failure must not itself become a
// document Fatal — the failure is already reported as an error, and Perl's
// convertBibliography likewise just returns empty-handed (L240-242).
use latexml_post::{
  Error, Info,
  document::{PostDocument, PostDocumentOptions},
  make_bibliography::{BibConversionRequest, RawBibSource, set_bib_converter},
};

use crate::core_interface::{DigestionAPI, DigestionOptions};

/// The commands a real `.bst`-generated `.bbl` provides before its entries.
///
/// BibTeX copies a field into the `.bbl`, whose preamble opens with a block of
/// `\providecommand`s so `\url{…}` in a `note` renders even in a document that
/// loads neither `hyperref` nor `url.sty`. We convert the `.bib` directly, one
/// step earlier, so without this block we raise `undefined:\url` where real
/// LaTeX renders text.
///
/// It is also what makes the percent-encoded-URL family work at all, and that is
/// the deeper reason to keep it: `\url` is declared with a **Semiverbatim**
/// parameter, so once it is defined, `\url{…/B130936%20Law%20of%20War.pdf}` reads
/// its argument with `%` at catcode 12. Measured against same-host Perl on the
/// same input: with `url.sty` present both engines emit 0 errors and identical
/// XML; without it both mis-read the `%`. So this block is not a Rust patch
/// papering over the engine — it is the condition under which the engine's own
/// parameter types can act.
///
/// `\providecommand` is the right primitive precisely because it defers: a
/// document that DOES load `hyperref` keeps hyperref's `\url`. Bodies mirror the
/// conventional `.bbl` definitions (plain/natbib/revtex all ship these shapes).
const BBL_STANDARD_FALLBACKS: &str = concat!(
  r"\providecommand{\url}[1]{\texttt{#1}}",
  r"\providecommand{\urlprefix}{URL }",
  r"\providecommand{\doi}[1]{doi:#1}",
  r"\providecommand{\bibinfo}[2]{#2}",
  r"\providecommand{\eprint}[2][]{\url{#2}}",
  r"\providecommand{\selectlanguage}[1]{\relax}",
  r"\providecommand{\newblock}{}",
);

/// Install this crate's recursive-session implementation into `latexml_post`.
///
/// Call once per thread before running the post-processing pipeline.
pub fn install() { set_bib_converter(convert); }

/// Assemble the sources into one conversion payload.
///
/// Perl `MakeBibliography.pm` L147-163: a lone file is converted as-is (so
/// locators name it), and several are concatenated into one `literal:` payload —
/// each followed by `%\n` — so that a single session sees them all and `@string`
/// macros defined in one file are in scope for the next.
///
/// The file reading deliberately does NOT use `read_to_string`: a strict UTF-8
/// read hard-errors on the first stray Cp1252 byte and would silently lose the
/// whole bibliography (witness 2605.00490). `decode_input_bytes` is the shared
/// decoder — UTF-8, else a lossless Latin-1 passthrough — that both `.bib` read
/// sites already use.
fn payload(sources: &[RawBibSource]) -> Option<String> {
  if let [RawBibSource::Path(path)] = sources {
    return Some(path.clone());
  }
  let mut combined = String::from("literal:");
  for source in sources {
    match source {
      RawBibSource::Literal(data) => combined.push_str(data),
      RawBibSource::Path(path) => match std::fs::read(path) {
        Ok(bytes) => combined.push_str(&latexml_core::mouth::decode_input_bytes(&bytes)),
        // Perl L162: `Info("open", ...)` — one unreadable file must not sink
        // the bibliographies that ARE readable.
        Err(e) => Info!(
          "open",
          path,
          "Couldn't open bibliography file {}: {}",
          path,
          e
        ),
      },
    }
    combined.push_str("%\n");
  }
  if combined == "literal:" {
    None
  } else {
    Some(combined)
  }
}

/// Make this thread's engine ready to digest BibTeX, **keeping the document's
/// own definitions**.
///
/// Perl spins a brand-new converter and rebuilds an approximation of the
/// article's environment from `[options]class.cls` + `[options]pkg.sty` preloads
/// recovered from the `<?latexml?>` PIs — and says so in a comment right above
/// `convertBibliography` (`MakeBibliography.pm` L174-177):
///
/// > In general, it should use the same STATE information as the main document,
/// > so IF that state is still around, we should use it! That's for future
/// > enhancement!!!
///
/// It is still around: post-processing runs in the same thread, right after
/// digestion. Using it is the enhancement Perl asks for, and it is strictly more
/// faithful to the *document*: preloads recover the class and packages but not
/// the author's own preamble macros, which is precisely what a `howpublished` or
/// `note` field tends to use (Perl's L184 comment — "custom macros often used in
/// e.g. howpublished field" — is the stated reason for preloading at all).
///
/// It also avoids two concrete hazards. A second `initialize_singletons` on one
/// thread re-runs the pools' `Let!`s (WISDOM #67 — that is a *non-unwinding*
/// abort, not a catchable failure), and it resets the `REPORT` the outer
/// document is still counting into, so Perl's `MergeStatus` would have to be
/// re-implemented by hand. Sharing the state makes both moot.
///
/// So all this needs is `BibTeX.pool` on top. The preloads are still consulted —
/// but only for the fallback path below, where there is no live session to
/// share (a `--post`-only run over an existing `.xml`, where nothing was ever
/// digested in this process).
fn ensure_session(request: &BibConversionRequest) -> Result<()> {
  if latexml_core::state::has_value("LATEXML_VERSION") {
    // Live session: add the pool the document never needed. Idempotent —
    // `input_definitions` early-returns on its own `_loaded` flag.
    input_definitions("BibTeX", InputDefinitionOptions {
      extension: Some("pool".into()),
      ..InputDefinitionOptions::default()
    })
  } else {
    // No conversion happened in this process (XML input). This IS the thread's
    // first initialization, so it is the safe case for `initialize_singletons`,
    // and Perl's preload reconstruction is exactly right here.
    let mut preloads = vec![s!("TeX.pool"), s!("BibTeX.pool")];
    preloads.extend(request.preloads.iter().cloned());
    let mut core = Core::new(CoreOptions {
      preload: Some(request.preloads.clone()),
      search_paths: Some(request.search_paths.clone()),
      ..CoreOptions::default()
    });
    core.initialize_singletons(preloads)
  }
}

/// Ensure `\url` exists, and exists with its REAL definition.
///
/// [`BBL_STANDARD_FALLBACKS`] alone would define it as
/// `\providecommand{\url}[1]{\texttt{#1}}` — an ordinary `{}` argument, which
/// renders but gives no catcode protection. Its argument is then read with `%`
/// at catcode 14, so a percent-encoded URL (`…/B130936%20Law%20of%20War.pdf` —
/// entirely routine in a `howpublished`) is truncated at the first `%`, taking
/// the closing brace with it.
///
/// Loading LaTeXML's own `url.sty` binding instead gives `\url` the
/// **Semiverbatim** parameter it is supposed to have (`url_sty.rs`), and the
/// truncation disappears. That is what a document which actually uses `\url`
/// would have had: a `.bst` emitting `\url{…}` into the `.bbl` assumes the
/// document loads `url` or `hyperref`, and nearly all do.
///
/// SURPASS-PERL, and deliberately: same-host Perl truncates this input (measured
/// — `latexmlc` renders `https://example.org/B130936` and spills the raw entry
/// into the note), because it defines nothing here at all. So does pdflatex
/// without a url package. This continues the beyond-Perl content-recovery line
/// already taken by the `.bbl` fallback block itself; it does not silence any
/// diagnostic Perl raises for a *correct* document, since a document that loads
/// `url`/`hyperref` keeps its own definition — `input_definitions` early-returns
/// on the `_loaded` flag, and `\providecommand` defers.
fn provide_url_command() -> Result<()> {
  if latexml_core::state::lookup_meaning(&latexml_core::T_CS!("\\url")).is_some() {
    return Ok(());
  }
  input_definitions("url", InputDefinitionOptions {
    extension: Some("sty".into()),
    handleoptions: true,
    ..InputDefinitionOptions::default()
  })
}

/// Run the recursive session and hand back its document.
fn convert(request: &BibConversionRequest) -> Option<PostDocument> {
  let source = payload(&request.sources)?;
  let stage = if source.starts_with("literal:") {
    s!("Recursive MakeBibliography Anonymous Bib String")
  } else {
    s!("Recursive MakeBibliography {source}")
  };
  note_begin(&stage);

  // The document's OWN state, still live, is the session — see the module docs
  // for why that is both better and cheaper than a fresh one. `Core` is a thin
  // handle over the thread-local engine (`Core::new` would install a *new*
  // State and throw the document's away), so it is constructed directly.
  let mut core = Core {
    preload: request.preloads.clone(),
  };

  let result = (|| -> Result<PostDocument> {
    ensure_session(request)?;
    // Perl builds a FRESH `catcodes => 'standard'` State per `.bib` (`Core.pm:39`),
    // where `@` is catcode OTHER. We REUSE the live document State (module docs),
    // so any catcode the document left non-standard leaks into the `.bib`
    // digestion. `@` is the one that matters: the emitted
    // `\begin{bibtex@bibliography}` wrapper carries `@` in its environment name, so
    // a document that leaves `@` ACTIVE — e.g. an unmatched `\makeatletter` or the
    // `\catcode`@=\active`+`\def@#1@{…}` red-annotation shorthand of
    // arXiv:2606.03480 (an unmatched `\shorthandon`) — makes that `@` EXPAND
    // mid-name, giving `readBalanced ran out of input` and losing the whole
    // bibliography. Reset `@` to its standard catcode to match Perl's fresh state.
    // Guard: `active_at_catcode_does_not_corrupt_bibtex_wrapper`.
    latexml_core::state::assign_catcode(
      '@',
      latexml_core::token::Catcode::OTHER,
      Some(latexml_core::state::Scope::Global),
    );
    // Give the fields what a `.bbl` would have given them, once the session is
    // ready and before any entry is digested.
    provide_url_command()?;
    latexml_core::stomach::digest(latexml_core::mouth::tokenize(BBL_STANDARD_FALLBACKS))?;

    // Digest only what the document cites — a `.bib` is a library, and
    // `bibtex(1)` reads the `.aux`'s `\citation` records rather than the whole
    // file. Set around `digest_file` only: the filter lives in the same
    // thread-local State the outer document uses, so leaving it installed
    // would leak into any later `.bib` in this process.
    latexml_engine::pre_bibtex::set_wanted_keys(request.wanted_keys.clone());
    // Hide the OUTER document's bibliography count for the duration of the
    // recursive digestion — the one value reusing the live State would
    // otherwise leak (see the module docs for why the State is shared).
    // `{bibtex@bibliography}` runs `begin_bibliography_clean`, which does
    // `n_bibliographies += 1` and names the session's ids
    // `<docid>bib<radix_alpha(n-1)>` (latex_constructs.rs:2392-2398). Perl
    // builds a FRESH State per `.bib` (MakeBibliography.pm:181-215), so its
    // inner session always sees 0 and mints `bib.bibN`; sharing ours made the
    // FIRST `.bib` of a one-bibliography document mint `biba.bibN` — the
    // second slot of the multi-bibliography sequence. Post then strips `^bib`
    // and re-prepends the OUTER bibid (MakeBibliography.pm:407-415), which is
    // exactly why the inner ids must be `bib`-rooted regardless of which
    // bibliography they end up in.
    let outer_n_bibliographies = latexml_core::state::lookup_int("n_bibliographies");
    latexml_core::state::assign_value(
      "n_bibliographies",
      0,
      Some(latexml_core::state::Scope::Global),
    );
    let digested = core.digest_file(source.clone(), DigestionOptions {
      mode: Some(DigestionMode::BibTeX),
      noinitialize: Some(true),
      ..DigestionOptions::default()
    });
    latexml_core::state::assign_value(
      "n_bibliographies",
      outer_n_bibliographies,
      Some(latexml_core::state::Scope::Global),
    );
    latexml_engine::pre_bibtex::clear_wanted_keys();
    let digested = digested?;
    let document = core.convert_document(digested)?;
    Ok(PostDocument::new(document.document, PostDocumentOptions {
      source_directory: Some(".".to_string()),
      ..PostDocumentOptions::default()
    }))
  })();

  // Perl `MergeStatus` (L237) ADDS the inner session's tally to the outer
  // document's (Common/Error.pm L669-686) — a `.bib` that raises errors makes
  // the document an error document, which is what the corpus signal depends on.
  // Sharing the live state gives that for free: the counters never left.
  note_end(&stage);

  match result {
    Ok(bibdoc) => Some(bibdoc),
    Err(e) => {
      Error!(
        "bibliography",
        "convert",
        "Recursive bibliography conversion failed: {}",
        e
      );
      None
    },
  }
}
