-- Move 372: Assurance. Identity and parameters: pokemon_data/assets/moves/372.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {flags = {"contact"},
    script = function(ctx) ctx:damage(ctx.target.hurt_this_turn and 120 or 60) end}

move.id = "assurance"
move.name = "Assurance"
move.type = Type.Dark
move.category = Category.Physical
move.pp = 10
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {"contact"}
move.base_power = 60
return move
