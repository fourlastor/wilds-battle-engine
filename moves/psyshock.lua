-- Move 473: Psyshock. Identity and parameters: pokemon_data/assets/moves/473.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {
    script = function(ctx) ctx:damage(80, {defense_stat = Stat.Defense}) end}

move.id = "psyshock"
move.name = "Psyshock"
move.type = Type.Psychic
move.category = Category.Special
move.pp = 10
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {}
return move
