-- Move 683. PokeAPI identity; Gen 8 behavior (see catalog.json).
return {
  id = "speed-swap", name = "Speed Swap", type = Type.Psychic,
  category = Category.Status, pp = 10, accuracy = Accuracy.Always,
  priority = 0, target = Target.AnyOther,
  flags = {"bypass_substitute"},
  script = function(ctx)
    if not ctx:try_hit(ctx.target) then return end
    local mine, theirs = ctx.user.raw_stats[Stat.Speed], ctx.target.raw_stats[Stat.Speed]
    ctx:set_raw_stat(ctx.user, Stat.Speed, theirs)
    ctx:set_raw_stat(ctx.target, Stat.Speed, mine)
  end,
}
