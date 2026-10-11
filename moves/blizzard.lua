-- Move 59: Blizzard. Identity and parameters: pokemon_data/assets/moves/59.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {accuracy = 0.7, target = Target.AllOpponents,
    script = function(ctx)
      ctx:damage(110, {never_miss = ctx.weather == Weather.Hail})
      for _, pokemon in ipairs(ctx.targets) do
        if ctx:reached(pokemon) then ctx:apply_status(pokemon, Status.Frozen, {chance = 0.1}) end
      end
    end}

move.id = "blizzard"
move.name = "Blizzard"
move.type = Type.Ice
move.category = Category.Special
move.pp = 5
move.accuracy = 0.7
move.priority = 0
move.target = Target.AllOpponents
move.flags = {"wind", "sheer_force"}
move.base_power = 110
return move
