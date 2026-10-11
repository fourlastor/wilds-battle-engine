-- Move 664: Sparkling Aria. Identity and parameters: pokemon_data/assets/moves/664.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {target = Target.AllOthers, flags = {"sound", "sheer_force"},
    script = function(ctx)
      ctx:damage(90)
      for _, pokemon in ipairs(ctx.targets) do
        if ctx:reached(pokemon) then ctx:cure_status(pokemon, Status.Burned) end
      end
    end}

move.id = "sparkling-aria"
move.name = "Sparkling Aria"
move.type = Type.Water
move.category = Category.Special
move.pp = 10
move.accuracy = 1
move.priority = 0
move.target = Target.AllOthers
move.flags = {"sound", "sheer_force"}
move.base_power = 90
return move
