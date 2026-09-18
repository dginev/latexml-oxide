//! wrapstuff.sty — text wrapped around a box (the expl3 successor of wrapfig).
//!
//! Raw, the package digests the environment body into a box register
//! (wrapstuff.sty:312-316) and places it from the LaTeX2e paragraph hooks
//! `para/begin`/`para/end` (:334, :539-552, :1905-1939), which neither
//! LaTeXML models (`\everypar` is inert: TeX_Paragraph.pool.ltxml:42,
//! tex_paragraph.rs:70): the box is built and never emitted — caption and
//! content lost at 0 errors (wrapstuff-doc-en, latex-via-exemplos ex15;
//! `~/data/pk_agents/w23/regr85/wrapstuff/NOTES.md`; raw, the content leaked
//! inline as a bare box while the float structure and the caption were
//! lost). The placement itself
//! needs `\prevgraf`, `\parshape` and `\vsplit` arithmetic no flow model
//! carries, so the environment is bound as the inline float wrapfig's
//! `wrapfigure`/`wraptable` are (`wrapfig_sty.rs`; Perl binds wrapfig the same
//! way, wrapfig.sty.ltxml:20-48): `type` (:2399) picks `ltx:table` /
//! `ltx:figure` (else a plain `ltx:float`), `l`/`r` (:2404-2405; the default
//! ratio `\c_one_fp`, :2461, is the right margin) the `float` side, `width`
//! (:2395) the width. The raw package is NOT loaded (bindings outrank raw).
use crate::{
  engine::latex_constructs::{after_float, before_float},
  prelude::*,
};

#[rustfmt::skip]
LoadDefinitions!({
  // The package's two public commands (wrapstuff.sty:2516-2521):
  // `\wrapstuffset{keys}` sets document-level defaults (`\keys_set:nn
  // {wrapstuff}{#1}`) — all typographic here, consumed; `\wrapstuffclear` =
  // `\par \__wstf_clear:` ends the wrapped paragraph, as wrapfig's `\WFclear`.
  def_macro_noop("\\wrapstuffset{}")?;
  DefMacro!("\\wrapstuffclear", "\\par");
  DefEnvironment!("{wrapstuff} OptionalKeyVals",
    "?#istable(<ltx:table xml:id='#id' inlist='#inlist' float='#float' ?#width(width='#width')>#tags#body</ltx:table>)\
     (?#isfigure(<ltx:figure xml:id='#id' inlist='#inlist' float='#float' ?#width(width='#width')>#tags#body</ltx:figure>)\
     (<ltx:float xml:id='#id' inlist='#inlist' float='#float' ?#width(width='#width')>#tags#body</ltx:float>))",
    mode => "internal_vertical",
    after_digest_begin => sub[whatsit] {
      let (ftype, side, width) = wrapstuff_keys(whatsit);
      whatsit.set_property("float", Stored::String(pin(side)));
      match ftype.as_str() {
        "table" => { whatsit.set_property("istable", Stored::Bool(true)); },
        "figure" => { whatsit.set_property("isfigure", Stored::Bool(true)); },
        _ => {},
      }
      // Only a typed box is a captioned float (wrapstuff.sty:2399 `type`
      // feeds `\@captype`); an untyped one is a plain wrapped box.
      if !ftype.is_empty() {
        whatsit.set_property("isfloat", Stored::Bool(true));
        before_float(&ftype, None);
      }
      if let Some(w) = width {
        set_wrap_width(whatsit, w);
      }
    },
    after_digest => sub[whatsit] {
      if whatsit.get_property("isfloat").is_some() {
        after_float(whatsit);
      }
      Ok(Vec::new())
    }
  );
});

/// `(type, float side, width)` from the environment's keyvals.
fn wrapstuff_keys(whatsit: &Whatsit) -> (String, &'static str, Option<Dimension>) {
  let mut ftype = String::new();
  let mut side = "right";
  let mut width = None;
  if let Some(arg) = whatsit.get_arg(1)
    && let DigestedData::KeyVals(ref kvs) = *arg.data()
  {
    if let Some(t) = kvs.get_value("type") {
      ftype = t.to_string().trim().to_string();
    }
    if kvs.get_value("l").is_some() {
      side = "left";
    }
    if kvs.get_value("r").is_some() {
      side = "right";
    }
    if let Some(w) = kvs.get_value("width") {
      let spec = w.to_string();
      if let Ok(d) = spec.trim().parse::<Dimension>() {
        width = Some(d);
      }
    }
  }
  (ftype, side, width)
}

/// As wrapfig's: the box width becomes the text width inside and a
/// percentage `width` on the element.
fn set_wrap_width(whatsit: &mut Whatsit, width: Dimension) {
  let v = width.value_of();
  if v == 0 {
    return;
  }
  let rv: RegisterValue = Dimension::new(v).into();
  let _ = assign_register("\\hsize", rv.clone(), None, Vec::new());
  let _ = assign_register("\\columnwidth", rv.clone(), None, Vec::new());
  let _ = assign_register("\\linewidth", rv, None, Vec::new());
  let Some(tw) = lookup_dimension("\\textwidth") else {
    return;
  };
  let tw_sp = tw.value_of();
  if tw_sp == 0 {
    return;
  }
  let pct = (100 * v) / tw_sp;
  whatsit.set_property("width", Stored::from(s!("{pct}%")));
}
