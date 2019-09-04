# rtx
A Rust port of [LaTeXML](https://github.com/brucemiller/latexml)

[![Build Status](https://travis-ci.com/dginev/rtx.svg?token=JKuszfgzJQJUFzH9JaLd&branch=master)](https://travis-ci.com/dginev/rtx#) ![version](https://img.shields.io/badge/version-0.1.21-orange.svg)
 [![Dependabot Status](https://api.dependabot.com/badges/status?host=github&repo=dginev/rtx&identifier=44036543)](https://dependabot.com)

## Progress

[![test porting progress](http://progressed.io/bar/10)](https://github.com/dginev/rtx/issues/30) | 21 of 207 core LaTeXML tests PASS

### Why?

The three main reasons:

  * LaTeXML is **too slow** for large-scale production use.
    - A recent independent quote from a [BIR 2019 paper](http://ceur-ws.org/Vol-2345/paper2.pdf):
     ![latexml vs tralics](https://i.imgur.com/6iOyCDo.png)
  
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
 These are the times of different TeX-like engines ran over the `xii.tex` example above. That is not representative to all of TeX, but gives a good early feeling:

| executable | time      |
|------------|-----------|
| tralics    |  0.011s   |
| rtx        |  0.033s   |
| tex        | 	0.067s   |
| pdftex     |  0.125s   |
| luatex     |  0.158s   |
| xetex      |  0.310s   |
| httex      |  0.333s   |
| latexml    |  0.562s   |

### Installation

Requires Rust `stable` v1.32, and newer.
