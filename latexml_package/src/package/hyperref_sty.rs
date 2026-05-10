use crate::package::url_sty::LEADING_BACKSLASH_RE;
use crate::prelude::*;
use latexml_core::document::{can_contain_qsym, get_node_qname, Document};
use latexml_core::common::error::Result as CoreResult;
use libxml::tree::NodeType;

LoadDefinitions!({
  // Perl #2736: newer hyperref.sty depends on etoolbox.sty
  RequirePackage!("iftex");
  RequirePackage!("etoolbox");
  // Perl: loads these packages to match hyperref's real dependencies
  RequirePackage!("ltxcmds");
  //RequirePackage("pdftexcmds");  // not ported
  //RequirePackage("infwarerr");   // not ported
  RequirePackage!("keyval");
  RequirePackage!("kvsetkeys");
  RequirePackage!("kvdefinekeys");
  //RequirePackage("pdfescape");   // not ported
  //RequirePackage("hycolor");     // not ported
  //RequirePackage("letltxmacro"); // not ported
  //RequirePackage("auxhook");     // not ported
  RequirePackage!("kvoptions");
  //RequirePackage("intcalc");     // not ported
  //RequirePackage("etexcmds");    // not ported
  RequirePackage!("nameref");
  RequirePackage!("url");
  RequirePackage!("bitset");
  //RequirePackage("atbegshi");    // not ported

  // Can we load hyperref, to get all it's random sundry definitions?
  // No, too many weird extra packages loaded.
  //// InputDefinitions('hyperref', type => 'sty', noltxml => 1);

  //%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
  // Follow hyperref's manual.pdf
  //%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%

  //%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
  // 3. Package Options
  // Most (all?) options currently ignored; seen handling at end. But:
  //  * the various color ones should be used for styling
  //  * The metadata could be used to augment the RDFa
  for option in [
    // 3.1 General Options
    "draft",
    "final",
    "debug",
    "verbose",
    "implicit",
    "hypertexnames",
    "naturalnames",
    "a4paper",
    "a5paper",
    "b5paper",
    "letterpaper",
    "legalpaper",
    "executivepaper",
    "setpagesizes",
    // 3.2 Configuration Options
    "raiselinks",
    "breaklinks",
    "pageanchor",
    "plainpages",
    "nesting",
    // 3.3 Backend Drivers
    "dvipdfm",
    "dvipdfmx",
    "dvips",
    "dvipsone",
    "dviwindo",
    "hypertex",
    "latex2html",
    "nativepdf",
    "pdfmark",
    "pdftex",
    "ps2pdf",
    "tex4ht",
    "textures",
    "vtex",
    "vtexpdfmark",
    "xetex",
    // 3.4 Extension Options
    "extension",
    "hyperfigures",
    "backref",
    "pagebackref",
    "hyperindex",
    "pageanchors",
    "plainpages",
    "hyperfootnotes",
    "encap",
    "linktocpage",
    "breaklinks",
    "colorlinks",
    "linkcolor",
    "anchorcolor",
    "citecolor",
    "filecolor",
    "menucolor",
    "pagecolor",
    "urlcolor",
    "frenchlinks",
    // 3.5 PDF-specific display options
    "bookmarks",
    "bookmarksopen",
    "bookmarksopenlevel",
    "bookmarksnumbered",
    "bookmarstype",
    "CJKbookmarks",
    "pdfhighlight",
    "citebordercolor",
    "filebordercolor",
    "linkbordercolor",
    "menubordercolor",
    "pagebordercolor",
    "urlbordercolor",
    "runbordercolor",
    "pdfborder",
    // 3.6 PDF display and information options
    "baseurl",
    "pdfpagemode",
    "pdfview",
    "pdfstartpage",
    "pdfstartview",
    "pdfpagescrop",
    "pdfcenterwindow",
    "pdfdirection",
    "pdfdisplaydoctitle",
    "pdfduplex",
    "pdffitwindow",
    "pdfmenubar",
    "pdfnewwindow",
    "pdfnonfullscreenpagemode",
    "pdfnumcopies",
    "pdfpagelayout",
    "pdfpagelabels",
    "pdfpagetransition",
    "pdfpicktrackbypdfsize",
    "pdfprintarea",
    "pdfprintclip",
    "pdfprintpagerange",
    "pdfprintscaling",
    "pdftoolbar",
    "pdfviewarea",
    "pdfviewclip",
    "pdfprintpagerange",
    "pdfprintscaling",
    "pdftoolbar",
    "pdfviewarea",
    "pdfviewclip",
    "pdfwindowui",
    "unicode",
    // PDF display and information options that provide interesting Metadata
    "pdftitle",
    "pdfauthor",
    "pdfsubject",
    "pdfcreator",
    "pdfproducer",
    "pdfkeywords",
    "pdflang",
  ] {
    DeclareOption!(option, None);
    // Rust-only divergence (paired with `21e730e71e` Info→Warn promotion):
    // also register each hyperref option as a Hyp keyval so `\hypersetup{
    // colorlinks=true,citecolor=…}` doesn't trip the unknown-key Warn path.
    // Perl `hyperref.sty.ltxml:110` only registers `baseurl`; everything
    // else falls through `KeyVals.pm:97` at Info level (silent). Without
    // this registration, every hyperref-using paper emits 3-10 Warn lines
    // per `\hypersetup`. Driver: 2304.12803 (4 Hyp warnings, all the
    // common color-link options).
    DefKeyVal!("Hyp", option, "");
  }

  // \hypersetup{keyvals} configures various parameters,
  // for each pdf keyword, provide [property,(content|resource),datatype]
  DefKeyVal!("Hyp", "baseurl", "Semiverbatim");

  // Digest & store the options
  // Perl: DefPrimitive('\hypersetup RequiredKeyVals:Hyp', sub {
  //   hyperref_setoption($key, Digest($value)); });
  DefPrimitive!("\\hypersetup RequiredKeyVals:Hyp", sub[(kv)] {
    for (key, value) in kv.get_pairs() {
      let value_str = value.to_string();
      if key == "colorlinks" && value_str == "true" {
        RequirePackage!("color");
      }
      // Perl digests the value tokens to apply font conversions (e.g. ' → ')
      let digested = stomach::digest(value.revert()?)?;
      let digested_str = digested.to_string();
      state::assign_mapping("Hyperref_options", key, Some(digested_str));
      if key == "baseurl" {
        AssignValue!("BASE_URL" => value_str);
      }
    }
  });

  state::push_value("@at@end@document", T_CS!("\\@add@PDF@RDFa@triples"))?;

  // \@add@PDF@RDFa@triples — emit <ltx:rdf> elements for PDF metadata
  {
    let replacement: ReplacementClosure = Rc::new(
      |document: &mut Document,
       _args: &Vec<Option<Digested>>,
       _props: &arena::SymHashMap<Stored>| {
        // pdfkey -> (property, object_attr)
        let pdfkey_property: &[(&str, &str, &str)] = &[
          ("pdfauthor", "dcterms:creator", "content"),
          ("pdfkeywords", "dcterms:subject", "content"),
          ("pdflang", "dcterms:language", "content"),
          ("pdfsubject", "dcterms:subject", "content"),
          ("pdftitle", "dcterms:title", "content"),
          ("pdfcopyright", "dcterms:rights", "content"),
          ("pdflicenseurl", "cc:licence", "resource"),
        ];

        let mut root = match document.document.get_root_element() {
          Some(r) => r,
          None => return Ok(()),
        };

        let mut keys = state::with_mapping_keys("Hyperref_options", |keys| {
          keys.into_iter().map(arena::to_string).collect::<Vec<_>>()
        });
        keys.sort();
        for key_str in &keys {
          if let Some((_, property, object_attr)) =
            pdfkey_property.iter().find(|(k, ..)| k == key_str)
          {
            if let Some(value) = state::lookup_mapping("Hyperref_options", key_str) {
              let value_str = value.to_string();
              let mut attrs = HashMap::default();
              attrs.insert("property".to_string(), property.to_string());
              attrs.insert(object_attr.to_string(), value_str);
              let mut node = document.open_element_at(&mut root, "ltx:rdf", Some(attrs), None)?;
              // Must set about="" directly — setAttribute omits empty attributes
              node.set_attribute("about", "")?;
              document.close_element_at(&mut node)?;
            }
          }
        }
        Ok(())
      },
    );
    let cs = T_CS!("\\@add@PDF@RDFa@triples");
    def_constructor(cs, None, Some(replacement), ConstructorOptions::default());
  }

  // Need some work here!?!?
  DefMacro!("\\pdfcatalog{}", None);
  DefRegister!("\\pdfcompresslevel", Number::new(0));

  // #%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
  // # Additional User Macros

  // \href{url}{text}
  DefMacro!(
    "\\href HyperVerbatim {}",
    "\\lx@hyper@url@\\href{}{}{#1}{#2}"
  );

  // Redefine \url{url} from url.sty...
  // It's slightly different in that it expands the argument
  // Redefine \@url to sanitize the argument less
  DefMacro!("\\lx@hyper@url Token", sub[(cmd)] {
    let open = gullet::read_token()?.unwrap();
    begin_semiverbatim(Some(&['%']));
    state::let_i(&T_ACTIVE!('~'), &T_OTHER!("~"), None); // Needs special protection?
    let (open,close,url) = if open.get_catcode() == Catcode::BEGIN {
      ( T_OTHER!("{"), T_OTHER!("}"),
        read_balanced(ExpansionLevel::Partial,false,false)?.unwrap_or_default()) // Expand as we go!
    } else {
      ( T_OTHER!("{"), T_OTHER!("}"),
        Tokens!(open.as_other()) )
    };
    end_semiverbatim()?;
    let toks : Vec<Token> = url.unlist().into_iter()
      .filter(|t| t.get_catcode() != Catcode::SPACE)
      // Identical with url's \@Url except, let CS's through!
      .map(|t| if t.get_catcode() == Catcode::CS { t } else { t.as_other() })
      .collect();
    let mut url_wrapped = vec![T_CS!("\\UrlFont"), T_CS!("\\UrlLeft")];
    url_wrapped.extend(toks.clone());
    url_wrapped.push(T_CS!("\\UrlRight"));
    let mut invocation_tokens = Invocation!(T_CS!("\\lx@hyper@url@"),vec![
        Tokens!(cmd.as_other()),
        Tokens!(open),
        Tokens!(close),
        Tokens::new(toks),
        Tokens::new(url_wrapped)]).unlist();
    invocation_tokens.push(T_CS!("\\endgroup"));
    Tokens::new(invocation_tokens)
  });

  // RE-define from url w
  DefMacro!("\\url", "\\begingroup\\lx@hyper@url\\url", locked => true);

  // Perl hyperref.sty.ltxml L187-194: bounded + enterHorizontal both
  // present. enter_horizontal => true was missing in the Rust port —
  // a `\url{...}` between paragraphs at top level opened <ltx:ref>
  // outside any <ltx:p>, producing invalid block-level structure.
  DefConstructor!("\\lx@hyper@url@ Undigested {}{} Semiverbatim {}",// Allow this to work in Math!
    "?#isMath(<ltx:XMWrap class='#class' href='#href'>#5</ltx:XMWrap>)(<ltx:ref href='#href' class='#class'>#5</ltx:ref>)",
    bounded   => true, enter_horizontal => true,
    properties => sub[args] {
      unref!(args => cmd, _open, _close, url, _formattedurl);
      let ltx_cmd = s!("ltx_{}", LEADING_BACKSLASH_RE.replace(&cmd.to_string(),""));
      Ok(stored_map!(
        "href" => compose_url(&state::lookup_string("BASE_URL"), &url.to_string(), None),
        "class"=> ltx_cmd
      ))
    },
    sizer     => "#5",
    reversion => "#1#2#4#3");
  // \nolinkurl{url} — Perl L197-199: enterHorizontal=>1
  DefConstructor!(
    "\\nolinkurl Semiverbatim",
    "<ltx:ref href='#1' class='ltx_nolink' >#1</ltx:ref>",
    enter_horizontal => true
  );

  // \hyperbaseurl{url}
  DefPrimitive!("\\hyperbaseurl Semiverbatim", sub[(url)] {
  AssignValue!("BASE_URL" => url.to_string()); });

  // \hyperimage{imageurl}{text} — Perl L205-207: enterHorizontal=>1
  DefConstructor!(
    "\\hyperimage Semiverbatim {}",
    "<ltx:graphic graphic='#1' description='#2'/>",
    enter_horizontal => true
  );

  DefMacro!("\\hyperref", "\\@ifnextchar[\\hyperref@@ii\\hyperref@@iv");
  // Perl L211-215: 2 argument form \hyperref[label]{text}
  DefConstructor!("\\hyperref@@ii OptionalSemiverbatim {}",
  "<ltx:ref labelref='#label'>#2</ltx:ref>",
  bounded => true, enter_horizontal => true,
  properties => sub[args] {
    let label = args[0].as_ref().map(|a| a.to_string()).unwrap_or_default();
    Ok(stored_map!("label" => clean_label(&label, None).into_owned()))
  });
  // Perl L217-222: 4 argument form \hyperref{url}{category}{name}{text}
  DefConstructor!("\\hyperref@@iv Semiverbatim Semiverbatim Semiverbatim Semiverbatim",
  "<ltx:ref href='#href'>#4</ltx:ref>",
  enter_horizontal => true,
  properties => sub[args] {
    let base_url = state::lookup_string("BASE_URL");
    let cat = args[1].as_ref().map(|a| a.to_string()).unwrap_or_default();
    let name = args[2].as_ref().map(|a| a.to_string()).unwrap_or_default();
    let fragment = clean_id(&format!("{}.{}", cat, name));
    let href = if base_url.is_empty() {
      format!("#{}", fragment)
    } else {
      format!("{}#{}", base_url, fragment)
    };
    Ok(stored_map!("href" => href))
  });

  // Perl L224-226: \htmlref{text}{label}
  DefConstructor!("\\htmlref Semiverbatim Semiverbatim",
  "<ltx:ref labelref='#label'>#1</ltx:ref>",
  enter_horizontal => true,
  properties => sub[args] {
    let label = args[1].as_ref().map(|a| a.to_string()).unwrap_or_default();
    Ok(stored_map!("label" => clean_label(&label, None).into_owned()))
  });

  // Perl L228-230: \hyperlink{name}{text}
  DefConstructor!("\\hyperlink Semiverbatim {}",
  "<ltx:ref idref='#id'>#2</ltx:ref>",
  enter_horizontal => true,
  properties => sub[args] {
    let name = args[0].as_ref().map(|a| a.to_string()).unwrap_or_default();
    Ok(stored_map!("id" => clean_id(&name)))
  });
  DefMacro!("\\hyper@@link{}{}{}", "\\hyperlink{#2}{#3}");

  // Perl L258-265: \hyperdef{category}{name}{text} and \hypertarget{name}{text}.
  // Both emit just `#3`/`#2` as content; an after_construct hook then DFS-walks
  // the just-built subtree and wraps the first descendant that ltx:anchor is
  // allowed to contain in <ltx:anchor xml:id='#id'>. This matches Perl's
  // localized_anchor and lets Pandoc-style `\hypertarget{n}{\section{T}}`
  // hoist the section out — the section structure stays intact and the
  // anchor lands inside the section title text rather than illegally
  // wrapping the section itself.
  DefConstructor!("\\hyperdef Semiverbatim Semiverbatim Semiverbatim",
  "#3",
  properties => sub[args] {
    let cat = args[0].as_ref().map(|a| a.to_string()).unwrap_or_default();
    let name = args[1].as_ref().map(|a| a.to_string()).unwrap_or_default();
    Ok(stored_map!("id" => clean_id(&format!("{}.{}", cat, name))))
  },
  after_construct => sub[document, whatsit] {
    localized_anchor(document, whatsit)?;
  });
  DefConstructor!("\\hypertarget Semiverbatim {}",
  "#2",
  properties => sub[args] {
    let name = args[0].as_ref().map(|a| a.to_string()).unwrap_or_default();
    Ok(stored_map!("id" => clean_id(&name)))
  },
  after_construct => sub[document, whatsit] {
    localized_anchor(document, whatsit)?;
  });

  // # Should create an anchor with automatically chosen name;
  // # But it's to be used where LaTeXML already would have created an anchor & link...
  // # Should leverage
  DefMacro!("\\phantomsection", None);

  Let!("\\footref", "\\ref"); // ?

  DefConditional!("\\ifHy@stoppedearly");
  DefConditional!("\\ifHy@typexml");
  DefConditional!("\\ifHy@activeanchor");
  DefConditional!("\\ifHy@backref");
  DefConditional!("\\ifHy@bookmarks");
  DefConditional!("\\ifHy@bookmarksnumbered");
  DefConditional!("\\ifHy@bookmarksopen");
  DefConditional!("\\ifHy@breaklinks");
  DefConditional!("\\ifHy@pdfcenterwindow");
  DefConditional!("\\ifHy@CJKbookmarks");
  DefConditional!("\\ifHy@colorlinks");
  DefConditional!("\\ifHy@destlabel");
  DefConditional!("\\ifHy@draft");
  Let!("\\Hy@finaltrue", "\\Hy@draftfalse");
  Let!("\\Hy@finalfalse", "\\Hy@drafttrue");
  DefConditional!("\\ifHy@pdfescapeform");
  DefConditional!("\\ifHy@hyperfigures");
  DefConditional!("\\ifHy@pdffitwindow");
  DefConditional!("\\ifHy@frenchlinks");
  DefConditional!("\\ifHy@hyperfootnotes");
  DefConditional!("\\ifHy@hyperindex");
  DefConditional!("\\ifHy@hypertexnames");
  DefConditional!("\\ifHy@implicit");
  DefConditional!("\\ifHy@linktocpage");
  DefConditional!("\\ifHy@localanchorname");
  DefConditional!("\\ifHy@pdfmenubar");
  DefConditional!("\\ifHy@naturalnames");
  DefConditional!("\\ifHy@nesting");
  DefConditional!("\\ifHy@pdfnewwindowset");
  DefConditional!("\\ifHy@pdfnewwindow");
  DefConditional!("\\ifHy@ocgcolorlinks");
  DefConditional!("\\ifHy@pageanchor");
  DefConditional!("\\ifHy@pdfpagelabels");
  DefConditional!("\\ifHy@pdfstring");
  DefConditional!("\\ifHy@plainpages");
  DefConditional!("\\ifHy@psize");
  DefConditional!("\\ifHy@raiselinks");
  DefConditional!("\\ifHy@seminarslides");
  DefConditional!("\\ifHy@setpagesize");
  DefConditional!("\\ifHy@texht");
  DefConditional!("\\ifHy@psdextra");
  DefConditional!("\\ifHy@pdftoolbar");
  DefConditional!("\\ifHy@unicode");
  DefConditional!("\\ifHy@pdfusetitle");
  DefConditional!("\\ifHy@verbose");
  Let!("\\Hy@debugtrue", "\\Hy@verbosetrue");
  Let!("\\Hy@debugfalse", "\\Hy@verbosefalse");
  DefConditional!("\\ifHy@pdfwindowui");
  DefConditional!("\\ifHy@pdfdisplaydoctitle");
  DefConditional!("\\ifHy@pdfa");
  TeX!(
    r"\Hy@backreffalse
\Hy@bookmarksnumberedfalse
\Hy@bookmarksopenfalse
\Hy@bookmarkstrue
\Hy@breaklinksfalse
\Hy@pdfcenterwindowfalse
\Hy@CJKbookmarksfalse
\Hy@destlabelfalse
\Hy@pdfescapeformfalse
\Hy@hyperfiguresfalse
\Hy@pdffitwindowfalse
\Hy@hyperfootnotestrue
\Hy@hyperindextrue
\Hy@hypertexnamestrue
\Hy@implicittrue
\Hy@linktocpagefalse
\Hy@localanchornamefalse
\Hy@pdfmenubartrue
\Hy@naturalnamesfalse
\Hy@nestingfalse
\Hy@pdfnewwindowsetfalse
\Hy@pdfnewwindowfalse
\Hy@pageanchortrue
\Hy@pdfpagelabelstrue
\Hy@pdfstringfalse
\Hy@plainpagesfalse
\Hy@raiselinksfalse
\Hy@setpagesizetrue
\Hy@texhtfalse
\Hy@psdextrafalse
\Hy@pdftoolbartrue
\Hy@typexmlfalse
\Hy@unicodetrue
"
  );
  DefMacro!("\\@bookmarksopenlevel", "\\maxdimen");
  // This only approximates the "contextual label" that should precede the number,
  // and ignores the user-definable macros.
  // But, we normally defer such bookkeeping until postprocessing....sigh
  // TODO: The star forms prevent nested double links.
  DefConstructor!("\\autoref OptionalMatch:* Semiverbatim",
  "<ltx:ref ?#1(class='ltx_refmacro_autoref ltx_nolink')(class='ltx_refmacro_autoref')
    show='autoref' labelref='#label' _force_font='true'/>",
  properties => sub[args] {
    let refarg = &args[1];
    Ok(stored_map!("label" => clean_label(&refarg.as_ref().unwrap().to_string(), None).to_string()))
  });

  DefMacro!("\\lx@autorefnum@@{}", sub[(ttype)] {
    let type_s  = ttype.unwrap().to_string();
    let mut tokens = if lookup_definition(&T_CS!(s!("\\{type_s}autorefname")))?.is_some() {
      vec![T_CS!(format!("\\{type_s}autorefname")), T_CS!("\\nobreakspace")]
    } else {
      Vec::new()
    };

    let counter_str = with_mapping("counter_for_type",&type_s, |mapping_opt|
      mapping_opt.map(ToString::to_string)).unwrap_or(type_s);

    let pcounter = T_CS!(s!("\\p@{counter_str}",));
    let thecounter = T_CS!(s!("\\the{counter_str}"));
    if lookup_definition(&pcounter)?.is_some() {
      tokens.push(pcounter);
    }
    tokens.push(thecounter);
    Tokens::new(tokens)
  });

  Let!("\\HyOrg@addtoreset", "\\@addtoreset");
  Let!("\\H@refstepcounter", "\\refstepcounter");

  AssignMapping!("type_tag_formatter", "autoref" => "\\lx@autorefnum@@");

  // Blech...
  DefMacro!(
    T_CS!("\\@itemiautorefname"),
    None,
    T_CS!("\\itemautorefname")
  );
  DefMacro!(
    T_CS!("\\@itemiiautorefname"),
    None,
    T_CS!("\\itemautorefname")
  );
  DefMacro!(
    T_CS!("\\@itemiiiautorefname"),
    None,
    T_CS!("\\itemautorefname")
  );
  DefMacro!(
    T_CS!("\\@itemivautorefname"),
    None,
    T_CS!("\\itemautorefname")
  );
  DefMacro!(
    T_CS!("\\@itemvautorefname"),
    None,
    T_CS!("\\itemautorefname")
  );
  DefMacro!(
    T_CS!("\\@itemviautorefname"),
    None,
    T_CS!("\\itemautorefname")
  );
  DefMacro!(
    T_CS!("\\enumiautorefname"),
    None,
    T_CS!("\\itemautorefname")
  );
  DefMacro!(
    T_CS!("\\enumiiautorefname"),
    None,
    T_CS!("\\itemautorefname")
  );
  DefMacro!(
    T_CS!("\\enumiiiautorefname"),
    None,
    T_CS!("\\itemautorefname")
  );
  DefMacro!(
    T_CS!("\\enumivautorefname"),
    None,
    T_CS!("\\itemautorefname")
  );
  DefMacro!(
    T_CS!("\\@desciautorefname"),
    None,
    T_CS!("\\itemautorefname")
  );
  DefMacro!(
    T_CS!("\\@desciiautorefname"),
    None,
    T_CS!("\\itemautorefname")
  );
  DefMacro!(
    T_CS!("\\@desciiiautorefname"),
    None,
    T_CS!("\\itemautorefname")
  );
  DefMacro!(
    T_CS!("\\@descivautorefname"),
    None,
    T_CS!("\\itemautorefname")
  );
  DefMacro!(
    T_CS!("\\@descvautorefname"),
    None,
    T_CS!("\\itemautorefname")
  );
  DefMacro!(
    T_CS!("\\@descviautorefname"),
    None,
    T_CS!("\\itemautorefname")
  );

  // Covered in LaTeX.pool, but non-ref character is ignored.
  // \ref*{label}
  // \pageref*{label}

  // I wonder if this is good enough for our context?
  // \pdfstringdef{macroname}{texstring}
  DefMacro!("\\pdfstringdef{Token}{}", "\\def#1{#2}");
  // Hopefully noop is sufficient for PDF-specific uses?
  DefMacro!("\\pdfstringdefDisableCommands", "");
  DefMacro!("\\pdfbookmark[]{}{}", "");
  DefMacro!("\\currentpdfbookmark{}{}", "");
  DefMacro!("\\subpdfbookmark{}{}", "");
  DefMacro!("\\belowpdfbookmark{}{}", "");

  //======================================================================
  // 4.1 Replacement macros

  // \texorpdfstring{TeXString}{PDFstring}
  DefMacro!("\\texorpdfstring{}{}", "#1");

  // Perl hyperref.sty.ltxml L413-416: guard against redefinition, then Let
  // the pdfstringdef hooks to sensible no-ops. Rust always defines them
  // because no package-level binding currently claims these CSes.
  Let!("\\pdfstringdefPreHook", "\\@empty");
  Let!("\\pdfstringdefPostHook", "\\@gobble");

  //======================================================================
  // 4.2 Utility macros
  // Perl L420-423: \hypercalcbp{dimen} — convert a dimension to its
  // value in big points (bp). Perl: Explode($dimen->valueOf / convertUnit('bp')).
  // In Rust, Dimension::value_of returns sp (scaled points); state::convert_unit("bp")
  // returns sp/bp. Their quotient is the bp value. Explode tokenizes the stringified
  // float into character tokens.
  DefMacro!("\\hypercalcbp {Dimension}", sub[(dimen)] {
    let sp = dimen.value_of() as f64;
    let bp = sp / state::convert_unit("bp");
    Ok(Tokens::new(Explode!(format!("{}", bp))))
  });

  //%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
  // 5 Acrobat-specific behaviour

  // \Acrobatmenu{menuoption}{text}
  // These create buttons that activate Acrobat Reader or Exchange actions.
  // It's doubtful that they have meaningful analogs in our context?
  DefMacro!("\\Acrobatmenu{}{}", "[#1 Button: #2]");

  //%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
  // 6 PDF and HTML forms
  // hmm... we might actually want to do this?
  // But, we need schema support!

  //----------------------------------------------------------------------
  // Fields:
  // \TextField[parameters]{label}
  // \CheckBox[parameters]{label}
  // \ChoiceMenu[parameters]{label}{choices}
  // \PushButton[parameters]{label}
  // \Submit[parameters]{label}
  // \Reset[parameters]{label}
  //----------------------------------------------------------------------
  // Layout:
  //  typically:  "#1 #2"
  // \LayoutTextField{label}{field}
  // \LayoutChoiceField{label}{field}
  // \LayoutCheckField{label}{field}
  //----------------------------------------------------------------------
  // What to show
  // \MakeRadioField{width}{height}
  // \MakeCheckField{width}{height}
  // \MakeChoiceField{width}{height}
  // \MakeButtonField{text}

  //======================================================================
  // 6.1 Forms environment parameters
  //   action   URL
  //   encoding name
  //   method   name (post|get)
  //======================================================================
  // 6.2 Forms optional parameters
  //  [a bunch] colors, events, etc; See the doc when we actually support.

  //%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
  // hyperref uses KeyVals for options!
  // until we come up with a nice, clean formal scheme, just hack through...

  // Process hyperref package options (keyval-style)
  if let Some(Stored::VecDequeStored(vdq)) = state::lookup_value("opt@hyperref.sty") {
    for entry in vdq {
      // Each entry is Stored::Strings from PassOptionsToPackage/\usepackage
      let opt_strs: Vec<String> = match entry {
        Stored::Strings(syms) => syms.iter().map(|s| arena::to_string(*s)).collect(),
        Stored::String(sym) => vec![arena::to_string(sym)],
        other => vec![other.to_string()],
      };
      for option in &opt_strs {
        if option == "colorlinks" {
          RequirePackage!("color");
        } else if let Some(eq_pos) = option.find('=') {
          let key = option[..eq_pos].trim();
          let value = option[eq_pos + 1..].trim();
          state::assign_mapping("Hyperref_options", key, Some(value.to_string()));
          if key == "baseurl" {
            AssignValue!("BASE_URL" => value.to_string());
          }
          // Perl hyperref.sty.ltxml L114-115: `if ($key eq 'colorlinks' and
          // ToString($value) eq 'true') { RequirePackage('color'); }`. Earlier
          // Rust port handled the bare `colorlinks` form but missed the
          // `colorlinks=true` keyval — papers passing the option as
          // `\usepackage[colorlinks=true]{hyperref}` had \textcolor undefined.
          // Witness: 0902.2912.
          if key == "colorlinks" && value == "true" {
            RequirePackage!("color");
          }
        }
      }
    }
  }

  TeX!(
    r#"
\def\HyLang@afrikaans{%
  \def\equationautorefname{Vergelyking}%
  \def\footnoteautorefname{Voetnota}%
  \def\itemautorefname{Item}%
  \def\figureautorefname{Figuur}%
  \def\tableautorefname{Tabel}%
  \def\partautorefname{Deel}%
  \def\appendixautorefname{Bylae}%
  \def\chapterautorefname{Hoofstuk}%
  \def\sectionautorefname{Afdeling}%
  \def\subsectionautorefname{Subafdeling}%
  \def\subsubsectionautorefname{Subsubafdeling}%
  \def\paragraphautorefname{Paragraaf}%
  \def\subparagraphautorefname{Subparagraaf}%
  \def\FancyVerbLineautorefname{Lyn}%
  \def\theoremautorefname{Teorema}%
  \def\pageautorefname{Bladsy}%
}
\def\HyLang@english{%
  \def\equationautorefname{Equation}%
  \def\footnoteautorefname{footnote}%
  \def\itemautorefname{item}%
  \def\figureautorefname{Figure}%
  \def\tableautorefname{Table}%
  \def\partautorefname{Part}%
  \def\appendixautorefname{Appendix}%
  \def\chapterautorefname{chapter}%
  \def\sectionautorefname{section}%
  \def\subsectionautorefname{subsection}%
  \def\subsubsectionautorefname{subsubsection}%
  \def\paragraphautorefname{paragraph}%
  \def\subparagraphautorefname{subparagraph}%
  \def\FancyVerbLineautorefname{line}%
  \def\theoremautorefname{Theorem}%
  \def\pageautorefname{page}%
}
\def\HyLang@french{%
  \def\equationautorefname{\'Equation}%
  \def\footnoteautorefname{note}%
  \def\itemautorefname{item}%
  \def\figureautorefname{Figure}%
  \def\tableautorefname{Tableau}%
  \def\partautorefname{Partie}%
  \def\appendixautorefname{Appendice}%
  \def\chapterautorefname{chapitre}%
  \def\sectionautorefname{section}%
  \def\subsectionautorefname{sous-section}%
  \def\subsubsectionautorefname{sous-sous-section}%
  \def\paragraphautorefname{paragraphe}%
  \def\subparagraphautorefname{sous-paragraphe}%
  \def\FancyVerbLineautorefname{ligne}%
  \def\theoremautorefname{Th\'eor\`eme}%
  \def\pageautorefname{page}%
}
\def\HyLang@german{%
  \def\equationautorefname{Gleichung}%
  \def\footnoteautorefname{Fu\ss note}%
  \def\itemautorefname{Punkt}%
  \def\figureautorefname{Abbildung}%
  \def\tableautorefname{Tabelle}%
  \def\partautorefname{Teil}%
  \def\appendixautorefname{Anhang}%
  \def\chapterautorefname{Kapitel}%
  \def\sectionautorefname{Abschnitt}%
  \def\subsectionautorefname{Unterabschnitt}%
  \def\subsubsectionautorefname{Unterunterabschnitt}%
  \def\paragraphautorefname{Absatz}%
  \def\subparagraphautorefname{Unterabsatz}%
  \def\FancyVerbLineautorefname{Zeile}%
  \def\theoremautorefname{Theorem}%
  \def\pageautorefname{Seite}%
}
\def\HyLang@italian{%
  \def\equationautorefname{Equazione}%
  \def\footnoteautorefname{nota}%
  \def\itemautorefname{punto}%
  \def\figureautorefname{Figura}%
  \def\tableautorefname{Tabella}%
  \def\partautorefname{Parte}%
  \def\appendixautorefname{Appendice}%
  \def\chapterautorefname{Capitolo}%
  \def\sectionautorefname{sezione}%
  \def\subsectionautorefname{sottosezione}%
  \def\subsubsectionautorefname{sottosottosezione}%
  \def\paragraphautorefname{paragrafo}%
  \def\subparagraphautorefname{sottoparagrafo}%
  \def\FancyVerbLineautorefname{linea}%
  \def\theoremautorefname{Teorema}%
  \def\pageautorefname{Pag.\@}%
}
\def\HyLang@magyar{%
  \def\equationautorefname{Egyenlet}%
  \def\footnoteautorefname{l\'abjegyzet}%
  \def\itemautorefname{Elem}%
  \def\figureautorefname{\'Abra}%
  \def\tableautorefname{T\'abl\'azat}%
  \def\partautorefname{R\'esz}%
  \def\appendixautorefname{F\"uggel\'ek}%
  \def\chapterautorefname{fejezet}%
  \def\sectionautorefname{szakasz}%
  \def\subsectionautorefname{alszakasz}%
  \def\subsubsectionautorefname{alalszakasz}%
  \def\paragraphautorefname{bekezd\'es}%
  \def\subparagraphautorefname{albekezd\'es}%
  \def\FancyVerbLineautorefname{sor}%
  \def\theoremautorefname{T\'etel}%
  \def\pageautorefname{oldal}%
}
\def\HyLang@portuges{%
  \def\equationautorefname{Equa\c c\~ao}%
  \def\footnoteautorefname{Nota de rodap\'e}%
  \def\itemautorefname{Item}%
  \def\figureautorefname{Figura}%
  \def\tableautorefname{Tabela}%
  \def\partautorefname{Parte}%
  \def\appendixautorefname{Ap\^endice}%
  \def\chapterautorefname{Cap\'itulo}%
  \def\sectionautorefname{Se\c c\~ao}%
  \def\subsectionautorefname{Subse\c c\~ao}%
  \def\subsubsectionautorefname{Subsubse\c c\~ao}%
  \def\paragraphautorefname{par\'agrafo}%
  \def\subparagraphautorefname{subpar\'agrafo}%
  \def\FancyVerbLineautorefname{linha}%
  \def\theoremautorefname{Teorema}%
  \def\pageautorefname{P\'agina}%
}
\def\HyLang@russian{%
  \def\equationautorefname{\cyr\cyrv\cyrery\cyrr.}%
  \def\footnoteautorefname{%
    \cyr\cyrp\cyro\cyrd\cyrs\cyrt\cyrr.\ \cyrp\cyrr\cyri\cyrm.%
  }%
  \def\itemautorefname{\cyr\cyrp.}%
  \def\figureautorefname{\cyr\cyrr\cyri\cyrs.}%
  \def\tableautorefname{\cyr\cyrt\cyra\cyrb\cyrl.}%
  \def\partautorefname{\cyr\cyrch.}%
  \def\chapterautorefname{\cyr\cyrg\cyrl.}%
  \def\sectionautorefname{\cyr\cyrr\cyra\cyrz\cyrd.}%
  \def\appendixautorefname{\cyr\cyrp\cyrr\cyri\cyrl.}%
  \def\subsectionautorefname{\cyr\cyrr\cyra\cyrz\cyrd.}%
  \def\subsubsectionautorefname{\cyr\cyrr\cyra\cyrz\cyrd.}%
  \def\paragraphautorefname{\cyr\cyrp.}%
  \def\subparagraphautorefname{\cyr\cyrp.}%
  \def\FancyVerbLineautorefname{\cyr\cyrs\cyrt\cyrr.}%
  \def\theoremautorefname{\cyr\cyrt\cyre\cyro\cyrr.}%
  \def\pageautorefname{\cyr\cyrs.}%
}
\def\HyLang@spanish{%
  \def\equationautorefname{Ecuaci\'on}%
  \def\footnoteautorefname{Nota a pie de p\'agina}%
  \def\itemautorefname{Elemento}%
  \def\figureautorefname{Figura}%
  \def\tableautorefname{Tabla}%
  \def\partautorefname{Parte}%
  \def\appendixautorefname{Ap\'endice}%
  \def\chapterautorefname{Cap\'itulo}%
  \def\sectionautorefname{Secci\'on}%
  \def\subsectionautorefname{Subsecci\'on}%
  \def\subsubsectionautorefname{Subsubsecci\'on}%
  \def\paragraphautorefname{P\'arrafo}%
  \def\subparagraphautorefname{Subp\'arrafo}%
  \def\FancyVerbLineautorefname{L\'inea}%
  \def\theoremautorefname{Teorema}%
  \def\pageautorefname{P\'agina}%
}
\def\HyLang@catalan{%
\def\equationautorefname{Equaci\'o}%
\def\footnoteautorefname{Nota al peu de p\`agina}%
\def\itemautorefname{Element}%
\def\figureautorefname{Figura}%
\def\tableautorefname{Taula}%
\def\partautorefname{Part}%
\def\appendixautorefname{Ap\`endix}%
\def\chapterautorefname{Cap\'itol}%
\def\sectionautorefname{Secci\'o}%
\def\subsectionautorefname{Subsecci\'o}%
\def\subsubsectionautorefname{Subsubsecci\'o}%
\def\paragraphautorefname{Par\`agraf}%
\def\subparagraphautorefname{Subpar\`agraf}%
\def\FancyVerbLineautorefname{L\'inia}%
\def\theoremautorefname{Teorema}%
\def\pageautorefname{P\`agina}%
}
\def\HyLang@vietnamese{%
  \def\equationautorefname{Ph\uhorn{}\ohorn{}ng tr\`inh}%
  \def\footnoteautorefname{Ch\'u th\'ich}%
  \def\itemautorefname{m\d{u}c}%
  \def\figureautorefname{H\`inh}%
  \def\tableautorefname{B\h{a}ng}%
  \def\partautorefname{Ph\`\acircumflex{}n}%
  \def\appendixautorefname{Ph\d{u} l\d{u}c}%
  \def\chapterautorefname{ch\uhorn{}\ohorn{}ng}%
  \def\sectionautorefname{m\d{u}c}%
  \def\subsectionautorefname{m\d{u}c}%
  \def\subsubsectionautorefname{m\d{u}c}%
  \def\paragraphautorefname{\dj{}o\d{a}n}%
  \def\subparagraphautorefname{\dj{}o\d{a}n}%
  \def\FancyVerbLineautorefname{d\`ong}%
  \def\theoremautorefname{\DJ{}\d{i}nh l\'y}%
  \def\pageautorefname{Trang}%
}
% For now...
\HyLang@english
"#
  );
  //%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
});

// Perl: hyperref.sty.ltxml `localized_anchor`. DFS walks the current node's
// subtree and wraps the first descendant that ltx:anchor is allowed to
// contain. The traversal mirrors Perl's pop+unshift order: candidates are
// popped from the back and child nodes are unshifted to the front, so we
// visit the rightmost element of each level before descending.
fn localized_anchor(document: &mut Document, whatsit: &Whatsit) -> CoreResult<()> {
  let id = match whatsit.get_property("id") {
    Some(v) => v.to_string(),
    None => return Ok(()),
  };
  let mut candidates: Vec<libxml::tree::Node> = vec![document.get_node().clone()];
  let mut found: Option<libxml::tree::Node> = None;
  while let Some(candidate) = candidates.pop() {
    match candidate.get_type() {
      Some(NodeType::ElementNode) => {
        let qname = get_node_qname(&candidate);
        if can_contain_qsym(pin!("ltx:anchor"), qname) {
          found = Some(candidate);
          break;
        }
        // Perl: unshift(@candidates, $candidate->childNodes); pushes the child
        // list to the front, so the rightmost child is popped next.
        let children = candidate.get_child_nodes();
        for child in children.into_iter().rev() {
          candidates.insert(0, child);
        }
      },
      // Perl: any non-element node short-circuits; the candidate (text node)
      // is then wrapped, which works because anchor.model = Inline allows text.
      _ => {
        found = Some(candidate);
        break;
      },
    }
  }
  if let Some(target) = found {
    if let Some(mut anchor) = document.wrap_nodes("ltx:anchor", vec![target])? {
      document.set_attribute(&mut anchor, "xml:id", &id)?;
      if document.is_open(&anchor) {
        document.close_node(&anchor)?;
      }
    } else {
      Warn!("malformed", "ltx:anchor",
        &s!("No available insertion point for ltx:anchor, failing \\hypertarget to {}", id));
    }
  } else {
    Warn!("malformed", "ltx:anchor",
      &s!("No available insertion point for ltx:anchor, failing \\hypertarget to {}", id));
  }
  Ok(())
}
