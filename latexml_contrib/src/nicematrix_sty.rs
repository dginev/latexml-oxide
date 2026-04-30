use latexml_package::prelude::*;

use crate::discard_env::discard_env_body;

#[rustfmt::skip]
LoadDefinitions!({
  RequirePackage!("pgfcore");
  RequirePackage!("amsmath");
  RequirePackage!("array");
  Warn!(
    "missing_file",
    "nicematrix.sty",
    "nicematrix.sty is not implemented and will not be interpreted raw."
  );

  // Perl ar5iv-bindings/nicematrix.sty.ltxml L43-126: sixteen
  // DefConstructorI entries wrapping `\begin{<name>}`. Each emits
  // `<ltx:ERROR>{<name>}</ltx:ERROR>` and discards the environment body
  // via discard_env_body. Paired `\end<name>` macros stay as `\relax`.
  DefConstructor!(T_CS!("\\begin{NiceTabular}"), None,
    "<ltx:ERROR>{NiceTabular}</ltx:ERROR>",
    bounded => true, mode => "text", locked => true,
    before_digest => { discard_env_body("NiceTabular", "nicematrix.sty.ltxml")?; });
  DefMacro!("\\endNiceTabular", "\\relax", locked => true);
  DefConstructor!(T_CS!("\\begin{NiceArray}"), None,
    "<ltx:ERROR>{NiceArray}</ltx:ERROR>",
    bounded => true, mode => "text", locked => true,
    before_digest => { discard_env_body("NiceArray", "nicematrix.sty.ltxml")?; });
  DefMacro!("\\endNiceArray", "\\relax", locked => true);
  DefConstructor!(T_CS!("\\begin{NiceMatrix}"), None,
    "<ltx:ERROR>{NiceMatrix}</ltx:ERROR>",
    bounded => true, mode => "text", locked => true,
    before_digest => { discard_env_body("NiceMatrix", "nicematrix.sty.ltxml")?; });
  DefMacro!("\\endNiceMatrix", "\\relax", locked => true);
  DefConstructor!(T_CS!("\\begin{NiceArrayWithDelims}"), None,
    "<ltx:ERROR>{NiceArrayWithDelims}</ltx:ERROR>",
    bounded => true, mode => "text", locked => true,
    before_digest => { discard_env_body("NiceArrayWithDelims", "nicematrix.sty.ltxml")?; });
  DefMacro!("\\endNiceArrayWithDelims", "\\relax", locked => true);
  DefConstructor!(T_CS!("\\begin{NiceTabular*}"), None,
    "<ltx:ERROR>{NiceTabular*}</ltx:ERROR>",
    bounded => true, mode => "text", locked => true,
    before_digest => { discard_env_body("NiceTabular*", "nicematrix.sty.ltxml")?; });
  DefConstructor!(T_CS!("\\begin{pNiceArray}"), None,
    "<ltx:ERROR>{pNiceArray}</ltx:ERROR>",
    bounded => true, mode => "text", locked => true,
    before_digest => { discard_env_body("pNiceArray", "nicematrix.sty.ltxml")?; });
  DefMacro!("\\endpNiceArray", "\\relax", locked => true);
  DefConstructor!(T_CS!("\\begin{pNiceMatrix}"), None,
    "<ltx:ERROR>{pNiceMatrix}</ltx:ERROR>",
    bounded => true, mode => "text", locked => true,
    before_digest => { discard_env_body("pNiceMatrix", "nicematrix.sty.ltxml")?; });
  DefMacro!("\\endpNiceMatrix", "\\relax", locked => true);
  DefConstructor!(T_CS!("\\begin{NiceTabularX}"), None,
    "<ltx:ERROR>{NiceTabularX}</ltx:ERROR>",
    bounded => true, mode => "text", locked => true,
    before_digest => { discard_env_body("NiceTabularX", "nicematrix.sty.ltxml")?; });
  DefMacro!("\\endNiceTabularX", "\\relax", locked => true);
  DefConstructor!(T_CS!("\\begin{bNiceArray}"), None,
    "<ltx:ERROR>{bNiceArray}</ltx:ERROR>",
    bounded => true, mode => "text", locked => true,
    before_digest => { discard_env_body("bNiceArray", "nicematrix.sty.ltxml")?; });
  DefMacro!("\\endbNiceArray", "\\relax", locked => true);
  DefConstructor!(T_CS!("\\begin{bNiceMatrix}"), None,
    "<ltx:ERROR>{bNiceMatrix}</ltx:ERROR>",
    bounded => true, mode => "text", locked => true,
    before_digest => { discard_env_body("bNiceMatrix", "nicematrix.sty.ltxml")?; });
  DefMacro!("\\endbNiceMatrix", "\\relax", locked => true);
  DefConstructor!(T_CS!("\\begin{BNiceArray}"), None,
    "<ltx:ERROR>{BNiceArray}</ltx:ERROR>",
    bounded => true, mode => "text", locked => true,
    before_digest => { discard_env_body("BNiceArray", "nicematrix.sty.ltxml")?; });
  DefMacro!("\\endBNiceArray", "\\relax", locked => true);
  DefConstructor!(T_CS!("\\begin{BNiceMatrix}"), None,
    "<ltx:ERROR>{BNiceMatrix}</ltx:ERROR>",
    bounded => true, mode => "text", locked => true,
    before_digest => { discard_env_body("BNiceMatrix", "nicematrix.sty.ltxml")?; });
  DefMacro!("\\endBNiceMatrix", "\\relax", locked => true);
  DefConstructor!(T_CS!("\\begin{vNiceArray}"), None,
    "<ltx:ERROR>{vNiceArray}</ltx:ERROR>",
    bounded => true, mode => "text", locked => true,
    before_digest => { discard_env_body("vNiceArray", "nicematrix.sty.ltxml")?; });
  DefMacro!("\\endvNiceArray", "\\relax", locked => true);
  DefConstructor!(T_CS!("\\begin{vNiceMatrix}"), None,
    "<ltx:ERROR>{vNiceMatrix}</ltx:ERROR>",
    bounded => true, mode => "text", locked => true,
    before_digest => { discard_env_body("vNiceMatrix", "nicematrix.sty.ltxml")?; });
  DefMacro!("\\endvNiceMatrix", "\\relax", locked => true);
  DefConstructor!(T_CS!("\\begin{VNiceArray}"), None,
    "<ltx:ERROR>{VNiceArray}</ltx:ERROR>",
    bounded => true, mode => "text", locked => true,
    before_digest => { discard_env_body("VNiceArray", "nicematrix.sty.ltxml")?; });
  DefMacro!("\\endVNiceArray", "\\relax", locked => true);
  DefConstructor!(T_CS!("\\begin{VNiceMatrix}"), None,
    "<ltx:ERROR>{VNiceMatrix}</ltx:ERROR>",
    bounded => true, mode => "text", locked => true,
    before_digest => { discard_env_body("VNiceMatrix", "nicematrix.sty.ltxml")?; });
  DefMacro!("\\endVNiceMatrix", "\\relax", locked => true);
});
