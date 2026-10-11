-- Move 279: Revenge. Identity and parameters: pokemon_data/assets/moves/279.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {priority = -4, flags = {"contact"},
    script = function(ctx) ctx:damage(ctx.user.last_attacker == ctx.target and 120 or 60) end}

move.id = "revenge"
move.name = "Revenge"
move.type = Type.Fighting
move.category = Category.Physical
move.pp = 10
move.accuracy = 1
move.priority = -4
move.target = Target.Selected
move.flags = {"contact"}
move.base_power = 60
return move
