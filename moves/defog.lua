-- Move 432. PokeAPI identity; Gen 8 behavior (see catalog.json).
return {
  id = "defog", name = "Defog", type = Type.Flying,
  category = Category.Status, pp = 15, accuracy = Accuracy.Always,
  priority = 0, target = Target.AnyOther,
  flags = {"bypass_substitute", "reflectable"},
  script = function(ctx)
    if not ctx:try_hit(ctx.target) then return end
    local changed = false
    local ability = (ctx.user.ability or ""):gsub("[-_]", "")
    if ctx.target.substitute_hp == 0 or ability == "infiltrator" then
      changed = ctx:change_stat(ctx.target, Stat.Evasion, -1) ~= 0
    end
    for _, name in ipairs({"reflect", "light_screen", "aurora_veil", "safeguard", "mist"}) do
      if ctx:end_effect(ctx.target.team, name) then changed = true end
    end
    for _, team in ipairs({ctx.user.team, ctx.target.team}) do
      for _, name in ipairs({"spikes", "toxic_spikes", "stealth_rock", "sticky_web"}) do
        if ctx:end_effect(team, name) then changed = true end
      end
    end
    for _, name in ipairs({"electric_terrain", "grassy_terrain", "misty_terrain", "psychic_terrain"}) do
      if ctx:end_effect(ctx.field, name) then changed = true end
    end
    if not changed then ctx:fail() end
  end,
}
