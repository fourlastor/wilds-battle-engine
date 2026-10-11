-- Move 377. PokeAPI identity; Gen 8 behavior (see catalog.json).
return {
  id = "heal-block", name = "Heal Block", type = Type.Psychic,
  category = Category.Status, pp = 15, accuracy = 1.0,
  priority = 0, target = Target.AllOpponents,
  flags = {"reflectable"},
  script = function(ctx)
    local applied = false
    for _, who in ipairs(ctx.foes) do
      if ctx:adjacent(ctx.user, who) and ctx:try_hit(who) then
        if ctx:start_effect(who, "heal_block", {turns = 5, rules = {
          {kind = Rule.BlockHealing},
          {kind = Rule.BlockMoves, flags = {"heal"}, message = "{name} cannot use healing moves!"}
        }}) then applied = true end
      end
    end
    if applied then ctx:succeed() else ctx:fail() end
  end,
}
