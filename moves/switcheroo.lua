-- Move 415. PokeAPI identity; Gen 8 behavior (see catalog.json).
return {
  id = "switcheroo", name = "Switcheroo", type = Type.Dark,
  category = Category.Status, pp = 10, accuracy = 1.0,
  priority = 0, target = Target.AnyOther,
  flags = {},
  script = function(ctx)
    if not ctx:try_hit(ctx.target) then return end
    if not ctx:swap_items(ctx.user, ctx.target) then ctx:fail() end
  end,
}
