-- Move 213. PokeAPI identity; Gen 8 behavior (see catalog.json).
return {
  id = "attract", name = "Attract", type = Type.Normal,
  category = Category.Status, pp = 15, accuracy = 1.0,
  priority = 0, target = Target.AnyOther,
  flags = {"bypass_substitute", "reflectable"},
  script = function(ctx)
    if not ctx:try_hit(ctx.target) then return end
    local a, b = ctx.user.gender, ctx.target.gender
    local opposite = (a == Gender.Male and b == Gender.Female) or (a == Gender.Female and b == Gender.Male)
    if not opposite or ctx.target.effects.attract then ctx:fail(); return end
    if not ctx:start_effect(ctx.target, "attract", {rules = {{kind = Rule.Infatuated, by = ctx.user}}}) then ctx:fail() end
  end,
}
