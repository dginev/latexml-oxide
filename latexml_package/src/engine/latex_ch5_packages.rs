use crate::prelude::*;
static OPTS_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r",\s*").unwrap());

LoadDefinitions!({
  // ======================================================================
  // C.5.2 Packages
  // ======================================================================
  // We'll prefer to load package.pm, but will try package.sty or
  // package.tex (the latter being unlikely to work, but....)
  // See Stomach.pm for details
  // Ignorable packages ??
  // pre-defined packages??

  DefMacro!("\\@clsextension", "cls");
  DefMacro!("\\@pkgextension", "sty");
  Let!("\\@currext", "\\@empty");
  Let!("\\@currname", "\\@empty");
  Let!("\\@classoptionslist", "\\relax");
  Let!("\\@raw@classoptionslist", "\\relax");
  DefMacro!("\\@declaredoptions", None);
  DefMacro!("\\@curroptions", None);
  DefMacro!("\\@unusedoptionlist", None);

  DefConstructor!("\\usepackage OptionalSemiverbatim Semiverbatim []",
                  "<?latexml package='#2' ?#1(options='#1')?>",
    before_digest => { only_preamble("\\usepackage") },
    after_digest => sub[whatsit] {
      let options: Option<&Digested> = whatsit.get_arg(1);
      let packages: Option<&Digested> = whatsit.get_arg(2);
      let package_list = match packages {
        Some(value) => OPTS_REGEX.split(&value.to_string())
          .map(ToString::to_string).filter(|s| !s.starts_with('%')).collect(),
        None => Vec::new(),
      };
      let options_list = match options {
        Some(opts) => OPTS_REGEX.split(&opts.to_string()).map(ToString::to_string).collect(),
        None => Vec::new(),
      };
      for package in package_list {
        require_package(&package, RequireOptions {
          options: options_list.clone(),
          ..RequireOptions::default()
        })?
      }
      Ok(Vec::new())
    }
  );

  DefConstructor!("\\RequirePackage OptionalSemiverbatim Semiverbatim []",
  "<?latexml package='#2' ?#1(options='#1')?>",
  before_digest =>  { only_preamble("\\RequirePackage") },
  after_digest => sub[whatsit] {
    // let options  = whatsit.get_arg(1);
    let packages = whatsit.get_arg(2).unwrap();
  //   $options = [($options ? split(/\s*,\s*/, (ToString($options))) : ())];
    for pkg in packages.to_string().split(',') {
      let pkg_trimmed = pkg.trim();
      if pkg_trimmed.is_empty() || pkg.starts_with('%') { continue; }
      require_package(pkg, RequireOptions::default())?;
    }
  });

  DefConstructor!("\\LoadClass OptionalSemiverbatim Semiverbatim []",
    "<?latexml class='#2' ?#1(options='#1')?>",
    before_digest => { only_preamble("\\LoadClass") }
    after_digest => sub[whatsit] {
      let options_arg: Option<&Digested> = whatsit.get_arg(1);
      let class_arg: Option<&Digested> = whatsit.get_arg(2);
      let class = class_arg.map(|c| c.to_string().replace(' ', "")).unwrap_or_default();
      let options: Vec<String> = match options_arg {
        Some(opts) => OPTS_REGEX.split(&opts.to_string())
          .map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect(),
        None => Vec::new(),
      };
      load_class(&class, options, Tokens!())?;
    }
  );

  // Related internal macros for package definition
  // Internals used in Packages
  DefMacro!("\\NeedsTeXFormat{}[]", None);

  DefPrimitive!("\\ProvidesClass{}[]", sub[(class, version_opt)] {
    let ver_cs = T_CS!(s!("\\ver@{class}.cls"));
    let version = version_opt.unwrap_or_default();
    DefMacro!(ver_cs, None, version, scope => Some(Scope::Global));
  });

  // Note that these, like LaTeX, define macros like \var@mypkg.sty to give the version info.
  DefMacro!("\\ProvidesPackage{}[]", sub[(package, version_opt)] {
    let ver_cs = T_CS!(s!("\\ver@{package}.sty"));
    let version = version_opt.unwrap_or_default();
    DefMacro!(ver_cs, None, version, scope => Some(Scope::Global));
  });

  DefMacro!("\\ProvidesFile{}[]", sub[(file, version_opt)] {
    let ver_cs = T_CS!(s!("\\ver@{file}"));
    let version = version_opt.unwrap_or_default();
    DefMacro!(ver_cs, None, version, scope => Some(Scope::Global));
  });

  // anything useful?
  //\DeclareRelease{v4.46}{2020-03-19}{glossaries-2020-03-19.sty}
  DefMacro!("\\DeclareRelease{}{}{}", None);
  //\DeclareCurrentRelease{v4.49}{2021-11-01}
  DefMacro!("\\DeclareCurrentRelease{}{}", None);
  DefMacro!("\\IncludeInRelease{}{}{} Until:\\EndIncludeInRelease", None);
  DefMacro!("\\NewModuleRelease{}{}{} Until:\\EndModuleRelease", None);

  DefPrimitive!("\\DeclareOption{}{}", sub[(option, code)] {
    let option_str = option.to_string();
    if option_str == "*" {
      DeclareOption!(None, code);
    } else {
      DeclareOption!(option_str, code);
    }
    Ok(Vec::new())
  });

  // Perl: latex_constructs.pool.ltxml lines 868-878
  DefPrimitive!("\\PassOptionsToPackage{}{}", sub[(options, name)] {
    let name_str = Expand!(name).to_string().replace(' ', "");
    let opts_str = Expand!(options).to_string();
    let opts: Vec<String> = opts_str.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();
    state::push_value(&s!("opt@{}.sty", name_str), opts)?;
  });

  DefPrimitive!("\\PassOptionsToClass{}{}", sub[(options, name)] {
    let name_str = Expand!(name).to_string().replace(' ', "");
    let opts_str = Expand!(options).to_string();
    let opts: Vec<String> = opts_str.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();
    state::push_value(&s!("opt@{}.cls", name_str), opts)?;
  });

  DefConstructor!("\\RequirePackageWithOptions Semiverbatim []",
  "<?latexml package='#1'?>",
  before_digest => { only_preamble("\\RequirePackage") }
  // afterDigest  => sub { my ($stomach, $whatsit) = @_;
  //   my $package = ToString($whatsit->getArg(1));
  //   $package =~ s/\s+//g;
  //   RequirePackage($package, withoptions => 1);
  //   return; }
  );

  DefConstructor!("\\LoadClassWithOptions Semiverbatim []", "<?latexml class='#1'?>",
    before_digest => { only_preamble("\\LoadClassWithOptions") }
    // afterDigest  => sub { my ($stomach, $whatsit) = @_;
    //   my $class = ToString($whatsit->getArg(1));
    //   $class =~ s/\s+//g;
    //   LoadClass($class, withoptions => 1);
    //   return; });
  );
  // Perl: latex_constructs.pool.ltxml L900-903
  DefPrimitive!("\\@onefilewithoptions {} [][] {}", sub[(name, option1, _option2, ext)] {
    let name_str = Expand!(name).to_string();
    let ext_str = Expand!(ext).to_string();
    let opts_str = match option1 {
      Some(o) => Expand!(o).to_string(),
      None => String::new(),
    };
    let options: Vec<String> = opts_str.split(',')
      .map(|s| s.trim().to_string())
      .filter(|s| !s.is_empty())
      .collect();
    let _ = input_definitions(&name_str, NewDefault!(InputDefinitionOptions,
      extension => Some(Cow::Owned(ext_str)),
      handleoptions => true,
      options => options
    ));
  });

  DefMacro!("\\CurrentOption", None);

  // Perl: latex_constructs.pool.ltxml lines 907-919
  DefPrimitive!("\\OptionNotUsed", {
    let option = Expand!(T_CS!("\\CurrentOption")).to_string();
    if !option.is_empty() {
      let ext = Expand!(T_CS!("\\@currext")).to_string();
      if ext == "cls" {
        state::push_value("@unusedoptionlist", option)?;
      }
    }
  });
  DefPrimitive!("\\@unknownoptionerror", {
    let option = Expand!(T_CS!("\\CurrentOption")).to_string();
    let name = Expand!(T_CS!("\\@currname")).to_string();
    Info!("unexpected", &option, &s!("Unknown option '{}' for {}", option, name));
  });

  DefPrimitive!("\\ExecuteOptions{}", sub[(options)] {
    let expanded = do_expand(options)?.to_string();
    let opts: Vec<&str> = expanded.split(',').map(|s| s.trim()).filter(|s| !s.is_empty()).collect();
    execute_options(&opts)?;
    Ok(Vec::new())
  });

  DefPrimitive!("\\ProcessOptions OptionalMatch:*", sub[(star)] {
    // Perl: ProcessOptions(($star ? (inorder => 1) : ()));
    let inorder = star.is_some();
    process_options(inorder)?;
    Ok(Vec::new())
  });
  DefMacro!("\\@options", "\\ProcessOptions*");

  Let!("\\@enddocumenthook", "\\@empty");
  DefMacro!("\\AtEndOfPackage{}", sub [(code)] {
    let name = Expand!(T_CS!("\\@currname")).to_string();
    let ttype = Expand!(T_CS!("\\@currext")).to_string();
    let hookcs = T_CS!(s!("\\{name}.{ttype}-h@@k"));
    AddToMacro!(hookcs, code);
  });

  DefMacro!("\\@ifpackageloaded", r"\@ifl@aded\@pkgextension");
  Let!("\\ltx@ifpackageloaded", r"\@ifpackageloaded");
  DefMacro!("\\@ifclassloaded", r"\@ifl@aded\@clsextension");
  Let!("\\ltx@ifclassloaded", r"\@ifclassloaded");
  DefMacro!("\\@ifl@aded{}{}", sub[(ext, name)] {
  let path = s!("{}.{}", Expand!(name), Expand!(ext));
  // If EITHER the raw TeX or ltxml version of this file was loaded.
  if lookup_bool(&s!("{path}_loaded")) || lookup_bool(&s!("{path}.ltxml_loaded")) {
    T_CS!("\\@firstoftwo")
  } else {
    T_CS!("\\@secondoftwo")
  }});

  DefMacro!("\\@ifpackagewith", r"\@if@ptions\@pkgextension");
  DefMacro!("\\@ifclasswith", r"\@if@ptions\@clsextension");
  // Perl: latex_constructs.pool.ltxml lines 952-958
  DefMacro!("\\@if@ptions{}{}{}", sub[(ext, name, option)] {
    let option_str = Expand!(option).to_string();
    let key = s!("opt@{}.{}", Expand!(name), Expand!(ext));
    let found = with_value(&key, |val_opt| {
      if let Some(Stored::VecDequeStored(values)) = val_opt {
        values.iter().any(|v| v.to_string() == option_str)
      } else {
        false
      }
    });
    if found {
      T_CS!("\\@firstoftwo")
    } else {
      T_CS!("\\@secondoftwo")
    }
  });
  DefMacro!(
    "\\@ptionlist {}",
    r"\@ifundefined{opt@#1}\@empty{\csname opt@#1\endcsname}"
  );

  DefPrimitive!("\\g@addto@macro DefToken {}", sub[(target, content)] {
    AddToMacro!(target, content);
  });
  DefMacro!("\\addto@hook DefToken {}", "#1\\expandafter{\\the#1#2}");

  // Alas, we're not tracking versions, so we'll assume it's "later" & cross fingers....
  DefMacro!("\\@ifpackagelater{}{}{}{}", "#3");
  DefMacro!("\\@ifclasslater{}{}{}{}", "#3");
  Let!("\\AtEndOfClass", "\\AtEndOfPackage");

  DefMacro!("\\AtBeginDvi {}", None);

  TeX!(
    r###"
  \def\@ifl@t@r#1#2{%
    \ifnum\expandafter\@parse@version@#1//00\@nil<%
          \expandafter\@parse@version@#2//00\@nil
      \expandafter\@secondoftwo
    \else
      \expandafter\@firstoftwo
    \fi}
  \def\@parse@version@#1{\@parse@version0#1}
  \def\@parse@version#1/#2/#3#4#5\@nil{%
  \@parse@version@dash#1-#2-#3#4\@nil
  }
  \def\@parse@version@dash#1-#2-#3#4#5\@nil{%
    \if\relax#2\relax\else#1\fi#2#3#4 }"###
  );

  //======================================================================
  // Somewhat related I/O stuff
  DefMacro!("\\filename@parse{}", sub[(pathname)] {
    let (mut dir, name, ext) = pathname::split(&Expand!(pathname).to_string());
    if !dir.is_empty() {
      dir.push('/');
    }
    let dir_tokens = Tokens!(ExplodeText!(dir));
    DefMacro!("\\filename@area", None, dir_tokens);
    let name_tokens = Tokens!(ExplodeText!(name));
    DefMacro!("\\filename@base", None, name_tokens);
    let ext_tokens = if !ext.is_empty() {
      Tokens!(ExplodeText!(ext))
    } else { Tokens!(T_CS!("\\relax")) };
    DefMacro!("\\filename@ext", None, ext_tokens);
    Vec::new()
  });

  // latex.ltx initializes \@filelist to \@gobble, which eats the leading comma
  // from the first \@addtofilelist call. We replicate this by using \@gobble.
  DefMacro!("\\@filelist", "\\@gobble");
  DefMacro!("\\@addtofilelist{}", sub[(arg)] {
    let expansion = Expand!(Tokens!(T_CS!("\\@filelist"), T_OTHER!(","), arg.unlist()));
    DefMacro!("\\@filelist",None,expansion);
    Vec::new()
  });
});
