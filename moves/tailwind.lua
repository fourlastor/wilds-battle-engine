-- Move 366. PokeAPI identity; Gen 8 behavior (see catalog.json).
return {
  id = "tailwind", name = "Tailwind", type = Type.Flying,
  category = Category.Status, pp = 15, accuracy = Accuracy.Always,
  priority = 0, target = Target.User,
  flags = {"snatch", "wind"},
  script = function(ctx)
    if not ctx:start_effect(ctx.user.team, "tailwind", {turns = 4, rules = {{kind = Rule.StatMultiplier, stat = Stat.Speed, factor = 2}}}) then ctx:fail() end
  end,
}
