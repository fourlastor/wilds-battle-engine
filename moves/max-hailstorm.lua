-- Move 763: Max Hailstorm. Identity and parameters: pokemon_data/assets/moves/763.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.
-- INCOMPLETE: Dynamax eligibility, power derived from the underlying move, and partial Protect damage are unsupported; this file uses the supplied JSON power.

local move = {script = function(ctx)
if not ctx:damage(10).hit then return end
ctx:set_weather(Weather.Hail, 5)
end}

move.id = "max-hailstorm"
move.name = "Max Hailstorm"
move.type = Type.Ice
move.category = Category.Physical
move.pp = 10
move.accuracy = Accuracy.Always
move.priority = 0
move.target = Target.Selected
move.flags = {"incomplete"}
return move
