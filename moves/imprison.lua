-- Move 286. PokeAPI identity; Gen 8 behavior (see catalog.json).
return {
  id = "imprison", name = "Imprison", type = Type.Psychic,
  category = Category.Status, pp = 10, accuracy = Accuracy.Always,
  priority = 0, target = Target.User,
  flags = {"bypass_substitute", "snatch", "pressure_all"},
  script = function(ctx)
    if not ctx:start_effect(ctx.user, "imprison", {rules = {{kind = Rule.SealMoves}}}) then ctx:fail() end
  end,
}
