-- Move 826: Eerie Spell. Identity and parameters: pokemon_data/assets/moves/826.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.
-- INCOMPLETE: PP is read-only in Lua; reducing the last-used move by three PP requires a new engine operation.

local move = {script = function(ctx)
ctx:damage(80)
end}

move.id = "eerie-spell"
move.name = "Eerie Spell"
move.type = Type.Psychic
move.category = Category.Special
move.pp = 5
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {"sound", "incomplete"}
return move
