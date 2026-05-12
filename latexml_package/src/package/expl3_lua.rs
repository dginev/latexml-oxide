/// Translation of expl3.lua.ltxml — native intarray operations for expl3
///
/// Provides efficient Rust-native implementations of intarray operations that
/// bypass the TeX-based fontdimen intarray code in expl3-code.tex.
use crate::prelude::*;
use std::cell::RefCell;
use rustc_hash::FxHashMap as HashMap;

thread_local! {
  static INTARRAYS: RefCell<HashMap<String, Vec<i64>>> = RefCell::new(HashMap::default());
}

fn intarray_key(token: &Token, index: i64) -> String { s!("__intarray_{}_{}", token, index) }

/// Read an intarray identifier: \__intarray:w <Number>
fn read_intarray_key() -> Result<String> {
  let tok = gullet::read_x_token(Some(false), false, None)?;
  if let Some(ref t) = tok {
    if t.defined_as(&T_CS!("\\__intarray:w")) {
      let n = gullet::read_number()?;
      return Ok(intarray_key(t, n.value_of()));
    } else {
      gullet::unread_one(*t);
    }
  }
  Error!(
    "expected",
    "__intarray:w",
    s!("Expected an intarray identifier, got {:?}", tok)
  );
  Ok(String::new())
}

fn with_intarray<F, R>(key: &str, f: F) -> R
where F: FnOnce(&mut Vec<i64>) -> R {
  INTARRAYS.with(|arrays| {
    let mut arrays = arrays.borrow_mut();
    let arr = arrays.entry(key.to_string()).or_default();
    f(arr)
  })
}

#[rustfmt::skip]
LoadDefinitions!({
  // 2026-05-11: Rust-native intarray bindings DISABLED in favor of the
  // OLD-path font-hack implementation provided by raw expl3-code.tex.
  //
  // The motivating issue was glossaries → mfirstuc → expl3 case folding
  // (`\Gls{cabbage}` → "Cabbage") returning empty case maps because the
  // codepoint setup block stores per-codepoint case data in IntArrays
  // (`\g__codepoint_<case>_blocks_intarray`). Two engine gaps fed the
  // mismatch:
  //   1. `\fontdimen` had no setter, so OLD-path writes were silent
  //      no-ops. Fixed in tex_fonts.rs.
  //   2. The font shared key normalised "at <N> sp" to "0.0pt" / "0.000000pt",
  //      collapsing every cmr10-derived intarray into the same fontdimen
  //      bucket. Also fixed in tex_fonts.rs.
  //
  // With (1)+(2) fixed, the OLD-path font-hack stores intarray state
  // faithfully. Keeping the Rust-native overrides AT THE SAME TIME caused
  // a split: `\intarray_count:N` (Rust) read from the empty Rust HashMap
  // and reported 0, then `\__intarray_gset:Nww`'s bounds check rejected
  // every write — so even though fontdimen storage worked, no value ever
  // reached it. Path B (read from font-hack inside Rust overrides) would
  // duplicate logic; Path A (drop Rust overrides) lets the
  // dump-loaded expl3-code.tex definitions run end-to-end. Choosing Path A.
  //
  // If this file ever resurrects Rust-native intarray, ALSO override
  // `\__intarray_gset:Nww`, `\__intarray_bounds:NNnTF`, `\__kernel_intarray_gset:Nnn`,
  // and the `\intarray_count:N` raw-load definition so the Rust storage
  // is the single source of truth.

  /*
  // \__intarray:w — marker token (protected primitive)
  DefPrimitive!(T_CS!("\\__intarray:w"), None, sub[_args] {
    Error!("unexpected", "\\__intarray:w", "Unexpected isolated \\__intarray:w?");
  });

  // \__intarray_gset_count:Nw <Intarray> <Number>
  DefPrimitive!(T_CS!("\\__intarray_gset_count:Nw"), None, sub[_args] {
    let key = read_intarray_key()?;
    let newlength = gullet::read_number()?.value_of() as usize;
    with_intarray(&key, |arr| arr.resize(newlength, 0));
  });

  // \intarray_count:N <Intarray> → digit tokens
  DefMacro!(T_CS!("\\intarray_count:N"), None, sub[_args] {
    let key = read_intarray_key()?;
    let len = with_intarray(&key, |arr| arr.len());
    let toks: Vec<Token> = Explode!(len as i64);
    Ok(Tokens::new(toks))
  });

  // \__intarray_gset:wF <Number> <Intarray> <Number> {<ifmissing>}
  DefMacro!(T_CS!("\\__intarray_gset:wF"), None, sub[_args] {
    let pos = gullet::read_number()?;
    let key = read_intarray_key()?;
    let value = gullet::read_number()?;
    let ifmissing = gullet::read_arg(ExpansionLevel::Off)?;
    let pos_val = pos.value_of() as usize;
    let value_val = value.value_of();
    let in_bounds = with_intarray(&key, |arr| {
      if pos_val > 0 && pos_val <= arr.len() {
        arr[pos_val - 1] = value_val;
        true
      } else {
        false
      }
    });
    if in_bounds { Ok(Tokens!()) } else { Ok(ifmissing) }
  });

  // \__intarray_gset:w <Number> <Intarray> <Number>
  DefPrimitive!(T_CS!("\\__intarray_gset:w"), None, sub[_args] {
    let pos = gullet::read_number()?;
    let key = read_intarray_key()?;
    let value = gullet::read_number()?;
    let pos_val = pos.value_of() as usize;
    let value_val = value.value_of();
    with_intarray(&key, |arr| {
      if pos_val > 0 {
        if pos_val > arr.len() { arr.resize(pos_val, 0); }
        arr[pos_val - 1] = value_val;
      }
    });
  });

  // \intarray_gzero:N <Intarray>
  DefPrimitive!(T_CS!("\\intarray_gzero:N"), None, sub[_args] {
    let key = read_intarray_key()?;
    with_intarray(&key, |arr| {
      for elem in arr.iter_mut() { *elem = 0; }
    });
  });

  // \__intarray_item:wF <Number> <Intarray> {<ifmissing>}
  DefMacro!(T_CS!("\\__intarray_item:wF"), None, sub[_args] {
    let pos = gullet::read_number()?;
    let key = read_intarray_key()?;
    let ifmissing = gullet::read_arg(ExpansionLevel::Off)?;
    let pos_val = pos.value_of() as usize;
    let result = with_intarray(&key, |arr| {
      if pos_val > 0 && pos_val <= arr.len() {
        Some(arr[pos_val - 1])
      } else { None }
    });
    if let Some(val) = result {
      Ok(Tokens::new(Explode!(val)))
    } else { Ok(ifmissing) }
  });

  // \__intarray_item:w <Number> <Intarray>
  DefMacro!(T_CS!("\\__intarray_item:w"), None, sub[_args] {
    let pos = gullet::read_number()?;
    let key = read_intarray_key()?;
    let pos_val = pos.value_of() as usize;
    let result = with_intarray(&key, |arr| {
      if pos_val > 0 && pos_val <= arr.len() {
        Some(arr[pos_val - 1])
      } else { None }
    });
    if let Some(val) = result {
      Ok(Tokens::new(Explode!(val)))
    } else { Ok(Tokens!()) }
  });

  // \__intarray_to_clist:Nn <Intarray>
  DefMacro!(T_CS!("\\__intarray_to_clist:Nn"), None, sub[_args] {
    let key = read_intarray_key()?;
    let s = with_intarray(&key, |arr| {
      arr.iter().map(|v| v.to_string()).collect::<Vec<_>>().join(", ")
    });
    Ok(Tokenize!(&s))
  });

  // \__intarray_range_to_clist:w <Intarray> <Number> <Number>
  DefMacro!(T_CS!("\\__intarray_range_to_clist:w"), None, sub[_args] {
    let key = read_intarray_key()?;
    let from = gullet::read_number()?;
    let to = gullet::read_number()?;
    let s = with_intarray(&key, |arr| {
      let len = arr.len();
      if len == 0 { return String::new(); }
      let from_idx = (from.value_of() as usize).max(1).min(len) - 1;
      let to_idx = (to.value_of() as usize).max(from_idx + 1).min(len) - 1;
      if from_idx <= to_idx && to_idx < len {
        arr[from_idx..=to_idx].iter().map(|v| v.to_string()).collect::<Vec<_>>().join(", ")
      } else { String::new() }
    });
    Ok(Tokenize!(&s))
  });

  // \__intarray_gset_range:w <Number> <Intarray>
  DefPrimitive!(T_CS!("\\__intarray_gset_range:w"), None, sub[_args] {
    let _from = gullet::read_number()?;
    let _key = read_intarray_key()?;
    Error!("unimplemented", "intarray_gset_range",
      "\\__intarray_gset_range:w is not yet implemented");
  });
  */
  // Reference unused fns to satisfy dead-code lint.
  let _: fn() -> Result<String> = read_intarray_key;
  let _ = intarray_key;
});

// Silence dead-code warning on `with_intarray` while bindings are commented out.
#[allow(dead_code)]
fn _unused_with_intarray(key: &str) -> usize {
  with_intarray(key, |arr| arr.len())
}
