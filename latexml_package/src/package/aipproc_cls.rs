use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: aipproc.cls.ltxml
  // American Institute of Physics Conference Proceedings.

  // Ignorable options
  for option in [
    "10pt", "11pt", "12pt", "twoside", "onecolumn", "twocolumn",
    "draft", "final", "referee",
    "letter",
  ].iter() {
    DeclareOption!(*option, None);
  }

  // Anything else gets passed to article.
  DeclareOption!(None, {
    Digest!("\\PassOptionsToClass{\\CurrentOption}{article}")?;
  });

  // Font options — store choice in state
  DeclareOption!("mathptmx", { AssignValue!("aipproc_font" => "mathptmx"); });
  DeclareOption!("mathptm",  { AssignValue!("aipproc_font" => "mathptm"); });
  DeclareOption!("mathtime", { AssignValue!("aipproc_font" => "mathptmx"); });
  DeclareOption!("mtpro",    { AssignValue!("aipproc_font" => "mathptmx"); });

  DeclareOption!("varioref",    None);
  DeclareOption!("nonvarioref", None);

  DeclareOption!("tnotealph",   None);
  DeclareOption!("tnotesymbol", None);

  DeclareOption!("numberedheadings",   None);
  DeclareOption!("unnumberedheadings", None);

  // Default font
  AssignValue!("aipproc_font" => "mathptmx");

  ProcessOptions!();
  load_class("article", Vec::new(), Tokens!())?;

  // Load the selected font package
  let font_pkg = state::lookup_value("aipproc_font")
    .map(|v| v.to_string())
    .unwrap_or_else(|| "mathptmx".to_string());
  if !font_pkg.is_empty() {
    require_package(&font_pkg, RequireOptions::default())?;
  }

  RequirePackage!("fixltx2e");
  RequirePackage!("fontenc");
  RequirePackage!("calc");
  RequirePackage!("varioref");
  RequirePackage!("times");
  RequirePackage!("graphicx");
  RequirePackage!("textcomp");
  RequirePackage!("url");
  RequirePackage!("textcase");
  RequirePackage!("natbib");

  //======================================================================
  // Frontmatter
  DefMacro!("\\layoutstyle{}", "");

  // keywords: address, altaddress, email
  // Perl: DefMacro('\author{} RequiredKeyVals', sub { ... complex Perl ... });
  // Simplified: define \author with keyvals support
  // The Perl version extracts address, altaddress, email from keyvals
  // and wraps them in \lx@author / \lx@contact invocations.
  // For now, provide the basic author definition.
  DefMacro!("\\author{} RequiredKeyVals", "\\lx@author{#1}");

  DefMacro!("\\keywordsname", "Keywords");
  DefMacro!("\\keywords{}", "\\@add@frontmatter{ltx:keywords}[name={\\keywordsname}]{#1}");
  DefMacro!("\\classification{}", "\\@add@frontmatter{ltx:classification}{#1}");

  DefEnvironment!("{theacknowledgments}",
    "<ltx:acknowledgements>#body</ltx:acknowledgements>");

  //======================================================================
  DefConstructor!("\\eqref Semiverbatim",
    "(<ltx:ref labelref='#label' _force_font='true'/>)",
    properties => sub[args] {
      unpack_opt_ref!(args => label_opt);
      let label = label_opt.as_ref().unwrap().to_string();
      Ok(stored_map!("label" => Stored::String(arena::pin(clean_label(&label, None)))))
    },
    enter_horizontal => true
  );

  //======================================================================
  DefMacro!("\\source{}", "\\lx@note{source}{#1}");
  DefMacro!("\\spaceforfigure{}{}", "");

  DefMacro!("\\tablehead{}{}{}{}", "\\multicolumn{#1}{#2}{\\parbox{#3}{#4}}");
  DefMacro!("\\tablenote OptionalMatch:* {}", "\\footnote{#2}");
});
