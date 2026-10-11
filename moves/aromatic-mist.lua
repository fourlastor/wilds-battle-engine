-- Move 597: Aromatic Mist. Identity and parameters: pokemon_data/assets/moves/597.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {target = Target.Ally,
    script = function(ctx) ctx:change_stat(ctx.target, Stat.SpDefense, 1) end}

move.id = "aromatic-mist"
move.name = "Aromatic Mist"
move.type = Type.Fairy
move.category = Category.Status
move.pp = 20
move.accuracy = Accuracy.Always
move.priority = 0
move.target = Target.Ally
move.flags = {"bypass_substitute"}
return move
