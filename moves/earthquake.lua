-- Move 89: Earthquake. Identity and parameters: pokemon_data/assets/moves/89.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local function each_target(ctx, power_for, options)
  for _, pokemon in ipairs(ctx.targets) do
    options = options or {}
    options.target = pokemon
    ctx:damage(power_for(pokemon), options)
  end
end

local move = {target = Target.AllOthers, hits_hidden = {Hidden.Underground},
    script = function(ctx)
      each_target(ctx, function(pokemon) return pokemon.hidden == Hidden.Underground and 200 or 100 end)
    end}

move.id = "earthquake"
move.name = "Earthquake"
move.type = Type.Ground
move.category = Category.Physical
move.pp = 10
move.accuracy = 1
move.priority = 0
move.target = Target.AllOthers
move.flags = {}
move.base_power = 100
return move
