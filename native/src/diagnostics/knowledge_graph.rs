//! Knowledge graph: concept network for C language (P2).
//!
//! Models discrete C-language knowledge points as a connected graph.
//! When a student encounters an error or browses code, the system dynamically
//! activates a relevant concept sub-graph and shows relationships.

use flutter_rust_bridge::frb;
use std::collections::{HashMap, HashSet};
use std::sync::LazyLock;

/// A concept node in the knowledge graph.
#[frb]
#[derive(Debug, Clone)]
pub struct ConceptNode {
    pub id: String,
    pub domain: String, // "Compile" | "Memory" | "ControlFlow"
    pub title: String,
    pub description: String,
    pub difficulty: i32, // 1-5
    pub related_card_ids: Vec<String>,
}

/// A directed edge between two concepts.
#[frb]
#[derive(Debug, Clone)]
pub struct ConceptEdge {
    pub from: String,
    pub to: String,
    pub relation: String, // "Prerequisite" | "LeadsTo" | "CommonMistake" | "UsedTogether" | "Contradicts"
    pub strength: f32,    // 0.0 ~ 1.0
}

/// An activated concept with its 1-hop neighbors.
#[frb]
#[derive(Debug, Clone)]
pub struct ActivatedConcept {
    pub node: ConceptNode,
    pub activated_by: String, // "Error" | "AST" | "Runtime"
    pub neighbors: Vec<NeighborConcept>,
}

/// A neighboring concept in the activated sub-graph.
#[frb]
#[derive(Debug, Clone)]
pub struct NeighborConcept {
    pub node: ConceptNode,
    pub relation: String,
    pub strength: f32,
    pub is_prerequisite: bool,
}

// ===================================================================
// Static data
// ===================================================================

static NODES: LazyLock<Vec<ConceptNode>> = LazyLock::new(|| {
    vec![
        // Compile domain
        ConceptNode {
            id: String::from("VarDecl"),
            domain: String::from("Compile"),
            title: String::from("变量声明"),
            description: String::from("在使用变量之前必须先声明其类型。C 语言不会自动创建变量。"),
            difficulty: 1,
            related_card_ids: vec![String::from("undeclared_var")],
        },
        ConceptNode {
            id: String::from("TypeSystem"),
            domain: String::from("Compile"),
            title: String::from("类型系统"),
            description: String::from("C 语言是静态类型语言，每个变量和表达式都有明确的类型。"),
            difficulty: 2,
            related_card_ids: vec![],
        },
        ConceptNode {
            id: String::from("ImplicitCast"),
            domain: String::from("Compile"),
            title: String::from("隐式转换"),
            description: String::from("编译器在某些情况下会自动转换类型，如 char → int、int → float。"),
            difficulty: 3,
            related_card_ids: vec![],
        },
        ConceptNode {
            id: String::from("PointerType"),
            domain: String::from("Compile"),
            title: String::from("指针类型"),
            description: String::from("指针存储的是内存地址，而不是数据本身。int* 和 int 是不同的类型。"),
            difficulty: 3,
            related_card_ids: vec![String::from("null_pointer")],
        },
        ConceptNode {
            id: String::from("ArithOp"),
            domain: String::from("Compile"),
            title: String::from("算术运算符"),
            description: String::from("+ - * / % 等运算符要求操作数是整数或浮点数。"),
            difficulty: 1,
            related_card_ids: vec![],
        },
        ConceptNode {
            id: String::from("LogicOp"),
            domain: String::from("Compile"),
            title: String::from("逻辑运算符"),
            description: String::from("&& || ! 用于条件判断，0 为假，非 0 为真。注意与位运算符 & | 区分。"),
            difficulty: 2,
            related_card_ids: vec![String::from("logic_vs_bitwise"), String::from("assignment_in_condition")],
        },
        ConceptNode {
            id: String::from("BitOp"),
            domain: String::from("Compile"),
            title: String::from("位运算符"),
            description: String::from("& | ^ ~ << >> 对二进制位进行操作，只能用于整数类型。"),
            difficulty: 3,
            related_card_ids: vec![String::from("logic_vs_bitwise")],
        },
        ConceptNode {
            id: String::from("Scope"),
            domain: String::from("Compile"),
            title: String::from("作用域"),
            description: String::from("变量只在声明它的代码块内有效。花括号定义了作用域边界。"),
            difficulty: 2,
            related_card_ids: vec![],
        },
        // Memory domain
        ConceptNode {
            id: String::from("StackMemory"),
            domain: String::from("Memory"),
            title: String::from("栈内存"),
            description: String::from("局部变量和函数参数存储在栈上，函数返回后自动释放。"),
            difficulty: 2,
            related_card_ids: vec![String::from("stack_overflow")],
        },
        ConceptNode {
            id: String::from("HeapMemory"),
            domain: String::from("Memory"),
            title: String::from("堆内存"),
            description: String::from("malloc/calloc/realloc 分配的内存位于堆上，必须手动 free 释放。"),
            difficulty: 3,
            related_card_ids: vec![String::from("use_after_free")],
        },
        ConceptNode {
            id: String::from("Pointer"),
            domain: String::from("Memory"),
            title: String::from("指针"),
            description: String::from("指针是存储内存地址的变量。所有指针的大小相同（通常为 4 或 8 字节）。"),
            difficulty: 3,
            related_card_ids: vec![String::from("null_pointer"), String::from("use_after_free")],
        },
        ConceptNode {
            id: String::from("AddressOf"),
            domain: String::from("Memory"),
            title: String::from("取地址 (&)"),
            description: String::from("& 运算符获取变量的内存地址，返回指针。scanf 需要传入地址。"),
            difficulty: 2,
            related_card_ids: vec![String::from("scanf_address")],
        },
        ConceptNode {
            id: String::from("Dereference"),
            domain: String::from("Memory"),
            title: String::from("解引用 (*)"),
            description: String::from("* 运算符访问指针指向的内存内容。对 NULL 指针解引用会导致崩溃。"),
            difficulty: 3,
            related_card_ids: vec![String::from("null_pointer")],
        },
        ConceptNode {
            id: String::from("PtrArithmetic"),
            domain: String::from("Memory"),
            title: String::from("指针算术"),
            description: String::from("指针加减整数时，步长自动按 pointee 类型大小缩放（如 int* 步长为 4）。"),
            difficulty: 4,
            related_card_ids: vec![],
        },
        ConceptNode {
            id: String::from("Array"),
            domain: String::from("Memory"),
            title: String::from("数组"),
            description: String::from("数组是相同类型元素的连续存储。大小在编译期确定（当前子集）。"),
            difficulty: 2,
            related_card_ids: vec![String::from("array_out_of_bounds")],
        },
        ConceptNode {
            id: String::from("ArrayDecay"),
            domain: String::from("Memory"),
            title: String::from("数组退化"),
            description: String::from("数组名在表达式中通常退化为指向首元素的指针，sizeof 行为会改变。"),
            difficulty: 4,
            related_card_ids: vec![],
        },
        ConceptNode {
            id: String::from("StructLayout"),
            domain: String::from("Memory"),
            title: String::from("结构体内存布局"),
            description: String::from("结构体成员按声明顺序存储，可能有内存对齐填充。"),
            difficulty: 3,
            related_card_ids: vec![String::from("struct_member")],
        },
        // ControlFlow domain
        ConceptNode {
            id: String::from("IfSwitch"),
            domain: String::from("ControlFlow"),
            title: String::from("条件分支"),
            description: String::from("if/switch 根据条件选择执行路径。条件表达式必须是整数或指针。"),
            difficulty: 1,
            related_card_ids: vec![String::from("assignment_in_condition")],
        },
        ConceptNode {
            id: String::from("ForLoop"),
            domain: String::from("ControlFlow"),
            title: String::from("for 循环"),
            description: String::from("for (init; cond; update) 适合已知循环次数的场景。"),
            difficulty: 2,
            related_card_ids: vec![],
        },
        ConceptNode {
            id: String::from("WhileLoop"),
            domain: String::from("ControlFlow"),
            title: String::from("while 循环"),
            description: String::from("while (cond) 在条件为真时重复执行，适合未知次数的循环。"),
            difficulty: 2,
            related_card_ids: vec![],
        },
        ConceptNode {
            id: String::from("BoundaryCondition"),
            domain: String::from("ControlFlow"),
            title: String::from("边界条件"),
            description: String::from("循环边界决定执行次数。大小为 N 的数组，有效索引是 0 ~ N-1。"),
            difficulty: 3,
            related_card_ids: vec![String::from("array_out_of_bounds")],
        },
        ConceptNode {
            id: String::from("FunctionCall"),
            domain: String::from("ControlFlow"),
            title: String::from("函数调用"),
            description: String::from("函数将代码组织为可复用的模块。调用前需要声明或定义。"),
            difficulty: 2,
            related_card_ids: vec![],
        },
        ConceptNode {
            id: String::from("ParameterPassing"),
            domain: String::from("ControlFlow"),
            title: String::from("参数传递"),
            description: String::from("C 语言使用值传递。函数内修改参数不会影响调用者（除非传指针）。"),
            difficulty: 3,
            related_card_ids: vec![],
        },
        ConceptNode {
            id: String::from("ReturnValue"),
            domain: String::from("ControlFlow"),
            title: String::from("返回值"),
            description: String::from("非 void 函数必须通过 return 返回对应类型的值，否则行为未定义。"),
            difficulty: 2,
            related_card_ids: vec![String::from("missing_return")],
        },
        ConceptNode {
            id: String::from("Recursion"),
            domain: String::from("ControlFlow"),
            title: String::from("递归"),
            description: String::from("函数调用自身。必须有终止条件，否则导致栈溢出。"),
            difficulty: 4,
            related_card_ids: vec![String::from("stack_overflow")],
        },
    ]
});

static EDGES: LazyLock<Vec<ConceptEdge>> = LazyLock::new(|| {
    vec![
        // Compile relationships
        ConceptEdge { from: String::from("VarDecl"), to: String::from("TypeSystem"), relation: String::from("Prerequisite"), strength: 0.9 },
        ConceptEdge { from: String::from("TypeSystem"), to: String::from("ImplicitCast"), relation: String::from("LeadsTo"), strength: 0.8 },
        ConceptEdge { from: String::from("TypeSystem"), to: String::from("PointerType"), relation: String::from("LeadsTo"), strength: 0.8 },
        ConceptEdge { from: String::from("LogicOp"), to: String::from("BitOp"), relation: String::from("CommonMistake"), strength: 0.9 },
        ConceptEdge { from: String::from("ArithOp"), to: String::from("PointerType"), relation: String::from("UsedTogether"), strength: 0.6 },
        ConceptEdge { from: String::from("Scope"), to: String::from("VarDecl"), relation: String::from("UsedTogether"), strength: 0.7 },
        // Memory relationships
        ConceptEdge { from: String::from("StackMemory"), to: String::from("HeapMemory"), relation: String::from("Contradicts"), strength: 0.8 },
        ConceptEdge { from: String::from("Pointer"), to: String::from("AddressOf"), relation: String::from("LeadsTo"), strength: 0.9 },
        ConceptEdge { from: String::from("Pointer"), to: String::from("Dereference"), relation: String::from("LeadsTo"), strength: 0.9 },
        ConceptEdge { from: String::from("Pointer"), to: String::from("PtrArithmetic"), relation: String::from("LeadsTo"), strength: 0.8 },
        ConceptEdge { from: String::from("Array"), to: String::from("Pointer"), relation: String::from("LeadsTo"), strength: 0.9 },
        ConceptEdge { from: String::from("Array"), to: String::from("ArrayDecay"), relation: String::from("LeadsTo"), strength: 0.8 },
        ConceptEdge { from: String::from("ArrayDecay"), to: String::from("PointerType"), relation: String::from("CommonMistake"), strength: 0.7 },
        ConceptEdge { from: String::from("HeapMemory"), to: String::from("Pointer"), relation: String::from("UsedTogether"), strength: 0.9 },
        ConceptEdge { from: String::from("StructLayout"), to: String::from("Pointer"), relation: String::from("UsedTogether"), strength: 0.6 },
        // ControlFlow relationships
        ConceptEdge { from: String::from("ForLoop"), to: String::from("BoundaryCondition"), relation: String::from("LeadsTo"), strength: 0.9 },
        ConceptEdge { from: String::from("WhileLoop"), to: String::from("BoundaryCondition"), relation: String::from("LeadsTo"), strength: 0.9 },
        ConceptEdge { from: String::from("IfSwitch"), to: String::from("LogicOp"), relation: String::from("UsedTogether"), strength: 0.9 },
        ConceptEdge { from: String::from("FunctionCall"), to: String::from("ParameterPassing"), relation: String::from("LeadsTo"), strength: 0.9 },
        ConceptEdge { from: String::from("FunctionCall"), to: String::from("ReturnValue"), relation: String::from("LeadsTo"), strength: 0.8 },
        ConceptEdge { from: String::from("Recursion"), to: String::from("FunctionCall"), relation: String::from("Prerequisite"), strength: 0.9 },
        ConceptEdge { from: String::from("Recursion"), to: String::from("BoundaryCondition"), relation: String::from("UsedTogether"), strength: 0.9 },
        // Cross-domain relationships
        ConceptEdge { from: String::from("BoundaryCondition"), to: String::from("Array"), relation: String::from("UsedTogether"), strength: 0.9 },
        ConceptEdge { from: String::from("PointerType"), to: String::from("Pointer"), relation: String::from("LeadsTo"), strength: 0.9 },
        ConceptEdge { from: String::from("ParameterPassing"), to: String::from("Pointer"), relation: String::from("UsedTogether"), strength: 0.7 },
    ]
});

// ===================================================================
// Error-code → concept mapping
// ===================================================================

static ERROR_CONCEPT_MAP: LazyLock<HashMap<i32, Vec<String>>> = LazyLock::new(|| {
    let mut m = HashMap::new();
    m.insert(3023, vec![String::from("VarDecl")]);
    m.insert(3004, vec![String::from("TypeSystem"), String::from("ImplicitCast")]);
    m.insert(3053, vec![String::from("ImplicitCast")]);
    m.insert(3054, vec![String::from("PointerType")]);
    m.insert(1004, vec![String::from("LogicOp"), String::from("BitOp")]);
    m.insert(3050, vec![String::from("LogicOp")]);
    m.insert(3048, vec![String::from("BitOp")]);
    m.insert(3021, vec![String::from("Pointer"), String::from("Dereference")]);
    m.insert(3035, vec![String::from("Pointer"), String::from("Dereference")]);
    m.insert(3060, vec![String::from("HeapMemory"), String::from("Pointer")]);
    m.insert(3061, vec![String::from("HeapMemory"), String::from("Pointer")]);
    m.insert(3051, vec![String::from("BoundaryCondition"), String::from("Array")]);
    m.insert(3052, vec![String::from("ArrayDecay"), String::from("Array")]);
    m.insert(3041, vec![String::from("StructLayout")]);
    m.insert(3042, vec![String::from("StructLayout")]);
    m.insert(3015, vec![String::from("IfSwitch"), String::from("LogicOp")]);
    m.insert(3010, vec![String::from("ForLoop"), String::from("WhileLoop")]);
    m.insert(3011, vec![String::from("ForLoop"), String::from("WhileLoop")]);
    m.insert(3036, vec![String::from("FunctionCall")]);
    m.insert(3037, vec![String::from("FunctionCall"), String::from("ParameterPassing")]);
    m.insert(3038, vec![String::from("FunctionCall"), String::from("ParameterPassing")]);
    m.insert(3013, vec![String::from("ReturnValue")]);
    m.insert(3020, vec![String::from("Recursion")]);
    m.insert(3030, vec![String::from("ArithOp")]);
    m.insert(3031, vec![String::from("ArithOp")]);
    m.insert(3032, vec![String::from("ArithOp")]);
    m.insert(3033, vec![String::from("ArithOp")]);
    m.insert(3034, vec![String::from("ArithOp")]);
    m.insert(3035, vec![String::from("ArithOp")]);
    m
});

// ===================================================================
// KnowledgeGraph API
// ===================================================================

/// Activate concepts from an error code.
#[frb]
pub fn activate_from_error(error_code: i32) -> Vec<ActivatedConcept> {
    let mut result = Vec::new();
    let node_map: HashMap<String, &ConceptNode> = NODES.iter().map(|n| (n.id.clone(), n)).collect();

    if let Some(concept_ids) = ERROR_CONCEPT_MAP.get(&error_code) {
        for cid in concept_ids {
            if let Some(node) = node_map.get(cid) {
                let neighbors = collect_neighbors(cid, &node_map);
                result.push(ActivatedConcept {
                    node: (*node).clone(),
                    activated_by: String::from("Error"),
                    neighbors,
                });
            }
        }
    }
    result
}

/// Activate concepts from a list of AST feature keywords.
#[frb]
pub fn activate_from_ast(features: Vec<String>) -> Vec<ActivatedConcept> {
    let mut result = Vec::new();
    let node_map: HashMap<String, &ConceptNode> = NODES.iter().map(|n| (n.id.clone(), n)).collect();
    let mut activated_ids = HashSet::new();

    for feature in &features {
        let matched = match feature.as_str() {
            "malloc" | "free" | "calloc" | "realloc" => vec![String::from("HeapMemory"), String::from("Pointer")],
            "array_index" | "arr[" | "a[" => vec![String::from("Array"), String::from("BoundaryCondition")],
            "pointer_deref" | "*p" => vec![String::from("Pointer"), String::from("Dereference")],
            "for_loop" => vec![String::from("ForLoop"), String::from("BoundaryCondition")],
            "while_loop" => vec![String::from("WhileLoop"), String::from("BoundaryCondition")],
            "if_statement" => vec![String::from("IfSwitch"), String::from("LogicOp")],
            "function_call" => vec![String::from("FunctionCall"), String::from("ParameterPassing")],
            "recursive_call" => vec![String::from("Recursion"), String::from("BoundaryCondition")],
            "struct_member" | "." | "->" => vec![String::from("StructLayout")],
            "scanf" => vec![String::from("AddressOf")],
            "printf" => vec![String::from("ArithOp")],
            _ => continue,
        };
        for cid in matched {
            if activated_ids.insert(cid.clone()) {
                if let Some(node) = node_map.get(&cid) {
                    let neighbors = collect_neighbors(&cid, &node_map);
                    result.push(ActivatedConcept {
                        node: (*node).clone(),
                        activated_by: String::from("AST"),
                        neighbors,
                    });
                }
            }
        }
    }
    result
}

/// Find the prerequisite learning path from basic concepts to the target.
#[frb]
pub fn find_prerequisite_path(target_id: String) -> Vec<ConceptNode> {
    let node_map: HashMap<String, &ConceptNode> = NODES.iter().map(|n| (n.id.clone(), n)).collect();
    let mut visited = HashSet::new();
    let mut path = Vec::new();

    fn dfs(
        id: &str,
        edges: &[ConceptEdge],
        node_map: &HashMap<String, &ConceptNode>,
        visited: &mut HashSet<String>,
        path: &mut Vec<ConceptNode>,
    ) {
        if visited.contains(id) {
            return;
        }
        visited.insert(id.to_string());
        for edge in edges {
            if edge.to == id && (edge.relation == "Prerequisite" || edge.relation == "LeadsTo") {
                dfs(&edge.from, edges, node_map, visited, path);
            }
        }
        if let Some(node) = node_map.get(id) {
            path.push((*node).clone());
        }
    }

    dfs(&target_id, &EDGES, &node_map, &mut visited, &mut path);
    path
}

/// Get all concept nodes (for rendering the full graph).
#[frb]
pub fn get_all_concept_nodes() -> Vec<ConceptNode> {
    NODES.clone()
}

/// Get all concept edges (for rendering the full graph).
#[frb]
pub fn get_all_concept_edges() -> Vec<ConceptEdge> {
    EDGES.clone()
}

// ===================================================================
// Helpers
// ===================================================================

fn collect_neighbors(
    node_id: &str,
    node_map: &HashMap<String, &ConceptNode>,
) -> Vec<NeighborConcept> {
    let mut neighbors = Vec::new();
    for edge in EDGES.iter() {
        let (other_id, is_prereq) = if edge.from == node_id {
            (edge.to.clone(), false)
        } else if edge.to == node_id {
            (edge.from.clone(), edge.relation == "Prerequisite")
        } else {
            continue;
        };
        if let Some(other) = node_map.get(&other_id) {
            neighbors.push(NeighborConcept {
                node: (*other).clone(),
                relation: edge.relation.clone(),
                strength: edge.strength,
                is_prerequisite: is_prereq,
            });
        }
    }
    neighbors
}

// ===================================================================
// Unit tests
// ===================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_activate_from_error_bounds() {
        let activated = activate_from_error(3051);
        assert!(!activated.is_empty());
        let ids: Vec<_> = activated.iter().map(|a| a.node.id.clone()).collect();
        assert!(ids.contains(&String::from("BoundaryCondition")));
    }

    #[test]
    fn test_activate_from_ast_malloc() {
        let activated = activate_from_ast(vec![String::from("malloc"), String::from("free")]);
        let ids: Vec<_> = activated.iter().map(|a| a.node.id.clone()).collect();
        assert!(ids.contains(&String::from("HeapMemory")));
        assert!(ids.contains(&String::from("Pointer")));
    }

    #[test]
    fn test_prerequisite_path() {
        let path = find_prerequisite_path(String::from("ImplicitCast"));
        let ids: Vec<_> = path.iter().map(|n| n.id.clone()).collect();
        assert!(ids.contains(&String::from("VarDecl")));
        assert!(ids.contains(&String::from("TypeSystem")));
        assert!(ids.contains(&String::from("ImplicitCast")));
    }

    #[test]
    fn test_get_all_nodes() {
        let nodes = get_all_concept_nodes();
        assert!(nodes.len() >= 20);
    }
}
