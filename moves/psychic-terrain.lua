-- Move 678. PokeAPI identity; Gen 8 behavior (see catalog.json).
return {
  id = "psychic-terrain", name = "Psychic Terrain", type = Type.Psychic,
  category = Category.Status, pp = 10, accuracy = Accuracy.Always,
  priority = 0, target = Target.User,
  flags = {"pressure_all"},
  script = function(ctx)
    if not ctx:start_effect(ctx.field, "psychic_terrain", {turns = 5, group = "terrain", rules = {
      {kind = Rule.BasePower, type = Type.Psychic, factor = 5325/4096, grounded_only = true},
      {kind = Rule.BlockPriority, grounded_only = true}
    }}) then ctx:fail() end
  end,
}
