#ifndef RYSYN_FFI_H
#define RYSYN_FFI_H

#pragma once

#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

/**
 * Initialize the bridge. Call once at startup from JUCE.
 */
bool rysyn_init(void);

/**
 * Get state snapshot as JSON string. Caller must free with rysyn_free_string.
 */
char *rysyn_get_state_json(void);

/**
 * Update state from JSON. Called by JUCE audio core.
 */
bool rysyn_update_state_json(const char *json);

/**
 * Send a command as JSON. Called by UI or scripts.
 */
bool rysyn_send_command_json(const char *json);

/**
 * Try to receive next command as JSON. Returns null if queue is empty.
 * Caller must free with rysyn_free_string.
 */
char *rysyn_recv_command_json(void);

/**
 * Free a string allocated by Rust
 */
void rysyn_free_string(char *s);

bool rysyn_transport_play(void);

bool rysyn_transport_pause(void);

bool rysyn_transport_stop(void);

bool rysyn_transport_set_bpm(double bpm);

bool rysyn_transport_set_playhead(double beats);

/**
 * Get current playhead position in beats (fast path, no JSON)
 */
double rysyn_get_playhead_beats(void);

/**
 * Get current BPM (fast path)
 */
double rysyn_get_bpm(void);

/**
 * Check if playing (fast path)
 */
bool rysyn_is_playing(void);

#endif  /* RYSYN_FFI_H */
