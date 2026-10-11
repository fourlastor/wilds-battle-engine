-- No local source JSON; parameters and behavior from Pokemon Showdown 942cda71.
return {
  id = "substitute", name = "Substitute", type = Type.Normal,
  category = Category.Status, pp = 10, accuracy = Accuracy.Always, priority = 0,
  target = Target.User, flags = {"snatch"},
  script = function(ctx)
    if not ctx:substitute() then ctx:fail() end
  end,
}
