//! ltxtable.sty — `\LTXtable{width}{file}`: a longtable with tabularx `X`
//! columns whose body lives in a separate file.
//!
//! The real macro (ltxtable.sty:9-56) runs tabularx's trial loop over
//! `\input{#2}` (a `longtable` environment) with longtable's `\LT@echunk`/
//! `\LT@get@widths` chunk machinery to converge the `X` widths, then inputs
//! the file once more for real. Neither Perl nor Rust had a binding, and the
//! raw macro reaches `\TX@target`/`\TX@col@width`/`\TX@newcol`/`\LT@echunk`/
//! `\LT@get@widths` — internals the tabularx/longtable bindings do not model
//! (tikzcodeblocks-documentation 64 errors, vhistory examples). As with the
//! xltabular binding, `X` is a global column type once tabularx is loaded and
//! longtable's alignment accepts it, so the width trial is unnecessary here:
//! `\LTXtable` sets the target width for the record and inputs the file once.
use latexml_package::prelude::*;

LoadDefinitions!({
  RequirePackage!("tabularx");
  RequirePackage!("longtable");
  // ltxtable.sty:8 `\TX@target#1\relax` is the width tabularx distributes
  // over the X columns; keep it as a real dimen so packages reading it
  // (`\TX@target`, `\TX@col@width`) find a length.
  RawTeX!(
    r"\ifx\TX@target\@undefined\newdimen\TX@target\fi\ifx\TX@col@width\@undefined\newdimen\TX@col@width\fi"
  );
  DefMacro!(
    "\\LTXtable{}{}",
    r"\begingroup\TX@target#1\relax\TX@col@width\TX@target\input{#2}\endgroup"
  );
});
