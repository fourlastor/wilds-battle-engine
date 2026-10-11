-- Move 364: Feint. Identity and parameters: pokemon_data/assets/moves/364.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {priority = 2,
    script = function(ctx)
      if ctx:break_protect(ctx.target) then ctx:message(ctx.target.name .. " fell for the feint!") end
      ctx:damage(30)
    end}

move.id = "feint"
move.name = "Feint"
move.type = Type.Normal
move.category = Category.Physical
move.pp = 10
move.accuracy = 1
move.priority = 2
move.target = Target.Selected
move.flags = {}
return move
