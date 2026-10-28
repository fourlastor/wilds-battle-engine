#ifndef WILDS_BATTLE_ENGINE_H
#define WILDS_BATTLE_ENGINE_H

#include <stdbool.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct WbeBattle WbeBattle;
typedef struct WbeCatalog WbeCatalog;
typedef struct WbeMoveRequest WbeMoveRequest;

/* Supplies the Lua source of the move `move_id` (a lowercase slug such as "thunder-wave").
 * When the host has it, call wbe_move_found(request, origin, text) and return true; return
 * false to use the built-in move, if any. Each ID is asked for at most once per catalog, when a
 * battle first needs it. Called on the thread using the catalog; it must not unwind. */
typedef bool (*wbe_move_source)(void *user_data, const char *move_id, WbeMoveRequest *request);
/* `origin` names the source in error messages (e.g. a file path). Both strings are copied. */
void wbe_move_found(WbeMoveRequest *request, const char *origin, const char *text);

/* Returned strings belong to the caller and must be freed with wbe_free_string. */
char *wbe_last_error(void);
void wbe_free_string(char *value);

/* Moves from `source` (may be null) over the built-in moves. Null on error. `source` and
 * `user_data` must stay valid until the catalog and all its battles are destroyed. */
WbeCatalog *wbe_catalog_create(wbe_move_source source, void *user_data);
/* As wbe_catalog_create, with item scripts from `item_source` (may be null) over the built-in
 * items, called the same way. */
WbeCatalog *wbe_catalog_create_with_items(wbe_move_source move_source,
                                          wbe_move_source item_source, void *user_data);
void wbe_catalog_destroy(WbeCatalog *catalog);
/* {"id","name","targets"} for an item, JSON null when no source has it, or null on error. */
char *wbe_catalog_item(const WbeCatalog *catalog, const char *item_id);
/* {"id","name","pp"} for a move, JSON null when no source has it, or null on a load error. */
char *wbe_catalog_move(const WbeCatalog *catalog, const char *move_id);

/* Null on error. setup_json must be a UTF-8, null-terminated string.
 * Optional setup field "wild_battle":true enables allied escape choices; "rules" (weather,
 * terrain, conditions_lua, no_escape) and "context" describe how the battle starts, "bags" the
 * items each side may use and "script_lua" the battle script, see README.
 * The battle shares the catalog's loaded moves; the catalog may be destroyed first. */
WbeBattle *wbe_create_with_catalog(const char *setup_json, const WbeCatalog *catalog);
/* As above, with the .lua files of moves_dir (named <move id>.lua) over the built-in moves,
 * or only the built-in moves when moves_dir is null. */
WbeBattle *wbe_create(const char *setup_json, const char *moves_dir);
void wbe_destroy(WbeBattle *battle);

/* JSON result or null on error; inspect wbe_last_error after a null/false result.
 * Terminal status.kind is "end", "escaped" (which includes the escaping side) or "captured"
 * (which includes the caught target).
 * Wild battle choices also include "escape" and "continue". Once a side answered an action
 * prompt this turn, its next action prompts end with an "undo" choice that takes the last
 * answer back and asks for it again; automated players should not pick it. */
char *wbe_advance(WbeBattle *battle);
bool wbe_respond(WbeBattle *battle, uint64_t prompt_id, uint32_t choice_id);
/* The items `side` ("allies" or "foes") may use, as a JSON array of item IDs; false on error. */
bool wbe_set_bag(WbeBattle *battle, const char *side, const char *items_json);
/* Loads every file of moves_dir (or nothing, when null) and lists it with the built-in moves. */
char *wbe_catalog(const char *moves_dir);

#ifdef __cplusplus
}
#endif
#endif
