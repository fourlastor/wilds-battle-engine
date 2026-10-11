-- Move 259. PokeAPI identity; Gen 8 behavior (see catalog.json).
return {
  id = "torment", name = "Torment", type = Type.Dark,
  category = Category.Status, pp = 15, accuracy = 1.0,
  priority = 0, target = Target.AnyOther,
  flags = {"bypass_substitute", "reflectable"},
  script = function(ctx)
    if not ctx:try_hit(ctx.target) then return end
    if not ctx:start_effect(ctx.target, "torment", {rules = {{kind = Rule.NoRepeat}}}) then ctx:fail() end
  end,
}
