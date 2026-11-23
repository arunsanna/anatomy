# Anatomy source and adapted method

- Original project: https://github.com/wheresryan22/anatomy
- Implementation fork: https://github.com/arunsanna/anatomy
- Pinned fork revision: `dbf697a527bbf7d358668e1557cb1e35ff725d46`
- Original skill at the pin: https://github.com/arunsanna/anatomy/blob/dbf697a527bbf7d358668e1557cb1e35ff725d46/SKILL.md
- Craft reference at the pin: https://github.com/arunsanna/anatomy/blob/dbf697a527bbf7d358668e1557cb1e35ff725d46/references/craft.md
- 3D method at the pin: https://github.com/arunsanna/anatomy/blob/dbf697a527bbf7d358668e1557cb1e35ff725d46/references/3d.md
- Reused GPU shader helpers: `kit/gl-shared.mjs`, embedded by `web/adapter.mjs` in its standalone HTML. Their `tonemap` function supplies the emissive surface transfer curve. Upstream kit and original skill remain unchanged.
- License: MIT, Copyright (c) 2026 Ryan; the full notice is in `LICENSE` beside this file and in exported HTML.

The upstream method uses an instrument or machine as the explanation. Its geometry, controls, readouts and movement share a truthful model. Supporting parts anchor the mechanism; shading and correct occlusion communicate volume. A framed figure provides a concise title, interaction hint, readout and sources. These principles carry over to Canvas.

This bounded native adaptation uses typed solids, scalar linear bindings, timeline events, a perspective camera and glow. It does not inherit the upstream minimum part counts, multi-hour workflows, SVG-specific construction API, full shader library, React integrations or geometry audit claims. The renderer is compiled trusted code, and agents supply scene data.
