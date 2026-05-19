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
  // `<ltx:note role='nicematrix-placeholder'>(<name>)</ltx:note>` and discards the environment body
  // via discard_env_body. Paired `\end<name>` macros stay as `\relax`.
  DefConstructor!(T_CS!("\\begin{NiceTabular}"), None,
    "<ltx:note role=\"nicematrix-placeholder\">NiceTabular (nicematrix)</ltx:note>",
    bounded => true, mode => "text", locked => true,
    before_digest => { discard_env_body("NiceTabular", "nicematrix.sty.ltxml")?; });
  DefMacro!("\\endNiceTabular", "\\relax", locked => true);
  DefConstructor!(T_CS!("\\begin{NiceArray}"), None,
    "<ltx:note role=\"nicematrix-placeholder\">NiceArray (nicematrix)</ltx:note>",
    bounded => true, mode => "text", locked => true,
    before_digest => { discard_env_body("NiceArray", "nicematrix.sty.ltxml")?; });
  DefMacro!("\\endNiceArray", "\\relax", locked => true);
  DefConstructor!(T_CS!("\\begin{NiceMatrix}"), None,
    "<ltx:note role=\"nicematrix-placeholder\">NiceMatrix (nicematrix)</ltx:note>",
    bounded => true, mode => "text", locked => true,
    before_digest => { discard_env_body("NiceMatrix", "nicematrix.sty.ltxml")?; });
  DefMacro!("\\endNiceMatrix", "\\relax", locked => true);
  DefConstructor!(T_CS!("\\begin{NiceArrayWithDelims}"), None,
    "<ltx:note role=\"nicematrix-placeholder\">NiceArrayWithDelims (nicematrix)</ltx:note>",
    bounded => true, mode => "text", locked => true,
    before_digest => { discard_env_body("NiceArrayWithDelims", "nicematrix.sty.ltxml")?; });
  DefMacro!("\\endNiceArrayWithDelims", "\\relax", locked => true);
  DefConstructor!(T_CS!("\\begin{NiceTabular*}"), None,
    "<ltx:note role=\"nicematrix-placeholder\">NiceTabular* (nicematrix)</ltx:note>",
    bounded => true, mode => "text", locked => true,
    before_digest => { discard_env_body("NiceTabular*", "nicematrix.sty.ltxml")?; });
  DefConstructor!(T_CS!("\\begin{pNiceArray}"), None,
    "<ltx:note role=\"nicematrix-placeholder\">pNiceArray (nicematrix)</ltx:note>",
    bounded => true, mode => "text", locked => true,
    before_digest => { discard_env_body("pNiceArray", "nicematrix.sty.ltxml")?; });
  DefMacro!("\\endpNiceArray", "\\relax", locked => true);
  DefConstructor!(T_CS!("\\begin{pNiceMatrix}"), None,
    "<ltx:note role=\"nicematrix-placeholder\">pNiceMatrix (nicematrix)</ltx:note>",
    bounded => true, mode => "text", locked => true,
    before_digest => { discard_env_body("pNiceMatrix", "nicematrix.sty.ltxml")?; });
  DefMacro!("\\endpNiceMatrix", "\\relax", locked => true);
  DefConstructor!(T_CS!("\\begin{NiceTabularX}"), None,
    "<ltx:note role=\"nicematrix-placeholder\">NiceTabularX (nicematrix)</ltx:note>",
    bounded => true, mode => "text", locked => true,
    before_digest => { discard_env_body("NiceTabularX", "nicematrix.sty.ltxml")?; });
  DefMacro!("\\endNiceTabularX", "\\relax", locked => true);
  DefConstructor!(T_CS!("\\begin{bNiceArray}"), None,
    "<ltx:note role=\"nicematrix-placeholder\">bNiceArray (nicematrix)</ltx:note>",
    bounded => true, mode => "text", locked => true,
    before_digest => { discard_env_body("bNiceArray", "nicematrix.sty.ltxml")?; });
  DefMacro!("\\endbNiceArray", "\\relax", locked => true);
  DefConstructor!(T_CS!("\\begin{bNiceMatrix}"), None,
    "<ltx:note role=\"nicematrix-placeholder\">bNiceMatrix (nicematrix)</ltx:note>",
    bounded => true, mode => "text", locked => true,
    before_digest => { discard_env_body("bNiceMatrix", "nicematrix.sty.ltxml")?; });
  DefMacro!("\\endbNiceMatrix", "\\relax", locked => true);
  DefConstructor!(T_CS!("\\begin{BNiceArray}"), None,
    "<ltx:note role=\"nicematrix-placeholder\">BNiceArray (nicematrix)</ltx:note>",
    bounded => true, mode => "text", locked => true,
    before_digest => { discard_env_body("BNiceArray", "nicematrix.sty.ltxml")?; });
  DefMacro!("\\endBNiceArray", "\\relax", locked => true);
  DefConstructor!(T_CS!("\\begin{BNiceMatrix}"), None,
    "<ltx:note role=\"nicematrix-placeholder\">BNiceMatrix (nicematrix)</ltx:note>",
    bounded => true, mode => "text", locked => true,
    before_digest => { discard_env_body("BNiceMatrix", "nicematrix.sty.ltxml")?; });
  DefMacro!("\\endBNiceMatrix", "\\relax", locked => true);
  DefConstructor!(T_CS!("\\begin{vNiceArray}"), None,
    "<ltx:note role=\"nicematrix-placeholder\">vNiceArray (nicematrix)</ltx:note>",
    bounded => true, mode => "text", locked => true,
    before_digest => { discard_env_body("vNiceArray", "nicematrix.sty.ltxml")?; });
  DefMacro!("\\endvNiceArray", "\\relax", locked => true);
  DefConstructor!(T_CS!("\\begin{vNiceMatrix}"), None,
    "<ltx:note role=\"nicematrix-placeholder\">vNiceMatrix (nicematrix)</ltx:note>",
    bounded => true, mode => "text", locked => true,
    before_digest => { discard_env_body("vNiceMatrix", "nicematrix.sty.ltxml")?; });
  DefMacro!("\\endvNiceMatrix", "\\relax", locked => true);
  DefConstructor!(T_CS!("\\begin{VNiceArray}"), None,
    "<ltx:note role=\"nicematrix-placeholder\">VNiceArray (nicematrix)</ltx:note>",
    bounded => true, mode => "text", locked => true,
    before_digest => { discard_env_body("VNiceArray", "nicematrix.sty.ltxml")?; });
  DefMacro!("\\endVNiceArray", "\\relax", locked => true);
  DefConstructor!(T_CS!("\\begin{VNiceMatrix}"), None,
    "<ltx:note role=\"nicematrix-placeholder\">VNiceMatrix (nicematrix)</ltx:note>",
    bounded => true, mode => "text", locked => true,
    before_digest => { discard_env_body("VNiceMatrix", "nicematrix.sty.ltxml")?; });
  DefMacro!("\\endVNiceMatrix", "\\relax", locked => true);
  // Configuration entry-points — `\NiceMatrixOptions{...}` / paper-level
  // `\NewCollectionOfColumnsType{...}` etc. Their bodies set internal
  // keys controlling visual styling of NiceMatrix's diagrams (rules,
  // spacing, colors). Since we render NiceMatrix envs as placeholder
  // notes, these options are visually irrelevant. No-op stubs prevent
  // Error:undefined for papers that call them in their preamble.
  // Witness 2312.01047.
  def_macro_noop("\\NiceMatrixOptions{}")?;
  def_macro_noop("\\NewCollectionOfColumnsType{}{}")?;
  def_macro_noop("\\RenewCollectionOfColumnsType{}{}")?;
  def_macro_noop("\\nicematrixoptions{}")?;
});
