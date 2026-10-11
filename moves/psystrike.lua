-- Move 540: Psystrike. Identity and parameters: pokemon_data/assets/moves/540.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {
    script = function(ctx) ctx:damage(100, {defense_stat = Stat.Defense}) end}

move.id = "psystrike"
move.name = "Psystrike"
move.type = Type.Psychic
move.category = Category.Special
move.pp = 10
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {}
move.base_power = 100
return move
