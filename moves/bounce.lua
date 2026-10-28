-- Move 340: Bounce. Identity and parameters: pokemon_data/assets/moves/340.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local function two_turns(place, message, strike)
  return function(ctx)
    if ctx.turn == 1 then
      ctx:message(message, ctx.user)
      if place then ctx:hide(place) end
      ctx:force_move(2, TargetPolicy.SameTarget)
      return
    end
    ctx:announce()
    strike(ctx)
  end
end

local move = {accuracy = 0.85, manual_announce = true, flags = {"contact", "sheer_force"},
    script = function(ctx)
      two_turns(Hidden.Air, "{0} sprang up!", function(inner)
        if inner:damage(85).hit then inner:apply_status(inner.target, Status.Paralyzed, {chance = 0.3}) end
      end)(ctx)
    end}

move.id = "bounce"
move.name = "Bounce"
move.type = Type.Flying
move.category = Category.Physical
move.pp = 5
move.accuracy = 0.85
move.priority = 0
move.target = Target.Selected
move.flags = {"charge", "contact", "gravity", "sheer_force"}
move.base_power = 85
return move
