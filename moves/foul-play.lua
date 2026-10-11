-- Move 492: Foul Play. Identity and parameters: pokemon_data/assets/moves/492.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {flags = {"contact"},
    script = function(ctx) ctx:damage(95, {attack_from = ctx.target}) end}

move.id = "foul-play"
move.name = "Foul Play"
move.type = Type.Dark
move.category = Category.Physical
move.pp = 15
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {"contact"}
return move
