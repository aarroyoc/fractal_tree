//#![windows_subsystem = "windows"] // compilar con MSVC para más rendimiento
extern crate glutin;
#[macro_use]
extern crate gfx;
extern crate gfx_device_gl;
extern crate gfx_window_glutin;
extern crate lyon;
extern crate winit;
extern crate rand;
extern crate palette;

use lyon::path::Path;
use lyon::path_builder::*;
use lyon::math::{point,Point};
use lyon::tessellation::{FillVertex,StrokeTessellator,StrokeOptions,StrokeVertex};
use lyon::tessellation::geometry_builder::{VertexConstructor, VertexBuffers, BuffersBuilder};
//use lyon::tessellation::basic_shapes::{stroke_circle,fill_quad};

use glutin::GlContext;
use std::f32;
use rand::Rng;
use palette::{Rgb,Hsl,RgbHue};
use palette::FromColor;

use gfx::traits::{Device, FactoryExt};

type ColorFormat = gfx::format::Rgba8;
type DepthFormat = gfx::format::DepthStencil;

gfx_defines!{
    vertex GpuVertex {
        position: [f32; 2] = "a_position",
        color: [f32; 3] = "in_color",
    }

    pipeline fill_pipeline {
        vbo: gfx::VertexBuffer<GpuVertex> = (),
        out_color: gfx::RenderTarget<ColorFormat> = "out_color",
    }
}

struct VertexCtor{
    color: [f32;3],
}
impl VertexConstructor<StrokeVertex, GpuVertex> for VertexCtor {
    fn new_vertex(&mut self, vertex: StrokeVertex) -> GpuVertex {
        GpuVertex {
            // (ugly hack) tweak the vertext position so that the logo fits roughly
            // within the (-1.0, 1.0) range.
            position: vertex.position.to_array(),
            color: self.color,
        }
    }
}
impl VertexConstructor<FillVertex, GpuVertex> for VertexCtor {
    fn new_vertex(&mut self, vertex: FillVertex) -> GpuVertex {
        GpuVertex {
            // (ugly hack) tweak the vertext position so that the logo fits roughly
            // within the (-1.0, 1.0) range.
            position: vertex.position.to_array(),
            color: self.color,
        }
    }
}

/*fn draw_line(mut mesh: &mut VertexBuffers<GpuVertex>,from: Point, to: Point, width: f32, color: [f32;3]) {
    let recto = f32::consts::FRAC_PI_2.sin();
    if from.x < to.x {
        fill_quad(from,to,point(to.x*recto,to.y*recto),point(from.x*recto,from.y*recto),&mut BuffersBuilder::new(&mut mesh, VertexCtor{color: color}));
    }else{
        fill_quad(from,to,point(to.x*recto,to.y*recto),point(from.x*recto,from.y*recto),&mut BuffersBuilder::new(&mut mesh, VertexCtor{color: color}));
    }
}*/

fn draw_line(mut mesh: &mut VertexBuffers<GpuVertex>,from: Point, to: Point, width: f32, color: &Hsl){
    let mut builder = SvgPathBuilder::new(Path::builder());
    builder.move_to(from);
    builder.line_to(to);
    let path = builder.build();
    let mut tes = StrokeTessellator::new();
    let stroke_options = StrokeOptions::default().with_line_width(width).with_tolerance(0.00001);
    let c = {
        let a = Rgb::from_hsl(color.clone());
        [a.red,a.green,a.blue]
    };
    tes.tessellate_path(path.path_iter(),&stroke_options,&mut BuffersBuilder::new(&mut mesh, VertexCtor{color: c}));
}

fn draw_branch(mut mesh: &mut VertexBuffers<GpuVertex>, x: f32, y: f32, len: f32, angle: f32,width: f32, c: &Hsl) {
    let to = point(x+len*angle.sin(),y+len*angle.cos());
    draw_line(&mut mesh,point(x,y),to,0.01,&c);

    if len < 0.01{
        return;
    }

    let factor = rand::thread_rng().gen_range(0.6,0.9);
    let new_angle = rand::thread_rng().gen_range(0.0,0.52);
    let color = {
        let mut a = c.clone();
        a.lightness = c.lightness*1.35;
        a
    };
    draw_branch(&mut mesh,to.x,to.y,len*factor,angle-new_angle,width*factor,&color);
    assert!(width > width*factor);
    let factor = rand::thread_rng().gen_range(0.5,0.9);
    let new_angle = rand::thread_rng().gen_range(0.0,0.52);
    let color = {
        let mut a = c.clone();
        a.lightness = c.lightness*1.35;
        a
    };
    draw_branch(&mut mesh,to.x,to.y,len*factor,angle+new_angle,width*factor,&color);
}

fn main() {
    println!("Fractal tree");
    
    /* Lyon */
    let mut builder = SvgPathBuilder::new(Path::builder());
    builder.move_to(point(-0.5,0.));
    builder.horizontal_line_to(0.5);
    //builder.close();

    //let path = builder.build();

    //let mut tessellator = StrokeTessellator::new();

    let mut mesh = VertexBuffers::new();

    //let stroke_options = StrokeOptions::default().with_line_width(0.1).with_tolerance(0.000000001);

    //tessellator.tessellate_path(path.path_iter(),&stroke_options,&mut BuffersBuilder::new(&mut mesh, VertexCtor{color: [0.0,0.0,1.0]}));
    //stroke_circle(point(0.0,0.0),0.2,&stroke_options,&mut BuffersBuilder::new(&mut mesh, VertexCtor{color: [0.0,1.0,0.0]}));
    //draw_line_beta(&mut mesh, point(0.0,0.0), point(0.5,0.5),0.01,[1.0,0.0,0.0]);
    let colors = [63.0,1.0,21.0,112.0,176.0,240.0,277.0,323.0];
    let hue = rand::thread_rng().gen_range(0,colors.len());
    let mut color = Hsl::new(RgbHue::from(colors[hue]),1.0,0.01);
    draw_branch(&mut mesh,0.0,-1.0,0.3,0.0,30.0,&mut color);

    println!(" -- fill: {} vertices {} indices", mesh.vertices.len(), mesh.indices.len());
    //println!("{:?}",mesh.vertices);

    /* GLUTIN */
    let mut events_loop = winit::EventsLoop::new();
    let glutin_builder = winit::WindowBuilder::new()
        .with_dimensions(700,700)
        .with_decorations(true)
        .with_title("Fractal trees".to_string());
    let context = glutin::ContextBuilder::new().with_vsync(true).with_multisampling(8);
    
    let (window, mut device, mut factory, mut main_fbo, mut main_depth) =
        gfx_window_glutin::init::<ColorFormat, DepthFormat>(glutin_builder,context,&events_loop);
    
    let shader = factory.link_program(
        VERTEX_SHADER.as_bytes(),
        FRAGMENT_SHADER.as_bytes()
    ).unwrap();

    let pso = factory.create_pipeline_from_program(
        &shader,
        gfx::Primitive::TriangleList,
        gfx::state::Rasterizer::new_fill(),
        fill_pipeline::new(),
    ).unwrap();

    let (vbo, ibo) = factory.create_vertex_buffer_with_slice(
        &mesh.vertices[..],
        &mesh.indices[..]
    );

    let mut cmd: gfx::Encoder<_, _> = factory.create_command_buffer().into();

    let mut running = true;
    while running{
        events_loop.poll_events(|event|{
            match event {
                winit::Event::WindowEvent{event,..} => match event {
                    winit::WindowEvent::Closed => running = false,
                    _ => ()
                },
                _ => ()
            }
        });
        gfx_window_glutin::update_views(&window,&mut main_fbo,&mut main_depth);

        cmd.clear(&main_fbo.clone(),[0.63,0.86,1.0,1.0]);
        cmd.draw(
            &ibo,
            &pso,
            &fill_pipeline::Data {
                vbo: vbo.clone(),
                out_color: main_fbo.clone(),
            },
        );
        cmd.flush(&mut device);

        window.swap_buffers().unwrap();

        device.cleanup();
    }
    

}

pub static VERTEX_SHADER: &'static str = &"
    #version 140
    #line 266

    in vec2 a_position;
    in vec3 in_color;

    out vec4 v_color;

    void main() {
        gl_Position = vec4(a_position, 0.0, 1.0);
        v_color = vec4(in_color, 1.0);
    }
";

// The fragment shader is dead simple. It just applies the color computed in the vertex shader.
// A more advanced renderer would probably compute texture coordinates in the vertex shader and
// sample the color from a texture here.
pub static FRAGMENT_SHADER: &'static str = &"
    #version 140
    in vec4 v_color;
    out vec4 out_color;

    void main() {
        out_color = v_color;
    }
";
