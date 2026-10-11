-- Move 472. PokeAPI identity; Gen 8 behavior (see catalog.json).
return {
  id = "wonder-room", name = "Wonder Room", type = Type.Psychic,
  category = Category.Status, pp = 10, accuracy = Accuracy.Always,
  priority = 0, target = Target.User,
  flags = {"pressure_all"},
  script = function(ctx)
    if not ctx:end_effect(ctx.field, "wonder_room") then
      ctx:start_effect(ctx.field, "wonder_room", {turns = 5, rules = {{kind = Rule.SwapDefenses}}})
    end
  end,
}
