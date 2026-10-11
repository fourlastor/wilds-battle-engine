-- Move 511. PokeAPI identity; Gen 8 behavior (see catalog.json).
return {
  id = "quash", name = "Quash", type = Type.Dark,
  category = Category.Status, pp = 15, accuracy = 1.0,
  priority = 0, target = Target.AnyOther,
  flags = {},
  script = function(ctx)
    if ctx.active_per_side == 1 then ctx:fail(); return end
    if not ctx:try_hit(ctx.target) then return end
    if not ctx:delay(ctx.target) then ctx:fail() end
  end,
}
