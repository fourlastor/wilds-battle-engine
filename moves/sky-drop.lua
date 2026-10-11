-- Move 507: Sky Drop. Identity and parameters: pokemon_data/assets/moves/507.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.
-- INCOMPLETE: Unavailable: the Lua API cannot carry, hide or disable another Pokemon across turns.

local move = {script = function(ctx)
ctx:message("Sky Drop is unavailable: carrying another Pokemon needs engine support.")
return ctx:fail()
end}

move.id = "sky-drop"
move.name = "Sky Drop"
move.type = Type.Flying
move.category = Category.Physical
move.pp = 10
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {"gravity", "incomplete"}
return move
