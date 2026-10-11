-- Move 503: Scald. Identity and parameters: pokemon_data/assets/moves/503.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {
  script = function(ctx)
    if ctx:damage(80).hit then
      ctx:cure_status(ctx.target, Status.Frozen)
      ctx:apply_status(ctx.target, Status.Burned, {chance = 0.3})
    end
  end,
}

move.id = "scald"
move.name = "Scald"
move.type = Type.Water
move.category = Category.Special
move.pp = 15
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {"sheer_force"}
move.usable_while_frozen = true
return move
