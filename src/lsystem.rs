use std::collections::HashMap;

use building_blocks::mesh::PosNormMesh;
use cgmath::{Deg, EuclideanSpace, InnerSpace, Point3, Quaternion, Rotation, Rotation3, Vector3, Zero};

use crate::transform::{Transform, TransformExtensions};

// Constant symbols
// F    Move forward some distance, drawing a line.
// f    Move forward some distance without drawing a line.
// +    Turn by some angle.
// &    Pitch by some angle.
// /    Roll by some angle.
// |    Turn around.
// [    Push the current state of the turtle onto the stack.
// ]    Pop a state from the stack and make it the current state of the turtle.
// {    Begin a polygon.
// }    End a polygon.
// .    Emit a vertex (only valid inside {}).
// G    Same as f, but for use inside {}.
type LSymbol = char;

#[derive(Clone, Debug)]
pub struct LModule(LSymbol, Vec<f32>);

macro_rules! symbol {
    ($sym:expr) => {
        LModule($sym, Vec::new())
    };
    ($sym:expr, $($param:expr),+) => {
        LModule($sym, vec![$($param),+])
    };
}

pub type LString = Vec<LModule>;

struct LRule {
    condition: Option<Box<dyn Fn(&[f32]) -> bool>>,
    transformation: Box<dyn Fn(&[f32], &mut LString)>,
}

pub struct LSystem {
    rules: HashMap<LSymbol, LRule>,
    string: LString,
}

impl LSystem {
    pub fn new() -> LSystem {
        LSystem {
            rules: HashMap::new(),
            string: LString::new(),
        }
    }
    pub fn add_rule<T>(&mut self, predecessor: LSymbol, transformation: T) where T: Fn(&[f32], &mut LString) + 'static {
        self.rules.insert(predecessor, LRule { condition: None, transformation: Box::new(transformation) });
    }
    pub fn add_conditional_rule<C, T>(&mut self, predecessor: LSymbol, condition: C, transformation: T) where C: Fn(&[f32]) -> bool + 'static, T: Fn(&[f32], &mut LString) + 'static {
        self.rules.insert(predecessor, LRule { condition: Some(Box::new(condition)), transformation: Box::new(transformation) });
    }

    pub fn start(&mut self, axiom: LString) {
        self.string = axiom;
    }
    pub fn step(&mut self) {
        assert!(!self.string.is_empty());
        let prev_string = self.string.split_off(0);
        for module in prev_string {
            if let Some(rule) = self.rules.get(&module.0) {
                if let Some(condition) = &rule.condition {
                    if !condition(&module.1) {
                        //self.string.push(module);
                        continue;
                    }
                }
                (rule.transformation)(&module.1, &mut self.string);
            } else {
                self.string.push(module);
            }
        }
    }
    pub fn step_by(&mut self, iterations: usize) {
        for _ in 0..iterations {
            self.step();
        }
    }

    pub fn current_string(&self) -> &LString {
        &self.string
    }
}

pub fn test_mesh() -> PosNormMesh {
    // const THETA: f32 = 10.0;
    // const STEP: f32 = 0.1;
    // let mut lsystem = LSystem::new();
    // lsystem.add_rule('A', |_, out| out.extend([
    //     symbol!('['), symbol!('+', THETA), symbol!('A'), symbol!('{'), symbol!('.'), symbol!(']'), symbol!('.'), symbol!('C'), symbol!('.'), symbol!('}')
    // ]));
    // lsystem.add_rule('B', |_, out| out.extend([
    //     symbol!('['), symbol!('+', -THETA), symbol!('B'), symbol!('{'), symbol!('.'), symbol!(']'), symbol!('.'), symbol!('C'), symbol!('.'), symbol!('}')
    // ]));
    // lsystem.add_rule('C', |_, out| out.extend([
    //     symbol!('G', STEP), symbol!('C')
    // ]));
    // lsystem.start(vec![
    //     symbol!('['), symbol!('A'), symbol!(']'), symbol!('['), symbol!('B'), symbol!(']')
    // ]);

    const THETA: f32 = 60.0;
    const LA: f32 = 5.0 / 10.0;
    const RA: f32 = 1.1;
    const LB: f32 = 1.5 / 10.0;
    const RB: f32 = 1.2;
    const PD: f32 = 1.0;
    let mut lsystem = LSystem::new();
    lsystem.add_rule('A', |params, out| out.extend([
        symbol!('G', LA, RA), symbol!('['), symbol!('+', -THETA), symbol!('B', params[0]), symbol!('.'), symbol!(']'),
        symbol!('['), symbol!('A', params[0] + 1.0), symbol!(']'), symbol!('['), symbol!('+', THETA), symbol!('B', params[0]), symbol!('.'), symbol!(']'),
    ]));
    lsystem.add_conditional_rule('B', |params| params[0] > 0.0, |params, out| out.extend([
        symbol!('G', LB, RB), symbol!('B', params[0] - PD)
    ]));
    lsystem.add_rule('G', |params, out| out.push(symbol!('G', params[0] * params[1], params[1])));
    lsystem.start(vec![
        symbol!('{'), symbol!('.'), symbol!('A', 0.0), symbol!('}')
    ]);

    lsystem.step_by(20);
    TurtleInterpreter::make_mesh(lsystem.current_string())
}

struct TurtleInterpreter {
    turtle: Transform,
    stack: Vec<Transform>,
    current_polygon: Option<Vec<Point3<f32>>>,
    last_polygon_normal: Vector3<f32>,
    mesh: PosNormMesh,
}

impl TurtleInterpreter {
    fn make_mesh(string: &LString) -> PosNormMesh {
        let mut interpreter = TurtleInterpreter {
            turtle: Transform::from_rotation(Quaternion::look_at(Vector3::unit_y(), -Vector3::unit_z())),
            stack: Vec::new(),
            current_polygon: None,
            last_polygon_normal: Vector3::zero(),
            mesh: PosNormMesh::default(),
        };
        for module in string.iter() {
            match module.0 {
                'F' => unimplemented!(),
                'f' => unimplemented!(),
                '+' => interpreter.rotate_turtle(Vector3::unit_y(), Deg(module.1[0])),
                '&' => interpreter.rotate_turtle(Vector3::unit_x(), Deg(module.1[0])),
                '/' => interpreter.rotate_turtle(Vector3::unit_z(), Deg(module.1[0])),
                '|' => interpreter.rotate_turtle(Vector3::unit_y(), Deg(180.0)),
                '[' => interpreter.stack.push(interpreter.turtle),
                ']' => interpreter.turtle = interpreter.stack.pop().expect("mismatched ']'"),
                '{' => interpreter.start_polygon(),
                '}' => interpreter.end_polygon(),
                '.' => interpreter.add_polygon_vertex(),
                'G' => interpreter.move_turtle(module.1[0]),
                _ => (),
            }
        }
        interpreter.mesh
    }

    fn move_turtle(&mut self, distance: f32) {
        self.turtle.disp += self.turtle.rot * Vector3::new(0.0, 0.0, distance);
    }
    fn rotate_turtle(&mut self, axis: Vector3<f32>, angle: Deg<f32>) {
        self.turtle.rot = self.turtle.rot * Quaternion::from_axis_angle(axis, angle);
    }

    fn start_polygon(&mut self) {
        assert!(self.current_polygon.is_none(), "mismatched '{{'");
        self.current_polygon = Some(Vec::new());
    }
    fn end_polygon(&mut self) {
        assert!(self.current_polygon.is_some(), "mismatched '}}'");
        let mut polygon = self.current_polygon.take().unwrap();
        polygon.dedup();
        if polygon.len() >= 3 {
            self.add_triangle_fan(polygon);
        }
    }
    fn add_polygon_vertex(&mut self) {
        assert!(self.current_polygon.is_some(), "'.' outside of {{ }}");
        self.current_polygon.as_mut().unwrap().push(Point3::from_vec(self.turtle.disp));
    }

    fn surface_normal(vertices: &[Point3<f32>]) -> Vector3<f32> {
        let mut normal = Vector3::zero();
        for i in 0..vertices.len() {
            let current = vertices[i];
            let next = vertices[(i + 1) % vertices.len()];
            normal.x += (current.y - next.y) * (current.z + next.z);
            normal.y += (current.z - next.z) * (current.x + next.x);
            normal.z += (current.x - next.x) * (current.y + next.y);
        }
        normal.normalize()
    }
    fn add_triangle_fan(&mut self, vertices: Vec<Point3<f32>>) {
        fn triangulate_face(indices: &mut Vec<u32>, face_indices: std::ops::Range<usize>) {
            let start = face_indices.start as u32;
            for i in 2..(face_indices.len() as u32) {
                indices.push(start + 0);
                indices.push(start + i - 1);
                indices.push(start + i);
            }
        }

        let mut normal = TurtleInterpreter::surface_normal(&vertices);
        // Ensure all normals are roughly in the same direction, so faces with opposite winding can have correct normals.
        // TODO would be better to enforce consistent winding.
        if normal.dot(self.last_polygon_normal) < 0.0 {
            normal = -normal;
        }
        self.last_polygon_normal = normal;

        let start_index = self.mesh.positions.len();
        self.mesh.positions.extend(vertices.iter().map(|v| Into::<[f32; 3]>::into(*v)));
        self.mesh.normals.extend(std::iter::repeat(Into::<[f32; 3]>::into(normal)).take(vertices.len()));
        triangulate_face(&mut self.mesh.indices, start_index..self.mesh.positions.len());
    }
}
