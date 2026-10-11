-- Move 391. PokeAPI identity; Gen 8 behavior (see catalog.json).
return {
  id = "heart-swap", name = "Heart Swap", type = Type.Psychic,
  category = Category.Status, pp = 10, accuracy = Accuracy.Always,
  priority = 0, target = Target.AnyOther,
  flags = {"bypass_substitute"},
  script = function(ctx)
    if not ctx:try_hit(ctx.target) then return end
    for _, stat in ipairs({Stat.Attack, Stat.Defense, Stat.SpAttack, Stat.SpDefense, Stat.Speed, Stat.Accuracy, Stat.Evasion}) do
      local mine, theirs = ctx.user:stage(stat), ctx.target:stage(stat)
      ctx:set_stage(ctx.user, stat, theirs)
      ctx:set_stage(ctx.target, stat, mine)
    end
  end,
}
