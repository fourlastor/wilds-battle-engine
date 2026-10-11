-- Move 379. PokeAPI identity; Gen 8 behavior (see catalog.json).
return {
  id = "power-trick", name = "Power Trick", type = Type.Psychic,
  category = Category.Status, pp = 10, accuracy = Accuracy.Always,
  priority = 0, target = Target.User,
  flags = {"snatch"},
  script = function(ctx)
    local attack, defense = ctx.user.raw_stats[Stat.Attack], ctx.user.raw_stats[Stat.Defense]
    ctx:set_raw_stat(ctx.user, Stat.Attack, defense)
    ctx:set_raw_stat(ctx.user, Stat.Defense, attack)
    if ctx.user.marks.power_trick then ctx:unmark(ctx.user, "power_trick")
    else ctx:mark(ctx.user, "power_trick") end
  end,
}
