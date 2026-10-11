-- Move 668. PokeAPI identity; Gen 8 behavior (see catalog.json).
return {
  id = "strength-sap", name = "Strength Sap", type = Type.Grass,
  category = Category.Status, pp = 10, accuracy = 1.0,
  priority = 0, target = Target.AnyOther,
  flags = {"reflectable", "heal"},
  script = function(ctx)
    if not ctx:try_hit(ctx.target) then return end
    local stage = ctx.target:stage(Stat.Attack)
    if stage == -6 then ctx:fail(); return end
    local factor = stage >= 0 and (2 + stage) / 2 or 2 / (2 - stage)
    local attack = math.floor(ctx.target.raw_stats[Stat.Attack] * factor)
    local changed = ctx:change_stat(ctx.target, Stat.Attack, -1)
    local healed = ctx:heal(ctx.user, math.min(65535, attack))
    if changed == 0 and healed == 0 then ctx:fail() end
  end,
}
