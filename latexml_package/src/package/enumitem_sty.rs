use crate::prelude::*;

LoadDefinitions!({
  // Package Options
  DeclareOption!("shortlabels", {
    AssignValue!("enumitem@shortlabels" => true);
  });
  DeclareOption!("inline", {
    AssignValue!("enumitem@inline" => true);
  });
  DeclareOption!("loadonly", {
    AssignValue!("enumitem@loadonly" => true);
  });
  ProcessOptions!();

  // KeyVals
  DefKeyVal!("enumitem", "label", "UndigestedKey");
  DefKeyVal!("enumitem", "label*", "UndigestedKey");
  DefKeyVal!("enumitem", "ref", "UndigestedKey");
  DefKeyVal!("enumitem", "font", "UndigestedKey");
  DefKeyVal!("enumitem", "format", "UndigestedKey");
  DefKeyVal!("enumitem", "start", "Number");
  DefKeyVal!("enumitem", "series", "UndigestedKey");
  DefKeyVal!("enumitem", "resume", "", "noseries");
  DefKeyVal!("enumitem", "resume*", "", "noseries");
  DefKeyVal!("enumitem", "style", "UndigestedKey");
  DefKeyVal!("enumitem", "itemjoin", "UndigestedKey");
  DefKeyVal!("enumitem", "itemjoin*", "UndigestedKey");
  DefKeyVal!("enumitem", "afterlabel", "UndigestedKey");
  DefKeyVal!("enumitem", "mode", "UndigestedKey");
  DefKeyVal!("enumitem", "align", "UndigestedKey");
  DefKeyVal!("enumitem", "labelindent", "Dimension");
  DefKeyVal!("enumitem", "left", "Dimension");
  DefKeyVal!("enumitem", "leftmargin", "UndigestedKey");
  DefKeyVal!("enumitem", "itemindent", "Dimension");
  DefKeyVal!("enumitem", "labelsep", "Dimension");
  DefKeyVal!("enumitem", "labelwidth", "Dimension");
  DefKeyVal!("enumitem", "widest", "UndigestedKey");
  DefKeyVal!("enumitem", "beginpenalty", "Number");
  DefKeyVal!("enumitem", "midpenalty", "Number");
  DefKeyVal!("enumitem", "endpenalty", "Number");
  DefKeyVal!("enumitem", "noitemsep", "", "true");
  DefKeyVal!("enumitem", "nolistsep", "", "true");
  DefKeyVal!("enumitem", "before", "UndigestedKey");
  DefKeyVal!("enumitem", "after", "UndigestedKey");

  if !has_value("enumitem@loadonly") {
    // Redefine itemize/enumerate/description to take OptionalKeyVals
    DefEnvironment!("{itemize} OptionalKeyVals:enumitem",
      "<ltx:itemize xml:id='#id'>#body</ltx:itemize>",
      properties => sub[_args] { BeginItemize!("itemize", "@item") },
      before_digest_end => { stomach::digest(Tokens!(T_CS!("\\par")))?; },
      mode => "internal_vertical",
      locked => true
    );
    DefEnvironment!("{enumerate} OptionalKeyVals:enumitem",
      "<ltx:enumerate xml:id='#id'>#body</ltx:enumerate>",
      properties => sub[_args] { BeginItemize!("enumerate", "enum") },
      before_digest_end => { stomach::digest(Tokens!(T_CS!("\\par")))?; },
      mode => "internal_vertical",
      locked => true
    );
    DefEnvironment!("{description} OptionalKeyVals:enumitem",
      "<ltx:description xml:id='#id'>#body</ltx:description>",
      before_digest => { Let!("\\makelabel", "\\descriptionlabel"); },
      properties => sub[_args] { BeginItemize!("description", "@desc") },
      before_digest_end => { stomach::digest(Tokens!(T_CS!("\\par")))?; },
      mode => "internal_vertical",
      locked => true
    );
  }

  if has_value("enumitem@inline") {
    DefEnvironment!("{itemize*} OptionalKeyVals:enumitem",
      "<ltx:inline-itemize xml:id='#id'>#body</ltx:inline-itemize>",
      properties => sub[_args] {
        begin_itemize("inline@itemize", Some("@item"), BeginItemizeOptions::default())
      },
      mode => "internal_vertical"
    );
    DefEnvironment!("{enumerate*} OptionalKeyVals:enumitem",
      "<ltx:inline-enumerate xml:id='#id'>#body</ltx:inline-enumerate>",
      properties => sub[_args] {
        begin_itemize("inline@enumerate", Some("enum"), BeginItemizeOptions::default())
      },
      mode => "internal_vertical"
    );
    DefEnvironment!("{description*} OptionalKeyVals:enumitem",
      "<ltx:inline-description xml:id='#id'>#body</ltx:inline-description>",
      properties => sub[_args] {
        begin_itemize("inline@description", Some("@desc"), BeginItemizeOptions::default())
      },
      mode => "internal_vertical"
    );
  }

  // \newlist, \renewlist — simplified stub
  DefMacro!("\\newlist{}{}{}", "");
  Let!("\\renewlist", "\\newlist");

  // \setlist — stub
  DefMacro!("\\setlist OptionalKeyVals:enumitem", "");
  DefMacro!("\\setitemize Optional {}", "");
  DefMacro!("\\setenumerate Optional {}", "");
  DefMacro!("\\setdescription Optional {}", "");

  DefMacro!("\\restartlist{}", "");

  // Not-yet-handled bits
  DefMacro!("\\SetLabelAlign{}{}", "");
  DefMacro!("\\EnumitemId", "");
  DefMacro!("\\SetEnumitemKey{}{}", "");
  DefMacro!("\\SetEnumerateShortLabel{}{}", "");
  DefMacro!("\\SetEnumitemValue{}{}{}", "");
  DefMacro!("\\SetEnumitemSize{}{}", "");
  DefMacro!("\\AddEnumerateCounter{}{}{}", "");
});
