-- Move 433. PokeAPI identity; Gen 8 behavior (see catalog.json).
return {
  id = "trick-room", name = "Trick Room", type = Type.Psychic,
  category = Category.Status, pp = 5, accuracy = Accuracy.Always,
  priority = -7, target = Target.User,
  flags = {"pressure_all"},
  script = function(ctx)
    if not ctx:end_effect(ctx.field, "trick_room") then
      ctx:start_effect(ctx.field, "trick_room", {turns = 5, rules = {{kind = Rule.ReverseSpeed}}})
    end
  end,
}
