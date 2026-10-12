use anatomy_native::model::Model;
const SCENE: &str = r#"{"schemaVersion":1,"title":"Assembly","parts":[{"id":"arm","kind":"box"}],"controls":[{"id":"angle","label":"Angle","min":0,"max":90,"step":1,"value":10,"unit":"deg"}],"bindings":[{"part":"arm","control":"angle","property":"rotationY","scale":2}],"readouts":[{"label":"Pose","control":"angle","scale":2,"unit":"deg"}]}"#;
#[test]
fn caller_control_drives_pose_and_readout() {
    let mut model = Model::load(SCENE.as_bytes()).unwrap();
    model.set_control("angle", 20.0).unwrap();
    let state = model.state(0.0).unwrap();
    assert_eq!(state["controls"]["angle"], 20.0);
    assert_eq!(state["parts"][0]["rotation"][1], 40.0);
    assert_eq!(state["readouts"][0]["value"], 40.0);
}
#[test]
fn timeline_is_reproducible_and_scrubbable() {
    let mut scene: serde_json::Value = serde_json::from_str(SCENE).unwrap();
    scene["timeline"] = serde_json::json!([{ "time":1,"control":"angle","value":30 },{ "time":2,"control":"angle","value":50 }]);
    let model = Model::load(scene.to_string().as_bytes()).unwrap();
    assert_eq!(model.state(2.).unwrap()["parts"][0]["rotation"][1], 100.);
    assert_eq!(model.state(1.).unwrap()["readouts"][0]["value"], 60.);
    assert_eq!(model.state(0.).unwrap()["controls"]["angle"], 10.);
    assert_eq!(model.state(1.).unwrap(), model.state(1.).unwrap());
}
#[test]
fn malformed_references_ranges_transforms_and_events_are_rejected() {
    let base: serde_json::Value = serde_json::from_str(SCENE).unwrap();
    let mut cases = Vec::new();
    let mut s = base.clone();
    s["bindings"][0]["control"] = "missing".into();
    cases.push(s);
    let mut s = base.clone();
    s["parts"][0]["size"] = serde_json::json!([1, -1, 1]);
    cases.push(s);
    let mut s = base.clone();
    s["controls"][0]["value"] = 100.into();
    cases.push(s);
    let mut s = base.clone();
    s["timeline"] = serde_json::json!([{"time":2,"control":"angle","value":5},{"time":1,"control":"angle","value":4}]);
    cases.push(s);
    let mut s = base.clone();
    s["parts"][0]["kind"] = "torus".into();
    cases.push(s);
    for s in cases {
        assert!(
            Model::load(s.to_string().as_bytes()).is_err(),
            "accepted {s}"
        );
    }
    let mut m = Model::load(SCENE.as_bytes()).unwrap();
    assert!(m.set_control("angle", f64::NAN).is_err());
    assert!(m.set_control("angle", 100.).is_err());
    assert!(m.set_control("unknown", 5.).is_err());
    assert!(m.state(f64::NAN).is_err());
}
#[test]
fn representative_fixture_has_shared_pose_readout_at_timeline_time() {
    let model = Model::load(include_bytes!("fixtures/representative.json")).unwrap();
    let state = model.state(2.).unwrap();
    assert_eq!(state["controls"]["stroke"], 2.);
    let carriage = state["parts"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["id"] == "carriage")
        .unwrap();
    assert_eq!(carriage["position"][0], -0.72);
    assert_eq!(state["readouts"][0]["value"], 2.);
}
#[test]
fn caller_override_wins_over_authored_timeline() {
    let mut scene: serde_json::Value = serde_json::from_str(SCENE).unwrap();
    scene["timeline"] = serde_json::json!([{"time":0,"control":"angle","value":30}]);
    let mut model = Model::load(scene.to_string().as_bytes()).unwrap();
    model.set_control("angle", 20.).unwrap();
    let state = model.state(3.).unwrap();
    assert_eq!(state["controls"]["angle"], 20.);
    assert_eq!(state["parts"][0]["rotation"][1], 40.);
    assert_eq!(state["readouts"][0]["value"], 40.);
}
#[test]
fn native_matches_web_golden_state_traces() {
    fn same(a: &serde_json::Value, b: &serde_json::Value) {
        match (a, b) {
            (serde_json::Value::Number(a), serde_json::Value::Number(b)) => {
                assert!((a.as_f64().unwrap() - b.as_f64().unwrap()).abs() < 1e-10)
            }
            (serde_json::Value::Array(a), serde_json::Value::Array(b)) => {
                assert_eq!(a.len(), b.len());
                for (a, b) in a.iter().zip(b) {
                    same(a, b);
                }
            }
            (serde_json::Value::Object(a), serde_json::Value::Object(b)) => {
                assert_eq!(a.len(), b.len());
                for (k, a) in a {
                    same(a, &b[k]);
                }
            }
            _ => assert_eq!(a, b),
        }
    }
    let traces: serde_json::Value =
        serde_json::from_slice(include_bytes!("fixtures/web-state-traces.json")).unwrap();
    for trace in traces.as_array().unwrap() {
        let bytes: &[u8] = match trace["fixture"].as_str().unwrap() {
            "representative" => include_bytes!("fixtures/representative.json"),
            "vector" => include_bytes!("fixtures/vector.json"),
            "effect" => include_bytes!("fixtures/effect.json"),
            "depth" => include_bytes!("fixtures/depth.json"),
            _ => panic!("unknown fixture"),
        };
        let mut model = Model::load(bytes).unwrap();
        if let Some(inputs) = trace["inputs"].as_object() {
            for (id, value) in inputs {
                model.set_control(id, value.as_f64().unwrap()).unwrap();
            }
        }
        let state = model.state(trace["time"].as_f64().unwrap()).unwrap();
        for field in ["title", "controls", "readouts", "parts"] {
            same(&state[field], &trace["state"][field]);
        }
    }
}
