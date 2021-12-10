use cgmath::{Deg, EuclideanSpace, InnerSpace, Rotation, Rotation3, Zero};

use crate::code::{VariableScope, VariableMap, Evaluable};
use crate::transform::{Transform, TransformExtensions, Quaternion, Point3f, Vector3f};
use crate::syntax::lsystem::*;
use crate::triangle_draw::TriangleMesh;

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

pub struct LSymbol {
    symbol: char,
    params: Vec<f64>,
}

impl LSymbol {
    pub fn new(symbol: char) -> LSymbol {
        LSymbol { symbol, params: Vec::new() }
    }
    pub fn new_params(symbol: char, params: Vec<f64>) -> LSymbol {
        LSymbol { symbol, params }
    }
}

impl std::fmt::Display for LSymbol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.symbol)?;
        if !self.params.is_empty() {
            write!(f, "(")?;
            VecFormatter::write(f, &self.params, ", ")?;
            write!(f, ")")?;
        }
        Ok(())
    }
}

struct VecFormatter;

impl VecFormatter {
    fn write<T: std::fmt::Display>(f: &mut std::fmt::Formatter<'_>, vec: &Vec<T>, sep: &str) -> std::fmt::Result {
        let mut first = true;
        for item in vec.iter() {
            if first {
                first = false;
            } else {
                write!(f, "{}", sep)?;
            }
            write!(f, "{}", item)?;
        }
        Ok(())
    }
}

pub struct LString(Vec<LSymbol>);

impl LString {
    fn new() -> LString {
        LString(Vec::new())
    }
    pub fn iter(&self) -> std::slice::Iter<'_, LSymbol> {
        self.0.iter()
    }
}

impl std::fmt::Display for LString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        VecFormatter::write(f, &self.0, " ")
    }
}

pub struct LSystem {
    system: System,
    string: LString,
}

impl LSystem {
    pub fn new(system: System) -> LSystem {
        LSystem { system, string: LString::new() }
    }

    fn create_local_variable_map(module: &LSymbol, production: &Production) -> VariableMap {
        let mut map = VariableMap::new();
        if let Some(param_names) = production.predecessor.params.as_ref() {
            let len = module.params.len().min(param_names.len());
            for i in 0..len {
                map.insert(param_names[i].clone(), module.params[i]);
            }
        }
        map
    }

    pub fn start(&mut self) {
        self.string = LString(vec![LSymbol::new('0')]);
        self.step();
    }
    pub fn step(&mut self) {
        assert!(!self.string.0.is_empty());
        let const_scope = VariableScope::new(&self.system.constants);
        let prev_string = self.string.0.split_off(0);
        for module in prev_string {
            let mut found_production = false;
            for production in self.system.productions.iter() {
                if module.symbol != production.predecessor.symbol { continue; }
                found_production = true;
                let local_variables = LSystem::create_local_variable_map(&module, production);
                let local_scope = const_scope.inner_scope(&local_variables);
                if production.conditions.evaluate(local_scope) {
                    for add_module in production.successor.iter() {
                        self.string.0.push(add_module.evaluate(local_scope));
                    }
                    break;
                }
            }
            if !found_production {
                self.string.0.push(module);
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

pub fn test_mesh() -> TriangleMesh {
    let lsystem_source = include_str!("../input/system.txt");
    let mut lsystem = match crate::syntax::parse_string(system(), lsystem_source) {
        Ok(system) => LSystem::new(system),
        Err(error) => {
            println!("{}", error);
            panic!("Failed to parse LSystem")
        }
    };  
    lsystem.start();
    lsystem.step_by(20);
    TurtleInterpreter::make_mesh(lsystem.current_string())
}

struct TurtleInterpreter {
    turtle: Transform,
    stack: Vec<Transform>,
    current_polygon: Option<Vec<Point3f>>,
    last_polygon_normal: Vector3f,
    mesh: TriangleMesh,
}

impl TurtleInterpreter {
    fn make_mesh(string: &LString) -> TriangleMesh {
        let mut interpreter = TurtleInterpreter {
            turtle: Transform::from_rotation(Quaternion::look_at(Vector3f::unit_y(), -Vector3f::unit_z())),
            stack: Vec::new(),
            current_polygon: None,
            last_polygon_normal: Vector3f::zero(),
            mesh: TriangleMesh::default(),
        };
        for module in string.iter() {
            match module.symbol {
                'F' => unimplemented!(),
                'f' => unimplemented!(),
                '+' => interpreter.rotate_turtle(Vector3f::unit_y(), Deg(module.params[0] as f32)),
                '-' => interpreter.rotate_turtle(-Vector3f::unit_y(), Deg(module.params[0] as f32)),
                '&' => interpreter.rotate_turtle(Vector3f::unit_x(), Deg(module.params[0] as f32)),
                '/' => interpreter.rotate_turtle(Vector3f::unit_z(), Deg(module.params[0] as f32)),
                '|' => interpreter.rotate_turtle(Vector3f::unit_y(), Deg(180.0)),
                '[' => interpreter.stack.push(interpreter.turtle),
                ']' => interpreter.turtle = interpreter.stack.pop().expect("mismatched ']'"),
                '{' => interpreter.start_polygon(),
                '}' => interpreter.end_polygon(),
                '.' => interpreter.add_polygon_vertex(),
                'G' => interpreter.move_turtle(module.params[0] as f32),
                _ => (),
            }
        }
        interpreter.mesh
    }

    fn move_turtle(&mut self, distance: f32) {
        self.turtle.disp += self.turtle.rot * Vector3f::new(0.0, 0.0, distance);
    }
    fn rotate_turtle(&mut self, axis: Vector3f, angle: Deg<f32>) {
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
        self.current_polygon.as_mut().unwrap().push(Point3f::from_vec(self.turtle.disp));
    }

    fn surface_normal(vertices: &[Point3f]) -> Vector3f {
        let mut normal = Vector3f::zero();
        for i in 0..vertices.len() {
            let current = vertices[i];
            let next = vertices[(i + 1) % vertices.len()];
            normal.x += (current.y - next.y) * (current.z + next.z);
            normal.y += (current.z - next.z) * (current.x + next.x);
            normal.z += (current.x - next.x) * (current.y + next.y);
        }
        normal.normalize()
    }
    fn add_triangle_fan(&mut self, vertices: Vec<Point3f>) {
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
