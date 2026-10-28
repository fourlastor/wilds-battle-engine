return {
  id = "x-accuracy", name = "X Accuracy", targets = "user",
  script = function(ctx) ctx:change_stat(ctx.target, Stat.Accuracy, 2) end,
}
