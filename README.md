# rtx
A Rust port of [LaTeXML](https://github.com/brucemiller/latexml)

[![CI](https://github.com/dginev/rtx/actions/workflows/CI.yml/badge.svg)](https://github.com/dginev/rtx/actions/workflows/CI.yml) ![version](https://img.shields.io/badge/version-0.2.0-orange.svg)

## Progress

[![test porting progress](http://progressed.io/bar/10)](https://github.com/dginev/rtx/issues/30) | 23 of ? core LaTeXML tests PASS

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

### Updates

 * `v0.1.12` is now capable of passing the `complex/xii.tex` test, which accurately converts the [infamous snippet](http://ctan.org/pkg/xii):

  ```
  \let~\catcode~`76~`A13~`F1~`j00~`P2jdefA71F~`7113jdefPALLF
  PA''FwPA;;FPAZZFLaLPA//71F71iPAHHFLPAzzFenPASSFthP;A$$FevP
  A@@FfPARR717273F737271P;ADDFRgniPAWW71FPATTFvePA**FstRsamP
  AGGFRruoPAqq71.72.F717271PAYY7172F727171PA??Fi*LmPA&&71jfi
  Fjfi71PAVVFjbigskipRPWGAUU71727374 75,76Fjpar71727375Djifx
  :76jelse&U76jfiPLAKK7172F71l7271PAXX71FVLnOSeL71SLRyadR@oL
  RrhC?yLRurtKFeLPFovPgaTLtReRomL;PABB71 72,73:Fjif.73.jelse
  B73:jfiXF71PU71 72,73:PWs;AMM71F71diPAJJFRdriPAQQFRsreLPAI
  I71Fo71dPA!!FRgiePBt'el@ lTLqdrYmu.Q.,Ke;vz vzLqpip.Q.,tz;
  ;Lql.IrsZ.eap,qn.i. i.eLlMaesLdRcna,;!;h htLqm.MRasZ.ilk,%
  s$;z zLqs'.ansZ.Ymi,/sx ;LYegseZRyal,@i;@ TLRlogdLrDsW,@;G
  LcYlaDLbJsW,SWXJW ree @rzchLhzsW,;WERcesInW qt.'oL.Rtrul;e
  doTsW,Wk;Rri@stW aHAHHFndZPpqar.tridgeLinZpe.LtYer.W,:
  ```

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

Requires Rust `stable` v1.32, and newer.

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
