use crate::prelude::*;

fn setup_cyrillic() -> Result<()> {
  DefMacro!("\\cyra", "\u{0430}");
  DefMacro!("\\cyrb", "\u{0431}");
  DefMacro!("\\cyrv", "\u{0432}");
  DefMacro!("\\cyrg", "\u{0433}");
  DefMacro!("\\cyrd", "\u{0434}");
  DefMacro!("\\cyre", "\u{0435}");
  DefMacro!("\\cyrzh", "\u{0436}");
  DefMacro!("\\cyrz", "\u{0437}");
  DefMacro!("\\cyri", "\u{0438}");
  DefMacro!("\\cyrishrt", "\u{0439}");
  DefMacro!("\\cyrk", "\u{043A}");
  DefMacro!("\\cyrl", "\u{043B}");
  DefMacro!("\\cyrm", "\u{043C}");
  DefMacro!("\\cyrn", "\u{043D}");
  DefMacro!("\\cyro", "\u{043E}");
  DefMacro!("\\cyrp", "\u{043F}");
  DefMacro!("\\cyrr", "\u{0440}");
  DefMacro!("\\cyrs", "\u{0441}");
  DefMacro!("\\cyrt", "\u{0442}");
  DefMacro!("\\cyru", "\u{0443}");
  DefMacro!("\\cyrf", "\u{0444}");
  DefMacro!("\\cyrh", "\u{0445}");
  DefMacro!("\\cyrc", "\u{0446}");
  DefMacro!("\\cyrch", "\u{0447}");
  DefMacro!("\\cyrsh", "\u{0448}");
  DefMacro!("\\cyrshch", "\u{0449}");
  DefMacro!("\\cyrhrdsn", "\u{044A}");
  DefMacro!("\\cyrery", "\u{044B}");
  DefMacro!("\\cyrsftsn", "\u{044C}");
  DefMacro!("\\cyrerev", "\u{044D}");
  DefMacro!("\\cyryu", "\u{044E}");
  DefMacro!("\\cyrya", "\u{044F}");
  DefMacro!("\\cyryo", "\u{0451}");
  DefMacro!("\\CYRA", "\u{0410}");
  DefMacro!("\\CYRB", "\u{0411}");
  DefMacro!("\\CYRV", "\u{0412}");
  DefMacro!("\\CYRG", "\u{0413}");
  DefMacro!("\\CYRD", "\u{0414}");
  DefMacro!("\\CYRE", "\u{0415}");
  DefMacro!("\\CYRZH", "\u{0416}");
  DefMacro!("\\CYRZ", "\u{0417}");
  DefMacro!("\\CYRI", "\u{0418}");
  DefMacro!("\\CYRISHRT", "\u{0419}");
  DefMacro!("\\CYRK", "\u{041A}");
  DefMacro!("\\CYRL", "\u{041B}");
  DefMacro!("\\CYRM", "\u{041C}");
  DefMacro!("\\CYRN", "\u{041D}");
  DefMacro!("\\CYRO", "\u{041E}");
  DefMacro!("\\CYRP", "\u{041F}");
  DefMacro!("\\CYRR", "\u{0420}");
  DefMacro!("\\CYRS", "\u{0421}");
  DefMacro!("\\CYRT", "\u{0422}");
  DefMacro!("\\CYRU", "\u{0423}");
  DefMacro!("\\CYRF", "\u{0424}");
  DefMacro!("\\CYRH", "\u{0425}");
  DefMacro!("\\CYRC", "\u{0426}");
  DefMacro!("\\CYRCH", "\u{0427}");
  DefMacro!("\\CYRSH", "\u{0428}");
  DefMacro!("\\CYRSHCH", "\u{0429}");
  DefMacro!("\\CYRHRDSN", "\u{042A}");
  DefMacro!("\\CYRERY", "\u{042B}");
  DefMacro!("\\CYRSFTSN", "\u{042C}");
  DefMacro!("\\CYREREV", "\u{042D}");
  DefMacro!("\\CYRYU", "\u{042E}");
  DefMacro!("\\CYRYA", "\u{042F}");
  DefMacro!("\\CYRYO", "\u{0401}");

  // Stub-define the remaining entries in @uclclist so that document-level
  // expansion of the uclclist doesn't trip undefined-CS errors. Perl's
  // LaTeXML processes @uclclist via a token-level scan (MakeUppercase /
  // MakeLowercase) that doesn't invoke the CSes, so Perl tolerates these
  // being undefined; Rust's digestion path expands them eagerly. We stub
  // them as identity-empty so they carry no output but don't error.
  // These are the T2A/T2B/T2C-extended cyrillic CSes; when the document
  // doesn't actually use them in text output (the common case for non-
  // cyrillic papers that nonetheless import babel), the stub is
  // semantically invisible.
  for cs in [
    "cyrabhch",
    "CYRABHCH",
    "cyrabhchdsc",
    "CYRABHCHDSC",
    "cyrabhdze",
    "CYRABHDZE",
    "cyrabhha",
    "CYRABHHA",
    "cyrae",
    "CYRAE",
    "cyrbyus",
    "CYRBYUS",
    "cyrchldsc",
    "CYRCHLDSC",
    "cyrchrdsc",
    "CYRCHRDSC",
    "cyrchvcrs",
    "CYRCHVCRS",
    "cyrdelta",
    "CYRDELTA",
    "cyrdje",
    "CYRDJE",
    "cyrdze",
    "CYRDZE",
    "cyrdzhe",
    "CYRDZHE",
    "cyreps",
    "CYREPS",
    "cyrfita",
    "CYRFITA",
    "cyrgdsc",
    "CYRGDSC",
    "cyrgdschcrs",
    "CYRGDSCHCRS",
    "cyrghcrs",
    "CYRGHCRS",
    "cyrghk",
    "CYRGHK",
    "cyrgup",
    "CYRGUP",
    "cyrhdsc",
    "CYRHDSC",
    "cyrhhcrs",
    "CYRHHCRS",
    "cyrhhk",
    "CYRHHK",
    "cyrie",
    "CYRIE",
    "cyrii",
    "CYRII",
    "cyrishrtdsc",
    "CYRISHRTDSC",
    "cyrizh",
    "CYRIZH",
    "cyrje",
    "CYRJE",
    "cyrkbeak",
    "CYRKBEAK",
    "cyrkdsc",
    "CYRKDSC",
    "cyrkhcrs",
    "CYRKHCRS",
    "cyrkhk",
    "CYRKHK",
    "cyrkvcrs",
    "CYRKVCRS",
    "cyrldsc",
    "CYRLDSC",
    "cyrlhk",
    "CYRLHK",
    "cyrlje",
    "CYRLJE",
    "cyrmdsc",
    "CYRMDSC",
    "cyrmhk",
    "CYRMHK",
    "cyrndsc",
    "CYRNDSC",
    "cyrng",
    "CYRNG",
    "cyrnhk",
    "CYRNHK",
    "cyrnje",
    "CYRNJE",
    "cyrnlhk",
    "CYRNLHK",
    "cyrotld",
    "CYROTLD",
    "cyrphk",
    "CYRPHK",
    "cyrq",
    "CYRQ",
    "cyrrdsc",
    "CYRRDSC",
    "cyrrhk",
    "CYRRHK",
    "cyrrtick",
    "CYRRTICK",
    "cyrsacrs",
    "CYRSACRS",
    "cyrschwa",
    "CYRSCHWA",
    "cyrsdsc",
    "CYRSDSC",
    "cyrsemisftsn",
    "CYRSEMISFTSN",
    "cyrshha",
    "CYRSHHA",
    "cyrtdsc",
    "CYRTDSC",
    "cyrtetse",
    "CYRTETSE",
    "cyrtshe",
    "CYRTSHE",
    "cyrushrt",
    "CYRUSHRT",
    "cyrw",
    "CYRW",
    "cyry",
    "CYRY",
    "cyryat",
    "CYRYAT",
    "cyryhcrs",
    "CYRYHCRS",
    "cyryi",
    "CYRYI",
    "cyrzdsc",
    "CYRZDSC",
    "cyrzhdsc",
    "CYRZHDSC",
  ]
  .iter()
  {
    let cs_tok = T_CS!(&s!("\\{}", cs));
    def_macro(cs_tok, None, Tokens!(), None)?;
  }
  AddToMacro!(
    "\\@uclclist",
    r"\cyra\CYRA\cyrabhch\CYRABHCH\cyrabhchdsc\CYRABHCHDSC\cyrabhdze\CYRABHDZE\cyrabhha\CYRABHHA\cyrae\CYRAE\cyrb\CYRB\cyrbyus\CYRBYUS\cyrc\CYRC\cyrch\CYRCH\cyrchldsc\CYRCHLDSC\cyrchrdsc\CYRCHRDSC\cyrchvcrs\CYRCHVCRS\cyrd\CYRD\cyrdelta\CYRDELTA\cyrdje\CYRDJE\cyrdze\CYRDZE\cyrdzhe\CYRDZHE\cyre\CYRE\cyreps\CYREPS\cyrerev\CYREREV\cyrery\CYRERY\cyrf\CYRF\cyrfita\CYRFITA\cyrg\CYRG\cyrgdsc\CYRGDSC\cyrgdschcrs\CYRGDSCHCRS\cyrghcrs\CYRGHCRS\cyrghk\CYRGHK\cyrgup\CYRGUP\cyrh\CYRH\cyrhdsc\CYRHDSC\cyrhhcrs\CYRHHCRS\cyrhhk\CYRHHK\cyrhrdsn\CYRHRDSN\cyri\CYRI\cyrie\CYRIE\cyrii\CYRII\cyrishrt\CYRISHRT\cyrishrtdsc\CYRISHRTDSC\cyrizh\CYRIZH\cyrje\CYRJE\cyrk\CYRK\cyrkbeak\CYRKBEAK\cyrkdsc\CYRKDSC\cyrkhcrs\CYRKHCRS\cyrkhk\CYRKHK\cyrkvcrs\CYRKVCRS\cyrl\CYRL\cyrldsc\CYRLDSC\cyrlhk\CYRLHK\cyrlje\CYRLJE\cyrm\CYRM\cyrmdsc\CYRMDSC\cyrmhk\CYRMHK\cyrn\CYRN\cyrndsc\CYRNDSC\cyrng\CYRNG\cyrnhk\CYRNHK\cyrnje\CYRNJE\cyrnlhk\CYRNLHK\cyro\CYRO\cyrotld\CYROTLD\cyrp\CYRP\cyrphk\CYRPHK\cyrq\CYRQ\cyrr\CYRR\cyrrdsc\CYRRDSC\cyrrhk\CYRRHK\cyrrtick\CYRRTICK\cyrs\CYRS\cyrsacrs\CYRSACRS\cyrschwa\CYRSCHWA\cyrsdsc\CYRSDSC\cyrsemisftsn\CYRSEMISFTSN\cyrsftsn\CYRSFTSN\cyrsh\CYRSH\cyrshch\CYRSHCH\cyrshha\CYRSHHA\cyrt\CYRT\cyrtdsc\CYRTDSC\cyrtetse\CYRTETSE\cyrtshe\CYRTSHE\cyru\CYRU\cyrushrt\CYRUSHRT\cyrv\CYRV\cyrw\CYRW\cyry\CYRY\cyrya\CYRYA\cyryat\CYRYAT\cyryhcrs\CYRYHCRS\cyryi\CYRYI\cyryo\CYRYO\cyryu\CYRYU\cyrz\CYRZ\cyrzdsc\CYRZDSC\cyrzh\CYRZH\cyrzhdsc\CYRZHDSC"
  );
  Ok(())
}

LoadDefinitions!({
  //%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
  // Font Encoding
  // We ALSO need to read in or set the char=>unicode mapping.

  DeclareOption!(None, {
    let current_option = Expand!(T_CS!("\\CurrentOption")).to_string();
    push_value("font_encodings", Stored::String(arena::pin(current_option)))?;
  });

  // WELL... Actually, some "encodings" map the normal 7bit (or 8)
  // apparently ASCII input characters to a completely different font.
  // EG. OT2 maps to cyrillic.

  ProcessOptions!();
  //%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
  if let Some(font_encodings) = lookup_vecdeque("font_encodings") {
    if !font_encodings.is_empty() {
      setup_cyrillic()?;
      for encoding_stored in font_encodings.into_iter() {
        if let Stored::String(enc_sym) = encoding_stored {
          let encoding = arena::to_string(enc_sym);
          DefMacro!(T_CS!("\\encodingdefault"), None, Tokens!(Explode!(encoding)),
            scope => Some(Scope::Global));
          let encfile = encoding.to_lowercase() + "enc";
          // Load fontmap BEFORE the enc def file, so \DeclareTextSymbol
          // can look up glyph positions during def loading.
          let has_fontmap = load_font_map(&encoding).is_some();
          InputDefinitions!(&encfile, extension => Some(Cow::Borrowed("def")));
          if has_fontmap {
            MergeFont!(encoding => encoding);
          }
        } else {
          let message = s!(
            "Only strings should be stored as font encoding names, at: {:?}",
            encoding_stored
          );
          Error!("fontenc", "font_encodings", message);
        }
      }
    }
  }

  //%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
});
