// The binding for tags_in_text_red.tex: \lxex numbers its argument with #tags inside ltx:text.
NewCounter("lxex");
DefConstructor("\\lxex{}", "<ltx:text class=\"lxex\">#tags #1</ltx:text>", #{
  properties: |x| RefStepCounter("lxex")
});
