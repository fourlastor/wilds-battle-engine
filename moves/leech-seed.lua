-- Move 73. PokeAPI identity; Gen 8 behavior (see catalog.json).
return {
  id = "leech-seed", name = "Leech Seed", type = Type.Grass,
  category = Category.Status, pp = 10, accuracy = 0.9,
  priority = 0, target = Target.AnyOther,
  flags = {"reflectable"},
  script = function(ctx)
    if not ctx:try_hit(ctx.target) then return end
    if ctx.target:has_type(Type.Grass) then return ctx:fail() end
    if not ctx:start_effect(ctx.target, "leech_seed", {rules = {{kind = Rule.DrainEachTurn, fraction = 1 / 8, to = ctx.user}}}) then ctx:fail() end
  end,
}
