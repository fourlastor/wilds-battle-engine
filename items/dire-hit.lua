return {
  id = "dire-hit", name = "Dire Hit", targets = "user",
  script = function(ctx)
    if ctx:start_effect(ctx.target, "focus_energy", {rules = {{kind = Rule.CriticalStage, stages = 2}}}) then
      ctx:message("{0} is getting pumped!", ctx.target)
    else
      ctx:message("But it failed!")
    end
  end,
}
