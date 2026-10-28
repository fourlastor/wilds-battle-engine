return {
  id = "x-attack", name = "X Attack", targets = "user",
  script = function(ctx) ctx:change_stat(ctx.target, Stat.Attack, 2) end,
}
