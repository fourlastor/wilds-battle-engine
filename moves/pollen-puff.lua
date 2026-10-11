-- Move 676: Pollen Puff. Identity and parameters: pokemon_data/assets/moves/676.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {target = Target.AnyOther,
    script = function(ctx)
      if ctx.target.is_ally then
        if ctx:reached() then ctx:heal(ctx.target, {fraction = 1/2}) end
      else
        ctx:damage(90)
      end
    end}

move.id = "pollen-puff"
move.name = "Pollen Puff"
move.type = Type.Bug
move.category = Category.Special
move.pp = 15
move.accuracy = 1
move.priority = 0
move.target = Target.AnyOther
move.flags = {"bullet"}
move.base_power = 90
return move
