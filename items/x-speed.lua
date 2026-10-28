return {
  id = "x-speed", name = "X Speed", targets = "user",
  script = function(ctx) ctx:change_stat(ctx.target, Stat.Speed, 2) end,
}
