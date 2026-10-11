-- Move 459: Roar of Time. Identity and parameters: pokemon_data/assets/moves/459.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {script = function(ctx)
if ctx:damage(150).hit then ctx:recharge() end
end}

move.id = "roar-of-time"
move.name = "Roar of Time"
move.type = Type.Dragon
move.category = Category.Special
move.pp = 5
move.accuracy = 0.9
move.priority = 0
move.target = Target.Selected
move.flags = {}
return move
