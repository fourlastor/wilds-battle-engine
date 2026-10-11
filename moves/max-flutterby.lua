-- Move 758: Max Flutterby. Identity and parameters: pokemon_data/assets/moves/758.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.
-- INCOMPLETE: Dynamax eligibility, power derived from the underlying move, and partial Protect damage are unsupported; this file uses the supplied JSON power.

local move = {script = function(ctx)
if not ctx:damage(10).hit then return end
for _, pokemon in ipairs(ctx.foes) do ctx:change_stat(pokemon, Stat.SpAttack, -1) end
end}

move.id = "max-flutterby"
move.name = "Max Flutterby"
move.type = Type.Bug
move.category = Category.Physical
move.pp = 10
move.accuracy = Accuracy.Always
move.priority = 0
move.target = Target.Selected
move.flags = {"incomplete"}
return move
