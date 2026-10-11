-- Move 120: Self-Destruct. Identity and parameters: pokemon_data/assets/moves/120.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {script = function(ctx)
ctx:damage(200)
ctx:faint_user()
end}

move.id = "self-destruct"
move.name = "Self-Destruct"
move.type = Type.Normal
move.category = Category.Physical
move.pp = 5
move.accuracy = 1
move.priority = 0
move.target = Target.AllOthers
move.flags = {}
move.base_power = 200
return move
