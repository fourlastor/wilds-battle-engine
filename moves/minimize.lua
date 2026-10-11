-- Move 107: Minimize. Identity and parameters: pokemon_data/assets/moves/107.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {target = Target.User,
    script = function(ctx)
      ctx:change_stat(ctx.user, Stat.Evasion, 2)
      ctx:mark(ctx.user, "minimized")
    end}

move.id = "minimize"
move.name = "Minimize"
move.type = Type.Normal
move.category = Category.Status
move.pp = 10
move.accuracy = Accuracy.Always
move.priority = 0
move.target = Target.User
move.flags = {"snatch"}
return move
