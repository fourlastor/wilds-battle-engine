-- Move 604. PokeAPI identity; Gen 8 behavior (see catalog.json).
return {
  id = "electric-terrain", name = "Electric Terrain", type = Type.Electric,
  category = Category.Status, pp = 10, accuracy = Accuracy.Always,
  priority = 0, target = Target.User,
  flags = {"pressure_all"},
  script = function(ctx)
    if not ctx:start_effect(ctx.field, "electric_terrain", {turns = 5, group = "terrain", rules = {
      {kind = Rule.BasePower, type = Type.Electric, factor = 5325/4096, grounded_only = true},
      {kind = Rule.BlockGroundedStatus, statuses = {Status.Asleep}}
    }}) then ctx:fail() end
  end,
}
