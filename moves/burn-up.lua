-- Move 682: Burn Up. Identity and parameters: pokemon_data/assets/moves/682.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {usable_while_frozen = true,
    script = function(ctx)
      if not ctx.user:has_type(Type.Fire) then return ctx:fail() end
      ctx:damage(130)
      ctx:lose_type(ctx.user, Type.Fire)
      ctx:message(ctx.user.name .. " burned itself out!")
    end}

move.id = "burn-up"
move.name = "Burn Up"
move.type = Type.Fire
move.category = Category.Special
move.pp = 5
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {}
move.usable_while_frozen = true
return move
