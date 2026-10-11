-- Move 23: Stomp. Identity and parameters: pokemon_data/assets/moves/23.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {flags = {"contact", "flinch_secondary"},
    script = function(ctx)
      local small = ctx.target.marks.minimized ~= nil
      local hit = ctx:damage(small and 130 or 65, {never_miss = small})
      if hit.hit then ctx:flinch(ctx.target, {chance = 0.3}) end
    end}

move.id = "stomp"
move.name = "Stomp"
move.type = Type.Normal
move.category = Category.Physical
move.pp = 20
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {"contact"}
move.base_power = 65
return move
