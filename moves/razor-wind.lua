-- Move 13: Razor Wind. Identity and parameters: pokemon_data/assets/moves/13.json.
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

local move = {target = Target.AllOpponents, manual_announce = true,
    script = two_turns(nil, " whipped up a whirlwind!", function(ctx) ctx:damage(80, {high_crit = true}) end)}

move.id = "razor-wind"
move.name = "Razor Wind"
move.type = Type.Normal
move.category = Category.Special
move.pp = 10
move.accuracy = 1
move.priority = 0
move.target = Target.AllOpponents
move.flags = {"charge"}
return move
