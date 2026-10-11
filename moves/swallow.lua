-- Move 256: Swallow. Identity and parameters: pokemon_data/assets/moves/256.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {
  script = function(ctx)
    local stored = ctx.user.marks.stockpile or 0
    if stored == 0 then return ctx:fail() end
    ctx:heal(ctx.user, {fraction = ({1/4, 1/2, 1})[stored]})
    -- Consume the stockpile even at full HP, undoing only boosts it actually granted.
    for _, boost in ipairs({{"stockpile_defense", Stat.Defense}, {"stockpile_sp_defense", Stat.SpDefense}}) do
      local key, stat = boost[1], boost[2]
      local stages = ctx.user.marks[key] or 0
      if stages > 0 then ctx:change_stat(ctx.user, stat, -stages) end
      ctx:unmark(ctx.user, key)
    end
    ctx:unmark(ctx.user, "stockpile")
  end,
}

move.id = "swallow"
move.name = "Swallow"
move.type = Type.Normal
move.category = Category.Status
move.pp = 10
move.accuracy = Accuracy.Always
move.priority = 0
move.target = Target.User
move.flags = {"snatch"}
return move
