-- Move 615: Thousand Waves. Identity and parameters: pokemon_data/assets/moves/615.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.
-- INCOMPLETE: Trapping is recorded as a Trapped rule for the host; the engine has no switching or bench.

local move = {script = function(ctx)
ctx:damage(90)
for _, pokemon in ipairs(ctx.targets) do
  if ctx:reached(pokemon) and not pokemon:has_type(Type.Ghost) then
    ctx:start_effect(pokemon, "thousand_waves", {rules = {{kind = Rule.Trapped}}})
  end
end
end}

move.id = "thousand-waves"
move.name = "Thousand Waves"
move.type = Type.Ground
move.category = Category.Physical
move.pp = 10
move.accuracy = 1
move.priority = 0
move.target = Target.AllOpponents
move.flags = {"incomplete"}
return move
