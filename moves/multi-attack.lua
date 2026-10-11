-- Move 718: Multi-Attack. Identity and parameters: pokemon_data/assets/moves/718.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {script = function(ctx)
local memories = {["fire-memory"] = Type.Fire, ["water-memory"] = Type.Water, ["grass-memory"] = Type.Grass, ["electric-memory"] = Type.Electric, ["ice-memory"] = Type.Ice, ["fighting-memory"] = Type.Fighting, ["poison-memory"] = Type.Poison, ["ground-memory"] = Type.Ground, ["flying-memory"] = Type.Flying, ["psychic-memory"] = Type.Psychic, ["bug-memory"] = Type.Bug, ["rock-memory"] = Type.Rock, ["ghost-memory"] = Type.Ghost, ["dragon-memory"] = Type.Dragon, ["dark-memory"] = Type.Dark, ["steel-memory"] = Type.Steel, ["fairy-memory"] = Type.Fairy}
local item = ctx.user.item_active and ctx.user.item and ctx.user.item:gsub("_", "-")
ctx:damage(120, {type = memories[item]})
end}

move.id = "multi-attack"
move.name = "Multi-Attack"
move.type = Type.Normal
move.category = Category.Physical
move.pp = 10
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {"contact"}
return move
