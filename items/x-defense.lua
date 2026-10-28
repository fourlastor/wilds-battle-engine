return {
  id = "x-defense", name = "X Defense", targets = "user",
  script = function(ctx) ctx:change_stat(ctx.target, Stat.Defense, 2) end,
}
