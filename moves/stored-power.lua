-- Move 500: Stored Power. Identity and parameters: pokemon_data/assets/moves/500.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {
    script = function(ctx) ctx:damage(20 + 20 * ctx.user:raised_stages()) end}

move.id = "stored-power"
move.name = "Stored Power"
move.type = Type.Psychic
move.category = Category.Special
move.pp = 10
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {}
return move
