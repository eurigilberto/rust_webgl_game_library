use std::rc::Rc;

use glam::*;
use rust_webgl2::{GlTexture2D, Graphics, Renderbuffer, Texture2DProps, TextureInternalFormat, FramebufferBinding};

pub fn is_depth(format: TextureInternalFormat) -> bool {
    match format {
        TextureInternalFormat::DEPTH24_STENCIL8
        | TextureInternalFormat::DEPTH_COMPONENT16
        | TextureInternalFormat::DEPTH_COMPONENT24
        | TextureInternalFormat::DEPTH_COMPONENT32F => true,
        _ => false,
    }
}

#[derive(Clone, Copy)]
pub enum FramebufferKind {
    Texture2D { properties: Texture2DProps },
    Renderbuffer { sample_count: u32 },
}

fn create_and_bind_attachment(
    graphics: &Graphics,
    size: UVec2,
    kind: FramebufferKind,
    format: TextureInternalFormat,
    framebuffer: &rust_webgl2::Framebuffer,
    attachment: rust_webgl2::FramebufferAttachment,
) -> Result<FramebufferAttachment, ()> {
    match kind {
        FramebufferKind::Texture2D { properties } => {
            let texture = GlTexture2D::new(graphics, properties, size, format, None)?;
            framebuffer.set_attachment_texture2d(attachment, Some(&texture));
            Ok(FramebufferAttachment::Texture2D(Rc::new(texture)))
        }
        FramebufferKind::Renderbuffer { sample_count } => {
            let renderbuffer = Renderbuffer::new(graphics, sample_count, size, format)?;
            framebuffer.set_attachment_renderbuffer(attachment, Some(&renderbuffer));
            Ok(FramebufferAttachment::Renderbuffer(Rc::new(renderbuffer)))
        }
    }
}

pub enum FramebufferAttachment {
    Renderbuffer(Rc<Renderbuffer>),
    Texture2D(Rc<GlTexture2D>),
}

pub struct Framebuffer {
    pub size: UVec2,
    pub kind: FramebufferKind,
    pub framebuffer: rust_webgl2::Framebuffer,
    pub color: Vec<FramebufferAttachment>,
    pub depth: Option<FramebufferAttachment>,
}

impl Framebuffer {
    pub fn new(graphics: &Graphics, size: UVec2, kind: FramebufferKind) -> Self {
        let framebuffer =
            rust_webgl2::Framebuffer::new(graphics).expect("Could not create Framebuffer");
        Self {
            size,
            kind,
            framebuffer: framebuffer,
            color: Vec::new(),
            depth: None,
        }
    }

    pub fn bind(&self, target: FramebufferBinding){
        self.framebuffer.bind(target);
        if target == FramebufferBinding::DRAW_FRAMEBUFFER{
            let mut buffers = Vec::new();
            for i in 0..self.color.len(){
                buffers.push(i as u32);
            }
            self.framebuffer.set_draw_buffers(buffers);
        }
    }
    
    pub fn create_color_texture(
        &mut self,
        graphics: &Graphics,
        format: TextureInternalFormat,
    ) -> Result<(), ()> {
        if is_depth(format) {
            panic!("Invalid format | It is not color")
        }

        let attachment = rust_webgl2::FramebufferAttachment::Color(self.color.len() as u32);
        let buffer = create_and_bind_attachment(
            graphics,
            self.size,
            self.kind,
            format,
            &self.framebuffer,
            attachment,
        )?;

        self.color.push(buffer);
        Ok(())
    }

    pub fn create_depth_texture(
        &mut self,
        graphics: &Graphics,
        format: TextureInternalFormat,
    ) -> Result<(), ()> {
        if !is_depth(format) {
            panic!("Invalid format | It is not depth")
        }

        let attachment = match format {
            TextureInternalFormat::DEPTH24_STENCIL8 => {
                rust_webgl2::FramebufferAttachment::DepthStencil
            }
            _ => rust_webgl2::FramebufferAttachment::Depth,
        };

        let buffer = create_and_bind_attachment(
            graphics,
            self.size,
            self.kind,
            format,
            &self.framebuffer,
            attachment,
        )?;
        self.depth = Some(buffer);
        Ok(())
    }
}
