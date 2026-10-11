-- Move 362: Brine. Identity and parameters: pokemon_data/assets/moves/362.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {
    script = function(ctx) ctx:damage(ctx.target.hp * 2 <= ctx.target.max_hp and 130 or 65) end}

move.id = "brine"
move.name = "Brine"
move.type = Type.Water
move.category = Category.Special
move.pp = 10
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {}
return move
