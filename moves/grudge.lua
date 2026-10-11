-- Move 288. PokeAPI identity; Gen 8 behavior (see catalog.json).
return {
  id = "grudge", name = "Grudge", type = Type.Ghost,
  category = Category.Status, pp = 5, accuracy = Accuracy.Always,
  priority = 0, target = Target.User,
  flags = {"bypass_substitute"},
  script = function(ctx)
    ctx:start_effect(ctx.user, "grudge", {rules={{kind=Rule.ExpiresOnAction}, {kind=Rule.DepleteAttackerPp}}})
  end,
}
