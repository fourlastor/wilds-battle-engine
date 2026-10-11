-- Move 769: Max Mindstorm. Identity and parameters: pokemon_data/assets/moves/769.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.
-- INCOMPLETE: Terrain is recorded, but grounded-only damage/status/healing rules and Psychic Terrain priority blocking require engine support.
-- INCOMPLETE: Dynamax eligibility, power derived from the underlying move, and partial Protect damage are unsupported; this file uses the supplied JSON power.

local move = {script = function(ctx)
if not ctx:damage(10).hit then return end
ctx:start_effect(ctx.field, "psychic_terrain", {turns=5, group="terrain", restart=true})
end}

move.id = "max-mindstorm"
move.name = "Max Mindstorm"
move.type = Type.Psychic
move.category = Category.Physical
move.pp = 10
move.accuracy = Accuracy.Always
move.priority = 0
move.target = Target.Selected
move.flags = {"incomplete"}
return move
