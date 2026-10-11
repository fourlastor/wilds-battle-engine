-- Move 50. PokeAPI identity; Gen 8 behavior (see catalog.json).
return {
  id = "disable", name = "Disable", type = Type.Normal,
  category = Category.Status, pp = 20, accuracy = 1.0,
  priority = 0, target = Target.AnyOther,
  flags = {"bypass_substitute", "reflectable"},
  script = function(ctx)
    if not ctx:try_hit(ctx.target) then return end
    local move = ctx.target.last_move
    if not move or move == "struggle" or ctx.target.pp[move] == 0 or ctx.target.effects.disable then ctx:fail(); return end
    local turns = 5
    for _, who in ipairs(ctx.pending) do if who == ctx.target then turns = 4; break end end
    if not ctx:start_effect(ctx.target, "disable", {turns = turns, rules = {
      {kind = Rule.BlockMove, id = move, message = "{name}'s move is disabled!"}
    }}) then ctx:fail() end
  end,
}
