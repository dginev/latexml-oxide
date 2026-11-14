// The binding for afterdigest_stomach_first_red.tex: a Perl-form afterDigest hook, `|stomach, whatsit|`.
DefConstructor("\\lxad{}", "<ltx:text class='#c'>#1</ltx:text>", #{
  afterDigest: |stomach, whatsit| { whatsit.setProperty("c", "ad"); }
});
