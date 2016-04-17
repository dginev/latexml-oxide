# rustexml
A Rust port of LaTeXML. Why?

The two main reasons:

  * LaTeXML is **too slow** for large-scale production use.
  * Perl 5 has **no street cred** anymore.

Design goals:

  * Faithfully rewrite the LaTeXML code base as-is, attempting to be as close as possible to the original Perl sources.
  * Use idiomatic Rust when possible, especially when refactoring idiomatic Perl idioms
  * Carefully address the newly required memory considerations in design and runtime

There is demonstrable need for LaTeXML in the domain of academic writing, as well as various research areas on math-heavy documents. So here is a fast, safe, hip reimplementation that can usher LaTeXML uses in the 2020s.