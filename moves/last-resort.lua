-- Move 387: Last Resort. Identity and parameters: pokemon_data/assets/moves/387.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {flags = {"contact"},
    script = function(ctx)
      if not ctx.user:used_all_other_moves(ctx.move.id) then return ctx:fail() end
      ctx:damage(140)
    end}

move.id = "last-resort"
move.name = "Last Resort"
move.type = Type.Normal
move.category = Category.Physical
move.pp = 5
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {"contact"}
return move
