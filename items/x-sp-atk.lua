return {
  id = "x-sp-atk", name = "X Sp. Atk", targets = "user",
  script = function(ctx) ctx:change_stat(ctx.target, Stat.SpAttack, 2) end,
}
