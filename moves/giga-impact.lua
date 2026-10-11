-- Move 416: Giga Impact. Identity and parameters: pokemon_data/assets/moves/416.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {script = function(ctx)
if ctx:damage(150).hit then ctx:recharge() end
end}

move.id = "giga-impact"
move.name = "Giga Impact"
move.type = Type.Normal
move.category = Category.Physical
move.pp = 5
move.accuracy = 0.9
move.priority = 0
move.target = Target.Selected
move.flags = {"contact"}
move.base_power = 150
return move
