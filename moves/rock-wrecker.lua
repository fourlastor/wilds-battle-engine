-- Move 439: Rock Wrecker. Identity and parameters: pokemon_data/assets/moves/439.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {script = function(ctx)
if ctx:damage(150).hit then ctx:recharge() end
end}

move.id = "rock-wrecker"
move.name = "Rock Wrecker"
move.type = Type.Rock
move.category = Category.Physical
move.pp = 5
move.accuracy = 0.9
move.priority = 0
move.target = Target.Selected
move.flags = {"bullet"}
move.base_power = 150
return move
