# rtx
A Rust port of [LaTeXML](https://github.com/brucemiller/latexml)

[![CI](https://github.com/dginev/rtx/actions/workflows/CI.yml/badge.svg)](https://github.com/dginev/rtx/actions/workflows/CI.yml) ![version](https://img.shields.io/badge/version-0.3.1-orange.svg)

## Porting Progress

[![test porting progress](https://progress-bar.dev/26/?title=passing%20tests)](https://github.com/dginev/rtx/issues/30)

### Why?

The three main reasons:

  * LaTeXML is **too slow** for large-scale production use.
    - A recent independent quote from a [BIR 2019 paper](http://ceur-ws.org/Vol-2345/paper2.pdf):

      <img alttext="latexml vs tralics" src="https://i.imgur.com/6iOyCDo.png" width=600>

    - Recent lamentations from social media:

       <img alttext="latexml too slow" src="https://i.imgur.com/lOOtSWa.png" width=300>

    - Recent request from a PhD student who maintains a dataset and wrote in for support:
      > With the newer version of LatexML, we are getting lower conversion failures. However, I have to run it on the whole collection and usually the other organizers who are also my advisors blame me for why it is taking so long to convert them!!

  * Perl 5 has **no street cred** anymore.
  * LaTeXML is **urgently needed** for transporting technical writing to the web and e-printing media.

Design goals:

  * Faithfully rewrite the LaTeXML code base as-is, attempting to be as close as possible to the original Perl sources.
  * Use idiomatic Rust when possible, especially when refactoring Perl idioms
  * Carefully address the newly required resource constraints in design, memory use and runtime

There is demonstrable need for LaTeXML in the domain of academic writing, as well as various research areas on math-heavy documents. So here is a fast, safe, hip reimplementation that can usher LaTeXML uses in the 2020s.

### Fake Benchmark
 These are the times of different TeX-like engines ran over the `xii.tex` example above. That is not representative to all of TeX, but gives a minimal early feeling.
 It will be a lot more telling to provide tikz and expl3 runtime numbers.

| executable | time      |
|------------|-----------|
| tralics    |  0.011s   |
| rtx        |  0.033s   |
| tex        |  0.096s   |
| pdftex     |  0.215s   |
| luatex     |  0.226s   |
| xetex      |  0.430s   |
| httex      |  0.608s   |
| latexml    |  0.745s   |

### Installation

Requires Rust `nightly` v1.72, and newer.

### Sample use

1. Make sure the tests pass first, via
    ```bash
    $ cargo test --release --tests
    ```

2. convert an example formula:
    ```bash
    $ cargo run --release --bin rtxmath '1+1=2'
    ```

3. convert an example document:
    ```bash
    $ cargo run --release --bin rtx rtx/tests/hello/hello.tex
    ```

### Docs

To generate the project documentation locally, run:
```bash
$ cargo doc --workspace --no-deps --open
```