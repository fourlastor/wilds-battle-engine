-- Move 265: Smelling Salts. Identity and parameters: pokemon_data/assets/moves/265.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {flags = {"contact"},
    script = function(ctx)
      local numb = ctx.target.status == Status.Paralyzed
      if ctx:damage(numb and 140 or 70).hit and numb then ctx:cure_status(ctx.target, Status.Paralyzed) end
    end}

move.id = "smelling-salts"
move.name = "Smelling Salts"
move.type = Type.Normal
move.category = Category.Physical
move.pp = 10
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {"contact"}
move.base_power = 70
return move
