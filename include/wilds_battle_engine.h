#ifndef WILDS_BATTLE_ENGINE_H
#define WILDS_BATTLE_ENGINE_H

#include <stdbool.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct WbeBattle WbeBattle;

/* Returned strings belong to the caller and must be freed with wbe_free_string. */
char *wbe_last_error(void);
void wbe_free_string(char *value);

/* Null on error. setup_json and moves_dir must be UTF-8, null-terminated strings.
 * Optional setup field "wild_battle":true enables allied escape choices. */
WbeBattle *wbe_create(const char *setup_json, const char *moves_dir);
void wbe_destroy(WbeBattle *battle);

/* JSON result or null on error; inspect wbe_last_error after a null/false result.
 * Terminal status.kind is "end" or "escaped" (which includes the escaping side).
 * Wild battle choices also include "escape" and "continue". Once a side answered an action
 * prompt this turn, its next action prompts end with an "undo" choice that takes the last
 * answer back and asks for it again; automated players should not pick it. */
char *wbe_advance(WbeBattle *battle);
bool wbe_respond(WbeBattle *battle, uint64_t prompt_id, uint32_t choice_id);
char *wbe_catalog(const char *moves_dir);

#ifdef __cplusplus
}
#endif
#endif
