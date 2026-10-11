-- Move 521: Volt Switch. Identity and parameters: pokemon_data/assets/moves/521.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {script = function(ctx)
  if ctx:damage(70).hit then ctx:switch_out() end
end}

move.id = "volt-switch"
move.name = "Volt Switch"
move.type = Type.Electric
move.category = Category.Special
move.pp = 20
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {}
return move
