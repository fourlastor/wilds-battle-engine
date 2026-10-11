-- Move 740: Sparkly Swirl. Identity and parameters: pokemon_data/assets/moves/740.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {script = function(ctx)
if not ctx:damage(120).hit then return end
for _, pokemon in ipairs(ctx.user.team.pokemon) do
  if pokemon.status then ctx:cure_status(pokemon, pokemon.status) end
end
end}

move.id = "sparkly-swirl"
move.name = "Sparkly Swirl"
move.type = Type.Fairy
move.category = Category.Special
move.pp = 5
move.accuracy = 0.85
move.priority = 0
move.target = Target.Selected
move.flags = {}
move.base_power = 120
return move
