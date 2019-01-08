# rtx
A Rust port of [LaTeXML](https://github.com/brucemiller/latexml)

[![Build Status](https://travis-ci.com/dginev/rtx.svg?token=JKuszfgzJQJUFzH9JaLd&branch=master)](https://travis-ci.com/dginev/rtx#) ![version](https://img.shields.io/badge/version-0.1.10-orange.svg)

### Why?

The two main reasons:

  * LaTeXML is **too slow** for large-scale production use.
  * Perl 5 has **no street cred** anymore.

Design goals:

  * Faithfully rewrite the LaTeXML code base as-is, attempting to be as close as possible to the original Perl sources.
  * Use idiomatic Rust when possible, especially when refactoring Perl idioms
  * Carefully address the newly required resource constraints in design, memory use and runtime

There is demonstrable need for LaTeXML in the domain of academic writing, as well as various research areas on math-heavy documents. So here is a fast, safe, hip reimplementation that can usher LaTeXML uses in the 2020s.
