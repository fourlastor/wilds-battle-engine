-- Move 6: Pay Day. Identity and parameters: pokemon_data/assets/moves/6.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {
    script = function(ctx)
      if ctx:damage(40).hit then
        ctx:add_payout(ctx.user.level * 5)
        ctx:message("Coins were scattered everywhere!")
      end
    end}

move.id = "pay-day"
move.name = "Pay Day"
move.type = Type.Normal
move.category = Category.Physical
move.pp = 20
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {}
return move
