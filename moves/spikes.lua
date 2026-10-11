-- No local source JSON; parameters and behavior from Pokemon Showdown 942cda71.
return {
  id = "spikes", name = "Spikes", type = Type.Ground,
  category = Category.Status, pp = 20, accuracy = Accuracy.Always, priority = 0,
  target = Target.Field, flags = {"reflectable", "hazard"},
  script = function(ctx)
    if not ctx:add_hazard(ctx.target.team, "spikes") then ctx:fail() end
  end,
}
