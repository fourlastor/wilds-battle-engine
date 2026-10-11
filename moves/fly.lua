-- Move 19: Fly. Identity and parameters: pokemon_data/assets/moves/19.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

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

local move = {accuracy = 0.95, manual_announce = true, flags = {"contact", "no_sleep_talk"},
    script = function(ctx)
      two_turns(Hidden.Air, " flew up high!", function(inner) inner:damage(90) end)(ctx)
    end}

move.id = "fly"
move.name = "Fly"
move.type = Type.Flying
move.category = Category.Physical
move.pp = 15
move.accuracy = 0.95
move.priority = 0
move.target = Target.Selected
move.flags = {"charge", "contact", "no_sleep_talk", "gravity"}
return move
