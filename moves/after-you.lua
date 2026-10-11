-- Move 495. PokeAPI identity; Gen 8 behavior (see catalog.json).
return {
  id = "after-you", name = "After You", type = Type.Normal,
  category = Category.Status, pp = 15, accuracy = Accuracy.Always,
  priority = 0, target = Target.AnyOther,
  flags = {"bypass_substitute"},
  script = function(ctx)
    if ctx.active_per_side == 1 then ctx:fail(); return end
    if not ctx:try_hit(ctx.target, {ignore_protect = true}) then return end
    for _, who in ipairs(ctx.pending) do
      if who == ctx.target then ctx:hurry(who); return end
    end
    ctx:fail()
  end,
}
