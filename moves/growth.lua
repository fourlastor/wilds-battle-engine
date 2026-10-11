-- Move 74: Growth. Identity and parameters: pokemon_data/assets/moves/74.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {target = Target.User,
    script = function(ctx)
      local stages = ctx.weather == Weather.Sun and 2 or 1
      ctx:change_stat(ctx.user, Stat.Attack, stages)
      ctx:change_stat(ctx.user, Stat.SpAttack, stages)
    end}

move.id = "growth"
move.name = "Growth"
move.type = Type.Normal
move.category = Category.Status
move.pp = 20
move.accuracy = Accuracy.Always
move.priority = 0
move.target = Target.User
move.flags = {"snatch"}
return move
