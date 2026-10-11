-- Move 221: Sacred Fire. Identity and parameters: pokemon_data/assets/moves/221.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {accuracy = 0.95, usable_while_frozen = true,
    script = function(ctx)
      if ctx:damage(100).hit then ctx:apply_status(ctx.target, Status.Burned, {chance = 0.5}) end
    end}

move.id = "sacred-fire"
move.name = "Sacred Fire"
move.type = Type.Fire
move.category = Category.Physical
move.pp = 5
move.accuracy = 0.95
move.priority = 0
move.target = Target.Selected
move.flags = {}
move.usable_while_frozen = true
return move
