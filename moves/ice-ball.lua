-- Move 301: Ice Ball. Identity and parameters: pokemon_data/assets/moves/301.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {accuracy = 0.9, flags = {"contact"},
    script = function(ctx)
      if ctx.turn == 1 then ctx:force_move(5, TargetPolicy.SameTarget) end
      local power = 30 * 2 ^ (ctx.turn - 1)
      if ctx.user.marks.defense_curl then power = power * 2 end
      if not ctx:damage(power).hit then ctx:break_sequence() end
    end}

move.id = "ice-ball"
move.name = "Ice Ball"
move.type = Type.Ice
move.category = Category.Physical
move.pp = 20
move.accuracy = 0.9
move.priority = 0
move.target = Target.Selected
move.flags = {"contact", "bullet"}
move.base_power = 30
return move
