-- Move 467: Shadow Force. Identity and parameters: pokemon_data/assets/moves/467.json.
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

local move = {manual_announce = true, flags = {"contact"},
    script = two_turns(Hidden.Vanished, " vanished instantly!", function(ctx)
      ctx:break_protect(ctx.target)
      ctx:damage(120)
    end)}

move.id = "shadow-force"
move.name = "Shadow Force"
move.type = Type.Ghost
move.category = Category.Physical
move.pp = 5
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {"contact", "incomplete"}
return move
