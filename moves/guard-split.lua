-- Move 470. PokeAPI identity; Gen 8 behavior (see catalog.json).
return {
  id = "guard-split", name = "Guard Split", type = Type.Psychic,
  category = Category.Status, pp = 10, accuracy = Accuracy.Always,
  priority = 0, target = Target.AnyOther,
  flags = {},
  script = function(ctx)
    if not ctx:try_hit(ctx.target) then return end
    for _, stat in ipairs({Stat.Defense, Stat.SpDefense}) do
      local value = math.floor((ctx.user.raw_stats[stat] + ctx.target.raw_stats[stat]) / 2)
      ctx:set_raw_stat(ctx.user, stat, value)
      ctx:set_raw_stat(ctx.target, stat, value)
    end
  end,
}
