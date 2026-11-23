use crate::{geometry, model::Model};
use objc2::{rc::Retained, runtime::ProtocolObject};
use objc2_foundation::NSString;
use objc2_metal::*;
use std::ptr::NonNull;
pub struct Metal {
    device: Retained<ProtocolObject<dyn MTLDevice>>,
    pipeline: Retained<ProtocolObject<dyn MTLRenderPipelineState>>,
    srgb_pipeline: Retained<ProtocolObject<dyn MTLRenderPipelineState>>,
    depth_state: Retained<ProtocolObject<dyn MTLDepthStencilState>>,
    vertices: Vec<geometry::Vertex>,
    slots: Vec<FrameSlot>,
}
#[derive(Default)]
struct FrameSlot {
    command: Option<Retained<ProtocolObject<dyn MTLCommandBuffer>>>,
    buffer: Option<Retained<ProtocolObject<dyn MTLBuffer>>>,
    depth: Option<Retained<ProtocolObject<dyn MTLTexture>>>,
}
impl FrameSlot {
    fn available(&self) -> bool {
        self.command.as_ref().is_none_or(|command| {
            matches!(
                command.status(),
                MTLCommandBufferStatus::Completed | MTLCommandBufferStatus::Error
            )
        })
    }
    fn prepare(
        &mut self,
        device: &ProtocolObject<dyn MTLDevice>,
        bytes: usize,
        width: usize,
        height: usize,
    ) -> Result<(), String> {
        // This slot's previous command has retired before any resource mutation.
        self.command = None;
        if bytes > 0 && self.buffer.as_ref().is_none_or(|b| b.length() < bytes) {
            self.buffer = Some(
                device
                    .newBufferWithLength_options(bytes, MTLResourceOptions::StorageModeShared)
                    .ok_or("Could not allocate vertex buffer")?,
            );
        }
        if self
            .depth
            .as_ref()
            .is_none_or(|d| d.width() != width || d.height() != height)
        {
            let d = MTLTextureDescriptor::new();
            d.setTextureType(MTLTextureType::Type2D);
            d.setPixelFormat(MTLPixelFormat::Depth32Float);
            unsafe {
                d.setWidth(width);
                d.setHeight(height);
            }
            d.setStorageMode(MTLStorageMode::Private);
            d.setUsage(MTLTextureUsage::RenderTarget);
            self.depth = Some(
                device
                    .newTextureWithDescriptor(&d)
                    .ok_or("Could not allocate depth texture")?,
            );
        }
        Ok(())
    }
}
impl Metal {
    pub fn new(device: Retained<ProtocolObject<dyn MTLDevice>>) -> Result<Self, String> {
        let library = device
            .newLibraryWithSource_options_error(
                &NSString::from_str(include_str!("../shaders/solids.metal")),
                None,
            )
            .map_err(|e| format!("Metal shader compilation: {e}"))?;
        let vertex = library
            .newFunctionWithName(&NSString::from_str("solid_vertex"))
            .ok_or("Missing vertex function")?;
        let fragment = library
            .newFunctionWithName(&NSString::from_str("solid_fragment"))
            .ok_or("Missing fragment function")?;
        let pipeline_for = |format| -> Result<_, String> {
            let d = MTLRenderPipelineDescriptor::new();
            d.setVertexFunction(Some(&vertex));
            d.setFragmentFunction(Some(&fragment));
            let color = unsafe { d.colorAttachments().objectAtIndexedSubscript(0) };
            color.setPixelFormat(format);
            color.setBlendingEnabled(true);
            color.setSourceRGBBlendFactor(MTLBlendFactor::SourceAlpha);
            color.setDestinationRGBBlendFactor(MTLBlendFactor::OneMinusSourceAlpha);
            d.setDepthAttachmentPixelFormat(MTLPixelFormat::Depth32Float);
            device
                .newRenderPipelineStateWithDescriptor_error(&d)
                .map_err(|e| format!("Metal pipeline: {e}"))
        };
        let pipeline = pipeline_for(MTLPixelFormat::BGRA8Unorm)?;
        let srgb_pipeline = pipeline_for(MTLPixelFormat::BGRA8Unorm_sRGB)?;
        let d = MTLDepthStencilDescriptor::new();
        d.setDepthCompareFunction(MTLCompareFunction::Less);
        d.setDepthWriteEnabled(true);
        let depth_state = device
            .newDepthStencilStateWithDescriptor(&d)
            .ok_or("Could not create depth state")?;
        Ok(Self {
            device,
            pipeline,
            srgb_pipeline,
            depth_state,
            vertices: Vec::new(),
            slots: Vec::new(),
        })
    }
    pub fn encode(
        &mut self,
        model: &Model,
        command: &ProtocolObject<dyn MTLCommandBuffer>,
        texture: &ProtocolObject<dyn MTLTexture>,
        time: f64,
        yaw: f64,
        pitch: f64,
        distance: f64,
    ) -> Result<(), String> {
        if [yaw, pitch, distance].iter().any(|v| !v.is_finite()) || distance <= 0. {
            return Err("Camera must be finite with positive distance".into());
        }
        if texture.width() == 0
            || texture.height() == 0
            || texture.sampleCount() != 1
            || texture.textureType() != MTLTextureType::Type2D
            || !texture.usage().contains(MTLTextureUsage::RenderTarget)
        {
            return Err("Target must be a nonempty single-sample 2D render-target texture".into());
        }
        let pipeline = match texture.pixelFormat() {
            MTLPixelFormat::BGRA8Unorm => &self.pipeline,
            MTLPixelFormat::BGRA8Unorm_sRGB => &self.srgb_pipeline,
            _ => return Err("Target format must be BGRA8Unorm or BGRA8Unorm_sRGB".into()),
        };
        let (_, parts) = model.evaluated(time)?;
        geometry::vertices(&parts, &mut self.vertices);
        let bytes = std::mem::size_of_val(self.vertices.as_slice());
        // Uncommitted/queued/scheduled work is busy too: no guessed frame cap.
        let index = if let Some(index) = self.slots.iter().position(FrameSlot::available) {
            index
        } else {
            self.slots.push(FrameSlot::default());
            self.slots.len() - 1
        };
        let slot = &mut self.slots[index];
        slot.prepare(&self.device, bytes, texture.width(), texture.height())?;
        if bytes > 0 {
            unsafe {
                std::ptr::copy_nonoverlapping(
                    self.vertices.as_ptr().cast::<u8>(),
                    slot.buffer
                        .as_ref()
                        .unwrap()
                        .contents()
                        .as_ptr()
                        .cast::<u8>(),
                    bytes,
                );
            }
        }
        let pass = MTLRenderPassDescriptor::new();
        let color = unsafe { pass.colorAttachments().objectAtIndexedSubscript(0) };
        color.setTexture(Some(texture));
        color.setLoadAction(MTLLoadAction::Clear);
        color.setStoreAction(MTLStoreAction::Store);
        let b = model.scene.background;
        color.setClearColor(MTLClearColor {
            red: b[0],
            green: b[1],
            blue: b[2],
            alpha: b[3],
        });
        pass.depthAttachment().setTexture(slot.depth.as_deref());
        pass.depthAttachment().setLoadAction(MTLLoadAction::Clear);
        pass.depthAttachment()
            .setStoreAction(MTLStoreAction::DontCare);
        pass.depthAttachment().setClearDepth(1.);
        let encoder = command
            .renderCommandEncoderWithDescriptor(&pass)
            .ok_or("Could not create Metal render encoder")?;
        encoder.setRenderPipelineState(pipeline);
        encoder.setDepthStencilState(Some(&self.depth_state));
        encoder.setCullMode(MTLCullMode::None);
        if bytes > 0 {
            let buffer = slot.buffer.as_ref().unwrap();
            let camera = geometry::camera(
                yaw,
                pitch,
                distance,
                texture.width() as f64 / texture.height() as f64,
            );
            unsafe {
                encoder.setVertexBuffer_offset_atIndex(Some(buffer), 0, 0);
                encoder.setVertexBytes_length_atIndex(
                    NonNull::from(&camera).cast(),
                    std::mem::size_of_val(&camera),
                    1,
                );
                encoder.drawPrimitives_vertexStart_vertexCount(
                    MTLPrimitiveType::Triangle,
                    0,
                    self.vertices.len(),
                );
            }
        }
        encoder.endEncoding();
        // setVertexBytes copies uniforms into command-owned storage; vertex and
        // depth resources stay immutable until this retained command retires.
        slot.command = Some(unsafe {
            Retained::retain(command as *const _ as *mut ProtocolObject<dyn MTLCommandBuffer>)
                .unwrap()
        });
        Ok(())
    }
}
