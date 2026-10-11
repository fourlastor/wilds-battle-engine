-- Move 222. PokeAPI identity; Gen 8 behavior (see catalog.json).
return {
  id = "magnitude", name = "Magnitude", type = Type.Ground,
  category = Category.Physical, pp = 30, accuracy = 1.0,
  priority = 0, target = Target.AllOthers,
  flags = {},
  hits_hidden = {Hidden.Underground},
  script = function(ctx)
    local roll = ctx:random_int(0, 99)
    local power, magnitude
    if roll < 5 then power, magnitude = 10, 4
    elseif roll < 15 then power, magnitude = 30, 5
    elseif roll < 35 then power, magnitude = 50, 6
    elseif roll < 65 then power, magnitude = 70, 7
    elseif roll < 85 then power, magnitude = 90, 8
    elseif roll < 95 then power, magnitude = 110, 9
    else power, magnitude = 150, 10 end
    ctx:message("Magnitude " .. magnitude .. "!")
    for _, who in ipairs(ctx.targets) do
      if ctx:adjacent(ctx.user, who) then
        ctx:damage(who.hidden == Hidden.Underground and power * 2 or power, {target = who})
      end
    end
  end,
}
