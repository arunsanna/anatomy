# SWIT-403 first scene contract

This is the bounded implementation contract for the approved build, not a general scripting language. No security-audit work is assigned. JSON supplies figure data; trusted compiled code supplies rendering/model behavior.

## Scene JSON v1

Required fields: `schemaVersion` (1), `title`, `parts`. Optional: `description`, `seed` (defaults zero), `camera`, `background`, `controls`, `bindings`, `readouts`, `timeline`, `sources`.

- `camera`: `yaw`, `pitch` in degrees, `distance` in world units. Defaults 35, 25, 8; these are document camera defaults, not operational caps.
- `background`: normalized RGBA array. Default dark paper `[0.025,0.04,0.06,1]`.
- `parts`: `{id, kind, position, rotation, size, color, effect}`. Kind is `box`, `cylinder` or `sphere`; position/size are 3-number arrays, rotation is Euler degrees, color is normalized RGBA, effect is absent/`none`/`glow`. Positive size. Default transforms are identity. Cylinder uses Y as its axis; `size` is its enclosing X/Y/Z dimensions.
- `controls`: `{id,label,min,max,step,value,unit}` finite scalar controls. Value lies in range, step positive. Unit optional.
- `bindings`: `{part,control,property,scale,offset}`. Properties: `rotationX`, `rotationY`, `rotationZ`, `positionX`, `positionY`, `positionZ`, `scaleX`, `scaleY`, `scaleZ`, `glow`. Bound value = control value * scale + offset (defaults 1/0). Rotation degrees. References must exist. Each bound property has one binding.
- `readouts`: `{label,control,scale,offset,unit}`; value from the same control/model formula. Defaults 1/0. No independent invented readout.
- `timeline`: optional sorted `{time,control,value}` events, seconds; apply events at or before the requested timestamp, starting from scene control defaults; apply caller overrides last so interactive changes win. Identical time/input trace gives identical state in native/web. No time from wall clock inside the core.
- `sources`: optional strings showing fact/model provenance. Explanatory approximations stay explicit.

The first fixtures demonstrate vector-style solids/technical readouts, a glow effect, and true depth/orbit with a control-driven moving assembly. More native effects are incremental extensions, not claims that all upstream shaders are already ported.

## Native C interface

Native static library/module name: `AnatomyNative`; C header `anatomy.h`.

```
void *anatomy_renderer_create(void *metalDevice);
int32_t anatomy_renderer_load_scene(void *renderer, const uint8_t *json, size_t length);
int32_t anatomy_renderer_set_control(void *renderer, const char *id, double value);
int32_t anatomy_renderer_encode(void *renderer, void *commandBuffer, void *texture,
                               double time, double yaw, double pitch, double distance);
char *anatomy_renderer_state_json(void *renderer, double time);
char *anatomy_renderer_last_error(void *renderer);
void anatomy_string_free(char *text);
void anatomy_renderer_destroy(void *renderer);
```

Return 1 for successful load/control/encode, 0 on functional error. Create returns null on failure. Returned strings are owned and freed with `anatomy_string_free`. State JSON contains `title`, `controls` (id -> value), `readouts` (label/value/unit) and evaluated parts. The last error explains the latest unsuccessful operation.

Swift owns the device, native view, command-buffer creation/presentation/commit and target texture. Rust retains the device as needed, owns trusted pipelines/scene/model/geometry and encodes Metal draw commands into the provided buffer. Passed Objective-C objects are borrowed valid objects for the call; command-buffer resource retention keeps encoded work alive. Wrapper serializes calls; video export uses its own renderer from the immutable scene snapshot. No generated Rust binary or shader code is loaded from JSON.

## Web/skill deliverables

`web/adapter.mjs`: export `evaluateScene(scene, time=0, controls={})`, `exportHTML(scene)` and optionally `exportSVG(scene, time=0, controls={})`; Node built-ins only. HTML is self-contained, with no remote dependency or local HTTP service. Controls/readouts/camera and typed scene match native. Use the existing fork kit where it fits; do not rewrite the whole upstream engine or introduce React/Three.js.

`skill/` packet: scoped Anatomy SKILL.md, upstream method/source/license references, this native scene shape and one representative scene. Default trigger is interactive figure generation inside Switchboard Canvas. Agent returns one fenced JSON scene and can refine it. App ships the skill as a resource and supplies it to the existing SDK session; project skill discovery can also use `.agents/skills/anatomy/`. Record fork pin.

Versioning, revisions and session association are client integration metadata around this scene. They must not mutate old completed source scenes. Web export and video encode the selected completed revision. Runtime FPS derives from local display capability; export size/rate are explicit user choices. No new environment knobs or gateway renderer.

## Locked camera and transform clarification

Right-handed coordinates, Y up, look at the origin. Yaw 0 puts the eye on +Z. Eye = distance × (sin(yaw)cos(pitch), sin(pitch), cos(yaw)cos(pitch)). Perspective vertical FOV 45°, near .1/far 100; aspect is width/height. Part transformations apply scale, Rx, Ry, Rz, then translation, using column vectors. Scale bindings multiply the part's original enclosing size dimension by the evaluated factor; position and rotation bindings assign the component. Effect glow starts at intensity 1; a glow binding supplies the intensity. Timeline evaluation starts from defaults, applies stable sorted events at or before time, then applies caller overrides last. An authored event at time zero must not defeat an interactive control.
