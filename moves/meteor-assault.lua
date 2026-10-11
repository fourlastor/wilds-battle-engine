-- Move 794: Meteor Assault. Identity and parameters: pokemon_data/assets/moves/794.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {script = function(ctx)
if ctx:damage(150).hit then ctx:recharge() end
end}

move.id = "meteor-assault"
move.name = "Meteor Assault"
move.type = Type.Fighting
move.category = Category.Physical
move.pp = 5
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {}
return move
