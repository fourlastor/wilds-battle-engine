-- Move 239: Twister. Identity and parameters: pokemon_data/assets/moves/239.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {target = Target.AllOpponents, hits_hidden = {Hidden.Air},
    script = function(ctx)
      for _, pokemon in ipairs(ctx.targets) do
        local hit = ctx:damage(pokemon.hidden == Hidden.Air and 80 or 40, {target = pokemon})
        if hit.hit then ctx:flinch(pokemon, {chance = 0.2}) end
      end
    end}

move.id = "twister"
move.name = "Twister"
move.type = Type.Dragon
move.category = Category.Special
move.pp = 20
move.accuracy = 1
move.priority = 0
move.target = Target.AllOpponents
move.flags = {"wind"}
move.base_power = 40
return move
