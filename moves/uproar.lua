-- Move 253: Uproar. Identity and parameters: pokemon_data/assets/moves/253.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {flags = {"sound", "no_sleep_talk"},
    script = function(ctx)
      if ctx.turn == 1 then
        local turns = 3
        ctx:force_move(turns, TargetPolicy.RandomOpponent)
        ctx:start_effect(ctx.field, "uproar", {turns = turns, restart = true, end_message = "The uproar ended.",
          rules = {{kind = Rule.BlockStatus, statuses = {Status.Asleep}}}})
        for _, pokemon in ipairs(ctx.everyone) do
          if ctx:cure_status(pokemon, Status.Asleep) then ctx:message(pokemon.name .. " woke up!") end
        end
      end
      ctx:damage(90)
    end,
    on_interrupt = function(ctx) ctx:end_effect(ctx.field, "uproar") end}

move.id = "uproar"
move.name = "Uproar"
move.type = Type.Normal
move.category = Category.Special
move.pp = 10
move.accuracy = 1
move.priority = 0
move.target = Target.RandomOpponent
move.flags = {"sound"}
return move
