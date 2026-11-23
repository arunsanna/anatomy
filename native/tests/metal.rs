use anatomy_native::*;
use objc2_metal::*;
use std::{
    ffi::{c_void, CStr},
    ptr::NonNull,
};
fn ptr<T: ?Sized>(v: &T) -> *mut c_void {
    v as *const T as *const () as *mut c_void
}
#[test]
fn metal_draws_depth_solids_and_changes_with_orbit() {
    objc2::rc::autoreleasepool(|_| unsafe {
        let device =
            MTLCreateSystemDefaultDevice().expect("Metal device required for native GPU gate");
        let queue = device.newCommandQueue().unwrap();
        let renderer = anatomy_renderer_create(ptr(&*device));
        assert!(!renderer.is_null(), "native Metal pipeline creation failed");
        let scene=br#"{"schemaVersion":1,"title":"GPU proof","background":[0,0,0,1],"parts":[{"id":"box","kind":"box","position":[-1,0,0],"size":[1,1,1],"color":[1,0,0,1]},{"id":"sphere","kind":"sphere","color":[0,1,0,1],"effect":"glow"},{"id":"cylinder","kind":"cylinder","position":[1,0,0],"color":[0,0,1,1]}]}"#;
        assert_eq!(
            anatomy_renderer_load_scene(renderer, scene.as_ptr(), scene.len()),
            1
        );
        let d = MTLTextureDescriptor::new();
        d.setTextureType(MTLTextureType::Type2D);
        d.setWidth(128);
        d.setHeight(128);
        d.setPixelFormat(MTLPixelFormat::BGRA8Unorm);
        d.setStorageMode(MTLStorageMode::Shared);
        d.setUsage(MTLTextureUsage::RenderTarget);
        let texture = device.newTextureWithDescriptor(&d).unwrap();
        let draw = |yaw, pitch| {
            let command = queue.commandBuffer().unwrap();
            assert_eq!(
                anatomy_renderer_encode(
                    renderer,
                    ptr(&*command),
                    ptr(&*texture),
                    0.,
                    yaw,
                    pitch,
                    6.
                ),
                1,
                "{}",
                CStr::from_ptr(anatomy_renderer_last_error(renderer)).to_string_lossy()
            );
            command.commit();
            command.waitUntilCompleted();
            assert_eq!(command.status(), MTLCommandBufferStatus::Completed);
            let mut bytes = vec![0u8; 128 * 128 * 4];
            texture.getBytes_bytesPerRow_fromRegion_mipmapLevel(
                NonNull::new(bytes.as_mut_ptr().cast()).unwrap(),
                128 * 4,
                MTLRegion {
                    origin: MTLOrigin { x: 0, y: 0, z: 0 },
                    size: MTLSize {
                        width: 128,
                        height: 128,
                        depth: 1,
                    },
                },
                0,
            );
            bytes
        };
        let front = draw(0., 25.);
        let orbit = draw(55., 25.);
        let red = front
            .chunks_exact(4)
            .filter(|p| p[2] > 80 && p[1] < 20 && p[0] < 20)
            .count();
        let green = front
            .chunks_exact(4)
            .filter(|p| p[1] > 80 && p[2] < 20 && p[0] < 20)
            .count();
        let blue = front
            .chunks_exact(4)
            .filter(|p| p[0] > 80 && p[1] < 20 && p[2] < 20)
            .count();
        assert!(
            red > 20 && green > 20 && blue > 20,
            "Missing GPU solids: red={red} green={green} blue={blue}"
        );
        assert_ne!(front, orbit, "Orbit must change GPU pixels");
        println!("METAL PASS: BGRA8 pixels red={red} green={green} blue={blue}; command completed; orbit differs");
        // Far geometry emitted last must not cover the near box.
        let depth_scene=br#"{"schemaVersion":1,"title":"Depth proof","background":[0,0,0,1],"parts":[{"id":"near","kind":"box","position":[0,0,0.8],"size":[2,2,1],"color":[1,0,0,1]},{"id":"far","kind":"box","position":[0,0,-0.8],"size":[2,2,1],"color":[0,1,0,1]}]}"#;
        assert_eq!(
            anatomy_renderer_load_scene(renderer, depth_scene.as_ptr(), depth_scene.len()),
            1
        );
        let depth = draw(0., 0.);
        let center = &depth[(64 * 128 + 64) * 4..][..4];
        assert!(
            center[2] > 80 && center[1] < 20,
            "Depth did not retain nearest box: {center:?}"
        );
        let glow_scene=br#"{"schemaVersion":1,"title":"Glow proof","background":[0,0,0,1],"parts":[{"id":"near","kind":"box","color":[0.3,0,0,1]}],"controls":[{"id":"power","label":"Power","min":0,"max":2,"step":1,"value":0}],"bindings":[{"part":"near","control":"power","property":"glow"}]}"#;
        assert_eq!(
            anatomy_renderer_load_scene(renderer, glow_scene.as_ptr(), glow_scene.len()),
            1
        );
        let dull = draw(0., 0.);
        assert_eq!(
            anatomy_renderer_set_control(renderer, c"power".as_ptr(), 2.),
            1
        );
        let glow = draw(0., 0.);
        assert!(
            glow[(64 * 128 + 64) * 4 + 2] > dull[(64 * 128 + 64) * 4 + 2],
            "Glow binding did not brighten material"
        );
        d.setPixelFormat(MTLPixelFormat::BGRA8Unorm_sRGB);
        let srgb = device.newTextureWithDescriptor(&d).unwrap();
        let command = queue.commandBuffer().unwrap();
        assert_eq!(
            anatomy_renderer_encode(renderer, ptr(&*command), ptr(&*srgb), 0., 0., 0., 6.),
            1
        );
        command.commit();
        command.waitUntilCompleted();
        assert_eq!(command.status(), MTLCommandBufferStatus::Completed);
        println!("METAL PASS: near box occludes later far box; control glow brightens pixels; sRGB pipeline completes");
        anatomy_renderer_destroy(renderer);
    });
}
#[test]
fn differently_posed_frames_keep_their_data_while_gpu_work_is_pending() {
    objc2::rc::autoreleasepool(|_| unsafe {
        let device = MTLCreateSystemDefaultDevice().unwrap();
        let queue = device.newCommandQueue().unwrap();
        let renderer = anatomy_renderer_create(ptr(&*device));
        assert!(!renderer.is_null());
        let scene=br#"{"schemaVersion":1,"title":"Pending pose proof","background":[0,0,0,1],"parts":[{"id":"box","kind":"box","size":[0.75,0.75,0.75],"color":[1,0,0,1]}],"controls":[{"id":"pose","label":"Pose","min":-1,"max":1,"step":1,"value":-1}],"bindings":[{"part":"box","control":"pose","property":"positionX"}]}"#;
        assert_eq!(
            anatomy_renderer_load_scene(renderer, scene.as_ptr(), scene.len()),
            1
        );
        let d = MTLTextureDescriptor::new();
        d.setTextureType(MTLTextureType::Type2D);
        d.setPixelFormat(MTLPixelFormat::BGRA8Unorm);
        d.setWidth(128);
        d.setHeight(128);
        d.setStorageMode(MTLStorageMode::Shared);
        d.setUsage(MTLTextureUsage::RenderTarget);
        let left = device.newTextureWithDescriptor(&d).unwrap();
        let right = device.newTextureWithDescriptor(&d).unwrap();
        let event = device.newSharedEvent().unwrap();
        let first = queue.commandBuffer().unwrap();
        first.encodeWaitForEvent_value(
            objc2::runtime::ProtocolObject::<dyn MTLEvent>::from_ref(&*event),
            1,
        );
        assert_eq!(
            anatomy_renderer_encode(renderer, ptr(&*first), ptr(&*left), 0., 0., 0., 6.),
            1
        );
        first.commit();
        assert_ne!(first.status(), MTLCommandBufferStatus::Completed);
        assert_eq!(
            anatomy_renderer_set_control(renderer, c"pose".as_ptr(), 1.),
            1
        );
        let second = queue.commandBuffer().unwrap();
        assert_eq!(
            anatomy_renderer_encode(renderer, ptr(&*second), ptr(&*right), 0., 0., 0., 6.),
            1
        );
        second.commit();
        event.setSignaledValue(1);
        first.waitUntilCompleted();
        second.waitUntilCompleted();
        assert_eq!(first.status(), MTLCommandBufferStatus::Completed);
        assert_eq!(second.status(), MTLCommandBufferStatus::Completed);
        let centroid = |texture: &objc2::runtime::ProtocolObject<dyn MTLTexture>| {
            let mut bytes = vec![0u8; 128 * 128 * 4];
            texture.getBytes_bytesPerRow_fromRegion_mipmapLevel(
                NonNull::new(bytes.as_mut_ptr().cast()).unwrap(),
                128 * 4,
                MTLRegion {
                    origin: MTLOrigin { x: 0, y: 0, z: 0 },
                    size: MTLSize {
                        width: 128,
                        height: 128,
                        depth: 1,
                    },
                },
                0,
            );
            let positions: Vec<_> = bytes
                .chunks_exact(4)
                .enumerate()
                .filter(|(_, p)| p[2] > 80 && p[1] < 20)
                .map(|(i, _)| (i % 128) as f64)
                .collect();
            assert!(!positions.is_empty());
            positions.iter().sum::<f64>() / positions.len() as f64
        };
        let left_x = centroid(&left);
        let right_x = centroid(&right);
        assert!(
            left_x < 64. && right_x > 64.,
            "Pending buffer overwritten: left={left_x}, right={right_x}"
        );
        anatomy_renderer_destroy(renderer);
        println!("METAL INFLIGHT PASS: event-blocked first frame x={left_x:.2}; second frame x={right_x:.2}");
    });
}
