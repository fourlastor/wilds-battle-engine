-- Move 57: Surf. Identity and parameters: pokemon_data/assets/moves/57.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local function each_target(ctx, power_for, options)
  for _, pokemon in ipairs(ctx.targets) do
    options = options or {}
    options.target = pokemon
    ctx:damage(power_for(pokemon), options)
  end
end

local move = {target = Target.AllOthers, hits_hidden = {Hidden.Underwater},
    script = function(ctx)
      each_target(ctx, function(pokemon) return pokemon.hidden == Hidden.Underwater and 180 or 90 end)
    end}

move.id = "surf"
move.name = "Surf"
move.type = Type.Water
move.category = Category.Special
move.pp = 15
move.accuracy = 1
move.priority = 0
move.target = Target.AllOthers
move.flags = {}
return move
