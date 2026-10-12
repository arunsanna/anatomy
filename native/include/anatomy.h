#ifndef ANATOMY_H
#define ANATOMY_H
#include <stddef.h>
#include <stdint.h>
#ifdef __cplusplus
extern "C" {
#endif
/* Serialized calls. Objective-C arguments are borrowed; device is retained.
   Target: 2D BGRA8Unorm or BGRA8Unorm_sRGB, sampleCount=1, renderTarget usage.
   Swift owns command-buffer presentation and commit. Link Metal, Foundation,
   objc, System, c, and m (verified Rust staticlib runtime).
   Timeline evaluation: scene defaults -> authored events <= time -> caller
   control overrides. set_control persists an override until the next load. */
void *anatomy_renderer_create(void *metalDevice);
int32_t anatomy_renderer_load_scene(void *renderer, const uint8_t *json, size_t length);
int32_t anatomy_renderer_set_control(void *renderer, const char *id, double value);
int32_t anatomy_renderer_encode(void *renderer, void *commandBuffer, void *texture,
                               double time, double yaw, double pitch, double distance);
char *anatomy_renderer_state_json(void *renderer, double time);
char *anatomy_renderer_last_error(void *renderer);
void anatomy_string_free(char *text);
void anatomy_renderer_destroy(void *renderer);
#ifdef __cplusplus
}
#endif
#endif
