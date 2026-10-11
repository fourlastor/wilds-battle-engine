-- Move 445: Captivate. Identity and parameters: pokemon_data/assets/moves/445.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {
    script = function(ctx)
      if not ctx:opposite_genders(ctx.user, ctx.target) then return ctx:fail() end
      if ctx:reached() then ctx:change_stat(ctx.target, Stat.SpAttack, -2) end
    end}

move.id = "captivate"
move.name = "Captivate"
move.type = Type.Normal
move.category = Category.Status
move.pp = 20
move.accuracy = 1
move.priority = 0
move.target = Target.AllOpponents
move.flags = {"reflectable"}
return move
