-- Move 580. PokeAPI identity; Gen 8 behavior (see catalog.json).
return {
  id = "grassy-terrain", name = "Grassy Terrain", type = Type.Grass,
  category = Category.Status, pp = 10, accuracy = Accuracy.Always,
  priority = 0, target = Target.User,
  flags = {"pressure_all"},
  script = function(ctx)
    if not ctx:start_effect(ctx.field, "grassy_terrain", {turns = 5, group = "terrain", rules = {
      {kind = Rule.BasePower, type = Type.Grass, factor = 5325/4096, grounded_only = true},
      {kind = Rule.IncomingBasePower, moves = {"earthquake", "bulldoze", "magnitude"}, factor = 0.5, grounded_only = true},
      {kind = Rule.HealEachTurn, fraction = 1/16, grounded_only = true}
    }}) then ctx:fail() end
  end,
}
