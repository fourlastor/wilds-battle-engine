-- Move 581. PokeAPI identity; Gen 8 behavior (see catalog.json).
return {
  id = "misty-terrain", name = "Misty Terrain", type = Type.Fairy,
  category = Category.Status, pp = 10, accuracy = Accuracy.Always,
  priority = 0, target = Target.User,
  flags = {"pressure_all"},
  script = function(ctx)
    if not ctx:start_effect(ctx.field, "misty_terrain", {turns = 5, group = "terrain", rules = {
      {kind = Rule.IncomingBasePower, type = Type.Dragon, factor = 0.5, grounded_only = true},
      {kind = Rule.BlockGroundedStatus, confusion = true}
    }}) then ctx:fail() end
  end,
}
