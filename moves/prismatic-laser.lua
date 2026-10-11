-- Move 711: Prismatic Laser. Identity and parameters: pokemon_data/assets/moves/711.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {script = function(ctx)
if ctx:damage(160).hit then ctx:recharge() end
end}

move.id = "prismatic-laser"
move.name = "Prismatic Laser"
move.type = Type.Psychic
move.category = Category.Special
move.pp = 10
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {}
move.base_power = 160
return move
