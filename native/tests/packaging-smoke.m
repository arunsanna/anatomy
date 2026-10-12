#import <Foundation/Foundation.h>
#import <Metal/Metal.h>
#include "anatomy.h"
#include <assert.h>
#include <stdio.h>
#include <string.h>
int main(void) {
    @autoreleasepool {
        id<MTLDevice> device = MTLCreateSystemDefaultDevice();
        assert(device);
        void *renderer = anatomy_renderer_create((__bridge void *)device);
        assert(renderer);
        const char *scene = "{\"schemaVersion\":1,\"title\":\"Packaged ABI proof\",\"parts\":[]}";
        assert(anatomy_renderer_load_scene(renderer,(const uint8_t *)scene,strlen(scene)) == 1);
        char *state = anatomy_renderer_state_json(renderer,0);
        assert(state && strstr(state,"Packaged ABI proof"));
        anatomy_string_free(state);
        anatomy_renderer_destroy(renderer);
        puts("PASS packaged C ABI: retained Metal device/pipeline, scene state, string ownership");
    }
    return 0;
}
