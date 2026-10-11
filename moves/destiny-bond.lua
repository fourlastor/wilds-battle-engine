-- Move 194. PokeAPI identity; Gen 8 behavior (see catalog.json).
return {
  id = "destiny-bond", name = "Destiny Bond", type = Type.Ghost,
  category = Category.Status, pp = 5, accuracy = Accuracy.Always,
  priority = 0, target = Target.User,
  flags = {"bypass_substitute"},
  script = function(ctx)
    if ctx:end_effect(ctx.user, "destiny_bond") then ctx:fail(); return end
    ctx:start_effect(ctx.user, "destiny_bond", {rules={
      {kind=Rule.ExpiresOnAction, except_move="destiny-bond"}, {kind=Rule.FaintAttacker},
    }})
  end,
}
