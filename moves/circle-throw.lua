-- Move 509: Circle Throw. Identity and parameters: pokemon_data/assets/moves/509.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {script = function(ctx)
  if ctx:damage(60).hit then ctx:force_switch(ctx.target) end
end}

move.id = "circle-throw"
move.name = "Circle Throw"
move.type = Type.Fighting
move.category = Category.Physical
move.pp = 10
move.accuracy = 0.9
move.priority = -6
move.target = Target.Selected
move.flags = {"contact"}
return move
