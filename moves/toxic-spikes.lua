-- No local source JSON; parameters and behavior from Pokemon Showdown 942cda71.
return {
  id = "toxic-spikes", name = "Toxic Spikes", type = Type.Poison,
  category = Category.Status, pp = 20, accuracy = Accuracy.Always, priority = 0,
  target = Target.Field, flags = {"reflectable", "hazard", "pressure_all"},
  script = function(ctx)
    if not ctx:add_hazard(ctx.target.team, "toxic_spikes") then ctx:fail() end
  end,
}
