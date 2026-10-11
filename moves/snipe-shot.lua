-- Move 745: Snipe Shot. Identity and parameters: pokemon_data/assets/moves/745.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.
-- INCOMPLETE: The engine has no redirection mechanic; targeting already goes directly to the selected opponent.

local move = {script = function(ctx)
ctx:damage(80, {high_crit = true})
end}

move.id = "snipe-shot"
move.name = "Snipe Shot"
move.type = Type.Water
move.category = Category.Special
move.pp = 15
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {"incomplete"}
return move
