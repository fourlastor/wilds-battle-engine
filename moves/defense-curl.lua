-- Move 111: Defense Curl. Identity and parameters: pokemon_data/assets/moves/111.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {target = Target.User,
    script = function(ctx)
      ctx:change_stat(ctx.user, Stat.Defense, 1)
      ctx:mark(ctx.user, "defense_curl")
    end}

move.id = "defense-curl"
move.name = "Defense Curl"
move.type = Type.Normal
move.category = Category.Status
move.pp = 40
move.accuracy = Accuracy.Always
move.priority = 0
move.target = Target.User
move.flags = {"snatch"}
return move
