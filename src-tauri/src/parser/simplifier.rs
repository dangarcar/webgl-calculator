use super::ast::Node;

pub fn simplify_tree(root: &mut Node) -> Option<f64> {
    match root {
        Node::Unknown {..} => None,
        Node::Constant {value} => Some(*value),
        Node::Unary { op_type, child } => {
            let child = child.as_mut().unwrap();
            if let Some(n) = simplify_tree(child) {
                let f = op_type.func().unwrap();
                let val = f(n);
                *root = Node::Constant { value: val };
                Some(val)
            } else { None }
        },
        Node::Binary { op_type, lhs, rhs } => {
            let lhs = simplify_tree(lhs.as_mut().unwrap());
            let rhs = simplify_tree(rhs.as_mut().unwrap());
            if lhs.is_some() && rhs.is_some() {
                let f = op_type.func().unwrap();
                let val = f(lhs.unwrap(), rhs.unwrap());
                *root = Node::Constant { value: val };
                Some(val)
            } else { 
                None
            }
        },
        Node::NAry { op_type, children } => {
            let cnst = children
                .iter_mut()
                .filter_map(|e| simplify_tree(e))
                .reduce(|acc, e| (op_type.func().unwrap())(acc, e));
            
            let mut new_children: Vec<Box<Node>> = children
                .into_iter()
                .filter_map(|e| { 
                    match simplify_tree(e) {
                        Some(..) => None,
                        None => Some(e.to_owned()),
                    }
                })
                .collect();

            if new_children.is_empty() {
                let x = cnst.unwrap();
                *root = Node::Constant { value: x };

                Some(x)
            } else {
                if let Some(x) = cnst {
                    new_children.push(Box::new(Node::Constant { value: x }));
                }

                if new_children.len() == 1 {
                    *root = *new_children.first().unwrap().to_owned()
                } else {
                    *root = Node::NAry { op_type: *op_type, children: new_children };
                }

                None
            }            
        },

        Node::Variable {..} => None,//TODO:
    }
}