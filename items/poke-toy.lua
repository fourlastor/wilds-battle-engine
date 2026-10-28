-- Ends a wild battle even while trapped.
return {
  id = "poke-toy", name = "Poké Toy", targets = "none",
  usable = function(_, ctx) return ctx.wild_battle and not ctx.no_escape end,
  script = function(ctx)
    if ctx:escape() then ctx:message("Got away safely!") end
  end,
}
