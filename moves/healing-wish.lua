-- Move 361. PokeAPI identity; Gen 8 behavior (see catalog.json).
return {
  id = "healing-wish", name = "Healing Wish", type = Type.Psychic,
  category = Category.Status, pp = 10, accuracy = Accuracy.Always,
  priority = 0, target = Target.User,
  flags = {"snatch", "heal"},
  script = function(ctx)
    if not ctx:prepare_replacement_heal(ctx.user, false) then ctx:fail(); return end
    ctx:hurt(ctx.user, ctx.user.hp)
  end,
}
