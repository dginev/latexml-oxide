# Change Log

## [0.1.8] (in active development)

  - work on passing `tokenize/verb` test
  - `DefParameter` has an `untokenized` flag that acts as a type designator. Unrealistic ergonomics in Rust. Instead, augment the `reader` paradigm with an optional follow-up closure called `predigest`, which has access to the stomach and can be ran immediately after a `read` is completed.
   

## [0.1.7] 2018-24-12
  - pass `tokenize/percent` and `tokenize/url` test 
  - Much improved `Def*` macro ergonomics since 0.1.4
  - Fleshed out more coverage, cleared some porting bugs in tokenization,
  - in particular `url.sty` and related bits of tex and latex pool files

## [0.1.4] 2018-27-08
  - First optimization release

