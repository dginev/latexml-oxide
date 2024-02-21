use crate::package::*;

#[rustfmt::skip]
LoadDefinitions!({
  //======================================================================
  // C.5.4 The Title Page and Abstract
  //======================================================================
  // See frontmatter support in TeX.ltxml

  Let!("\\@title", "\\@empty");
  DefMacro!("\\title{}", "\\def\\@title{#1}\\@add@frontmatter{ltx:title}{#1}", locked => true);
  DefMacro!("\\@date", "\\@empty");
  DefMacro!(
    "\\date{}",
    r"\def\@date{#1}\
\@add@frontmatter{ltx:date}[role=creation,name={\@ifundefined{datename}{}{\datename}}]{#1}"
  );
  DefConstructor!("\\person@thanks{}", "^ <ltx:contact role='thanks'>#1</ltx:contact>",
    alias => "\\thanks", mode => "text");
  DefConstructor!("\\@personname{}", "<ltx:personname>#1</ltx:personname>",
    before_digest => { Let!("\\thanks", "\\person@thanks"); },
    bounded => true,
    mode => "text"
  );

  // Sanitize person names for (obvious) punctuation abuse at start+end
  Tag!("ltx:personname", after_close => sub[_document, node] {
    if let Some(mut first) = node.get_first_child() {
      if first.get_type() == Some(NodeType::TextNode) {
        let first_text = first.get_content();
        let mut first_text_iter = first_text.chars().peekable();
        while let Some(peeked) = first_text_iter.peek() {
          if peeked.is_whitespace() || matches!(peeked, ',' | '!' | ';' | '.' | ':' | '?') {
            first_text_iter.next();
          } else {
            break;
          }
        }
        let new_text = first_text_iter.collect::<String>();
        if first_text != new_text {
          first.set_content(&new_text)?;
        }
      }
      if let Some(mut last) = node.get_last_child() {
        if last.get_type() == Some(NodeType::TextNode) {
          let last_text = last.get_content();
          let mut last_text_iter  = last_text.chars().rev().peekable();
          while let Some(peeked) = last_text_iter.peek() {
            if peeked.is_whitespace() || matches!(peeked, ',' | '!' | ';' | '.' | ':' | '?') {
              last_text_iter.next();
            } else {
              break;
            }
          }
          let new_text = last_text_iter.rev().collect::<String>();

          if last_text != new_text {
            last.set_content(&new_text)?;
          }
        }
      }
    }
  });

  DefConstructor!("\\and", " and ");

  AssignValue!("NUMBER_OF_AUTHORS" => 0);
  DefPrimitive!("\\lx@count@author", {
    let current = lookup_int("NUMBER_OF_AUTHORS");
    AssignValue!("NUMBER_OF_AUTHORS" => current + 1, Some(Scope::Global));
  });
  DefMacro!(
    "\\lx@author{}",
    r"\lx@count@author\@add@frontmatter{ltx:creator}[role=author]{\lx@author@prefix\@personname{#1}}"
  );
  DefConstructor!("\\lx@@@contact{}{}", "^ <ltx:contact role='#1'>#2</ltx:contact>");
  DefMacro!("\\lx@contact{}{}",
  r"\@add@to@frontmatter{ltx:creator}{\lx@@@contact{#1}{#2}}");
  DefMacro!("\\lx@author@sep", "\\qquad");
  DefMacro!("\\lx@author@conj", "\\qquad");
  DefConstructor!("\\lx@author@prefix", sub[document, _args, _props] {
    let mut node   = document.get_element().unwrap();
    let nauthors   = lookup_int("NUMBER_OF_AUTHORS");
    let i          = document.findnodes("//ltx:creator[@role='author']", None).len() as i64;
    if i <= 1 { }
    else if i == nauthors {
      let author_conj = Digest!(T_CS!("\\lx@author@conj"))?;
      document.set_attribute(&mut node, "before", &author_conj.to_string())?;

    } else {
      let author_sep = Digest!(T_CS!("\\lx@author@sep"))?;
      document.set_attribute(&mut node, "before", &author_sep.to_string())?;
    }
  });

  DefMacro!("\\@author", "\\@empty");
  DefMacro!("\\author{}", "\\def\\@author{#1}\\lx@make@authors@anded{#1}", locked => true);
  DefMacro!("\\lx@make@authors@anded{}", sub[(authors)] {
    and_split(T_CS!("\\lx@author"), authors)
  });
  DefPrimitive!("\\ltx@authors@oneline", {
    AssignMapping!("DOCUMENT_CLASSES", "ltx_authors_1line" => true);
  });
  DefPrimitive!("\\ltx@authors@multiline", {
    AssignMapping!("DOCUMENT_CLASSES", "ltx_authors_multiline" => true);
  });

  DefMacro!(
    "\\@add@conversion@date",
    "\\@add@frontmatter{ltx:date}[role=creation]{\\today}"
  );

  // Doesn"t produce anything (we're already inserting frontmatter),
  // But, it does make the various frontmatter macros into no-ops.
  DefMacro!(
    "\\maketitle",
    r"\@startsection@hook\global\let\thanks\relax\global\let\maketitle\relax\
\global\let\@maketitle\relax\global\let\@thanks\@empty\global\let\@author\@empty\
\global\let\@date\@empty\global\let\@title\@empty\global\let\title\relax\
\global\let\author\relax\global\let\date\relax\global\let\and\relax"
  );

  DefMacro!("\\@thanks", "\\@empty");
  DefMacro!("\\thanks{}", r"\def\@thanks{#1}\lx@make@thanks{#1}");
  DefConstructor!(
    "\\lx@make@thanks{}",
    "<ltx:note role='thanks'>#1</ltx:note>"
  );

  // Abstract SHOULD have been so simple, but seems to be a magnet for abuse.
  // For one thing, we'd like to just write
  //   DefEnvironment('{abstract}','<ltx:abstract>//body</ltx:abstract>');
  // However, we don't want to place the <ltx:abstract> environment directly where
  // we found it, but we want to add it to frontmatter. This requires capturing the
  // recently digested list and storing it in the frontmatter structure.

  // The really messy stuff comes from the way authors -- and style designers -- misuse it.
  // Basic LaTeX wants it to be an environment WITHIN the document environment,
  // and AFTER the \maketitle.
  // However, since all it really does is typeset "Abstract" in bold, it allows:
  //   \abstract stuff...
  // without even an \endabstract!  We MUST know when the abstract ends, so we've got
  // to recognize when we've moved on to other stuff... \sections at the VERY LEAST.

  // Additional complications come from certain other classes and styles that
  // redefine abstract to take the text as an argument. And some treat it
  // like \title, \author, and such, that are expected to appear in the preamble!!
  // The treatment below allows an abstract environment in the preamble,
  // (even though straight latex doesn't) but does not cover the 1-arg case in preamble!
  //
  // Probably there are other places (eg in titlepage?) that should force the close??

  DefEnvironment!("{abstract}", "",
    after_digest_begin => {
      AssignValue!("inPreamble" => false);
      AddToMacro!("\\@startsection@hook", "\\maybe@end@abstract");
    },
    after_digest => {
      let abstract_title = stomach::digest(Tokens!(T_CS!("\\format@title@abstract"),
        T_BEGIN!(), T_CS!("\\abstractname"), T_END!()))?;
      let regurgitated = List::new(clone_box_list());

      with_value_mut("frontmatter",|frontmatter_opt| {
        let frontmatter = match frontmatter_opt {
          Some(&mut Stored::HashTagData(ref mut frnt)) => frnt,
          _ => Fatal!(TexPool, Expected,
              "Global TeX Frontmatter hash was not available, should never happen"),
        };
        let abstr = frontmatter.entry("ltx:abstract".to_string()).or_insert_with(Vec::new);
        abstr.push(("ltx:abstract".to_string(),
          Some(string_map!("name" => abstract_title)), regurgitated.into()));
        Ok(())
      })?;
      DefMacro!("\\maybe@end@abstract", "", scope => Some(Scope::Global));
    },
    locked => true,
    mode => "text"
  );
  // If we get a plain \abstract, instead of an environment, look for \abstract{the abstract}
  AssignValue!("\\abstract:locked" => false); // REDEFINE the above locked definition!
  DefMacro!("\\abstract", {
    if gullet::if_next(&TOKEN_BEGIN)? {
      T_CS!("\\abstract@onearg")
    } else {
      T_CS!("\\begin{abstract}")
    }
  },
  locked => true);
  DefMacro!("\\abstract@onearg{}", "\\begin{abstract}#1\\end{abstract}");
  DefMacro!("\\maybe@end@abstract", "\\endabstract");
  DefMacro!("\\abstractname", "Abstract");
  DefMacro!("\\format@title@abstract{}", "#1");

  // Hmm, titlepage is likely to be hairy, low-level markup,
  // without even title, author, etc, specified as such!
  // Hmm, should this even redefine author, title, etc so that they
  // are simply output?
  // This is horrible hackery; What we really need, I think, is the
  // ability to bind some sort of "Do <this> when we create a text box"...
  // ON Second Thought...
  // For the time being, ignore titlepage!
  // Maybe we could do some of this if there is no title/author
  // otherwise defined? Ugh!

  //DefEnvironment('{titlepage}','');
  // Or perhaps it's better just to ignore the markers?
  //DefMacro('\titlepage','');
  //DefMacro('\endtitlepage','');

  // Or perhaps not....
  // There's a title and other stuff in here, but how could we guess?
  // Well, there's likely to be a sequence of <p><text font="xx" fontsize="yy">...</text></p>
  // Presumably the earlier, larger one is title, rest are authors/affiliations...
  // Particularly, if they start with a pseudo superscript or other "marker", they're probably
  // affil! For now, we just give an info message
  DefEnvironment!("{titlepage}", "<ltx:titlepage>#body",
    // TODO
    // before_digest => sub { Let('\centering', '\relax');
    //   DefEnvironment('{abstract}',
    //     '<ltx:abstract>#body</ltx:abstract>');
    //   Info('unexpected', 'titlepage', $_[0],
    //     "When using titlepage, Frontmatter will not be well-structured");
    //   return; },
    // beforeDigestEnd => sub { Digest(T_CS('\maybe@end@title')); },
    locked => true,
    mode => "text"
  );

  DefConstructor!("\\maybe@end@title", sub[document,_args,_props] {
    if document.is_closeable("ltx:titlepage").is_some() {
      document.close_element("ltx:titlepage")?;
    }
  });

  DefMacro!("\\sectionmark{}", "");
  DefMacro!("\\subsectionmark{}", "");
  DefMacro!("\\subsubsectionmark{}", "");
  DefMacro!("\\paragraphmark{}", "");
  DefMacro!("\\subparagraphmark{}", "");
  DefMacro!("\\@oddfoot", "");
  DefMacro!("\\@oddhed", "");
  DefMacro!("\\@evenfoot", "");
  DefMacro!("\\@evenfoot", "");
});
