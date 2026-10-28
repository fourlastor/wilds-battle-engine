return {
  id = "x-sp-def", name = "X Sp. Def", targets = "user",
  script = function(ctx) ctx:change_stat(ctx.target, Stat.SpDefense, 2) end,
}
