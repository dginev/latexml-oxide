//! Shared helper: discard an environment body by reading tokens up to
//! matching `\end{kind}`. Mirrors the Perl `discard_env_body` closures
//! used in ar5iv-bindings/nicematrix / forest / diagrams / pb-diagram
//! stub packages.
//!
//! Perl reference (identical across those stubs):
//! ```perl
//! sub discard_env_body {
//!   my ($stomach, $kind) = @_;
//!   my $gullet = $stomach->getGullet;
//!   $stomach->bgroup;
//!   if (!$reported{$kind}) {
//!     $reported{$kind} = 1;
//!     Error('undefined', "{$kind}", $gullet,
//!       "$kind has no support, this is a stub binding."); }
//!   while (my $_ = $gullet->readUntil(T_CS('\end'))) {
//!     my $_drop_open = $gullet->readToken;
//!     my $env        = $gullet->readBalanced;
//!     last if ToString($env) eq $kind; }
//!   $stomach->egroup;
//!   return; }
//! ```

use std::cell::RefCell;

use latexml_package::prelude::*;
use rustc_hash::FxHashSet as HashSet;

thread_local! {
  /// One error per kind per conversion run, matching Perl's
  /// `our $reported{$kind}` cache inside each stub package.
  static REPORTED: RefCell<HashSet<String>> = RefCell::new(HashSet::default());
}

/// Read the raw (unexpanded) tokens of the current environment body up to
/// the first `\end{kind}`, which is consumed and not returned; an `\end` of
/// another environment inside the body is kept. The one mechanism behind
/// every binding that captures or discards a body wholesale (forest's bracket
/// parser, animate's frame discard, the stub discards below) — batch 56bc.
/// `read_balanced(…, false, false)`: the `{` after `\end` was just consumed,
/// so the balanced read starts inside it (Perl's argless `readBalanced`).
pub fn read_env_body_tokens(kind: &str) -> Result<Vec<Token>> {
  let end_delim = Tokens!(T_CS!("\\end"));
  let mut body: Vec<Token> = Vec::new();
  loop {
    if let Some(toks) = read_until(&end_delim)? {
      body.extend(toks.unlist());
    }
    let Some(open) = read_token()? else {
      break; // end of input
    };
    let env = read_balanced(ExpansionLevel::Off, false, false)?;
    if env.to_string() == kind {
      break;
    }
    body.push(T_CS!("\\end"));
    body.push(open);
    body.extend(env.unlist());
    body.push(T_END!());
  }
  Ok(body)
}

/// Read and discard the body up to and including the matching `\end{kind}`,
/// with a one-time stub warning per `kind`.
pub fn discard_env_body(kind: &str, source: &str) -> Result<()> {
  bgroup();
  report_stub_once(kind, source)?;
  let _body = read_env_body_tokens(kind)?;
  egroup()?;
  Ok(())
}

/// The one-per-kind stub diagnostic shared by the discard helpers.
fn report_stub_once(kind: &str, source: &str) -> Result<()> {
  let first_time = REPORTED.with(|cell| {
    let mut set = cell.borrow_mut();
    if set.contains(kind) {
      false
    } else {
      set.insert(kind.to_string());
      true
    }
  });
  if first_time {
    let obj = format!("{{{}}}", kind);
    let msg = format!(
      "{} has no support in {}, this is a stub binding.",
      kind, source
    );
    // A Warn, not an Error: the body IS discarded cleanly, and the ar5iv
    // binding's Error (forest.sty.ltxml:37-38) was the only diagnostic of
    // forest-quickstart, fragoli_doc and milsymb (pdflatex clean; sweep #41).
    // Guard: `perfect_kernel_batch56::forest_stub_is_a_warning`.
    Warn!("undefined", &obj, msg);
  }
  Ok(())
}

/// Discard the body of the BARE-CS environment form `\<kind> … \end<kind>`
/// (what `\NewDocumentEnvironment` defines alongside `\begin{<kind>}`):
/// raw, non-expanding scan up to the `end_cs` token, sharing the
/// one-per-kind stub report with [`discard_env_body`].
pub fn discard_body_until_cs(kind: &str, end_cs: &str, source: &str) -> Result<()> {
  bgroup();
  report_stub_once(kind, source)?;
  let _body = read_until(&Tokens!(T_CS!(end_cs)))?;
  egroup()?;
  Ok(())
}
