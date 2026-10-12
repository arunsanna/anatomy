import test from 'node:test';
import assert from 'node:assert/strict';
import vm from 'node:vm';
import { evaluateScene, exportHTML, exportSVG } from './adapter.mjs';

const scene = {
  schemaVersion: 1, title: 'Bench',
  parts: [{ id: 'slide', kind: 'box', size: [2, 1, 1], effect: 'glow' }],
  controls: [{ id: 'stroke', label: 'Stroke', min: 0, max: 10, step: 0.5, value: 2 }],
  bindings: [{ part: 'slide', control: 'stroke', property: 'positionX', scale: 0.5, offset: -1 }],
  readouts: [{ label: 'Displacement', control: 'stroke', scale: 10, offset: 1, unit: 'mm' }],
  timeline: [{ time: 1, control: 'stroke', value: 4 }, { time: 3, control: 'stroke', value: 8 }]
};
test('one control drives both physical position and readout without mutating source', () => {
  const before = JSON.stringify(scene);
  const state = evaluateScene(scene, 0, { stroke: 6 });
  assert.equal(state.parts[0].position[0], 2);
  assert.deepEqual(state.readouts, [{ label: 'Displacement', value: 61, unit: 'mm' }]);
  assert.equal(JSON.stringify(scene), before);
});
test('timeline seeks deterministically including exact event boundaries', () => {
  assert.equal(evaluateScene(scene, 0.999, { stroke: 6 }).controls.stroke, 6);
  assert.equal(evaluateScene(scene, 1).controls.stroke, 4);
  assert.equal(evaluateScene(scene, 1, { stroke: 6 }).controls.stroke, 6);
  assert.equal(evaluateScene(scene, 4).controls.stroke, 8);
  assert.equal(evaluateScene(scene, 0).controls.stroke, 2);
  assert.deepEqual(evaluateScene(scene, 3), evaluateScene(scene, 3));
});
test('controls enforce finite range and resolve scene defaults', () => {
  assert.throws(() => evaluateScene(scene, 0, { stroke: 11 }), /range/);
  assert.throws(() => evaluateScene(scene, 0, { stroke: NaN }), /finite/);
  assert.deepEqual(evaluateScene(scene).parts[0].rotation, [0, 0, 0]);
  assert.deepEqual(evaluateScene(scene).camera, { yaw: 35, pitch: 25, distance: 8 });
});
test('standalone export has an executable classic script with scene and no resource requests', () => {
  const html = exportHTML(scene);
  const scripts = [...html.matchAll(/<script>([\s\S]*?)<\/script>/g)];
  assert.equal(scripts.length, 1);
  new vm.Script(scripts[0][1]);
  assert.doesNotMatch(html, /(?:src|href)=["'](?:https?:|\.\.\/|\.\/)/);
  assert.doesNotMatch(scripts[0][1], /\b(?:fetch|import)\s*\(/);
  assert.match(html, /type="range"/);
  assert.match(html, /Anatomy/);
});
test('scale binding multiplies enclosing base size while rotations assign degrees', () => {
  const assembly = { ...scene, bindings: [
    { part: 'slide', control: 'stroke', property: 'scaleX', scale: 0.5 },
    { part: 'slide', control: 'stroke', property: 'rotationY', scale: 15 }
  ] };
  assert.deepEqual(evaluateScene(assembly, 0, { stroke: 6 }).parts[0].size, [6, 1, 1]);
  assert.equal(evaluateScene(assembly, 0, { stroke: 6 }).parts[0].rotation[1], 90);
});
test('native parity rejects negative time and nonphysical evaluated dimensions/glow', () => {
  assert.throws(() => evaluateScene(scene, -1), /nonnegative/);
  for (const [property,offset] of [['scaleX', 0], ['glow', -1]]) {
    assert.throws(() => evaluateScene({ ...scene, bindings:[{part:'slide',control:'stroke',property,scale:0,offset}] }), /size|glow/);
  }
});

test('interactive caller override wins an authored time-zero event for geometry and readout', () => {
  const recorded = { ...scene, timeline:[{time:0,control:'stroke',value:1}] };
  const state = evaluateScene(recorded,0,{stroke:6});
  assert.equal(state.controls.stroke,6);
  assert.equal(state.parts[0].position[0],2);
  assert.equal(state.readouts[0].value,61);
});

test('scalable SVG snapshot includes physical solids and readout from changed control', () => {
  const before=exportSVG(scene,0,{stroke:2});
  const after=exportSVG(scene,0,{stroke:6});
  assert.match(after, /^<svg[^>]*xmlns="http:\/\/www.w3.org\/2000\/svg"/);
  assert.match(after,/viewBox="0 0 960 640"/);
  assert.match(after,/<polygon /);
  assert.match(after,/Bench/);
  assert.match(after,/Displacement: 61 mm/);
  assert.notEqual(before.match(/<polygon[^>]*points="([^"]+)/)[1],after.match(/<polygon[^>]*points="([^"]+)/)[1]);
  const orbit=exportSVG({...scene,camera:{yaw:90,pitch:25,distance:8}},0,{stroke:6});
  assert.notEqual(after.match(/<polygon[^>]*points="([^"]+)/)[1],orbit.match(/<polygon[^>]*points="([^"]+)/)[1]);
  assert.match(after,/MIT License/);
});
test('HTML export preserves initial overrides outside authored scene defaults', () => {
  const html=exportHTML(scene,{stroke:6});
  assert.doesNotMatch(html,/__ANATOMY_INITIAL_CONTROLS__/);
  assert.match(html,/"stroke":6/);
  assert.match(html,/Save SVG/);
  new vm.Script(html.match(/<script>([\s\S]*?)<\/script>/)[1]);
});
