-- Move 772: Max Darkness. Identity and parameters: pokemon_data/assets/moves/772.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.
-- INCOMPLETE: Dynamax eligibility, power derived from the underlying move, and partial Protect damage are unsupported; this file uses the supplied JSON power.

local move = {script = function(ctx)
if not ctx:damage(10).hit then return end
for _, pokemon in ipairs(ctx.foes) do ctx:change_stat(pokemon, Stat.SpDefense, -1) end
end}

move.id = "max-darkness"
move.name = "Max Darkness"
move.type = Type.Dark
move.category = Category.Physical
move.pp = 10
move.accuracy = Accuracy.Always
move.priority = 0
move.target = Target.Selected
move.flags = {"incomplete"}
return move
