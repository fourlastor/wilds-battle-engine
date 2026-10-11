-- Move 193. PokeAPI identity; Gen 8 behavior (see catalog.json).
return {
  id = "foresight", name = "Foresight", type = Type.Normal,
  category = Category.Status, pp = 40, accuracy = Accuracy.Always,
  priority = 0, target = Target.AnyOther,
  flags = {"bypass_substitute", "reflectable"},
  script = function(ctx)
    if not ctx:try_hit(ctx.target) then return end
    if ctx.target.effects.foresight or ctx.target.effects.miracle_eye then ctx:fail(); return end
    ctx:start_effect(ctx.target, "foresight", {rules = {
      {kind = Rule.IgnoreEvasionBoost},
      {kind = Rule.NegateTypeImmunity, defending = Type.Ghost, incoming = {Type.Normal, Type.Fighting}}
    }})
  end,
}
