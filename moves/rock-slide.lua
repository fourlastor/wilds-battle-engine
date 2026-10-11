-- Move 157: Rock Slide. Identity and parameters: pokemon_data/assets/moves/157.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {script = function(ctx)
  local connected = false
  for _, pokemon in ipairs(ctx.targets) do
    local hit = ctx:damage(75, {target = pokemon})
    if hit.hit then
      connected = true
      ctx:flinch(pokemon, {chance = 0.3})
    end
  end
end}

move.id = "rock-slide"
move.name = "Rock Slide"
move.type = Type.Rock
move.category = Category.Physical
move.pp = 10
move.accuracy = 0.9
move.priority = 0
move.target = Target.AllOpponents
move.flags = {}
return move
