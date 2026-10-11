-- Move 753. PokeAPI identity; Gen 8 behavior (see catalog.json).
return {
  id = "octolock", name = "Octolock", type = Type.Fighting,
  category = Category.Status, pp = 15, accuracy = 1.0,
  priority = 0, target = Target.AnyOther,
  flags = {},
  script = function(ctx)
    if not ctx:try_hit(ctx.target) then return end
    if ctx.target:has_type(Type.Ghost) then ctx:fail(); return end
    if not ctx:start_effect(ctx.target, "octolock", {rules = {
      {kind = Rule.Trapped, by = ctx.user},
      {kind = Rule.StatEachTurn, stat = Stat.Defense, stages = -1, by = ctx.user},
      {kind = Rule.StatEachTurn, stat = Stat.SpDefense, stages = -1, by = ctx.user}
    }}) then ctx:fail() end
  end,
}
