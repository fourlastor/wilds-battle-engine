-- Move 367. PokeAPI identity; Gen 8 behavior (see catalog.json).
return {
  id = "acupressure", name = "Acupressure", type = Type.Normal,
  category = Category.Status, pp = 30, accuracy = Accuracy.Always,
  priority = 0, target = Target.UserOrAlly,
  flags = {},
  script = function(ctx)
    if not ctx:try_hit(ctx.target, {ignore_protect = true}) then return end
    local stats = {}
    for _, stat in ipairs({Stat.Attack, Stat.Defense, Stat.SpAttack, Stat.SpDefense, Stat.Speed, Stat.Accuracy, Stat.Evasion}) do
      if ctx.target:stage(stat) < 6 then stats[#stats + 1] = stat end
    end
    if #stats == 0 then return ctx:fail() end
    if ctx:change_stat(ctx.target, stats[ctx:random_int(1, #stats)], 2) == 0 then ctx:fail() end
  end,
}
