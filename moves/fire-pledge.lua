-- Move 519: Fire Pledge. Identity and parameters: pokemon_data/assets/moves/519.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local pledge = 1
local kinds = {["fire-pledge"] = 1, ["water-pledge"] = 2, ["grass-pledge"] = 3}

local move = {
  script = function(ctx)
    local partner_kind = ctx.move_state.partner_kind or ctx.user.marks.pledge_partner
    ctx.move_state.partner_kind = partner_kind
    ctx:unmark(ctx.user, "pledge_partner")
    if not partner_kind then
      for _, partner in ipairs(ctx.pending) do
        local kind = partner.selected_move and kinds[partner.selected_move.id]
        if partner.is_ally and kind and kind ~= pledge then
          ctx:mark(partner, "pledge_partner", pledge, 1)
          ctx:hurry(partner)
          ctx:message("{0} is waiting for {1}!", ctx.user, partner)
          return
        end
      end
      ctx:damage(80)
      return
    end

    local pair = pledge + partner_kind
    local kind = pair == 3 and Type.Water or pair == 4 and Type.Fire or Type.Grass
    ctx:message("The two pledges combined!")
    if not ctx:damage(150, {type = kind, force_stab = true}).hit then return end
    if pair == 3 then
      ctx:start_effect(ctx.user.team, "rainbow", {turns = 4, rules = {{kind = Rule.EffectChance, factor = 2}}})
      ctx:message("A rainbow appeared on the team's side!")
    elseif pair == 4 then
      ctx:start_effect(ctx.target.team, "sea_of_fire", {turns = 4, rules = {{kind = Rule.DamageEachTurn,
        fraction = 1/8, except_types = {Type.Fire}, message = "{0} is hurt by the sea of fire!"}}})
      ctx:message("A sea of fire enveloped the opposing team!")
    else
      ctx:start_effect(ctx.target.team, "swamp", {turns = 4, rules = {{kind = Rule.StatMultiplier, stat = Stat.Speed, factor = 0.25}}})
      ctx:message("A swamp enveloped the opposing team!")
    end
  end,
}

move.id = "fire-pledge"
move.name = "Fire Pledge"
move.type = Type.Fire
move.category = Category.Special
move.pp = 10
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {"no_gem"}
return move
