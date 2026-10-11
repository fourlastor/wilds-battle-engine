-- Move 775: identity and parameters from pokemon_data/assets/moves/775.json.
return {
  id = "clangorous-soul", name = "Clangorous Soul",
  type = Type.Dragon, category = Category.Status,
  pp = 5, accuracy = 1, priority = 0, target = Target.User,
  flags = {"sound", "dance"},
  script = function(ctx)
    if ctx.user.max_hp == 1 or ctx.user.hp <= ctx.user.max_hp * 33 / 100 then
      return ctx:fail()
    end
    local changed = false
    for _, stat in ipairs({Stat.Attack, Stat.Defense, Stat.SpAttack, Stat.SpDefense, Stat.Speed}) do
      if ctx:change_stat(ctx.user, stat, 1) ~= 0 then changed = true end
    end
    if not changed then return ctx:fail() end
    ctx:hurt(ctx.user, math.max(1, math.floor(ctx.user.max_hp * 33 / 100)))
  end,
}
