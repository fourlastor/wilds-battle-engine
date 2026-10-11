-- Move 461. PokeAPI identity; Gen 8 behavior (see catalog.json).
return {
  id = "lunar-dance", name = "Lunar Dance", type = Type.Psychic,
  category = Category.Status, pp = 10, accuracy = Accuracy.Always,
  priority = 0, target = Target.User,
  flags = {"snatch", "dance", "heal"},
  script = function(ctx)
    if not ctx:prepare_replacement_heal(ctx.user, true) then ctx:fail(); return end
    ctx:hurt(ctx.user, ctx.user.hp)
  end,
}
