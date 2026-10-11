-- Move 187. PokeAPI identity; Gen 8 behavior (see catalog.json).
return {
  id = "belly-drum", name = "Belly Drum", type = Type.Normal,
  category = Category.Status, pp = 10, accuracy = Accuracy.Always,
  priority = 0, target = Target.User,
  flags = {"snatch"},
  script = function(ctx)
    if ctx.user.max_hp <= 1 or ctx.user.hp <= ctx.user.max_hp / 2 or ctx.user:stage(Stat.Attack) >= 6 then return ctx:fail() end
    ctx:hurt(ctx.user, math.floor(ctx.user.max_hp / 2))
    ctx:change_stat(ctx.user, Stat.Attack, 12)
  end,
}
