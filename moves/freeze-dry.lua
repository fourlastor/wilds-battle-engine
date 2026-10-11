-- Move 573: Freeze-Dry. Identity and parameters: pokemon_data/assets/moves/573.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {
    script = function(ctx)
      if ctx:damage(70, {effective = {[Type.Water] = 2}}).hit then ctx:apply_status(ctx.target, Status.Frozen, {chance = 0.1}) end
    end}

move.id = "freeze-dry"
move.name = "Freeze-Dry"
move.type = Type.Ice
move.category = Category.Special
move.pp = 20
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {}
move.base_power = 70
return move
