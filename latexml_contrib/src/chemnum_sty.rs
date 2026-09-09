//! chemnum.sty — comprehensive numbering of chemical compounds
//! (`\cmpd`, `\refcmpd`, `\initcmpd`, `\setchemnum`, ...).
//!
//! Numbering model following Clemens Niederberger's `chemnum.sty` (v1.3a):
//! - Lines 228-250: Counter `cmpdmain` (starts at 0, arabic) and subcounter (`\alph`).
//! - Lines 260-261: Compound separator `.` (`\l_chemnum_compound_separator_tl`).
//! - Lines 640-646: `\chemnum_define_compound:n`: increments `cmpdmain` on first use.
//! - Lines 660-701: `\chemnum_define_subcompound:nn`: subcompounds per main compound
//!   are 1-indexed and formatted with `\alph` (`1` -> `'a'`, `2` -> `'b'`, etc.).
//!   Combined representation is `<main><sub_alph>` (e.g. `1a`).
//! - Lines 1373-1380: `\chemnum_write_main`: first use creates target
//!   (`\chemnum_hyper_target`), subsequent uses create hyperlink (`\chemnum_hyper_link`).
//! - Lines 1676-1701: `\chemnum_cmpd:nnnn`: list handling, comma splitting, star/plus handling.
//! - Lines 1976-1983: User macros: `\cmpd`, `\refcmpd`, `\labelcmpd`.
//! - Lines 2016-2026: `\initcmpd` (`\cmpdinit`), `\resetcmpd` (`\cmpdreset`).
//!
//! Witness arXiv:2103.03138 — loads `chemfig + chemnum`.

use std::cell::RefCell;

use latexml_package::prelude::*;

#[derive(Default, Debug)]
struct CompoundEntry {
  number:         usize,
  target_emitted: bool,
  sub_counter:    usize,
  sub_compounds:  HashMap<String, SubCompoundEntry>,
}

#[derive(Clone, Debug)]
struct SubCompoundEntry {
  index:          usize,
  target_emitted: bool,
}

#[derive(Default, Debug)]
struct ChemnumState {
  counter:   usize,
  compounds: HashMap<String, CompoundEntry>,
}

thread_local! {
  static CHEMNUM_STATE: RefCell<ChemnumState> = RefCell::new(ChemnumState::default());
}

fn reset_chemnum_state(start_val: usize) {
  CHEMNUM_STATE.with(|st| {
    let mut state = st.borrow_mut();
    state.counter = start_val;
    state.compounds.clear();
  });
}

fn format_sub(n: usize) -> String {
  if n == 0 {
    return String::new();
  }
  let mut res = String::new();
  let mut num = n - 1;
  loop {
    let rem = num % 26;
    res.insert(0, (b'a' + rem as u8) as char);
    if num < 26 {
      break;
    }
    num = (num / 26) - 1;
  }
  res
}

fn split_top_level_commas(input: &str) -> Vec<String> {
  let mut parts = Vec::new();
  let mut current = String::new();
  let mut depth = 0;
  for ch in input.chars() {
    match ch {
      '{' => {
        depth += 1;
        current.push(ch);
      },
      '}' => {
        if depth > 0 {
          depth -= 1;
        }
        current.push(ch);
      },
      ',' if depth == 0 => {
        let trimmed = current.trim();
        if !trimmed.is_empty() {
          parts.push(trimmed.to_string());
        }
        current.clear();
      },
      _ => current.push(ch),
    }
  }
  let trimmed = current.trim();
  if !trimmed.is_empty() {
    parts.push(trimmed.to_string());
  }
  parts
}

fn expand_compound_spec(item: &str) -> Vec<(String, Option<String>)> {
  let item = item.trim();
  if let Some((main_part, sub_part)) = item.split_once('.') {
    let main_trimmed = main_part.trim().to_string();
    let sub_trimmed = sub_part.trim();
    if sub_trimmed.starts_with('{') && sub_trimmed.ends_with('}') {
      let inner = &sub_trimmed[1..sub_trimmed.len() - 1];
      let sub_items = split_top_level_commas(inner);
      if sub_items.is_empty() {
        vec![(main_trimmed, None)]
      } else {
        sub_items
          .into_iter()
          .map(|s| (main_trimmed.clone(), Some(s)))
          .collect()
      }
    } else {
      vec![(main_trimmed, Some(sub_trimmed.to_string()))]
    }
  } else {
    vec![(item.to_string(), None)]
  }
}

fn register_compound(main_label: &str, sub_label: Option<&str>) {
  CHEMNUM_STATE.with(|st| {
    let mut state = st.borrow_mut();
    if !state.compounds.contains_key(main_label) {
      state.counter += 1;
      let counter = state.counter;
      state
        .compounds
        .insert(main_label.to_string(), CompoundEntry {
          number:         counter,
          target_emitted: false,
          sub_counter:    0,
          sub_compounds:  HashMap::default(),
        });
    }
    let entry = state.compounds.get_mut(main_label).unwrap();
    if let Some(sub_name) = sub_label
      && !entry.sub_compounds.contains_key(sub_name)
    {
      entry.sub_counter += 1;
      let sub_counter = entry.sub_counter;
      entry
        .sub_compounds
        .insert(sub_name.to_string(), SubCompoundEntry {
          index:          sub_counter,
          target_emitted: false,
        });
    }
  });
}

fn lookup_or_register(
  main_label: &str,
  sub_label: Option<&str>,
  is_ref: bool,
  sub_only: bool,
) -> (String, String, bool) {
  CHEMNUM_STATE.with(|st| {
    let mut state = st.borrow_mut();
    if !state.compounds.contains_key(main_label) {
      state.counter += 1;
      let counter = state.counter;
      state
        .compounds
        .insert(main_label.to_string(), CompoundEntry {
          number:         counter,
          target_emitted: false,
          sub_counter:    0,
          sub_compounds:  HashMap::default(),
        });
    }
    let entry = state.compounds.get_mut(main_label).unwrap();
    let main_num = entry.number;

    if let Some(sub_name) = sub_label {
      if !entry.sub_compounds.contains_key(sub_name) {
        entry.sub_counter += 1;
        let sub_counter = entry.sub_counter;
        entry
          .sub_compounds
          .insert(sub_name.to_string(), SubCompoundEntry {
            index:          sub_counter,
            target_emitted: false,
          });
      }
      let sub_entry = entry.sub_compounds.get_mut(sub_name).unwrap();
      let sub_index = sub_entry.index;
      let sub_str = format_sub(sub_index);
      let display_text = if sub_only {
        sub_str
      } else {
        format!("{main_num}{sub_str}")
      };
      let clean_target_id = clean_id(&format!("cmpd.{main_label}.{sub_name}"));
      let emit_as_target = !sub_entry.target_emitted && !is_ref;
      if emit_as_target {
        sub_entry.target_emitted = true;
      }
      (clean_target_id, display_text, emit_as_target)
    } else {
      let display_text = format!("{main_num}");
      let clean_target_id = clean_id(&format!("cmpd.{main_label}"));
      let emit_as_target = !entry.target_emitted && !is_ref;
      if emit_as_target {
        entry.target_emitted = true;
      }
      (clean_target_id, display_text, emit_as_target)
    }
  })
}

fn chemnum_process_labels(
  labels_str: &str,
  is_silent: bool,
  is_ref: bool,
  sub_only: bool,
) -> Tokens {
  if is_silent {
    for item in split_top_level_commas(labels_str) {
      for (main, sub) in expand_compound_spec(&item) {
        register_compound(&main, sub.as_deref());
      }
    }
    return Tokens::new(Vec::new());
  }

  let mut result_toks = Vec::new();
  let items = split_top_level_commas(labels_str);
  let mut first_item = true;

  for item in items {
    for (main, sub) in expand_compound_spec(&item) {
      if !first_item {
        result_toks.push(T_OTHER!(","));
        result_toks.push(T_SPACE!());
      }
      first_item = false;

      let (id, display, is_target) = lookup_or_register(&main, sub.as_deref(), is_ref, sub_only);
      result_toks.push(if is_target {
        T_CS!("\\chemnumEmitTarget")
      } else {
        T_CS!("\\chemnumEmitRef")
      });
      result_toks.push(T_BEGIN!());
      result_toks.extend(Explode!(id));
      result_toks.push(T_END!());
      result_toks.push(T_BEGIN!());
      result_toks.extend(Explode!(display));
      result_toks.push(T_END!());
    }
  }

  Tokens::new(result_toks)
}

LoadDefinitions!({
  // chemnum.sty:51-55 requires translations, chemgreek (its label alphabet;
  // chemgreek.sty:486 defines the preamble-only `\activatechemgreekmapping`
  // that documents call after `\usepackage{chemnum}` — undefined while the
  // binding loaded neither) and psfrag. Batch 56ax.
  RequirePackage!("translations");
  RequirePackage!("chemgreek");
  RequirePackage!("psfrag");
  reset_chemnum_state(0);
  model::add_tag_attribute("ltx:text", vec!["idref"]);

  DefConstructor!(
    "\\chemnumEmitTarget Semiverbatim {}",
    "<ltx:text class='ltx_cmpd' xml:id='#1'>#2</ltx:text>"
  );
  DefConstructor!(
    "\\chemnumEmitRef Semiverbatim {}",
    "<ltx:text class='ltx_cmpd' idref='#1'>#2</ltx:text>"
  );

  DefMacro!(
    "\\cmpd OptionalMatch:* OptionalMatch:+ [] {}",
    sub[(star, plus, opt, labels)] {
      let is_silent = star.is_some();
      let is_ref = plus.is_some();
      let opt_str = opt.as_ref().map(|o| o.to_string()).unwrap_or_default();
      let sub_only = opt_str.contains("sub-only");
      let labels_str = labels.to_string();
      Ok(chemnum_process_labels(&labels_str, is_silent, is_ref, sub_only))
    },
    protected => true
  );

  DefMacro!(
    "\\refcmpd [] {}",
    sub[(opt, labels)] {
      let opt_str = opt.as_ref().map(|o| o.to_string()).unwrap_or_default();
      let sub_only = opt_str.contains("sub-only");
      let labels_str = labels.to_string();
      Ok(chemnum_process_labels(&labels_str, false, true, sub_only))
    },
    protected => true
  );

  DefMacro!(
    "\\labelcmpd [] {}",
    sub[(_opt, labels)] {
      let labels_str = labels.to_string();
      Ok(chemnum_process_labels(&labels_str, true, false, false))
    },
    protected => true
  );

  DefMacro!(
    "\\initcmpd [] {}",
    sub[(_opt, labels)] {
      let labels_str = labels.to_string();
      Ok(chemnum_process_labels(&labels_str, true, false, false))
    },
    protected => true
  );

  DefMacro!(
    "\\cmpdinit [] {}",
    sub[(_opt, labels)] {
      let labels_str = labels.to_string();
      Ok(chemnum_process_labels(&labels_str, true, false, false))
    },
    protected => true
  );

  DefMacro!(
    "\\resetcmpd []",
    sub[(opt)] {
      let val = opt
        .as_ref()
        .and_then(|o| o.to_string().trim().parse::<usize>().ok())
        .unwrap_or(1);
      reset_chemnum_state(val.saturating_sub(1));
      Ok(Tokens::new(Vec::new()))
    },
    protected => true
  );

  DefMacro!(
    "\\cmpdreset []",
    sub[(opt)] {
      let val = opt
        .as_ref()
        .and_then(|o| o.to_string().trim().parse::<usize>().ok())
        .unwrap_or(1);
      reset_chemnum_state(val.saturating_sub(1));
      Ok(Tokens::new(Vec::new()))
    },
    protected => true
  );

  DefMacro!(
    "\\cmpdplain {}",
    sub[(label)] {
      let label_str = label.to_string();
      let (_id, display, _) = lookup_or_register(label_str.trim(), None, true, false);
      let mut toks = Vec::new();
      toks.extend(Explode!(display));
      Ok(Tokens::new(toks))
    },
    protected => true
  );

  DefMacro!(
    "\\subcmpdplain {} {}",
    sub[(main_lab, sub_lab)] {
      let main_str = main_lab.to_string();
      let sub_str = sub_lab.to_string();
      let (_id, display, _) =
        lookup_or_register(main_str.trim(), Some(sub_str.trim()), true, true);
      let mut toks = Vec::new();
      toks.extend(Explode!(display));
      Ok(Tokens::new(toks))
    },
    protected => true
  );

  DefMacro!(
    "\\submaincmpdplain {} {}",
    sub[(main_lab, sub_lab)] {
      let main_str = main_lab.to_string();
      let sub_str = sub_lab.to_string();
      let (_id, display, _) =
        lookup_or_register(main_str.trim(), Some(sub_str.trim()), true, false);
      let mut toks = Vec::new();
      toks.extend(Explode!(display));
      Ok(Tokens::new(toks))
    },
    protected => true
  );

  // Stubs for configuration and secondary commands:
  DefMacro!("\\setchemnum {}", "");
  DefMacro!("\\cmpdsetup {}", "");
  DefMacro!("\\setcmpdproperty {} {} {}", "");
  DefMacro!("\\setcmpdlabel {} {}", "");
  DefMacro!("\\newcmpdcounterformat {} {}", "");
  DefMacro!("\\replacecmpd OptionalMatch:* {}", "");
  DefMacro!("\\cmpdprintlabelid {}", "");
  DefMacro!("\\cmpdshowlabelmargin {}", "");
  DefMacro!("\\cmpdshowlabelinline {}", "");
  DefMacro!("\\chemnumshowdef {}", "");
  DefMacro!("\\chemnumshowref {}", "");
  DefMacro!("\\cmpdshowdef {}", "");
  DefMacro!("\\cmpdshowref {}", "");
  DefMacro!("\\subcmpdshowdef {} {}", "");
  DefMacro!("\\subcmpdshowref {} {}", "");
});
