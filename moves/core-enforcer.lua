-- Move 687: Core Enforcer. Identity and parameters: pokemon_data/assets/moves/687.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {target = Target.AllOpponents,
    script = function(ctx)
      ctx:damage(100)
      for _, pokemon in ipairs(ctx.targets) do
        if ctx:reached(pokemon) and pokemon.acted and pokemon.turns_on_field > 0 then ctx:suppress_ability(pokemon) end
      end
    end}

move.id = "core-enforcer"
move.name = "Core Enforcer"
move.type = Type.Dragon
move.category = Category.Special
move.pp = 10
move.accuracy = 1
move.priority = 0
move.target = Target.AllOpponents
move.flags = {}
move.base_power = 100
return move
