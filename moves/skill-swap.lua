-- Move 285. PokeAPI identity; Gen 8 behavior (see catalog.json).
return {
  id = "skill-swap", name = "Skill Swap", type = Type.Psychic,
  category = Category.Status, pp = 10, accuracy = Accuracy.Always,
  priority = 0, target = Target.AnyOther,
  flags = {"bypass_substitute"},
  script = function(ctx)
    if ctx.user.ability_flags.failskillswap or ctx.target.ability_flags.failskillswap then ctx:fail(); return end
    if not ctx:try_hit(ctx.target) then return end
    if not ctx:swap_abilities(ctx.user, ctx.target) then ctx:fail() end
  end,
}
