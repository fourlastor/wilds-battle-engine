-- Move 277. PokeAPI identity; Gen 8 behavior (see catalog.json).
return {
  id = "magic-coat", name = "Magic Coat", type = Type.Psychic,
  category = Category.Status, pp = 15, accuracy = Accuracy.Always,
  priority = 4, target = Target.User,
  flags = {},
  script = function(ctx)
    ctx:start_effect(ctx.user, "magic_coat", {turns = 1, restart = true,
      rules = {{kind = Rule.ReflectMoves}}
    })
  end,
}
