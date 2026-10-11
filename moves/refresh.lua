-- Move 287. PokeAPI identity; Gen 8 behavior (see catalog.json).
return {
  id = "refresh", name = "Refresh", type = Type.Normal,
  category = Category.Status, pp = 20, accuracy = Accuracy.Always,
  priority = 0, target = Target.User,
  flags = {"snatch"},
  script = function(ctx)
    local status = ctx.user.status
    if status ~= Status.Burned and status ~= Status.Poisoned and status ~= Status.BadlyPoisoned and status ~= Status.Paralyzed then return ctx:fail() end
    ctx:cure_status(ctx.user)
  end,
}
