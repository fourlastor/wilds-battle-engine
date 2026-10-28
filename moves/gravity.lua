-- Move 356: Gravity. No local source JSON; identity and parameters from PokeAPI.
-- Behavior checked against the reference recorded in catalog.json.

return {
  id = "gravity",
  name = "Gravity",
  type = Type.Psychic,
  category = Category.Status,
  pp = 5,
  accuracy = Accuracy.Always,
  priority = 0,
  target = Target.Field,
  flags = {"pressure_all"},
  script = function(ctx)
    if not ctx:start_effect(ctx.field, "gravity", {
      turns = 5,
      rules = {
        {kind = Rule.Grounded},
        {kind = Rule.BlockMoves, flags = {"gravity"}, message = "{0} cannot use that move because of gravity!"},
        {kind = Rule.AccuracyMultiplier, factor = 6840 / 4096},
      },
      end_message = "Gravity returned to normal!",
    }) then return ctx:fail() end

    ctx:message("Gravity intensified!")
    for _, pokemon in ipairs(ctx.everyone) do
      -- Dig, Dive and disappearing moves must keep their hidden state and continuation.
      if pokemon.hidden == Hidden.Air and ctx:unhide(pokemon) then
        ctx:message("{0} couldn't stay airborne because of gravity!", pokemon)
      end
      ctx:end_effect(pokemon, "magnet_rise")
      ctx:end_effect(pokemon, "telekinesis")
    end
  end,
}
