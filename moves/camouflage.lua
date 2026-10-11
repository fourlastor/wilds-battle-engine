-- Move 293. PokeAPI identity; Gen 8 behavior (see catalog.json).
return {
  id = "camouflage", name = "Camouflage", type = Type.Normal,
  category = Category.Status, pp = 20, accuracy = Accuracy.Always,
  priority = 0, target = Target.User,
  flags = {"snatch"},
  script = function(ctx)
    local user = ctx.user
    local species = string.lower(user.species):match("^[^-_]+")
    if user.species_id == 493 or user.species_id == 773 or species == "arceus" or species == "silvally" then ctx:fail(); return end
    local environments = {
      grass=Type.Grass, tallgrass=Type.Grass, water=Type.Water, movingwater=Type.Water,
      stillwater=Type.Water, puddle=Type.Water, underwater=Type.Water, cave=Type.Rock,
      mountain=Type.Rock, rock=Type.Ground, sand=Type.Ground, desert=Type.Ground, road=Type.Normal,
      forest=Type.Bug, forestgrass=Type.Bug, snow=Type.Ice, ice=Type.Ice, volcano=Type.Fire,
      graveyard=Type.Ghost, sky=Type.Flying, space=Type.Dragon, ultraspace=Type.Psychic,
    }
    local kind = environments[string.lower(ctx.environment or ""):gsub("[-_ ]", "")] or Type.Normal
    local field = ctx.field.effects
    if field.electric_terrain then kind = Type.Electric
    elseif field.grassy_terrain then kind = Type.Grass
    elseif field.misty_terrain then kind = Type.Fairy
    elseif field.psychic_terrain then kind = Type.Psychic end
    if #user.types == 1 and user:has_type(kind) then ctx:fail(); return end
    ctx:set_types(user, {kind})
  end,
}
