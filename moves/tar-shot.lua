-- Move 749. PokeAPI identity; Gen 8 behavior (see catalog.json).
return {
  id = "tar-shot", name = "Tar Shot", type = Type.Rock,
  category = Category.Status, pp = 15, accuracy = 1.0,
  priority = 0, target = Target.AnyOther,
  flags = {"reflectable"},
  script = function(ctx)
    if not ctx:try_hit(ctx.target) then return end
    local changed = ctx:change_stat(ctx.target, Stat.Speed, -1)
    local added = ctx:start_effect(ctx.target, "tar_shot", {rules = {
      {kind = Rule.TypeEffectiveness, type = Type.Fire, factor = 2}
    }})
    if changed == 0 and not added then ctx:fail() end
  end,
}
