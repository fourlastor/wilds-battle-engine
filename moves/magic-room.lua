-- Move 478. PokeAPI identity; Gen 8 behavior (see catalog.json).
return {
  id = "magic-room", name = "Magic Room", type = Type.Psychic,
  category = Category.Status, pp = 10, accuracy = Accuracy.Always,
  priority = 0, target = Target.Field,
  flags = {"pressure_all"},
  script = function(ctx)
if not ctx:end_effect(ctx.field, "magic_room") then
      ctx:start_effect(ctx.field, "magic_room", {turns=5, rules={{kind=Rule.BlockItems}}})
    end
  end,
}
