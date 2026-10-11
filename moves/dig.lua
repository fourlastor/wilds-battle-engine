-- Move 91: Dig. Identity and parameters: pokemon_data/assets/moves/91.json.
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

local move = {manual_announce = true, flags = {"contact", "no_sleep_talk"},
    script = two_turns(Hidden.Underground, " burrowed its way under the ground!", function(ctx) ctx:damage(80) end)}

move.id = "dig"
move.name = "Dig"
move.type = Type.Ground
move.category = Category.Physical
move.pp = 10
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {"charge"}
return move
