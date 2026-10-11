-- Move 685. PokeAPI identity; Gen 8 behavior (see catalog.json).
return {
  id = "purify", name = "Purify", type = Type.Poison,
  category = Category.Status, pp = 20, accuracy = Accuracy.Always,
  priority = 0, target = Target.AnyOther,
  flags = {"reflectable", "heal"},
  script = function(ctx)
    if not ctx:try_hit(ctx.target) then return end
    if not ctx:cure_status(ctx.target) then ctx:message("But it failed!"); return end
    ctx:heal(ctx.user, math.ceil(ctx.user.max_hp / 2))
  end,
}
