-- Move 474: Venoshock. Identity and parameters: pokemon_data/assets/moves/474.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {script = function(ctx)
ctx:damage((ctx.target.status == Status.Poisoned or ctx.target.status == Status.BadlyPoisoned) and 130 or 65)
end}

move.id = "venoshock"
move.name = "Venoshock"
move.type = Type.Poison
move.category = Category.Special
move.pp = 10
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {}
move.base_power = 65
return move
