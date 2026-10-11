-- Move 744: Dynamax Cannon. Identity and parameters: pokemon_data/assets/moves/744.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.
-- INCOMPLETE: Dynamax state is not exposed by the Lua API, so damage cannot double against a Dynamaxed target.

local move = {script = function(ctx)
ctx:damage(100)
end}

move.id = "dynamax-cannon"
move.name = "Dynamax Cannon"
move.type = Type.Dragon
move.category = Category.Special
move.pp = 5
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {"incomplete"}
return move
