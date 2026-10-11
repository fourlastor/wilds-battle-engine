-- Move 582. PokeAPI identity; Gen 8 behavior (see catalog.json).
return {
  id = "electrify", name = "Electrify", type = Type.Electric,
  category = Category.Status, pp = 20, accuracy = Accuracy.Always,
  priority = 0, target = Target.AnyOther,
  flags = {},
  script = function(ctx)
    if not ctx:try_hit(ctx.target) then return end
    local pending = false
    for _, who in ipairs(ctx.pending) do if who == ctx.target then pending = true end end
    if not pending and ctx.target.turns_on_field > 0 then ctx:fail(); return end
    local rules = {}
    for _, kind in ipairs({Type.Normal, Type.Fighting, Type.Flying, Type.Poison, Type.Ground, Type.Rock, Type.Bug, Type.Ghost, Type.Steel, Type.Fire, Type.Water, Type.Grass, Type.Electric, Type.Psychic, Type.Ice, Type.Dragon, Type.Dark, Type.Fairy}) do
      table.insert(rules, {kind = Rule.MoveType, from = kind, to = Type.Electric})
    end
    if not ctx:start_effect(ctx.target, "electrify", {turns = 1, rules = rules}) then ctx:fail() end
  end,
}
