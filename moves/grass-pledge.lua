-- Move 520: Grass Pledge. Identity and parameters: pokemon_data/assets/moves/520.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.
-- INCOMPLETE: The example implements side-effect combinations only. Combined 150-power attacks, forced STAB and cross-move queue promotion need engine support.

local move = {
    script = function(ctx)
      local hit = ctx:damage(80)
      if hit.hit and ctx.user.team.marks.fire_pledge then
        ctx:message("A sea of fire enveloped the opposing team!")
        ctx:start_effect(ctx.target.team, "sea_of_fire", {turns = 4, rules = {{kind = Rule.DamageEachTurn, fraction = 1/8,
          except_types = {Type.Fire}, message = "{name} is hurt by the sea of fire!"}}})
      end
      ctx:mark(ctx.user.team, "grass_pledge", 1, 1)
    end}

move.id = "grass-pledge"
move.name = "Grass Pledge"
move.type = Type.Grass
move.category = Category.Special
move.pp = 10
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {"incomplete"}
return move
