-- Move 514: Retaliate. Identity and parameters: pokemon_data/assets/moves/514.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {flags = {"contact"},
    script = function(ctx) ctx:damage(ctx.user.team.fainted_last_turn and 140 or 70) end}

move.id = "retaliate"
move.name = "Retaliate"
move.type = Type.Normal
move.category = Category.Physical
move.pp = 5
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {"contact"}
move.base_power = 70
return move
