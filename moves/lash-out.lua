-- Move 808: Lash Out. Identity and parameters: pokemon_data/assets/moves/808.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.
-- INCOMPLETE: The API does not expose whether any stat was lowered earlier this turn, so the conditional power doubling is pending.

local move = {script = function(ctx)
ctx:damage(75)
end}

move.id = "lash-out"
move.name = "Lash Out"
move.type = Type.Dark
move.category = Category.Physical
move.pp = 5
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {"contact", "incomplete"}
return move
