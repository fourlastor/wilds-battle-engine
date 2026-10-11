-- Move 569. PokeAPI identity; Gen 8 behavior (see catalog.json).
return {
  id = "ion-deluge", name = "Ion Deluge", type = Type.Electric,
  category = Category.Status, pp = 25, accuracy = Accuracy.Always,
  priority = 1, target = Target.User,
  flags = {},
  script = function(ctx)
    if not ctx:start_effect(ctx.field, "ion_deluge", {turns = 1,
      rules = {{kind = Rule.MoveType, from = Type.Normal, to = Type.Electric}}
    }) then ctx:fail() end
  end,
}
