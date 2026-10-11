-- Move 283. PokeAPI identity; Gen 8 behavior (see catalog.json).
return {
  id = "endeavor", name = "Endeavor", type = Type.Normal,
  category = Category.Physical, pp = 5, accuracy = 1.0,
  priority = 0, target = Target.AnyOther,
  flags = {"contact", "no_parental_bond"},
  script = function(ctx)
    if ctx.user.hp >= ctx.target.hp then return ctx:fail() end
    if not ctx:try_hit(ctx.target) then return end
    ctx:direct_damage(ctx.target, ctx.target.hp - ctx.user.hp)
  end,
}
