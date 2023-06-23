use crate::package::*;
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
    // afterDigest  => sub { my ($stomach, $whatsit) = @_;
    //   my $options = $whatsit->getArg(1);
    //   my $class   = ToString($whatsit->getArg(2));
    //   $class =~ s/\s+//g;
    //   $options = [($options ? split(/\s*,\s*/, (ToString($options))) : ())];
    //   LoadClass($class, options => $options);
    //   return; }
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

  // DefPrimitive!("\\PassOptionsToPackage{}{}", sub[(name, options)] {
  //   // $name = ToString($name);
  //   // $name =~ s/\s+//g;
  //   // PassOptions($name, 'sty', split(/\s*,\s*/, ToString(Expand($options))));
  //   Ok(Vec::new())
  // });

  // DefPrimitive!("\\PassOptionsToClass{}{}", sub[(name, options)] {
  //   // $name = ToString($name);
  //   // $name =~ s/\s+//g;
  //   // PassOptions($name, 'cls', split(/\s*,\s*/, ToString(Expand($options))));
  //   Ok(Vec::new())
  // });

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
  // DefPrimitive!("\\@onefilewithoptions {} [][] {}",
  // sub[(name,option1,option2,ext)] {
  //   // InputDefinitions(ToString(Expand($name)),
  //   //    type => ToString(Expand($ext)), options => $option1);
  //   Ok(Vec::new())
  // });

  DefMacro!("\\CurrentOption", None);

  DefPrimitive!("\\ExecuteOptions{}", sub[(options)] {
    // TODO!
    // ExecuteOptions!(split(/\s*,\s*/, ToString(Expand($options))));
    Info!("TODO","\\ExecuteOptions","implement fully, it's an empty stub.");
    Ok(Vec::new())
  });

  DefPrimitive!("\\ProcessOptions OptionalMatch:*", sub[(star)] {
    // TODO:
    // if star.is_some() {
    //   "inorder"
    // }
    // ProcessOptions!(($star ? (inorder => 1) : ()));
    Info!("TODO","\\ProcessOptions","implement fully, missing 'inorder'");
    process_options()?;
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
  if lookup_bool(&s!("{path}_loaded")) || lookup_bool(&s!("{path}_binding_loaded")) {
    T_CS!("\\@firstoftwo")
  } else {
    T_CS!("\\@secondoftwo")
  }});

  DefMacro!("\\@ifpackagewith", r"\@if@ptions\@pkgextension");
  DefMacro!("\\@ifclasswith", r"\@if@ptions\@clsextension");
  DefMacro!(
    "\\@ptionlist {}",
    r"\@ifundefined{opt@#1}\@empty{\csname opt@#1\endcsname}"
  );

  DefPrimitive!("\\g@addto@macro DefToken {}", sub[(target, content)] {
    AddToMacro!(target, content);
  });
  DefMacro!("\\addto@hook DefToken {}", "#1\\expandafter{\\the#1#2}");
});
