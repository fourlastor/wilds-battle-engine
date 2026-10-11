-- Move 757: Max Flare. Identity and parameters: pokemon_data/assets/moves/757.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.
-- INCOMPLETE: Dynamax eligibility, power derived from the underlying move, and partial Protect damage are unsupported; this file uses the supplied JSON power.

local move = {script = function(ctx)
if not ctx:damage(100).hit then return end
ctx:set_weather(Weather.Sun, 5)
end}

move.id = "max-flare"
move.name = "Max Flare"
move.type = Type.Fire
move.category = Category.Physical
move.pp = 10
move.accuracy = Accuracy.Always
move.priority = 0
move.target = Target.Selected
move.flags = {"incomplete"}
return move
