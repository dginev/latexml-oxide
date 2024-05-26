use crate::prelude::*;

LoadDefinitions!({
  //======================================================================
  // Alignments
  //
  // & gives an error except within the right context
  // (which should redefine it!)
  DefConstructor!("&", { Error!("unexpected", "&", "Stray alignment \"&\""); });

  //**********************************************************************
  // Plain;  Extracted from Appendix B.
  //**********************************************************************
  //
  //======================================================================
  // TeX Book, Appendix B, p. 344
  //======================================================================
  TeX!(r"\outer\def^^L{\par}");
  DefMacro!("\\dospecials", r"\do\ \do\\\do\{\do\}\do\$\do\&\do\#\do\^\do\^^K\do\_\do\^^A\do\%\do\~");


  //======================================================================
  // TeX Book, Appendix B, p. 345

  TeX!(
    r"\chardef\active=13
    \chardef\@ne=1
    \chardef\tw@=2
    \chardef\thr@@=3
    \chardef\sixt@@n=16
    \chardef\@cclv=255
    \mathchardef\@cclvi=256
    \mathchardef\@m=1000
    \mathchardef\@M=10000
    \mathchardef\@MM=20000
    \countdef\m@ne=21\relax
    \m@ne=-1"
  );

  //======================================================================
  // TeX Book, Appendix B, p. 346
  TeX!(
    r"\countdef\count@=255
  \toksdef\toks@=0
  \skipdef\skip@=0
  \dimendef\dimen@=0
  \dimendef\dimen@i=1
  \dimendef\dimen@ii=2
  \count10=22
  \count11=9
  \count12=9
  \count13=9
  \count14=9
  \count15=9
  \count16=-1
  \count17=-1
  \count18=3
  \count19=0
  \count20=255
  \countdef\insc@unt=20
  \countdef\allocationnumber=21
  \countdef\m@ne=22 \m@ne=-1"
  );
  // Various \count's are set; should we?

  //======================================================================
  // TeX Book, Appendix B, p. 347
  DefPrimitive!("\\wlog{}", sub[(arg)] {
    eprintln!("{}", Expand!(arg));
    Ok(Vec::new())
  }, locked => true);
  // From plain.tex
  DefPrimitive!("\\newcount DefToken", sub[(name)] {
    DefRegister!(name, None, Number::new(0), allocate=>"\\count");
  });
  DefPrimitive!("\\newdimen DefToken", sub[(name)] {
    DefRegister!(name, None, Dimension::new(0), allocate=>"\\dimen");
  });
  DefPrimitive!("\\newskip DefToken", sub[(name)] {
    DefRegister!(name, None, Glue::new(0), allocate=>"\\skip");
  });
  DefPrimitive!("\\newmuskip DefToken", sub[(name)] {
    DefRegister!(name, None, MuGlue::new(0), allocate=>"\\muskip");
  });
  AssignValue!("allocated_boxes" => 0);
  DefPrimitive!("\\newbox DefToken", sub[(t)] {
    let n = lookup_int("allocated_boxes");
    AssignValue!("allocated_boxes" => n + 1, Some(Scope::Global));
    let empty_list = List::new(Vec::new());
    AssignValue!(&s!("box{n}"), empty_list);
    DefRegister!(t, None, Number(n), readonly => true);
  });
  DefPrimitive!("\\newhelp DefToken {}", sub[(token,arg)] {
    state::assign_value(&token.to_string(), arg, None);
  });
  DefPrimitive!("\\newtoks DefToken", sub[(name)] {
    DefRegister!(name, None, Tokens!(), allocate=>"\\toks");
  });

  // the next 4 actually work by doing a \chardef instead of \countdef, etc.
  // which means they actually work quite differently
  DefPrimitive!("\\alloc@@ {}", sub[(atype)] {
    let c = s!("allocation @{}", atype);
    let n = state::lookup_int(&c);
    state::assign_value(&c, n + 1, Some(Scope::Global));
    state::assign_register("\\allocationnumber", Number::new(n).into(), Some(Scope::Global), Vec::new())?;
  });
  DefMacro!(
    "\\newread DefToken",
    r"\alloc@@{read}\global\chardef#1=\allocationnumber"
  );
  DefMacro!(
    "\\newwrite DefToken",
    r"\alloc@@{write}\global\chardef#1=\allocationnumber"
  );
  DefMacro!(
    "\\newfam DefToken",
    r"\alloc@@{fam}\global\chardef#1=\allocationnumber"
  );
  DefMacro!(
    "\\newlanguage DefToken",
    r"\alloc@@{language}\global\chardef#1=\allocationnumber"
  );
  DefMacro!("\\e@alloc{}{}{}{}{}{}",
  r"\global\advance#3\@ne
  \allocationnumber#3\relax
  \global#2#6\allocationnumber");
  DefMacro!("\\alloc@{}{}{}{}", r"\e@alloc#2#3{\count1#1}#4\float@count");
  DefMacro!("\\newread",        r"\e@alloc\read \chardef{\count16}\m@ne\sixt@@n");
  DefMacro!("\\newwrite", r"\e@alloc\write
                  {\ifnum\allocationnumber=18
                      \advance\count17\@ne
                      \allocationnumber\count17 %
                    \fi
                    \global\chardef}%
                    {\count17}%
                    \m@ne
                    {128}");

  // This implementation is quite wrong
  DefPrimitive!("\\newinsert Token", sub[(t)] {
    DefRegister!(t, None, Number::new(0));
  });
  // \alloc@, \ch@ck

  // TeX plain uses \newdimen, etc. for these.
  // Is there any advantage to that?
  // note: rust complains about the 16_383.99999 having excessive precision, hence simplifying
  DefRegister!("\\maxdimen", Dimension::new_f64(16383.99999 * UNITY_F64));
  // DefRegister!("\\hideskip", Glue!(-1000 * 65536, "1fill"));
  DefRegister!("\\centering", Glue!("0pt plus 1000pt minus 1000pt"));
  DefRegister!("\\p@", Dimension::new(UNITY));
  DefRegister!("\\z@", Dimension::new(0));
  DefRegister!("\\z@skip", Glue::new(0));

  // First approximation. till I figure out \newbox
  TeX!(r"\newbox\voidb@x");
  //======================================================================
  // TeX Book, Appendix B, p. 348

  DefMacro!("\\newif DefToken", sub[(cs)] {
    def_conditional(cs, None,None,ConditionalOptions::default())
  });

  // See the section Registers & Parameters, above for setting default values.
  //======================================================================
  // TeX Book, Appendix B, p. 349
  // See the section Registers & Parameters, above for setting default values.

  // These are originally defined with \newskip, etc
  DefRegister!("\\smallskipamount", Glue!("3pt plus1pt minus1pt"));
  DefRegister!("\\medskipamount", Glue!("6pt plus2pt minus2pt"));
  DefRegister!("\\bigskipamount", Glue!("12pt plus4pt minus4pt"));
  DefRegister!("\\normalbaselineskip", Glue!("12pt"));
  DefRegister!("\\normallineskip", Glue!("1pt"));
  DefRegister!("\\normallineskiplimit", Dimension!("0pt"));
  DefRegister!("\\jot", Dimension!("3pt"));

  let jot_val = LookupRegister!("\\jot");

  DefRegister!("\\lx@default@jot", jot_val);
  DefRegister!("\\interdisplaylinepenalty", Number(100));
  DefRegister!("\\interfootnotelinepenalty", Number(100));

  DefMacro!("\\magstephalf", "1095");
  DefMacro!("\\magstep{}", sub[(mag)] {
    Explode!(match mag.to_string().as_str() {
      "0" => "1000",
      "1" => "1200",
      "2" => "1440",
      "3" => "1728",
      "4" => "2074",
      "5" => "2488",
      _ => ""
    })
  });

});
