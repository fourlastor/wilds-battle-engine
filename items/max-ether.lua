return {
  id = "max-ether", name = "Max Ether", targets = "party_move",
  usable = function(target, ctx, move) return target.pp[move] < target.max_pp[move] end,
  script = function(ctx)
    if ctx:restore_pp(ctx.target, ctx.item_move, nil) > 0 then
      ctx:message("{0}'s PP was restored.", ctx.target)
    end
  end,
}
