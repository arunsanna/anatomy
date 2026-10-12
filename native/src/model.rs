use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::BTreeMap;
fn one() -> f64 {
    1.0
}
fn size() -> [f64; 3] {
    [1.; 3]
}
fn color() -> [f64; 4] {
    [0.65, 0.75, 0.8, 1.]
}
fn background() -> [f64; 4] {
    [0.025, 0.04, 0.06, 1.]
}
#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Part {
    pub id: String,
    pub kind: String,
    #[serde(default)]
    pub position: [f64; 3],
    #[serde(default)]
    pub rotation: [f64; 3],
    #[serde(default = "size")]
    pub size: [f64; 3],
    #[serde(default = "color")]
    pub color: [f64; 4],
    #[serde(default)]
    pub effect: Option<String>,
    #[serde(skip_deserializing, default)]
    pub glow: f64,
}
#[derive(Clone, Deserialize)]
pub struct Control {
    pub id: String,
    pub label: String,
    pub min: f64,
    pub max: f64,
    pub step: f64,
    pub value: f64,
    #[serde(default)]
    pub unit: String,
}
#[derive(Clone, Deserialize)]
pub struct Binding {
    pub part: String,
    pub control: String,
    pub property: String,
    #[serde(default = "one")]
    pub scale: f64,
    #[serde(default)]
    pub offset: f64,
}
#[derive(Clone, Deserialize)]
pub struct Readout {
    pub label: String,
    pub control: String,
    #[serde(default = "one")]
    pub scale: f64,
    #[serde(default)]
    pub offset: f64,
    #[serde(default)]
    pub unit: String,
}
#[derive(Clone, Deserialize)]
pub struct Event {
    pub time: f64,
    pub control: String,
    pub value: f64,
}
#[derive(Clone, Deserialize)]
#[serde(default)]
pub struct Camera {
    pub yaw: f64,
    pub pitch: f64,
    pub distance: f64,
}
impl Default for Camera {
    fn default() -> Self {
        Self {
            yaw: 35.,
            pitch: 25.,
            distance: 8.,
        }
    }
}
#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Scene {
    pub schema_version: u32,
    pub title: String,
    pub parts: Vec<Part>,
    #[serde(default)]
    pub camera: Camera,
    #[serde(default = "background")]
    pub background: [f64; 4],
    #[serde(default)]
    pub controls: Vec<Control>,
    #[serde(default)]
    pub bindings: Vec<Binding>,
    #[serde(default)]
    pub readouts: Vec<Readout>,
    #[serde(default)]
    pub timeline: Vec<Event>,
}
pub struct Model {
    pub scene: Scene,
    values: BTreeMap<String, f64>,
}
impl Model {
    pub fn load(bytes: &[u8]) -> Result<Self, String> {
        let scene: Scene =
            serde_json::from_slice(bytes).map_err(|e| format!("Invalid scene JSON: {e}"))?;
        if scene.schema_version != 1 {
            return Err("schemaVersion must be 1".into());
        }
        validate(&scene)?;
        let values = BTreeMap::new();
        let model = Self { scene, values };
        Ok(model)
    }
    pub fn set_control(&mut self, id: &str, value: f64) -> Result<(), String> {
        let control = self
            .scene
            .controls
            .iter()
            .find(|c| c.id == id)
            .ok_or_else(|| format!("Unknown control: {id}"))?;
        if !value.is_finite() || value < control.min || value > control.max {
            return Err(format!("Control {id} value is outside its finite range"));
        }
        self.values.insert(id.to_owned(), value);
        Ok(())
    }
    pub fn evaluated(&self, time: f64) -> Result<(BTreeMap<String, f64>, Vec<Part>), String> {
        if !time.is_finite() || time < 0. {
            return Err("time must be finite and nonnegative".into());
        }
        let mut values: BTreeMap<String, f64> = self
            .scene
            .controls
            .iter()
            .map(|c| (c.id.clone(), c.value))
            .collect();
        for event in self.scene.timeline.iter().take_while(|e| e.time <= time) {
            values.insert(event.control.clone(), event.value);
        }
        values.extend(self.values.clone());
        let mut parts = self.scene.parts.clone();
        for p in &mut parts {
            p.glow = if p.effect.as_deref() == Some("glow") {
                1.
            } else {
                0.
            };
        }
        for b in &self.scene.bindings {
            let value = values[&b.control] * b.scale + b.offset;
            let p = parts
                .iter_mut()
                .find(|p| p.id == b.part)
                .ok_or("Unknown bound part")?;
            match b.property.as_str() {
                "rotationX" => p.rotation[0] = value,
                "rotationY" => p.rotation[1] = value,
                "rotationZ" => p.rotation[2] = value,
                "positionX" => p.position[0] = value,
                "positionY" => p.position[1] = value,
                "positionZ" => p.position[2] = value,
                "scaleX" => p.size[0] *= value,
                "scaleY" => p.size[1] *= value,
                "scaleZ" => p.size[2] *= value,
                "glow" => p.glow = value,
                _ => return Err("Unknown bound property".into()),
            }
        }
        for p in &parts {
            if p.size.iter().any(|v| *v <= 0. || !v.is_finite())
                || p.position.iter().chain(&p.rotation).any(|v| !v.is_finite())
                || !p.glow.is_finite()
                || p.glow < 0.
            {
                return Err("Evaluated transform or glow is invalid".into());
            }
        }
        Ok((values, parts))
    }
    pub fn state(&self, time: f64) -> Result<Value, String> {
        let (values, parts) = self.evaluated(time)?;
        let readouts:Vec<_>=self.scene.readouts.iter().map(|r|json!({"label":r.label,"value":values[&r.control]*r.scale+r.offset,"unit":r.unit})).collect();
        Ok(json!({"title":self.scene.title,"controls":values,"readouts":readouts,"parts":parts}))
    }
}

fn validate(s: &Scene) -> Result<(), String> {
    let mut ids = std::collections::BTreeSet::new();
    for p in &s.parts {
        if p.id.is_empty() || !ids.insert(&p.id) {
            return Err("Part ids must be nonempty and unique".into());
        }
        if !["box", "cylinder", "sphere"].contains(&p.kind.as_str()) {
            return Err(format!("Unsupported part kind: {}", p.kind));
        }
        if p.position
            .iter()
            .chain(&p.rotation)
            .chain(&p.size)
            .any(|v| !v.is_finite())
            || p.size.iter().any(|v| *v <= 0.)
        {
            return Err("Transforms must be finite and size positive".into());
        }
        if p.color
            .iter()
            .any(|v| !v.is_finite() || !(0. ..=1.).contains(v))
        {
            return Err("Color components must be in 0...1".into());
        }
        if !matches!(p.effect.as_deref(), None | Some("none") | Some("glow")) {
            return Err("Unsupported effect".into());
        }
    }
    let mut controls = BTreeMap::new();
    for c in &s.controls {
        if c.id.is_empty() || controls.insert(&c.id, c).is_some() {
            return Err("Control ids must be nonempty and unique".into());
        }
        if [c.min, c.max, c.step, c.value]
            .iter()
            .any(|v| !v.is_finite())
            || c.min > c.max
            || c.step <= 0.
            || c.value < c.min
            || c.value > c.max
        {
            return Err("Invalid control range, step or value".into());
        }
    }
    if s.background
        .iter()
        .any(|v| !v.is_finite() || !(0. ..=1.).contains(v))
        || !s.camera.yaw.is_finite()
        || !s.camera.pitch.is_finite()
        || !s.camera.distance.is_finite()
        || s.camera.distance <= 0.
    {
        return Err("Invalid camera or background".into());
    }
    let mut bound = std::collections::BTreeSet::new();
    for b in &s.bindings {
        if !ids.contains(&b.part) || !controls.contains_key(&b.control) {
            return Err("Binding references an unknown part or control".into());
        }
        if ![
            "rotationX",
            "rotationY",
            "rotationZ",
            "positionX",
            "positionY",
            "positionZ",
            "scaleX",
            "scaleY",
            "scaleZ",
            "glow",
        ]
        .contains(&b.property.as_str())
            || !b.scale.is_finite()
            || !b.offset.is_finite()
            || !bound.insert((&b.part, &b.property))
        {
            return Err("Invalid or duplicate binding property".into());
        }
    }
    for r in &s.readouts {
        if !controls.contains_key(&r.control) || !r.scale.is_finite() || !r.offset.is_finite() {
            return Err("Invalid readout reference or formula".into());
        }
    }
    let mut previous = 0.;
    for e in &s.timeline {
        let c = controls
            .get(&e.control)
            .ok_or("Timeline references unknown control")?;
        if !e.time.is_finite()
            || e.time < previous
            || !e.value.is_finite()
            || e.value < c.min
            || e.value > c.max
        {
            return Err(
                "Timeline must be sorted with nonnegative finite times and in-range values".into(),
            );
        }
        previous = e.time;
    }
    Ok(())
}
