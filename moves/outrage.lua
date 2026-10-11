-- Move 200: Outrage. Identity and parameters: pokemon_data/assets/moves/200.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local function end_thrash(ctx)
  if ctx.total_turns and ctx.turn == ctx.total_turns then ctx:confuse_self() end
  ctx:break_sequence()
end

local move = {target=Target.RandomOpponent, on_interrupt=end_thrash,
    script=function(ctx)
      if not ctx.total_turns then
        ctx:force_move(ctx:random_int(2, 3), ctx.TargetPolicy.RandomOpponent)
      end
      local result = ctx:damage(120)
      if not result.hit then return end_thrash(ctx) end
      if ctx.total_turns and ctx.turn == ctx.total_turns then ctx:confuse_self() end
    end }

move.id = "outrage"
move.name = "Outrage"
move.type = Type.Dragon
move.category = Category.Physical
move.pp = 10
move.accuracy = 1
move.priority = 0
move.target = Target.RandomOpponent
move.flags = {"contact"}
return move
