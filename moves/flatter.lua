-- Move 260. PokeAPI identity; Gen 8 behavior (see catalog.json).
return {
  id = "flatter", name = "Flatter", type = Type.Dark,
  category = Category.Status, pp = 15, accuracy = 1.0,
  priority = 0, target = Target.AnyOther,
  flags = {"reflectable"},
  script = function(ctx)
    if not ctx:try_hit(ctx.target) then return end
    local raised = ctx:change_stat(ctx.target, Stat.SpAttack, 1)
    local confused = ctx:confuse(ctx.target)
    if raised == 0 and not confused then ctx:fail() end
  end,
}
