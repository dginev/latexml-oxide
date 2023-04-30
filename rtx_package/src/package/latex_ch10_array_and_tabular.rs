use crate::package::*;
//======================================================================
// C.10.2 The array and tabular Environments
//======================================================================
// Tabular are a bit tricky in that we have to arrange for tr and td to
// be openned and closed at the right times; the only real markup is
// the & and \\. Also \multicolumn has to be cooperative.
// Along with this, we have to track which column specification applies
// to the current column.
// To simulate LaTeX's tabular borders & hlines, we simply add border
// attributes to all cells.  For HTML, CSS will be necessary to display them.
// [We'll ignore HTML's frame, rules and colgroup mechanisms.]

LoadDefinitions!(state, {
  DefRegister!("\\lx@arstrut", Dimension!("0pt"));
  DefRegister!("\\lx@default@tabcolsep", Dimension!("6pt"));
  DefRegister!("\\tabcolsep", Dimension!("6pt"));
  DefMacro!("\\arraystretch", None, T_OTHER!("1"));
  Let!("\\@tabularcr", "\\@alignment@newline");
  if LookupValue!("GUESS_TABULAR_HEADERS").is_none() {
    AssignValue!("GUESS_TABULAR_HEADERS" => true); // Defaults to yes
  }

  // Keyvals are for attributes for the alignment.
  // Typical keys are width, vattach,...
  DefKeyVal!("tabular", "width", "Dimension");
  DefPrimitive!("\\@tabular@bindings AlignmentTemplate OptionalKeyVals:tabular",
    sub[stomach, (template, attributes_opt), state] {
    let mut attrs = attributes_opt.map(KeyVals::as_flat_hash).unwrap_or_default();
    if let Some(va) = attrs.get("vattach") {
      attrs.insert(String::from("vattach"), Stored::String(arena::pin_static(translate_attachment(va))));
    }
    let gullet = stomach.get_gullet_mut();
    tabular_bindings(template, attrs, gullet, state)?;
  });

  DefMacro!("\\@tabular@before", None);
  DefMacro!("\\@tabular@after", None);
  DefMacro!("\\@tabular@row@before", None);
  DefMacro!("\\@tabular@row@after", None);
  DefMacro!("\\@tabular@column@before", None);
  DefMacro!("\\@tabular@column@after", None);

  // The Core alignment support is in LaTeXML::Core::Alignment and in TeX.ltxml
  DefMacro!("\\tabular[]{}",
    r"\@tabular@bindings{#2}[vattach=#1]\@@tabular[#1]{#2}\@start@alignment\@tabular@before",
    locked => true);
  DefMacro!("\\endtabular", r"\@tabular@after\@finish@alignment\@end@tabular",
    locked => true);
  DefPrimitive!("\\@end@tabular", sub[stomach,_a,state] { stomach.egroup(state)?; });
  DefConstructor!("\\@@tabular[] Undigested DigestedBody",
    "#3",
    reversion    => r"\begin{tabular}[#1]{#2}#3\end{tabular}",
    before_digest => sub[stomach,state] { stomach.bgroup(state); },
    sizer        => "#3",
    // TODO: vattach
    // after_digest  => sub[stomach,whatsit,state] {
    //   if let Some(Stored::Alignment(alignment)) = state.lookup_value("Alignment") {
    //     let attr = alignment.borrow_mut().get_property_mut("attributes");
    //     attr.insert(String::from("vattach"), Stored::String(arena::pin_static(translate_attachment(whatsit.get_arg(1)))));
    //   }
    // },
    locked => true,
    mode   => "text");

  // DefMacro!(T_CS!("tabular*"),"{Dimension}[]{}",
  //   r"\@tabular@bindings{#3}[width=#1,vattach=#2]\@@tabular@{#1}[#2]{#3}\@start@alignment");
  // DefMacro!(T_CS!("endtabular*"),
  //   r"\@finish@alignment\@end@tabular@");
  // DefConstructor!("\\@@tabular@{Dimension}[] Undigested DigestedBody",
  //   "#4",
  //   before_digest => sub[stomach,_a,state] { stomach.bgroup(); },
  //   reversion    => r"\begin{tabular*}{#1}[#2]{#3}#4\end{tabular*",
  //   mode         => "text");
  DefPrimitive!("\\@end@tabular@", sub [stomach,_args,state] { stomach.egroup(state)?; });
  Let!("\\multicolumn", "\\@multicolumn");

  // A weird bit that sometimes gets invoked by Cargo Cult programmers...
  // to \noalign in the defn of \hline! Bizarre! (see latex.ltx)
  // However, the really weird thing is the way this provides the } to close the argument
  DefMacro!("\\@xhline", r"\ifnum0=`{\fi}");

  DefMacro!("\\cline{}", r"\noalign{\@cline{#1}}");
  DefConstructor!("\\@cline{}", "",
    after_digest => sub[_stomach, whatsit,state] {
      let cols = whatsit.get_arg(1).map(ToString::to_string).unwrap_or_default();
      let mut cols_vec = Vec::new();
      let cols_chars = cols.chars();
      let mut from : Option<usize> = None;
      let mut num = String::new();
      for c_next in cols_chars {
        match c_next {
          ',' => if !num.is_empty() {
            let this_num = num.parse::<usize>().unwrap();
            if let Some(from_num) = from {
              for num_in_range in from_num..=this_num {
                cols_vec.push(num_in_range);
              }
            } else {
              cols_vec.push(this_num);
            }
            from = None;
            num = String::new();
          },
          '-' => {
            from = Some(num.parse::<usize>().unwrap());
            num = String::new();
          }
          c if c.is_ascii_digit() => num.push(c_next),
          _ => break
        }
      }
      if !num.is_empty() {
        let this_num = num.parse::<usize>().unwrap();
        if let Some(from_num) = from {
          for num_in_range in from_num..=this_num {
            cols_vec.push(num_in_range);
          }
        } else {
          cols_vec.push(this_num);
        }
      }
      if let Some(alignment_stored) = state.lookup_alignment("Alignment") {
        alignment_stored.alignment_cell().unwrap().borrow_mut()
          .add_line("t", cols_vec);
      }
      ()
    },
    sizer      => 0, alias => "\\cline",
    // properties => { "isHorizontalRule" => true }
  );

  DefConstructor!("\\vline", "",   // ???
    // properties => { "isVerticalRule" => true },
    sizer      => 0,
  );
  DefRegister!("\\lx@default@arraycolsep", Dimension!("5pt"));
  DefRegister!("\\arraycolsep", Dimension!("5pt"));
  DefRegister!("\\arrayrulewidth", Dimension!("0.4pt"));
  DefRegister!("\\doublerulesep", Dimension!("2pt"));
  DefMacro!("\\extracolsep{}", None);

  // Array and similar environments

  // DefPrimitive!("\\@array@bindings [] AlignmentTemplate", sub[stomach, (pos,template), state] {
  // my $attr = { vattach => translateAttachment($pos),
  //   role => 'ARRAY' };
  // # Determine column and row separations, if non default
  // my $colsep = LookupDimension('\arraycolsep');
  // if ($colsep && ($colsep->valueOf != LookupDimension('\lx@default@arraycolsep')->valueOf)) {
  //   $$attr{colsep} = $colsep; }
  // my $str = ToString(Expand(T_CS('\arraystretch')));
  // if ($str != 1) {
  //   $$attr{rowsep} = Dimension(($str - 1) . 'em'); }
  // alignmentBindings($template, 'math', attributes => $attr);
  // MergeFont(mathstyle => 'text');
  // Let("\\\\", '\@alignment@newline');

  // });

  DefMacro!(
    "\\array[]{}",
    r"\@array@bindings[#1]{#2}\@@array[#1]{#2}\@start@alignment"
  );
  DefMacro!("\\endarray", None, r"\@finish@alignment\@end@array");
  DefPrimitive!("\\@end@array", sub[stomach,_args,state] { stomach.egroup(state)?; });
  DefConstructor!("\\@@array[] Undigested DigestedBody",
    "#3",
    before_digest => sub[stomach,state] { stomach.bgroup(state); },
    reversion    => r"\begin{array}[#1]{#2}#3\end{array}");

  DefMacro!("\\@tabarray", r"\m@th\@@array[c]");
});
