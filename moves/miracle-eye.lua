-- Move 357. PokeAPI identity; Gen 8 behavior (see catalog.json).
return {
  id = "miracle-eye", name = "Miracle Eye", type = Type.Psychic,
  category = Category.Status, pp = 40, accuracy = Accuracy.Always,
  priority = 0, target = Target.AnyOther,
  flags = {"bypass_substitute", "reflectable"},
  script = function(ctx)
    if not ctx:try_hit(ctx.target) then return end
    if ctx.target.effects.foresight or ctx.target.effects.miracle_eye then ctx:fail(); return end
    ctx:start_effect(ctx.target, "miracle_eye", {rules = {
      {kind = Rule.IgnoreEvasionBoost},
      {kind = Rule.NegateTypeImmunity, defending = Type.Dark, incoming = {Type.Psychic}}
    }})
  end,
}
