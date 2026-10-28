-- Move 364: Feint. Identity and parameters: pokemon_data/assets/moves/364.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {priority = 2,
    script = function(ctx)
      if ctx:break_protect(ctx.target) then ctx:message("{0} fell for the feint!", ctx.target) end
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
move.base_power = 30
return move
