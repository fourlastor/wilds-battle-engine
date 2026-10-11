-- Move 795: Eternabeam. Identity and parameters: pokemon_data/assets/moves/795.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {script = function(ctx)
if ctx:damage(160).hit then ctx:recharge() end
end}

move.id = "eternabeam"
move.name = "Eternabeam"
move.type = Type.Dragon
move.category = Category.Special
move.pp = 5
move.accuracy = 0.9
move.priority = 0
move.target = Target.Selected
move.flags = {}
move.base_power = 160
return move
