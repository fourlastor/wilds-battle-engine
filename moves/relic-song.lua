-- Move 547: Relic Song. Identity and parameters: pokemon_data/assets/moves/547.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {script = function(ctx)
  local connected = false
  for _, pokemon in ipairs(ctx.targets) do
    local hit = ctx:damage(75, {target = pokemon})
    if hit.hit then
      connected = true
      ctx:apply_status(pokemon, Status.Asleep, {chance = 0.1})
    end
  end
  if connected and ctx.user.hp > 0 and (ctx.user.species_id == 648 or ctx.user.species:match("^meloetta")) then
    if (ctx.user.form or ctx.user.species) == "meloetta-pirouette" then
      if not ctx:restore_form(ctx.user) then ctx:change_form(ctx.user, "meloetta-aria") end
    else
      ctx:change_form(ctx.user, "meloetta-pirouette")
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
move.flags = {"sound"}
return move
