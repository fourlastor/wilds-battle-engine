-- Move 672. PokeAPI identity; Gen 8 behavior (see catalog.json).
return {
  id = "toxic-thread", name = "Toxic Thread", type = Type.Poison,
  category = Category.Status, pp = 20, accuracy = 1.0,
  priority = 0, target = Target.AnyOther,
  flags = {"reflectable"},
  script = function(ctx)
    if not ctx:try_hit(ctx.target) then return end
    local slowed = ctx:change_stat(ctx.target, Stat.Speed, -1)
    local poisoned = ctx:apply_status(ctx.target, Status.Poisoned)
    if slowed == 0 and not poisoned then ctx:fail() end
  end,
}
