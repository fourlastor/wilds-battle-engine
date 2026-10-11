-- Move 704: Shell Trap. Identity and parameters: pokemon_data/assets/moves/704.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.
-- INCOMPLETE: The trap records opposing physical hits, but reprioritizing the user immediately after the triggering attacker requires an ordering hook.

local move = {priority = -3, target = Target.AllOpponents,
    on_turn_start = function(ctx)
      ctx:message(ctx.user.name .. " set a shell trap!")
      ctx:watch_hits_until_next_action()
    end,
    on_hit = function(ctx)
      if ctx.target.is_foe and ctx.hit.category == Category.Physical then ctx:mark(ctx.user, "shell_trap", 1, 1) end
    end,
    script = function(ctx)
      if not ctx.user.marks.shell_trap then
        ctx:message(ctx.user.name .. "'s shell trap didn't work!")
        return ctx:fail()
      end
      ctx:damage(150)
    end}

move.id = "shell-trap"
move.name = "Shell Trap"
move.type = Type.Fire
move.category = Category.Special
move.pp = 5
move.accuracy = 1
move.priority = -3
move.target = Target.AllOpponents
move.flags = {"incomplete"}
return move
