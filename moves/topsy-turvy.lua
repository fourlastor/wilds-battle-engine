-- Move 576. PokeAPI identity; Gen 8 behavior (see catalog.json).
return {
  id = "topsy-turvy", name = "Topsy-Turvy", type = Type.Dark,
  category = Category.Status, pp = 20, accuracy = Accuracy.Always,
  priority = 0, target = Target.AnyOther,
  flags = {"reflectable"},
  script = function(ctx)
    if not ctx:try_hit(ctx.target) then return end
    local changed = false
    for _, stat in ipairs({Stat.Attack, Stat.Defense, Stat.SpAttack, Stat.SpDefense, Stat.Speed, Stat.Accuracy, Stat.Evasion}) do
      local stage = ctx.target:stage(stat)
      if stage ~= 0 then ctx:set_stage(ctx.target, stat, -stage); changed = true end
    end
    if not changed then ctx:fail() end
  end,
}
