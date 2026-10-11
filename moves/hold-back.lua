-- Move 610: Hold Back. Identity and parameters: pokemon_data/assets/moves/610.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {script = function(ctx)
ctx:damage(40, {min_target_hp = 1})
end}

move.id = "hold-back"
move.name = "Hold Back"
move.type = Type.Normal
move.category = Category.Physical
move.pp = 40
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {"contact"}
return move
