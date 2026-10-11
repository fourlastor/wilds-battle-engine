-- Move 746: Jaw Lock. Identity and parameters: pokemon_data/assets/moves/746.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.
-- INCOMPLETE: Trapping is recorded as a Trapped rule for the host; the engine has no switching or bench.

local move = {flags = {"contact"},
    script = function(ctx)
      if not ctx:damage(80).hit or ctx.target.fainted or ctx.user.trapped or ctx.target.trapped then return end
      if not ctx.target:has_type(Type.Ghost) then ctx:start_effect(ctx.target, "jaw_lock", {rules = {{kind = Rule.Trapped}}}) end
      if not ctx.user:has_type(Type.Ghost) then ctx:start_effect(ctx.user, "jaw_lock", {rules = {{kind = Rule.Trapped}}}) end
    end}

move.id = "jaw-lock"
move.name = "Jaw Lock"
move.type = Type.Dark
move.category = Category.Physical
move.pp = 10
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {"contact", "bite", "incomplete"}
return move
