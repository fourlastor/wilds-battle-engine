-- Move 386. PokeAPI identity; Gen 8 behavior (see catalog.json).
return {
  id = "punishment", name = "Punishment", type = Type.Dark,
  category = Category.Physical, pp = 5, accuracy = 1.0,
  priority = 0, target = Target.AnyOther,
  flags = {"contact"},
  script = function(ctx)
    local positive = 0
    for _, stat in ipairs({Stat.Attack, Stat.Defense, Stat.SpAttack, Stat.SpDefense, Stat.Speed, Stat.Accuracy, Stat.Evasion}) do
      positive = positive + math.max(0, ctx.target:stage(stat))
    end
    ctx:damage(math.min(200, 60 + 20 * positive))
  end,
}
