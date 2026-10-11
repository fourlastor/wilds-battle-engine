-- Move 389: Sucker Punch. Identity and parameters: pokemon_data/assets/moves/389.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {priority = 1, flags = {"contact"},
    script = function(ctx)
      local chosen = ctx.target.selected_move
      if ctx.target.acted or not chosen or not chosen.damaging then return ctx:fail() end
      ctx:damage(70)
    end}

move.id = "sucker-punch"
move.name = "Sucker Punch"
move.type = Type.Dark
move.category = Category.Physical
move.pp = 5
move.accuracy = 1
move.priority = 1
move.target = Target.Selected
move.flags = {"contact"}
move.base_power = 70
return move
