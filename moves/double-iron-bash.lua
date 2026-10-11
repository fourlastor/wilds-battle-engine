-- Move 742: Double Iron Bash. Identity and parameters: pokemon_data/assets/moves/742.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {
  script = function(ctx)
    for hit = 1, 2 do
      if not ctx:damage(60, {never_miss = hit > 1}).hit then break end
      ctx:flinch(ctx.target, {chance = 0.3})
      if ctx.target.fainted then break end
    end
  end,
}

move.id = "double-iron-bash"
move.name = "Double Iron Bash"
move.type = Type.Steel
move.category = Category.Physical
move.pp = 5
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {"contact", "punch"}
return move
