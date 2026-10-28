-- Heavier Pokémon are easier to catch: the catch rate itself changes, not the multiplier.
local function bonus(weight)
  if weight >= 300 then return 30 elseif weight >= 200 then return 20 elseif weight >= 100 then return 0 end
  return -20
end

return {
  id = "heavy-ball", name = "Heavy Ball", targets = "foe",
  usable = function(_, ctx) return ctx.wild_battle end,
  script = function(ctx)
    ctx:attempt_capture(ctx.target, 1, {catch_rate_bonus = bonus(ctx.target.weight)})
  end,
}
