-- Move 570: Parabolic Charge. Identity and parameters: pokemon_data/assets/moves/570.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {target = Target.AllOthers,
    script = function(ctx) ctx:damage(65, {drain = 0.5}) end}

move.id = "parabolic-charge"
move.name = "Parabolic Charge"
move.type = Type.Electric
move.category = Category.Special
move.pp = 20
move.accuracy = 1
move.priority = 0
move.target = Target.AllOthers
move.flags = {"heal"}
return move
