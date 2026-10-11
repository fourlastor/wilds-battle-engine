-- Move 41: Twineedle. Identity and parameters: pokemon_data/assets/moves/41.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {
  script = function(ctx)
    for hit = 1, 2 do
      if not ctx:damage(25, {never_miss = hit > 1}).hit then break end
      ctx:apply_status(ctx.target, Status.Poisoned, {chance = 0.2})
      if ctx.target.fainted then break end
    end
  end,
}

move.id = "twineedle"
move.name = "Twineedle"
move.type = Type.Bug
move.category = Category.Physical
move.pp = 20
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {}
move.base_power = 25
return move
