-- Move 504. PokeAPI identity; Gen 8 behavior (see catalog.json).
return {
  id = "shell-smash", name = "Shell Smash", type = Type.Normal,
  category = Category.Status, pp = 15, accuracy = Accuracy.Always,
  priority = 0, target = Target.User,
  flags = {"snatch"},
  script = function(ctx)
    local changed = 0
    for _, boost in ipairs({{Stat.Defense, -1}, {Stat.SpDefense, -1}, {Stat.Attack, 2}, {Stat.SpAttack, 2}, {Stat.Speed, 2}}) do
      changed = changed + math.abs(ctx:change_stat(ctx.user, boost[1], boost[2]))
    end
    if changed == 0 then ctx:fail() end
  end,
}
