-- Eaten from the bag: restores 10 PP to the chosen move.
return {
  id = "leppa-berry", name = "Leppa Berry", targets = "party_move",
  usable = function(target, ctx, move) return target.pp[move] < target.max_pp[move] end,
  script = function(ctx)
    if ctx:restore_pp(ctx.target, ctx.item_move, 10) > 0 then
      ctx:message("{0}'s PP was restored.", ctx.target)
    end
  end,
}
