-- Move 255. PokeAPI identity; Gen 8 behavior (see catalog.json).
return {
  id = "spit-up", name = "Spit Up", type = Type.Normal,
  category = Category.Special, pp = 10, accuracy = 1.0,
  priority = 0, target = Target.AnyOther,
  flags = {},
  script = function(ctx)
    local stored = ctx.user.marks.stockpile or 0
    if stored == 0 then return ctx:fail() end
    ctx:damage(stored * 100)
    if not ctx.last_hit then return end
    for _, boost in ipairs({{"stockpile_defense", Stat.Defense}, {"stockpile_sp_defense", Stat.SpDefense}}) do
      local stages = ctx.user.marks[boost[1]] or 0
      if stages > 0 then ctx:change_stat(ctx.user, boost[2], -stages) end
      ctx:unmark(ctx.user, boost[1])
    end
    ctx:unmark(ctx.user, "stockpile")
  end,
}
