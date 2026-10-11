-- Move 547: Relic Song. Identity and parameters: pokemon_data/assets/moves/547.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.
-- INCOMPLETE: Damage and sleep work; Meloetta form/stat transformation needs host or engine form support.

local move = {script = function(ctx)
  local connected = false
  for _, pokemon in ipairs(ctx.targets) do
    local hit = ctx:damage(75, {target = pokemon})
    if hit.hit then
      connected = true
      ctx:apply_status(pokemon, Status.Asleep, {chance = 0.1})
    end
  end
end}

move.id = "relic-song"
move.name = "Relic Song"
move.type = Type.Normal
move.category = Category.Special
move.pp = 10
move.accuracy = 1
move.priority = 0
move.target = Target.AllOpponents
move.flags = {"sound", "incomplete"}
return move
