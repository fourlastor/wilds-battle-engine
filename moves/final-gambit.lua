-- Move 515. PokeAPI identity; Gen 8 behavior (see catalog.json).
return {
  id = "final-gambit", name = "Final Gambit", type = Type.Fighting,
  category = Category.Special, pp = 5, accuracy = 1.0,
  priority = 0, target = Target.AnyOther,
  flags = {},
  script = function(ctx)
    if not ctx:try_hit(ctx.target) then return end
    if ctx:direct_damage(ctx.target, ctx.user.hp).hit then ctx:faint_user() end
  end,
}
