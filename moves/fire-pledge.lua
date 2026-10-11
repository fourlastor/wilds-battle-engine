-- Move 519: Fire Pledge. Identity and parameters: pokemon_data/assets/moves/519.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.
-- INCOMPLETE: The example implements side-effect combinations only. Combined 150-power attacks, forced STAB and cross-move queue promotion need engine support.

local move = {
    script = function(ctx)
      local hit = ctx:damage(80)
      if hit.hit and ctx.user.team.marks.water_pledge then
        ctx:message("A rainbow appeared in the sky on your team's side!")
        ctx:start_effect(ctx.user.team, "rainbow", {turns = 4, rules = {{kind = Rule.EffectChance, factor = 2}}})
      end
      ctx:mark(ctx.user.team, "fire_pledge", 1, 1)
    end}

move.id = "fire-pledge"
move.name = "Fire Pledge"
move.type = Type.Fire
move.category = Category.Special
move.pp = 10
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {"incomplete"}
return move
