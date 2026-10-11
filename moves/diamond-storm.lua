-- Move 591: Diamond Storm. Identity and parameters: pokemon_data/assets/moves/591.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {accuracy = 0.95, target = Target.AllOpponents,
    script = function(ctx)
      local hit = ctx:damage(100)
      for _ = 1, hit.hits do ctx:change_stat(ctx.user, Stat.Defense, 2, {chance = 0.5}) end
    end}

move.id = "diamond-storm"
move.name = "Diamond Storm"
move.type = Type.Rock
move.category = Category.Physical
move.pp = 5
move.accuracy = 0.95
move.priority = 0
move.target = Target.AllOpponents
move.flags = {}
return move
