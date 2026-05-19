//! moderncv.cls — Modern CV class
//! Perl: moderncv.cls.ltxml (115 lines)
use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: LoadClass('article');
  load_class("article", Vec::new(), Tokens!())?;

  RequirePackage!("calc");
  RequirePackage!("ifthen");
  RequirePackage!("url");
  // Pre-load xcolor with [dvipsnames, table] options so user xcolor
  // calls don't silently option-clash and miss dvipsnam.def/colortbl.
  RequirePackage!("xcolor", options => vec!["dvipsnames".to_string(), "table".to_string()]);
  RequirePackage!("fancyhdr");
  RequirePackage!("hyperref");

  RequireResource!("ltx-cv.css");

  TeX!(r"\@add@frontmatter{ltx:creator}[role=cv]{}");

  DefConstructor!("\\@@@address{}", "^ <ltx:contact role='address'>#1</ltx:contact>");
  DefMacro!("\\address{}{}", "\\@add@to@frontmatter{ltx:creator}{\\@@@address{#1\\newline #2}}");

  def_macro_noop("\\addressfont")?;
  def_macro_noop("\\addressstyle")?;
  def_macro_noop("\\addresssymbol")?;

  def_macro_noop("\\cvcolumn")?;
  def_macro_noop("\\cvcolumncell")?;
  def_macro_noop("\\cvdoubleitem")?;

  // Perl L43, L49: enterHorizontal => 1 on both.
  DefConstructor!("\\cventry{}{}{}{}{}{}",
    "<ltx:para class='ltx_cv_entry'><ltx:block class='ltx_cv_entry_date'>#1</ltx:block><ltx:block class='ltx_cv_entry_content'><ltx:inline-block class='ltx_font_bold'>#2,</ltx:inline-block><ltx:inline-block> #4, #5</ltx:inline-block></ltx:block></ltx:para>",
    enter_horizontal => true
  );

  DefConstructor!("\\cvitem{}{}",
    "<ltx:para class='ltx_cv_item'><ltx:block class='ltx_cv_item_label'>#1</ltx:block><ltx:block class='ltx_cv_item_content'>#2</ltx:block></ltx:para>",
    enter_horizontal => true
  );

  DefConstructor!("\\@@@homepage{}", "^ <ltx:contact role='homepage'>#1</ltx:contact>");
  DefMacro!("\\homepage{}", "\\@add@to@frontmatter{ltx:creator}{\\@@@homepage{\\url{#1}}}");
  def_macro_noop("\\homepagesymbol")?;

  DefConstructor!("\\@@@mobile{}", "^ <ltx:contact role='mobile'>#1</ltx:contact>");
  DefMacro!("\\mobile{}", "\\@add@to@frontmatter{ltx:creator}{\\@@@mobile{#1}}");

  // @@cv@section: replaces both @@numbered@section and @@unnumbered@section
  DefConstructor!("\\@@cv@section{} Undigested OptionalUndigested Undigested",
    sub[document, args, props] {
      let stype = args[0].as_ref().unwrap();
      let inlist = args[1].as_ref().unwrap();
      let id = props.get("id").unwrap().to_string();
      document.open_element(&s!("ltx:{stype}"),
        Some(string_map!(
          "xml:id" => clean_id(&id),
          "inlist" => inlist.to_string()
        )), None)?;
      // Open ltx:title with class="ltx_cv"
      document.open_element("ltx:title",
        Some(string_map!("class" => "ltx_cv")),
        None)?;
      // Insert ltx:text with class="ltx_section_mark" (empty content)
      document.insert_element("ltx:text", vec![],
        Some(string_map!("class" => "ltx_section_mark")))?;
      // Insert ltx:text with class="ltx_cv_heading" and title content
      let title = prop_digested!(props, "title");
      document.insert_element("ltx:text", title,
        Some(string_map!("class" => "ltx_cv_heading")))?;
      document.close_element("ltx:title")?;
      // Optionally insert ltx:toctitle
      let toctitle = prop_digested!(props, "toctitle");
      if !toctitle.is_empty() {
        document.insert_element("ltx:toctitle", toctitle, None)?;
      }
    },
    properties => sub[args] {
      use DigestedData::*;
      let stype = args[0].as_ref().unwrap();
      let toctitle_arg = args[2].as_ref();
      let title = args[3].as_ref().unwrap();
      let stype_str = stype.to_string();
      let mut props = RefStepID!(&stype_str)?;

      let title_digested = if let Postponed(tokens) = title.data() {
        stomach::digest(
          Tokens!(T_CS!("\\lx@hidden@bgroup"), tokens.clone().unlist(), T_CS!("\\lx@hidden@egroup")))?
      } else {
        title.clone()
      };
      props.insert("title", title_digested.into());

      if let Some(toctitle) = toctitle_arg {
        if let Postponed(toctokens) = toctitle.data() {
          if !toctokens.is_empty() {
            let toctitle_digested = stomach::digest(
              Tokens!(T_CS!("\\lx@hidden@bgroup"),
                toctokens.clone().unlist(), T_CS!("\\lx@hidden@egroup")))?;
            props.insert("toctitle", toctitle_digested.into());
          }
        }
      }
      Ok(props)
    },
    locked => true
  );

  Let!("\\@@numbered@section",   "\\@@cv@section");
  Let!("\\@@unnumbered@section", "\\@@cv@section");

  DefMacro!("\\closesection{}", "", locked => true);

  // Redefine \title, \email, \firstname, \familyname for CV frontmatter
  DefMacro!("\\title Semiverbatim",      "\\@add@to@frontmatter{ltx:creator}{\\@@@position{#1}}");
  DefMacro!("\\email Semiverbatim",      "\\@add@to@frontmatter{ltx:creator}{\\@@@email{#1}}");
  DefMacro!("\\firstname Semiverbatim",  "\\@add@to@frontmatter{ltx:creator}{\\@@@firstname{#1}}");
  DefMacro!("\\familyname Semiverbatim", "\\@add@to@frontmatter{ltx:creator}{\\@@@familyname{#1}}");
  def_macro_noop("\\photo[]{}")?; // TODO

  DefConstructor!("\\@@@position{}",   "^ <ltx:contact role='position'>#1</ltx:contact>");
  DefConstructor!("\\@@@email{}",      "^ <ltx:contact role='email'>#1</ltx:contact>");
  DefConstructor!("\\@@@firstname{}",  "^ <ltx:contact role='firstname'>#1</ltx:contact>");
  DefConstructor!("\\@@@familyname{}", "^ <ltx:contact role='familyname'>#1</ltx:contact>");

  // Style-dependent
  def_macro_noop("\\moderncvtheme[]{}")?;
  def_macro_noop("\\moderncvcolor")?;
  def_macro_noop("\\moderncvicons")?;
  def_macro_noop("\\moderncvstyle")?;

  // Classic theme icon macros
  def_macro_noop("\\marvosymbol {}")?;
  def_macro_noop("\\addresssymbol")?;
  DefMacro!("\\mobilephonesymbol",    "\u{1F4F1}"); // 📱
  DefMacro!("\\fixedphonesymbol",     "\u{260E}");  // ☎
  DefMacro!("\\faxphonesymbol",       "\u{1F4E0}"); // 📠
  DefMacro!("\\emailsymbol",          "\u{2709}");  // ✉
  DefMacro!("\\homepagesymbol",       "\u{1F5B0}"); // 🖰
  def_macro_noop("\\linkedinsocialsymbol")?;
  def_macro_noop("\\twittersocialsymbol")?;
  def_macro_noop("\\githubsocialsymbol")?;
});
