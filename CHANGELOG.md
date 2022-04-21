# Change Log

## [0.2.1] (in active development)
  - continue increasing test coverage

## [0.2]
  - update to 03.2022 state of the mainline LaTeXML test suite
  - unblock math parsing with the inclusion of a Marpa grammar
  - pass most of `tokenize` and `grouping` tests
  - `DefParameter` has an `untokenized` flag that acts as a type designator. Unrealistic ergonomics in Rust. Instead, augment the `reader` paradigm with an optional follow-up closure called `reader_predigest`, which has access to the stomach and can be ran immediately after a `read` is completed. One can still use an `reader_predigest => undigested!()` macro call to allow arguments to pass through digestion untouched.
  - Note: "SEARCHPATHS" no longer needs to be looked up, it's in `state.search_paths`



## [0.1.7] 2018-24-12
  - pass `tokenize/percent` and `tokenize/url` test
  - Much improved `Def*` macro ergonomics since 0.1.4
  - Fleshed out more coverage, cleared some porting bugs in tokenization,
  - in particular `url.sty` and related bits of tex and latex pool files

## [0.1.4] 2018-27-08
  - First optimization release
