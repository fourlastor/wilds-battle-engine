-- Move 603. PokeAPI identity; Gen 8 behavior (see catalog.json).
return {
  id = "happy-hour", name = "Happy Hour", type = Type.Normal,
  category = Category.Status, pp = 30, accuracy = Accuracy.Always,
  priority = 0, target = Target.User,
  flags = {},
  script = function(ctx)
    ctx:set_payout_multiplier(2)
    ctx:message("Everyone is caught up in the happy atmosphere!")
  end,
}
