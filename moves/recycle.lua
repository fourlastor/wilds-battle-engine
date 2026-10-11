-- Move 278. PokeAPI identity; Gen 8 behavior (see catalog.json).
return {
  id = "recycle", name = "Recycle", type = Type.Normal,
  category = Category.Status, pp = 10, accuracy = Accuracy.Always,
  priority = 0, target = Target.User,
  flags = {"snatch"},
  script = function(ctx)
    if not ctx:recycle_item(ctx.user) then ctx:fail() end
  end,
}
