-- Move 812: Flip Turn. Identity and parameters: pokemon_data/assets/moves/812.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {script = function(ctx)
  if ctx:damage(60).hit then ctx:switch_out() end
end}

move.id = "flip-turn"
move.name = "Flip Turn"
move.type = Type.Water
move.category = Category.Physical
move.pp = 20
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {"contact"}
move.base_power = 60
return move
