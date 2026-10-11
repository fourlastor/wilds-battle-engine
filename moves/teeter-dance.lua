-- Move 298: Teeter Dance. Identity and parameters: pokemon_data/assets/moves/298.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {target = Target.AllOthers,
    script = function(ctx)
      for _, pokemon in ipairs(ctx.targets) do
        if ctx:reached(pokemon) then ctx:confuse(pokemon) end
      end
    end}

move.id = "teeter-dance"
move.name = "Teeter Dance"
move.type = Type.Normal
move.category = Category.Status
move.pp = 20
move.accuracy = 1
move.priority = 0
move.target = Target.AllOthers
move.flags = {"dance"}
return move
