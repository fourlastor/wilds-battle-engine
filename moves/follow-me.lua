-- Move 266. PokeAPI identity; Gen 8 behavior (see catalog.json).
return {
  id = "follow-me", name = "Follow Me", type = Type.Normal,
  category = Category.Status, pp = 20, accuracy = Accuracy.Always,
  priority = 2, target = Target.User,
  flags = {},
  script = function(ctx)
    if ctx.active_per_side == 1 then ctx:fail(); return end
    ctx:start_effect(ctx.user.team, "follow_me_" .. ctx.user.index, {turns = 1, restart = true,
      rules = {{kind = Rule.Attention, to = ctx.user, priority = 1}}})
  end,
}
