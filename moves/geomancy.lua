-- Move 601: Geomancy. Identity and parameters: pokemon_data/assets/moves/601.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.
-- INCOMPLETE: Power Herb item activation is not available; normal charge and weather shortcuts are implemented.

local function two_turns(place, message, strike)
  return function(ctx)
    if ctx.turn == 1 then
      ctx:message(ctx.user.name .. message)
      if place then ctx:hide(place) end
      ctx:force_move(2, TargetPolicy.SameTarget)
      return
    end
    ctx:announce()
    strike(ctx)
  end
end

local move = {target = Target.User, manual_announce = true,
    script = two_turns(nil, " is absorbing power!", function(ctx)
      for _, stat in ipairs({Stat.SpAttack, Stat.SpDefense, Stat.Speed}) do ctx:change_stat(ctx.user, stat, 2) end
    end)}

move.id = "geomancy"
move.name = "Geomancy"
move.type = Type.Fairy
move.category = Category.Status
move.pp = 10
move.accuracy = Accuracy.Always
move.priority = 0
move.target = Target.User
move.flags = {"incomplete"}
return move
