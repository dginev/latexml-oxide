use crate::package::*;
LoadDefinitions!(state, {
  //======================================================================
  // C.5.4 The Title Page and Abstract
  //======================================================================
  // See frontmatter support in TeX.ltxml

  Let!("\\@title", "\\@empty");
  DefMacro!("\\title{}", "\\def\\@title{#1}\\@add@frontmatter{ltx:title}{#1}", locked => true);
  DefMacro!("\\@date", "\\@empty");
  DefMacro!("\\date{}", "\\def\\@date{#1}\
    \\@add@frontmatter{ltx:date}[role=creation,\
    name={\\@ifundefined{datename}{}{\\datename}}]{#1}");

  // TODO: ^
  // DefConstructor!("\\person@thanks{}", "^ <ltx:contact role='thanks'>#1</ltx:contact>",
  //   alias => "\\thanks".into_option(), mode => "text".into_option());
  DefConstructor!("\\@personname{}", "<ltx:personname>#1</ltx:personname>",
    before_digest => before_digest!(stomach, state, { Let!("\\thanks", "\\person@thanks"); }),
    bounded => true, 
    mode => "text".into_option());

  DefConstructor!("\\and", " and ");

  AssignValue!("NUMBER_OF_AUTHORS" => 0);
  DefPrimitive!("\\lx@count@author", sub[stomach, args, state] {
    let current = state.lookup_int("NUMBER_OF_AUTHORS");
    AssignValue!("NUMBER_OF_AUTHORS" => current + 1, Some(Scope::Global));
  });
  DefMacro!("\\lx@author{}", "\\lx@count@author\
    \\@add@frontmatter{ltx:creator}[role=author]{\\lx@author@prefix\\@personname{#1}}");
  DefMacro!("\\lx@author@sep",  "\\qquad");
  DefMacro!("\\lx@author@conj", "\\qquad");
  DefConstructor!("\\lx@author@prefix", sub[document, args, props, state] {
    let mut node       = document.get_element().unwrap();
    let nauthors   = state.lookup_int("NUMBER_OF_AUTHORS");
    let i          = document.findnodes("//ltx:creator[@role='author']", None, state).len() as i32;
    if i <= 1 { }
    else if i == nauthors {
      let author_conj = Digest!(T_CS!("\\lx@author@conj"), state)?;
      document.set_attribute(&mut node, "before", &author_conj.to_string())?;
    } else {
      let author_sep = Digest!(T_CS!("\\lx@author@sep"), state)?;
      document.set_attribute(&mut node, "before", &author_sep.to_string())?;
    }
  });

  DefMacro!("\\@author", "\\@empty");
  DefMacro!("\\author{}", "\\def\\@author{#1}\\lx@make@authors@anded{#1}", locked => true);
  // TODO:
  // DefMacro!("\\lx@make@authors@anded{}", sub[gullet, args, state] { andSplit(T_CS!("\\lx@author"), args); });
  DefPrimitive!("\\ltx@authors@oneline", sub[stomach, args, state] {
    AssignMapping!("DOCUMENT_CLASSES", "ltx_authors_1line" => true);
  });
  DefPrimitive!("\\ltx@authors@multiline", sub[stomach, args, state] {
    AssignMapping!("DOCUMENT_CLASSES", "ltx_authors_multiline" => true);
  });

  DefMacro!("\\@add@conversion@date", "\\@add@frontmatter{ltx:date}[role=creation]{\\today}");

  // Doesn"t produce anything (we're already inserting frontmatter),
  // But, it does make the various frontmatter macros into no-ops.
  DefMacro!("\\maketitle", "\\@startsection@hook\
      \\global\\let\\thanks\\relax\
      \\global\\let\\maketitle\\relax\
      \\global\\let\\@maketitle\\relax\
      \\global\\let\\@thanks\\@empty\
      \\global\\let\\@author\\@empty\
      \\global\\let\\@date\\@empty\
      \\global\\let\\@title\\@empty\
      \\global\\let\\title\\relax\
      \\global\\let\\author\\relax\
      \\global\\let\\date\\relax\
      \\global\\let\\and\\relax");

  DefMacro!("\\@thanks",  "\\@empty");
  DefMacro!("\\thanks{}", "\\def\\@thanks{#1}\\lx@make@thanks{#1}");
  DefConstructor!("\\lx@make@thanks{}", "<ltx:note role='thanks'>#1</ltx:note>");

  DefMacro!("\\sectionmark{}", "");
  DefMacro!("\\subsectionmark{}", "");
  DefMacro!("\\subsubsectionmark{}", "");
  DefMacro!("\\paragraphmark{}", "");
  DefMacro!("\\subparagraphmark{}", "");
});
