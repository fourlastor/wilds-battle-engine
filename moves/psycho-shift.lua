-- Move 375. PokeAPI identity; Gen 8 behavior (see catalog.json).
return {
  id = "psycho-shift", name = "Psycho Shift", type = Type.Psychic,
  category = Category.Status, pp = 10, accuracy = 1.0,
  priority = 0, target = Target.AnyOther,
  flags = {},
  script = function(ctx)
    if not ctx.user.status then return ctx:fail() end
    if not ctx:try_hit(ctx.target) then return end
    if ctx:apply_status(ctx.target, ctx.user.status) then ctx:cure_status(ctx.user) else ctx:fail() end
  end,
}
