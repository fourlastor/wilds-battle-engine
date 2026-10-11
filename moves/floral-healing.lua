-- Move 666: Floral Healing. Identity and parameters: pokemon_data/assets/moves/666.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {target = Target.AnyOther,
    script = function(ctx)
      if ctx:reached() then ctx:heal(ctx.target, {fraction = ctx.field.effects.grassy_terrain and 2/3 or 1/2}) end
    end}

move.id = "floral-healing"
move.name = "Floral Healing"
move.type = Type.Fairy
move.category = Category.Status
move.pp = 10
move.accuracy = Accuracy.Always
move.priority = 0
move.target = Target.AnyOther
move.flags = {"reflectable", "heal"}
return move
