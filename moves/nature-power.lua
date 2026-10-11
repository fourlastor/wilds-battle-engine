-- Move 267. PokeAPI identity; Gen 8 behavior (see catalog.json).
return {
  id = "nature-power", name = "Nature Power", type = Type.Normal,
  category = Category.Status, pp = 20, accuracy = Accuracy.Always,
  priority = 0, target = Target.AnyOther,
  flags = {"calls_move"},
  script = function(ctx)
    local environments = {
      grass="energy-ball", tallgrass="energy-ball", forest="energy-ball", forestgrass="energy-ball",
      water="hydro-pump", movingwater="hydro-pump", stillwater="hydro-pump", underwater="hydro-pump",
      puddle="mud-bomb", cave="power-gem", rock="earth-power", sand="earth-power", desert="earth-power",
      snow="ice-beam", ice="ice-beam", volcano="lava-plume", graveyard="shadow-ball",
      sky="air-slash", space="draco-meteor", ultraspace="psyshock",
    }
    local move = environments[(ctx.environment or ""):lower():gsub("[-_ ]", "")] or "tri-attack"
    local field = ctx.field.effects
    if field.electric_terrain then move="thunderbolt"
    elseif field.grassy_terrain then move="energy-ball"
    elseif field.misty_terrain then move="moonblast"
    elseif field.psychic_terrain then move="psychic" end
    ctx:call_move(move, ctx.target)
  end,
}
