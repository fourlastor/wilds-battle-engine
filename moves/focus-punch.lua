-- Move 264: Focus Punch. Identity and parameters: pokemon_data/assets/moves/264.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {priority = -3, flags = {"contact", "no_sleep_talk"},
    on_turn_start = function(ctx)
      ctx:start_effect(ctx.user, "focus_punch", {turns = 1, rules = {{kind = Rule.BlockFlinch}}})
      ctx:message(ctx.user.name .. " is tightening its focus!")
    end,
    script = function(ctx)
      if ctx.user.last_hit ~= nil then
        ctx:message(ctx.user.name .. " lost its focus and couldn't move!")
        return ctx:fail()
      end
      ctx:damage(150)
    end}

move.id = "focus-punch"
move.name = "Focus Punch"
move.type = Type.Fighting
move.category = Category.Physical
move.pp = 20
move.accuracy = 1
move.priority = -3
move.target = Target.Selected
move.flags = {"contact", "no_sleep_talk"}
move.base_power = 150
return move
