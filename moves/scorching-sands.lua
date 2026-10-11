-- Move 815: Scorching Sands. Identity and parameters: pokemon_data/assets/moves/815.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {
  script = function(ctx)
    if ctx:damage(70).hit then
      ctx:cure_status(ctx.target, Status.Frozen)
      ctx:apply_status(ctx.target, Status.Burned, {chance = 0.3})
    end
  end,
}

move.id = "scorching-sands"
move.name = "Scorching Sands"
move.type = Type.Ground
move.category = Category.Special
move.pp = 10
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {"sheer_force"}
move.usable_while_frozen = true
move.base_power = 70
return move
