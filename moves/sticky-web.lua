-- No local source JSON; parameters and behavior from Pokemon Showdown 942cda71.
return {
  id = "sticky-web", name = "Sticky Web", type = Type.Bug,
  category = Category.Status, pp = 20, accuracy = Accuracy.Always, priority = 0,
  target = Target.Field, flags = {"reflectable", "hazard"},
  script = function(ctx)
    if not ctx:add_hazard(ctx.target.team, "sticky_web") then ctx:fail() end
  end,
}
