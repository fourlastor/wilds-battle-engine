local function end_thrash(ctx)
  if ctx.total_turns and ctx.turn == ctx.total_turns then ctx:confuse_self() end
  ctx:break_sequence()
end

return { id="thrash", name="Thrash", type=Type.Normal, category=Category.Physical, pp=10,
    accuracy=1.0, target=Target.RandomOpponent, on_interrupt=end_thrash,
    script=function(ctx)
      if not ctx.total_turns then
        ctx:force_move(ctx:random_int(2, 3), ctx.TargetPolicy.RandomOpponent)
      end
      local result = ctx:damage(120)
      if not result.hit then return end_thrash(ctx) end
      if ctx.total_turns and ctx.turn == ctx.total_turns then ctx:confuse_self() end
    end }
