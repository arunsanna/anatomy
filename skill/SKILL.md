---
name: anatomy
description: Create and conversationally refine interactive explanatory figures inside Switchboard Canvas, using the typed Anatomy scene format. Default for Canvas interactive figure requests; does not trigger for general diagrams outside Canvas.
---

# Anatomy for Switchboard Canvas

Make an idea understandable through a recognizable physical object whose mechanism explains it: a slider on a bench, a valve in a housing, a lamp on a test rig. Choose solid, purposeful parts; avoid a floating stack of labeled boxes. The part that moves and the number that changes must come from the same control.

Read [the scene contract](references/scene-contract.md) before generating or refining a scene. Consult [the upstream method and source](references/upstream.md) when choosing a physical metaphor or checking how Anatomy frames an explanation. [The representative scene](fixtures/representative.json) shows a moving, measurable bench assembly; it is an example, not an object to copy for every request.

The deliverable is **exactly one fenced `json` block containing one complete Scene JSON v1 object**, with no prose outside the fence. Use `description` for the short explanation and approximation caveats, and `sources` for factual/model provenance. The Canvas client renders the object; do not return HTML, SVG, executable code, Markdown diagrams or renderer instructions.

1. Identify the one relationship the user wants to understand. Pick an object whose movement expresses it. If needed, state explanatory simplifications in `description`; do not invent physical measurements, source claims or scientific fidelity.
2. Build the smallest useful object from `box`, `cylinder` and `sphere`. Match the user's requested detail rather than applying upstream's 40–80-part minimum. First deliver a coherent mechanism with a base, supports and working part. Add detail only where it explains function; stay within one scene and one focused modeling pass.
3. Expose useful scalar controls with honest labels, bounds, steps and units. Bind them with the contract's linear formulas. A technical readout references the same control and formula as the relevant part. State when a model is illustrative; do not suggest a linear binding implements nonlinear physics.
4. Use the supported `glow` effect where it communicates a powered or active part. True 3D orbit/depth are provided by the renderer. Do not claim the upstream water, exhaust, caustics, shaders, custom meshes, labels or springs are implemented by this bounded format.
5. Check all ids/references, finite numbers, positive sizes, control ranges and steps, sorted timeline events and one binding per part property. Camera and rotations are degrees; Y is up; cylinder's axis is Y. Size is enclosing dimensions, not radius.
6. Return the single complete scene. On conversational refinement, preserve existing part/control ids where their meaning survives, apply the requested change and return a new complete scene. Never overwrite a completed old revision; the app associates the new result with its session and revision.

For a request requiring unsupported geometry or a nonlinear model, produce a useful supported approximation only if it preserves the explanatory meaning, and describe the limitation inside the scene. A larger project or a new renderer effect requires a separate implementation decision. Canvas's Webpage and MP4 actions export the selected completed revision; generation does not start services or publish anything.

Adapted from Anatomy by Ryan, MIT licensed. Fork source pin: `arunsanna/anatomy@dbf697a527bbf7d358668e1557cb1e35ff725d46`. Keep [the upstream license](references/LICENSE) with distributed resources.
