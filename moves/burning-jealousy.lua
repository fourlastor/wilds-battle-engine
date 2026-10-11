-- Move 807: Burning Jealousy. Identity and parameters: pokemon_data/assets/moves/807.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.
-- INCOMPLETE: The API does not expose whether targets raised any stat earlier this turn, so conditional burning is pending.

local move = {script = function(ctx)
ctx:damage(70)
end}

move.id = "burning-jealousy"
move.name = "Burning Jealousy"
move.type = Type.Fire
move.category = Category.Special
move.pp = 5
move.accuracy = 1
move.priority = 0
move.target = Target.AllOpponents
move.flags = {"incomplete"}
return move
