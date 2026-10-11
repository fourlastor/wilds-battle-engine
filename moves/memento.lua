-- Move 262. PokeAPI identity; Gen 8 behavior (see catalog.json).
return {
  id = "memento", name = "Memento", type = Type.Dark,
  category = Category.Status, pp = 10, accuracy = 1.0,
  priority = 0, target = Target.AnyOther,
  flags = {},
  script = function(ctx)
    if not ctx:try_hit(ctx.target) then return end
    ctx:change_stat(ctx.target, Stat.Attack, -2)
    ctx:change_stat(ctx.target, Stat.SpAttack, -2)
    ctx:faint_user()
  end,
}
