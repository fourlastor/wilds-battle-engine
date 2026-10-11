-- Move 449: Judgment. Identity and parameters: pokemon_data/assets/moves/449.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local plates = {flame_plate = Type.Fire, splash_plate = Type.Water, meadow_plate = Type.Grass,
  zap_plate = Type.Electric, icicle_plate = Type.Ice, fist_plate = Type.Fighting, toxic_plate = Type.Poison,
  earth_plate = Type.Ground, sky_plate = Type.Flying, mind_plate = Type.Psychic, insect_plate = Type.Bug,
  stone_plate = Type.Rock, spooky_plate = Type.Ghost, draco_plate = Type.Dragon, dread_plate = Type.Dark,
  iron_plate = Type.Steel, pixie_plate = Type.Fairy}

local move = {
    script = function(ctx)
      local item = ctx.user.item and ctx.user.item:gsub("-", "_")
      ctx:damage(100, {type = plates[item]})
    end}

move.id = "judgment"
move.name = "Judgment"
move.type = Type.Normal
move.category = Category.Special
move.pp = 10
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {}
return move
