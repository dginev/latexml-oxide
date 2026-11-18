// item 3: the stomach's group and mode methods
DefPrimitive("\\opengrp", |stomach| { stomach.bgroup(); return; });
DefPrimitive("\\closegrp", |stomach| { stomach.egroup(); return; });
DefPrimitive("\\startpara", |stomach| { stomach.enterHorizontal(); return; });
DefPrimitive("\\endpara", |stomach| { stomach.leaveHorizontal(); return; });
// item 4: the boxes a digest hook returns
DefConstructor("\\withpost{}", "<ltx:text class=\"w\">#1</ltx:text>", #{
  afterDigest: |stomach, whatsit| Digest("[after]")
});
DefPrimitive("\\pafter", |stomach| { return; }, #{
  afterDigest: |stomach| Digest("[pafter]")
});
DefPrimitive("\\pbefore", |stomach| { return; }, #{
  beforeDigest: |stomach| Digest("[pbefore]")
});
// item 5: a properties map whose values are closures
DefConstructor("\\pm{}", "<ltx:text class=\"#cls\">#1</ltx:text>", #{
  properties: #{ cls: |stomach, x| "c-" + ToString(x) }
});
