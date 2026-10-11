-- Move 803: Grassy Glide. Identity and parameters: pokemon_data/assets/moves/803.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local function grounded(who)
  return who.grounded or (not who:has_type(Type.Flying) and who.ability ~= "levitate"
    and who.item ~= "air-balloon" and who.item ~= "air_balloon" and who.hidden ~= Hidden.Air and not who.airborne)
end

local move = {
  on_turn_start = function(ctx)
    if ctx.field.effects.grassy_terrain and grounded(ctx.user) then
      ctx:start_effect(ctx.user, "grassy_glide_priority", {turns = 1, rules = {{kind = Rule.MovePriority, amount = 1}}})
    end
  end,
  script = function(ctx) ctx:damage(55) end,
}

move.id = "grassy-glide"
move.name = "Grassy Glide"
move.type = Type.Grass
move.category = Category.Physical
move.pp = 20
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {"contact"}
return move
