-- No local source JSON; parameters and behavior from Pokemon Showdown 942cda71.
return {
  id = "stealth-rock", name = "Stealth Rock", type = Type.Rock,
  category = Category.Status, pp = 20, accuracy = Accuracy.Always, priority = 0,
  target = Target.Field, flags = {"reflectable", "hazard", "pressure_all"},
  script = function(ctx)
    if not ctx:add_hazard(ctx.target.team, "stealth_rock") then ctx:fail() end
  end,
}
