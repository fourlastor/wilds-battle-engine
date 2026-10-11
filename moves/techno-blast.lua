-- Move 546: Techno Blast. Identity and parameters: pokemon_data/assets/moves/546.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {script = function(ctx)
local drives = {["shock-drive"] = Type.Electric, ["burn-drive"] = Type.Fire, ["chill-drive"] = Type.Ice, ["douse-drive"] = Type.Water}
local item = ctx.user.item_active and ctx.user.item and ctx.user.item:gsub("_", "-")
ctx:damage(120, {type = drives[item]})
end}

move.id = "techno-blast"
move.name = "Techno Blast"
move.type = Type.Normal
move.category = Category.Special
move.pp = 5
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {}
return move
