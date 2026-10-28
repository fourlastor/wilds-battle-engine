-- Move 607. PokeAPI identity; Gen 8 behavior (see catalog.json).
return {
  id = "hold-hands", name = "Hold Hands", type = Type.Normal,
  category = Category.Status, pp = 40, accuracy = Accuracy.Always,
  priority = 0, target = Target.Ally,
  flags = {"bypass_substitute"},
  script = function(ctx)
    if not ctx:try_hit(ctx.target, {ignore_protect = true}) then return end
    ctx:message("{0} and {1} held hands!", ctx.user, ctx.target)
    ctx:succeed()
  end,
}
