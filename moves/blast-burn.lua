-- Move 307: Blast Burn. Identity and parameters: pokemon_data/assets/moves/307.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {script = function(ctx)
if ctx:damage(150).hit then ctx:recharge() end
end}

move.id = "blast-burn"
move.name = "Blast Burn"
move.type = Type.Fire
move.category = Category.Special
move.pp = 5
move.accuracy = 0.9
move.priority = 0
move.target = Target.Selected
move.flags = {}
return move
