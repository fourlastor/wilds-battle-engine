-- Move 244. PokeAPI identity; Gen 8 behavior (see catalog.json).
return {
  id = "psych-up", name = "Psych Up", type = Type.Normal,
  category = Category.Status, pp = 10, accuracy = Accuracy.Always,
  priority = 0, target = Target.AnyOther,
  flags = {"bypass_substitute"},
  script = function(ctx)
    if not ctx:try_hit(ctx.target, {ignore_protect = true}) then return end
    for _, stat in ipairs({Stat.Attack, Stat.Defense, Stat.SpAttack, Stat.SpDefense, Stat.Speed, Stat.Accuracy, Stat.Evasion}) do
      ctx:set_stage(ctx.user, stat, ctx.target:stage(stat))
    end
    ctx:end_effect(ctx.user, "focus_energy")
    ctx:end_effect(ctx.user, "laser_focus")
    if ctx.target.marks.focus_energy then ctx:start_effect(ctx.user, "focus_energy", {rules = {{kind = Rule.CriticalStage, stages = 2}}}) end
    if ctx.target.marks.laser_focus then ctx:start_effect(ctx.user, "laser_focus", {turns = 2, rules = {{kind = Rule.CriticalStage, stages = 3}}}) end
  end,
}
