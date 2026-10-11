-- Move 615: Thousand Waves. Identity and parameters: pokemon_data/assets/moves/615.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {script = function(ctx)
ctx:damage(90)
for _, pokemon in ipairs(ctx.targets) do
  if ctx:reached(pokemon) and not pokemon.fainted and not pokemon.linked_trap and not ctx.user.fainted then
    ctx:start_effect(pokemon, "trapped", {rules = {{kind = Rule.Trapped, by = ctx.user}}})
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
move.flags = {}
return move
