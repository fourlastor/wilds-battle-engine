-- Moves from the "Saved moves" sheet that needed more than the first script API, written with the
-- current one. They are examples and a test (sheet_moves.rs plays every one of them), not part of
-- the game's move list. Marks and effects are named by the moves themselves: "minimized",
-- "reflect", "gravity" and so on only mean something because the moves here agree on the names.

local function move(id, name, kind, category, fields)
  fields.id, fields.name, fields.type, fields.category = id, name, kind, category
  fields.pp = fields.pp or 10
  return fields
end

-- A move that gets ready on its first turn, optionally hiding somewhere, and strikes on its second.
-- Moves made with it need manual_announce = true.
local function two_turns(place, message, strike)
  return function(ctx)
    if ctx.turn == 1 then
      ctx:message(ctx.user.name .. message)
      if place then ctx:hide(place) end
      ctx:force_move(2, TargetPolicy.SameTarget)
      return
    end
    ctx:announce()
    strike(ctx)
  end
end

local function gravity(ctx)
  return ctx.field.effects.gravity ~= nil
end

-- One hit per Pokémon the move is aimed at, each with its own power.
local function each_target(ctx, power_for, options)
  for _, pokemon in ipairs(ctx.targets) do
    options = options or {}
    options.target = pokemon
    ctx:damage(power_for(pokemon), options)
  end
end

local screen = function(name, category, wore_off)
  return function(ctx)
    local started = ctx:start_effect(ctx.user.team, name, {turns = 5, end_message = wore_off,
      rules = {{kind = Rule.DamageTaken, category = category, factor = 0.5, not_on_crit = true}}})
    if not started then return ctx:fail() end
  end
end

local terrain = function(name, rules)
  return function(ctx)
    local started = ctx:start_effect(ctx.field, name, {turns = 5, group = "terrain", rules = rules,
      end_message = "The terrain returned to normal."})
    if not started then return ctx:fail() end
  end
end

local plates = {flame_plate = Type.Fire, splash_plate = Type.Water, meadow_plate = Type.Grass,
  zap_plate = Type.Electric, icicle_plate = Type.Ice, fist_plate = Type.Fighting, toxic_plate = Type.Poison,
  earth_plate = Type.Ground, sky_plate = Type.Flying, mind_plate = Type.Psychic, insect_plate = Type.Bug,
  stone_plate = Type.Rock, spooky_plate = Type.Ghost, draco_plate = Type.Dragon, dread_plate = Type.Dark,
  iron_plate = Type.Steel, pixie_plate = Type.Fairy}

return {
  -- Helpers for the test battles.
  move("wait", "Wait", Type.Normal, Category.Status, {target = Target.User, script = function(_) end}),
  move("scratch", "Scratch", Type.Normal, Category.Physical, {flags = {"contact"}, effects = {{kind = Effect.Damage, power = 40}}}),
  move("ember", "Ember", Type.Fire, Category.Special, {effects = {{kind = Effect.Damage, power = 40}}}),
  move("rain_dance", "Rain Dance", Type.Water, Category.Status, {target = Target.Field,
    effects = {{kind = Effect.Weather, weather = Weather.Rain, turns = 5}}}),
  move("hail", "Hail", Type.Ice, Category.Status, {target = Target.Field,
    effects = {{kind = Effect.Weather, weather = Weather.Hail, turns = 5}}}),
  move("guard", "Guard", Type.Normal, Category.Status, {target = Target.User, priority = 4,
    script = function(ctx) ctx:protect() end}),

  -- Moves marked "Major issues" that the built-in list does not have yet.
  move("skull_bash", "Skull Bash", Type.Normal, Category.Physical, {manual_announce = true,
    script = function(ctx)
      if ctx.turn == 1 then
        ctx:message(ctx.user.name .. " tucked in its head!")
        ctx:change_stat(ctx.user, Stat.Defense, 1)
        ctx:force_move(2, TargetPolicy.SameTarget)
        return
      end
      ctx:announce()
      ctx:damage(130)
    end}),
  move("high_jump_kick", "High Jump Kick", Type.Fighting, Category.Physical, {accuracy = 0.9, flags = {"contact"},
    script = function(ctx)
      if gravity(ctx) then return ctx:fail() end
      local hit = ctx:damage(130)
      if not hit.hit then
        ctx:message(ctx.user.name .. " kept going and crashed!")
        ctx:hurt(ctx.user, {fraction = 1/2})
      end
    end}),
  move("struggle_like", "Flail About", Type.Normal, Category.Physical, {
    script = function(ctx)
      ctx:damage(50, {typeless = true, never_miss = true})
      ctx:recoil_max_hp(1/4)
    end}),
  move("thief", "Thief", Type.Dark, Category.Physical, {flags = {"contact", "no_metronome", "no_assist"},
    script = function(ctx)
      local hit = ctx:damage(60)
      if hit.hit then
        local item = ctx:steal_item(ctx.target, ctx.user)
        if item then ctx:message(ctx.user.name .. " stole " .. ctx.target.name .. "'s " .. item .. "!") end
      end
    end}),
  move("fury_cutter", "Fury Cutter", Type.Bug, Category.Physical, {accuracy = 0.95, flags = {"contact"},
    script = function(ctx)
      local uses = ctx.user.marks.fury_cutter or 0
      local hit = ctx:damage(40 * 2 ^ math.min(uses, 4))
      if hit.hit then ctx:mark(ctx.user, "fury_cutter", uses + 1) else ctx:unmark(ctx.user, "fury_cutter") end
    end}),
  -- The engine has no switching, so Pursuit is its plain hit.
  move("pursuit", "Pursuit", Type.Dark, Category.Physical, {flags = {"contact"}, effects = {{kind = Effect.Damage, power = 40}}}),
  move("rapid_spin", "Rapid Spin", Type.Normal, Category.Physical, {flags = {"contact"},
    script = function(ctx)
      local hit = ctx:damage(50)
      if not hit.hit then return end
      ctx:free(ctx.user)
      ctx:end_effect(ctx.user, "leech_seed")
      for _, hazard in ipairs({"spikes", "toxic_spikes", "stealth_rock"}) do
        ctx:end_effect(ctx.user.team, hazard)
      end
    end}),
  move("hidden_power", "Hidden Power", Type.Normal, Category.Special, {
    script = function(ctx)
      local ivs = ctx.user.ivs or {hp = 31, attack = 31, defense = 31, sp_attack = 31, sp_defense = 31, speed = 31}
      local kind, strength = 0, 0
      for bit, iv in ipairs({ivs.hp, ivs.attack, ivs.defense, ivs.speed, ivs.sp_attack, ivs.sp_defense}) do
        kind = kind + (iv % 2) * 2 ^ (bit - 1)
        strength = strength + (iv // 2 % 2) * 2 ^ (bit - 1)
      end
      ctx:damage(strength * 40 // 63 + 30, {type = ctx:type_number(kind * 15 // 63)})
    end}),
  move("stockpile", "Stockpile", Type.Normal, Category.Status, {target = Target.User,
    script = function(ctx)
      local stored = ctx.user.marks.stockpile or 0
      if stored >= 3 then return ctx:fail() end
      ctx:mark(ctx.user, "stockpile", stored + 1)
      ctx:message(ctx.user.name .. " stockpiled " .. (stored + 1) .. "!")
      ctx:change_stat(ctx.user, Stat.Defense, 1)
      ctx:change_stat(ctx.user, Stat.SpDefense, 1)
    end}),
  move("swallow", "Swallow", Type.Normal, Category.Status, {target = Target.User,
    script = function(ctx)
      local stored = ctx.user.marks.stockpile
      if not stored then return ctx:fail() end
      ctx:heal(ctx.user, {fraction = ({1/4, 1/2, 1})[stored]})
      ctx:unmark(ctx.user, "stockpile")
      ctx:change_stat(ctx.user, Stat.Defense, -stored)
      ctx:change_stat(ctx.user, Stat.SpDefense, -stored)
    end}),
  move("focus_punch", "Focus Punch", Type.Fighting, Category.Physical, {priority = -3, flags = {"contact", "no_sleep_talk"},
    on_turn_start = function(ctx) ctx:message(ctx.user.name .. " is tightening its focus!") end,
    script = function(ctx)
      if ctx.user.hurt_this_turn then
        ctx:message(ctx.user.name .. " lost its focus and couldn't move!")
        return ctx:fail()
      end
      ctx:damage(150)
    end}),
  move("secret_power", "Secret Power", Type.Normal, Category.Physical, {
    script = function(ctx)
      local hit = ctx:damage(70)
      if not hit.hit or not ctx:effect_chance(0.3) then return end
      local place = ctx.environment
      if place == "cave" or place == "mountain" then ctx:flinch(ctx.target)
      elseif place == "water" then ctx:change_stat(ctx.target, Stat.Attack, -1)
      elseif place == "desert" or place == "road" then ctx:change_stat(ctx.target, Stat.Accuracy, -1)
      elseif place == "grass" then ctx:apply_status(ctx.target, Status.Asleep)
      elseif place == "snow" then ctx:apply_status(ctx.target, Status.Frozen)
      else ctx:apply_status(ctx.target, Status.Paralyzed) end
    end}),

  -- Marks that one move leaves for another.
  move("minimize", "Minimize", Type.Normal, Category.Status, {target = Target.User,
    script = function(ctx)
      ctx:change_stat(ctx.user, Stat.Evasion, 2)
      ctx:mark(ctx.user, "minimized")
    end}),
  move("stomp", "Stomp", Type.Normal, Category.Physical, {flags = {"contact"},
    script = function(ctx)
      local small = ctx.target.marks.minimized ~= nil
      local hit = ctx:damage(small and 130 or 65, {never_miss = small})
      if hit.hit then ctx:flinch(ctx.target, {chance = 0.3}) end
    end}),
  move("defense_curl", "Defense Curl", Type.Normal, Category.Status, {target = Target.User,
    script = function(ctx)
      ctx:change_stat(ctx.user, Stat.Defense, 1)
      ctx:mark(ctx.user, "defense_curl")
    end}),
  move("ice_ball", "Ice Ball", Type.Ice, Category.Physical, {accuracy = 0.9, flags = {"contact"},
    script = function(ctx)
      if ctx.turn == 1 then ctx:force_move(5, TargetPolicy.SameTarget) end
      local power = 30 * 2 ^ (ctx.turn - 1)
      if ctx.user.marks.defense_curl then power = power * 2 end
      if not ctx:damage(power).hit then ctx:break_sequence() end
    end}),
  move("round", "Round", Type.Normal, Category.Special, {flags = {"sound"},
    script = function(ctx)
      ctx:damage(ctx.field.marks.round and 120 or 60)
      ctx:mark(ctx.field, "round", 1, 1)
      ctx:hurry()
    end}),
  move("echoed_voice", "Echoed Voice", Type.Normal, Category.Special, {flags = {"sound"},
    script = function(ctx)
      local echoes = ctx.user.team.marks.echoed_voice or 0
      ctx:damage(math.min(40 * (echoes + 1), 200))
      ctx:mark(ctx.user.team, "echoed_voice", echoes + 1, 2)
    end}),
  move("fusion_flare", "Fusion Flare", Type.Fire, Category.Special, {usable_while_frozen = true,
    script = function(ctx)
      ctx:damage(ctx.field.marks.fusion_bolt and 200 or 100)
      ctx:mark(ctx.field, "fusion_flare", 1, 1)
    end}),
  move("fusion_bolt", "Fusion Bolt", Type.Electric, Category.Physical, {
    script = function(ctx)
      ctx:damage(ctx.field.marks.fusion_flare and 200 or 100)
      ctx:mark(ctx.field, "fusion_bolt", 1, 1)
    end}),
  move("grass_pledge", "Grass Pledge", Type.Grass, Category.Special, {
    script = function(ctx)
      local hit = ctx:damage(80)
      if hit.hit and ctx.user.team.marks.fire_pledge then
        ctx:message("A sea of fire enveloped the opposing team!")
        ctx:start_effect(ctx.target.team, "sea_of_fire", {turns = 4, rules = {{kind = Rule.DamageEachTurn, fraction = 1/8,
          except_types = {Type.Fire}, message = "{name} is hurt by the sea of fire!"}}})
      end
      ctx:mark(ctx.user.team, "grass_pledge", 1, 1)
    end}),
  move("fire_pledge", "Fire Pledge", Type.Fire, Category.Special, {
    script = function(ctx)
      local hit = ctx:damage(80)
      if hit.hit and ctx.user.team.marks.water_pledge then
        ctx:message("A rainbow appeared in the sky on your team's side!")
        ctx:start_effect(ctx.user.team, "rainbow", {turns = 4, rules = {{kind = Rule.EffectChance, factor = 2}}})
      end
      ctx:mark(ctx.user.team, "fire_pledge", 1, 1)
    end}),
  move("water_pledge", "Water Pledge", Type.Water, Category.Special, {
    script = function(ctx)
      local hit = ctx:damage(80)
      if hit.hit and ctx.user.team.marks.grass_pledge then
        ctx:message("A swamp enveloped the opposing team!")
        ctx:start_effect(ctx.target.team, "swamp", {turns = 4, rules = {{kind = Rule.StatMultiplier, stat = Stat.Speed, factor = 0.5}}})
      end
      ctx:mark(ctx.user.team, "water_pledge", 1, 1)
    end}),

  -- Hiding places, and the moves that reach them.
  move("fly", "Fly", Type.Flying, Category.Physical, {accuracy = 0.95, manual_announce = true, flags = {"contact", "no_sleep_talk"},
    script = function(ctx)
      if gravity(ctx) then return ctx:fail() end
      two_turns(Hidden.Air, " flew up high!", function(inner) inner:damage(90) end)(ctx)
    end}),
  move("dig", "Dig", Type.Ground, Category.Physical, {manual_announce = true, flags = {"contact", "no_sleep_talk"},
    script = two_turns(Hidden.Underground, " burrowed its way under the ground!", function(ctx) ctx:damage(80) end)}),
  move("bounce", "Bounce", Type.Flying, Category.Physical, {accuracy = 0.85, manual_announce = true, flags = {"contact"},
    script = function(ctx)
      if gravity(ctx) then return ctx:fail() end
      two_turns(Hidden.Air, " sprang up!", function(inner)
        if inner:damage(85).hit then inner:apply_status(inner.target, Status.Paralyzed, {chance = 0.3}) end
      end)(ctx)
    end}),
  move("shadow_force", "Shadow Force", Type.Ghost, Category.Physical, {manual_announce = true, flags = {"contact"},
    script = two_turns(Hidden.Vanished, " vanished instantly!", function(ctx)
      ctx:break_protect(ctx.target)
      ctx:damage(120)
    end)}),
  -- Sky Drop without the carrying: the engine cannot keep another Pokémon from acting for a turn.
  move("sky_drop", "Sky Drop", Type.Flying, Category.Physical, {manual_announce = true, flags = {"contact"},
    script = function(ctx)
      if gravity(ctx) then return ctx:fail() end
      two_turns(Hidden.Air, " took " .. ctx.target.name .. " into the sky!", function(inner)
        if inner.target:has_type(Type.Flying) then
          if inner:reached() then inner:message("It doesn't affect " .. inner.target.name .. "...") end
        else
          inner:damage(60)
        end
      end)(ctx)
    end}),
  move("gust", "Gust", Type.Flying, Category.Special, {hits_hidden = {Hidden.Air},
    script = function(ctx) ctx:damage(ctx.target.hidden == Hidden.Air and 80 or 40) end}),
  move("twister", "Twister", Type.Dragon, Category.Special, {target = Target.AllOpponents, hits_hidden = {Hidden.Air},
    script = function(ctx)
      for _, pokemon in ipairs(ctx.targets) do
        local hit = ctx:damage(pokemon.hidden == Hidden.Air and 80 or 40, {target = pokemon})
        if hit.hit then ctx:flinch(pokemon, {chance = 0.2}) end
      end
    end}),
  move("sky_uppercut", "Sky Uppercut", Type.Fighting, Category.Physical, {accuracy = 0.9, hits_hidden = {Hidden.Air},
    flags = {"contact"}, effects = {{kind = Effect.Damage, power = 85}}}),
  move("earthquake", "Earthquake", Type.Ground, Category.Physical, {target = Target.AllOthers, hits_hidden = {Hidden.Underground},
    script = function(ctx)
      each_target(ctx, function(pokemon) return pokemon.hidden == Hidden.Underground and 200 or 100 end)
    end}),
  move("surf", "Surf", Type.Water, Category.Special, {target = Target.AllOthers, hits_hidden = {Hidden.Underwater},
    script = function(ctx)
      each_target(ctx, function(pokemon) return pokemon.hidden == Hidden.Underwater and 180 or 90 end)
    end}),
  move("smack_down", "Smack Down", Type.Rock, Category.Physical, {hits_hidden = {Hidden.Air},
    script = function(ctx)
      local hit = ctx:damage(50)
      if not hit.hit then return end
      if ctx:unhide(ctx.target) then ctx:message(ctx.target.name .. " fell straight down!") end
      ctx:start_effect(ctx.target, "smacked_down", {rules = {{kind = Rule.Grounded}}})
    end}),
  move("thousand_arrows", "Thousand Arrows", Type.Ground, Category.Physical, {target = Target.AllOpponents, hits_hidden = {Hidden.Air},
    script = function(ctx)
      for _, pokemon in ipairs(ctx.targets) do
        -- Grounding comes first, so that the hit lands on a Flying type too.
        if ctx:reached(pokemon) then ctx:start_effect(pokemon, "smacked_down", {rules = {{kind = Rule.Grounded}}}) end
        ctx:damage(90, {target = pokemon})
      end
    end}),
  move("gravity", "Gravity", Type.Psychic, Category.Status, {target = Target.Field,
    script = function(ctx)
      local started = ctx:start_effect(ctx.field, "gravity", {turns = 5, end_message = "Gravity returned to normal!",
        rules = {{kind = Rule.Grounded}}})
      if not started then return ctx:fail() end
      ctx:message("Gravity intensified!")
      for _, pokemon in ipairs(ctx.everyone) do
        if ctx:unhide(pokemon) then ctx:message(pokemon.name .. " couldn't stay airborne because of gravity!") end
      end
    end}),

  -- Weather.
  move("thunder", "Thunder", Type.Electric, Category.Special, {accuracy = 0.7, hits_hidden = {Hidden.Air},
    script = function(ctx)
      local hit = ctx:damage(110, {never_miss = ctx.weather == Weather.Rain, accuracy = ctx.weather == Weather.Sun and 0.5 or nil})
      if hit.hit then ctx:apply_status(ctx.target, Status.Paralyzed, {chance = 0.3}) end
    end}),
  move("hurricane", "Hurricane", Type.Flying, Category.Special, {accuracy = 0.7, hits_hidden = {Hidden.Air},
    script = function(ctx)
      local hit = ctx:damage(110, {never_miss = ctx.weather == Weather.Rain, accuracy = ctx.weather == Weather.Sun and 0.5 or nil})
      if hit.hit then ctx:confuse(ctx.target, {chance = 0.3}) end
    end}),
  move("blizzard", "Blizzard", Type.Ice, Category.Special, {accuracy = 0.7, target = Target.AllOpponents,
    script = function(ctx)
      ctx:damage(110, {never_miss = ctx.weather == Weather.Hail})
      for _, pokemon in ipairs(ctx.targets) do
        if ctx:reached(pokemon) then ctx:apply_status(pokemon, Status.Frozen, {chance = 0.1}) end
      end
    end}),
  move("solar_blade", "Solar Blade", Type.Grass, Category.Physical, {manual_announce = true, flags = {"contact"},
    script = function(ctx)
      if ctx.turn == 1 and ctx.weather ~= Weather.Sun then
        ctx:message(ctx.user.name .. " absorbed light!")
        ctx:force_move(2, TargetPolicy.SameTarget)
        return
      end
      ctx:announce()
      local dim = ctx.weather ~= nil and ctx.weather ~= Weather.Sun
      ctx:damage(dim and 62 or 125)
    end}),
  move("synthesis", "Synthesis", Type.Grass, Category.Status, {target = Target.User,
    script = function(ctx)
      if ctx.user.hp == ctx.user.max_hp then return ctx:fail() end
      local share = ctx.weather == nil and 1/2 or ctx.weather == Weather.Sun and 2/3 or 1/4
      ctx:heal(ctx.user, {fraction = share})
      ctx:message(ctx.user.name .. " regained health!")
    end}),
  move("shore_up", "Shore Up", Type.Ground, Category.Status, {target = Target.User,
    script = function(ctx)
      if ctx.user.hp == ctx.user.max_hp then return ctx:fail() end
      ctx:heal(ctx.user, {fraction = ctx.weather == Weather.Sandstorm and 2/3 or 1/2})
    end}),
  move("weather_ball", "Weather Ball", Type.Normal, Category.Special, {
    script = function(ctx)
      local kinds = {[Weather.Sun] = Type.Fire, [Weather.Rain] = Type.Water, [Weather.Hail] = Type.Ice, [Weather.Sandstorm] = Type.Rock}
      local kind = ctx.weather and kinds[ctx.weather]
      ctx:damage(kind and 100 or 50, {type = kind})
    end}),
  move("growth", "Growth", Type.Normal, Category.Status, {target = Target.User,
    script = function(ctx)
      local stages = ctx.weather == Weather.Sun and 2 or 1
      ctx:change_stat(ctx.user, Stat.Attack, stages)
      ctx:change_stat(ctx.user, Stat.SpAttack, stages)
    end}),

  -- Screens, terrains and other effects with a duration.
  move("reflect", "Reflect", Type.Psychic, Category.Status, {target = Target.User,
    script = screen("reflect", Category.Physical, "Reflect wore off!")}),
  move("light_screen", "Light Screen", Type.Psychic, Category.Status, {target = Target.User,
    script = screen("light_screen", Category.Special, "Light Screen wore off!")}),
  move("brick_break", "Brick Break", Type.Fighting, Category.Physical, {flags = {"contact"},
    script = function(ctx)
      local broke = ctx:end_effect(ctx.target.team, "reflect")
      broke = ctx:end_effect(ctx.target.team, "light_screen") or broke
      if broke then ctx:message("It shattered the barrier!") end
      ctx:damage(75)
    end}),
  move("snipe_shot", "Glitzy Shot", Type.Fairy, Category.Special, {
    script = function(ctx)
      if ctx:damage(80).hit then
        ctx:start_effect(ctx.user.team, "light_screen", {turns = 5, end_message = "Light Screen wore off!",
          rules = {{kind = Rule.DamageTaken, category = Category.Special, factor = 0.5, not_on_crit = true}}})
      end
    end}),
  move("grassy_terrain", "Grassy Terrain", Type.Grass, Category.Status, {target = Target.Field,
    script = terrain("grassy_terrain", {
      {kind = Rule.HealEachTurn, fraction = 1/16, message = "{name} is healed by the grassy terrain!"},
      {kind = Rule.DamageDealt, type = Type.Grass, factor = 1.3}})}),
  move("electric_terrain", "Electric Terrain", Type.Electric, Category.Status, {target = Target.Field,
    script = terrain("electric_terrain", {
      {kind = Rule.BlockStatus, statuses = {Status.Asleep}},
      {kind = Rule.DamageDealt, type = Type.Electric, factor = 1.3}})}),
  move("genesis_supernova", "Genesis Supernova", Type.Psychic, Category.Special, {
    script = function(ctx)
      if ctx:damage(185).hit then
        ctx:start_effect(ctx.field, "psychic_terrain", {turns = 5, group = "terrain", restart = true,
          rules = {{kind = Rule.DamageDealt, type = Type.Psychic, factor = 1.3}}})
      end
    end}),
  move("splintered_stormshards", "Splintered Stormshards", Type.Rock, Category.Physical, {
    script = function(ctx)
      ctx:damage(190)
      if ctx:end_group(ctx.field, "terrain") then ctx:message("The terrain returned to normal.") end
    end}),
  move("floral_healing", "Floral Healing", Type.Fairy, Category.Status, {target = Target.AnyOther,
    script = function(ctx)
      if ctx:reached() then ctx:heal(ctx.target, {fraction = ctx.field.effects.grassy_terrain and 2/3 or 1/2}) end
    end}),
  move("safeguard", "Safeguard", Type.Normal, Category.Status, {target = Target.User,
    script = function(ctx)
      ctx:start_effect(ctx.user.team, "safeguard", {turns = 5, end_message = "Safeguard wore off!",
        rules = {{kind = Rule.BlockStatus, confusion = true}}})
    end}),
  move("uproar", "Uproar", Type.Normal, Category.Special, {flags = {"sound", "no_sleep_talk"},
    script = function(ctx)
      if ctx.turn == 1 then
        local turns = ctx:random_int(2, 5)
        ctx:force_move(turns, TargetPolicy.RandomOpponent)
        ctx:start_effect(ctx.field, "uproar", {turns = turns, restart = true, end_message = "The uproar ended.",
          rules = {{kind = Rule.BlockStatus, statuses = {Status.Asleep}}}})
        for _, pokemon in ipairs(ctx.everyone) do
          if ctx:cure_status(pokemon, Status.Asleep) then ctx:message(pokemon.name .. " woke up!") end
        end
      end
      ctx:damage(90)
    end,
    on_interrupt = function(ctx) ctx:end_effect(ctx.field, "uproar") end}),
  move("leech_seed", "Leech Seed", Type.Grass, Category.Status, {accuracy = 0.9,
    script = function(ctx)
      if not ctx:reached() then return end
      if ctx.target:has_type(Type.Grass) then return ctx:fail() end
      local seeded = ctx:start_effect(ctx.target, "leech_seed", {rules = {{kind = Rule.DrainEachTurn, fraction = 1/8,
        to = ctx.user, message = "{name}'s health is sapped by Leech Seed!"}}})
      if not seeded then return ctx:fail() end
      ctx:message(ctx.target.name .. " was seeded!")
    end}),
  move("jaw_lock", "Jaw Lock", Type.Dark, Category.Physical, {flags = {"contact"},
    script = function(ctx)
      if not ctx:damage(80).hit then return end
      ctx:start_effect(ctx.target, "jaw_lock", {rules = {{kind = Rule.Trapped}}})
      ctx:start_effect(ctx.user, "jaw_lock", {rules = {{kind = Rule.Trapped}}})
    end}),
  move("spirit_shackle", "Spirit Shackle", Type.Ghost, Category.Physical, {
    script = function(ctx)
      if ctx:damage(80).hit then ctx:start_effect(ctx.target, "shackled", {rules = {{kind = Rule.Trapped}}}) end
    end}),
  move("lock_on", "Lock-On", Type.Normal, Category.Status, {
    script = function(ctx)
      if not ctx:reached() then return end
      ctx:start_effect(ctx.user, "lock_on", {turns = 2, restart = true, rules = {{kind = Rule.AlwaysHit, against = ctx.target}}})
      ctx:message(ctx.user.name .. " took aim at " .. ctx.target.name .. "!")
    end}),
  move("sheer_cold", "Sheer Cold", Type.Ice, Category.Special, {accuracy = Accuracy.Ohko, effects = {{kind = Effect.Ohko}}}),
  move("endure", "Endure", Type.Normal, Category.Status, {target = Target.User, priority = 4,
    script = function(ctx)
      if not ctx:chance((1/3) ^ ctx.user.streak) then return ctx:fail() end
      ctx:start_effect(ctx.user, "endure", {turns = 1, rules = {{kind = Rule.Endure}}})
      ctx:message(ctx.user.name .. " braced itself!")
    end}),
  move("plasma_fists", "Plasma Fists", Type.Electric, Category.Physical, {flags = {"contact"},
    script = function(ctx)
      if ctx:damage(100).hit then
        ctx:start_effect(ctx.field, "ion_deluge", {turns = 1, restart = true,
          rules = {{kind = Rule.MoveType, from = Type.Normal, to = Type.Electric}}})
        ctx:message("A deluge of ions showers the battlefield!")
      end
    end}),
  move("roost", "Roost", Type.Flying, Category.Status, {target = Target.User,
    script = function(ctx)
      if ctx.user.hp == ctx.user.max_hp then return ctx:fail() end
      ctx:heal_self(1/2)
      ctx:start_effect(ctx.user, "roosting", {turns = 1, rules = {{kind = Rule.WithoutType, type = Type.Flying}}})
    end}),

  -- Types and stats that are not the usual ones.
  move("burn_up", "Burn Up", Type.Fire, Category.Special, {usable_while_frozen = true,
    script = function(ctx)
      if not ctx.user:has_type(Type.Fire) then return ctx:fail() end
      ctx:damage(130)
      ctx:lose_type(ctx.user, Type.Fire)
      ctx:message(ctx.user.name .. " burned itself out!")
    end}),
  move("flying_press", "Flying Press", Type.Fighting, Category.Physical, {accuracy = 0.95, flags = {"contact"},
    script = function(ctx)
      if gravity(ctx) then return ctx:fail() end
      local small = ctx.target.marks.minimized ~= nil
      ctx:damage(small and 200 or 100, {also_type = Type.Flying, never_miss = small})
    end}),
  move("freeze_dry", "Freeze-Dry", Type.Ice, Category.Special, {
    script = function(ctx)
      if ctx:damage(70, {effective = {[Type.Water] = 2}}).hit then ctx:apply_status(ctx.target, Status.Frozen, {chance = 0.1}) end
    end}),
  move("revelation_dance", "Revelation Dance", Type.Normal, Category.Special, {
    script = function(ctx)
      local kind = ctx.user.types[1]
      if kind == nil or kind == Type.None then ctx:damage(90, {typeless = true}) else ctx:damage(90, {type = kind}) end
    end}),
  move("judgment", "Judgment", Type.Normal, Category.Special, {
    script = function(ctx) ctx:damage(100, {type = plates[ctx.user.item]}) end}),
  move("psyshock", "Psyshock", Type.Psychic, Category.Special, {
    script = function(ctx) ctx:damage(80, {defense_stat = Stat.Defense}) end}),
  move("foul_play", "Foul Play", Type.Dark, Category.Physical, {flags = {"contact"},
    script = function(ctx) ctx:damage(95, {attack_from = ctx.target}) end}),
  move("chip_away", "Chip Away", Type.Normal, Category.Physical, {flags = {"contact"},
    script = function(ctx) ctx:damage(70, {ignore_stages = true}) end}),
  move("photon_geyser", "Photon Geyser", Type.Psychic, Category.Special, {
    script = function(ctx)
      local physical = ctx.user.attack > ctx.user.sp_attack
      ctx:damage(100, {category = physical and Category.Physical or Category.Special})
    end}),

  -- Power that depends on what is going on.
  move("eruption", "Eruption", Type.Fire, Category.Special, {target = Target.AllOpponents,
    script = function(ctx) ctx:damage(math.max(1, 150 * ctx.user.hp / ctx.user.max_hp)) end}),
  move("brine", "Brine", Type.Water, Category.Special, {
    script = function(ctx) ctx:damage(ctx.target.hp * 2 <= ctx.target.max_hp and 130 or 65) end}),
  move("hex", "Hex", Type.Ghost, Category.Special, {
    script = function(ctx) ctx:damage(ctx.target.status and 130 or 65) end}),
  move("smelling_salts", "Smelling Salts", Type.Normal, Category.Physical, {flags = {"contact"},
    script = function(ctx)
      local numb = ctx.target.status == Status.Paralyzed
      if ctx:damage(numb and 140 or 70).hit and numb then ctx:cure_status(ctx.target, Status.Paralyzed) end
    end}),
  move("wake_up_slap", "Wake-Up Slap", Type.Fighting, Category.Physical, {flags = {"contact"},
    script = function(ctx)
      local asleep = ctx.target.status == Status.Asleep
      if ctx:damage(asleep and 140 or 70).hit and asleep then ctx:cure_status(ctx.target, Status.Asleep) end
    end}),
  move("sacred_fire", "Sacred Fire", Type.Fire, Category.Physical, {accuracy = 0.95, usable_while_frozen = true,
    script = function(ctx)
      if ctx:damage(100).hit then ctx:apply_status(ctx.target, Status.Burned, {chance = 0.5}) end
    end}),
  move("sparkling_aria", "Sparkling Aria", Type.Water, Category.Special, {target = Target.AllOthers, flags = {"sound"},
    script = function(ctx)
      ctx:damage(90)
      for _, pokemon in ipairs(ctx.targets) do
        if ctx:reached(pokemon) then ctx:cure_status(pokemon, Status.Burned) end
      end
    end}),
  move("revenge", "Revenge", Type.Fighting, Category.Physical, {priority = -4, flags = {"contact"},
    script = function(ctx) ctx:damage(ctx.user.last_attacker == ctx.target and 120 or 60) end}),
  move("payback", "Payback", Type.Dark, Category.Physical, {flags = {"contact"},
    script = function(ctx) ctx:damage(ctx.target.acted and 100 or 50) end}),
  move("assurance", "Assurance", Type.Dark, Category.Physical, {flags = {"contact"},
    script = function(ctx) ctx:damage(ctx.target.hurt_this_turn and 120 or 60) end}),
  move("sucker_punch", "Sucker Punch", Type.Dark, Category.Physical, {priority = 1, flags = {"contact"},
    script = function(ctx)
      local chosen = ctx.target.selected_move
      if ctx.target.acted or not chosen or not chosen.damaging then return ctx:fail() end
      ctx:damage(70)
    end}),
  move("fake_out", "Fake Out", Type.Normal, Category.Physical, {priority = 3, flags = {"contact"},
    script = function(ctx)
      if ctx.user.turns_on_field > 1 then return ctx:fail() end
      if ctx:damage(40).hit then ctx:flinch(ctx.target) end
    end}),
  move("last_resort", "Last Resort", Type.Normal, Category.Physical, {flags = {"contact"},
    script = function(ctx)
      if not ctx.user:used_all_other_moves(ctx.move.id) then return ctx:fail() end
      ctx:damage(140)
    end}),
  move("retaliate", "Retaliate", Type.Normal, Category.Physical, {flags = {"contact"},
    script = function(ctx) ctx:damage(ctx.user.team.fainted_last_turn and 140 or 70) end}),
  move("stomping_tantrum", "Stomping Tantrum", Type.Ground, Category.Physical, {flags = {"contact"},
    script = function(ctx) ctx:damage(ctx.user.last_move_failed and 150 or 75) end}),
  move("stored_power", "Stored Power", Type.Psychic, Category.Special, {
    script = function(ctx) ctx:damage(20 + 20 * ctx.user:raised_stages()) end}),
  move("fell_stinger", "Fell Stinger", Type.Bug, Category.Physical, {flags = {"contact"},
    script = function(ctx)
      if ctx:damage(50).fainted then ctx:change_stat(ctx.user, Stat.Attack, 3) end
    end}),
  move("mind_blown", "Mind Blown", Type.Fire, Category.Special, {target = Target.AllOthers,
    script = function(ctx)
      ctx:damage(150)
      ctx:hurt(ctx.user, {fraction = 1/2})
    end}),

  -- Items, abilities and the like: only names to the engine.
  move("pay_day", "Pay Day", Type.Normal, Category.Physical, {
    script = function(ctx)
      if ctx:damage(40).hit then
        ctx:add_payout(ctx.user.level * 5)
        ctx:message("Coins were scattered everywhere!")
      end
    end}),
  move("acrobatics", "Acrobatics", Type.Flying, Category.Physical, {flags = {"contact"},
    script = function(ctx) ctx:damage(ctx.user.item and 55 or 110) end}),
  move("knock_off", "Knock Off", Type.Dark, Category.Physical, {flags = {"contact"},
    script = function(ctx)
      local hit = ctx:damage(ctx.target.item and 97 or 65)
      if hit.hit and ctx.target.item then
        ctx:message(ctx.user.name .. " knocked off " .. ctx.target.name .. "'s " .. ctx:take_item(ctx.target) .. "!")
      end
    end}),
  move("incinerate", "Incinerate", Type.Fire, Category.Special, {target = Target.AllOpponents,
    script = function(ctx)
      ctx:damage(60)
      for _, pokemon in ipairs(ctx.targets) do
        if ctx:reached(pokemon) and pokemon.item and pokemon.item:find("berry", 1, true) then ctx:take_item(pokemon) end
      end
    end}),
  -- The user gets the berry, but nothing makes it work: what items do is the game's side.
  move("bug_bite", "Bug Bite", Type.Bug, Category.Physical, {flags = {"contact"},
    script = function(ctx)
      local hit = ctx:damage(60)
      if hit.hit and ctx.target.item and ctx.target.item:find("berry", 1, true) then
        ctx:message(ctx.user.name .. " stole and ate its target's " .. ctx:take_item(ctx.target) .. "!")
        ctx:mark(ctx.user, "ate_berry")
      end
    end}),
  move("belch", "Belch", Type.Poison, Category.Special, {accuracy = 0.9,
    script = function(ctx)
      if not ctx.user.marks.ate_berry then return ctx:fail() end
      ctx:damage(120)
    end}),
  move("core_enforcer", "Core Enforcer", Type.Dragon, Category.Special, {target = Target.AllOpponents,
    script = function(ctx)
      ctx:damage(100)
      for _, pokemon in ipairs(ctx.targets) do
        if ctx:reached(pokemon) and pokemon.acted then ctx:suppress_ability(pokemon) end
      end
    end}),
  move("magnetic_flux", "Magnetic Flux", Type.Electric, Category.Status, {target = Target.UserAndAllies,
    script = function(ctx)
      for _, pokemon in ipairs(ctx.targets) do
        if pokemon.ability == "plus" or pokemon.ability == "minus" then
          ctx:change_stat(pokemon, Stat.Defense, 1)
          ctx:change_stat(pokemon, Stat.SpDefense, 1)
        end
      end
    end}),
  move("chatter", "Chatter", Type.Flying, Category.Special, {flags = {"sound", "no_mimic", "no_metronome"},
    script = function(ctx)
      if ctx:damage(65).hit and ctx.user.species == "chatot" then ctx:confuse(ctx.target) end
    end}),
  move("captivate", "Captivate", Type.Normal, Category.Status, {
    script = function(ctx)
      if not ctx:opposite_genders(ctx.user, ctx.target) then return ctx:fail() end
      if ctx:reached() then ctx:change_stat(ctx.target, Stat.SpAttack, -2) end
    end}),
  move("autotomize", "Autotomize", Type.Steel, Category.Status, {target = Target.User,
    script = function(ctx)
      ctx:change_stat(ctx.user, Stat.Speed, 2)
      if not ctx.user.marks.autotomized then
        ctx:set_weight(ctx.user, math.max(0.1, ctx.user.weight / 2))
        ctx:mark(ctx.user, "autotomized")
        ctx:message(ctx.user.name .. " became nimble!")
      end
    end}),

  -- Getting through, or not.
  move("feint", "Feint", Type.Normal, Category.Physical, {priority = 2,
    script = function(ctx)
      if ctx:break_protect(ctx.target) then ctx:message(ctx.target.name .. " fell for the feint!") end
      ctx:damage(30)
    end}),
  move("hyperspace_hole", "Hyperspace Hole", Type.Psychic, Category.Special, {accuracy = Accuracy.Always,
    script = function(ctx)
      ctx:break_protect(ctx.target)
      ctx:damage(80, {ignore_protect = true})
    end}),
  move("clear_smog", "Clear Smog", Type.Poison, Category.Special, {accuracy = Accuracy.Always,
    script = function(ctx)
      if ctx:damage(50).hit and ctx:reset_stats(ctx.target) then ctx:message(ctx.target.name .. "'s stat changes were removed!") end
    end}),
  move("spectral_thief", "Spectral Thief", Type.Ghost, Category.Physical, {flags = {"contact"},
    script = function(ctx)
      -- The boosts are taken first, so that the hit already has them; the hit follows the same check.
      if ctx:reached() then ctx:steal_boosts(ctx.target, ctx.user) end
      ctx:damage(90)
    end}),
  move("haze", "Haze", Type.Ice, Category.Status, {target = Target.Field,
    script = function(ctx)
      for _, pokemon in ipairs(ctx.everyone) do ctx:reset_stats(pokemon) end
      ctx:message("All stat changes were eliminated!")
    end}),

  -- Moves that react.
  move("shell_trap", "Shell Trap", Type.Fire, Category.Special, {priority = -3, target = Target.AllOpponents,
    on_turn_start = function(ctx)
      ctx:message(ctx.user.name .. " set a shell trap!")
      ctx:watch_hits_until_next_action()
    end,
    on_hit = function(ctx)
      if ctx.hit.category == Category.Physical then ctx:mark(ctx.user, "shell_trap", 1, 1) end
    end,
    script = function(ctx)
      if not ctx.user.marks.shell_trap then
        ctx:message(ctx.user.name .. "'s shell trap didn't work!")
        return ctx:fail()
      end
      ctx:damage(150)
    end}),
  move("beak_blast", "Beak Blast", Type.Flying, Category.Physical, {priority = -3,
    on_turn_start = function(ctx)
      ctx:message(ctx.user.name .. " started heating up its beak!")
      ctx:watch_hits_until_next_action()
    end,
    on_hit = function(ctx)
      if ctx.hit.contact then ctx:apply_status(ctx.target, Status.Burned) end
    end,
    script = function(ctx) ctx:damage(100) end}),

  -- Moves aimed at allies, at everyone, or at whoever the player picks.
  move("pollen_puff", "Pollen Puff", Type.Bug, Category.Special, {target = Target.AnyOther,
    script = function(ctx)
      if ctx.target.is_ally then
        if ctx:reached() then ctx:heal(ctx.target, {fraction = 1/2}) end
      else
        ctx:damage(90)
      end
    end}),
  move("aromatic_mist", "Aromatic Mist", Type.Fairy, Category.Status, {target = Target.Ally,
    script = function(ctx) ctx:change_stat(ctx.target, Stat.SpDefense, 1) end}),
  move("rototiller", "Rototiller", Type.Ground, Category.Status, {target = Target.All,
    script = function(ctx)
      for _, pokemon in ipairs(ctx.everyone) do
        if pokemon:has_type(Type.Grass) then
          ctx:change_stat(pokemon, Stat.Attack, 1)
          ctx:change_stat(pokemon, Stat.SpAttack, 1)
        end
      end
    end}),
  move("teeter_dance", "Teeter Dance", Type.Normal, Category.Status, {target = Target.AllOthers,
    script = function(ctx)
      for _, pokemon in ipairs(ctx.targets) do
        if ctx:reached(pokemon) then ctx:confuse(pokemon) end
      end
    end}),
  move("parabolic_charge", "Parabolic Charge", Type.Electric, Category.Special, {target = Target.AllOthers,
    script = function(ctx) ctx:damage(65, {drain = 0.5}) end}),
  move("diamond_storm", "Diamond Storm", Type.Rock, Category.Physical, {accuracy = 0.95, target = Target.AllOpponents,
    script = function(ctx)
      local hit = ctx:damage(100)
      for _ = 1, hit.hits do ctx:change_stat(ctx.user, Stat.Defense, 2, {chance = 0.5}) end
    end}),
  move("clangorous_soulblaze", "Clangorous Soulblaze", Type.Dragon, Category.Special, {target = Target.AllOpponents, flags = {"sound"},
    script = function(ctx)
      if not ctx:damage(185).hit then return end
      for _, stat in ipairs({Stat.Attack, Stat.Defense, Stat.SpAttack, Stat.SpDefense, Stat.Speed}) do
        ctx:change_stat(ctx.user, stat, 1)
      end
    end}),
  move("flame_burst", "Flame Burst", Type.Fire, Category.Special, {
    script = function(ctx)
      if not ctx:damage(70).hit then return end
      for _, pokemon in ipairs(ctx.target.team.pokemon) do
        if pokemon ~= ctx.target then ctx:hurt(pokemon, {fraction = 1/16}) end
      end
    end}),
  move("geomancy", "Geomancy", Type.Fairy, Category.Status, {target = Target.User, manual_announce = true,
    script = two_turns(nil, " is absorbing power!", function(ctx)
      for _, stat in ipairs({Stat.SpAttack, Stat.SpDefense, Stat.Speed}) do ctx:change_stat(ctx.user, stat, 2) end
    end)}),
  move("razor_wind", "Razor Wind", Type.Normal, Category.Special, {target = Target.AllOpponents, manual_announce = true,
    script = two_turns(nil, " whipped up a whirlwind!", function(ctx) ctx:damage(80, {high_crit = true}) end)}),
}
