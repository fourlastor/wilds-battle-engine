-- Move 381. PokeAPI identity; Gen 8 behavior (see catalog.json).
return {
  id = "lucky-chant", name = "Lucky Chant", type = Type.Normal,
  category = Category.Status, pp = 30, accuracy = Accuracy.Always,
  priority = 0, target = Target.User,
  flags = {"snatch"},
  script = function(ctx)
    if not ctx:start_effect(ctx.user.team, "lucky_chant", {turns = 5, rules = {{kind = Rule.BlockCritical}}}) then ctx:fail() end
  end,
}
