-- Move 818: Surging Strikes. Identity and parameters: pokemon_data/assets/moves/818.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {script = function(ctx)
ctx:multi_hit(25, 3, 3, {always_crit = true})
end}

move.id = "surging-strikes"
move.name = "Surging Strikes"
move.type = Type.Water
move.category = Category.Physical
move.pp = 5
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {"contact", "punch"}
return move
