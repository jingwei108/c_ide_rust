use super::*;

// ============================================================================
// BFS
// ============================================================================

pub(crate) fn infer_bfs(source_line: &str, vars: &VarMap, algorithm: &AlgorithmMatch) -> Option<AlgorithmStepSnapshot> {
    let line_lower = source_line.to_lowercase();
    let u = vars.get_int_any(&["u", "front", "start"]).unwrap_or(-1);
    let front = vars.get_int("front").unwrap_or(-1);
    let rear = vars.get_int("rear").unwrap_or(-1);

    if line_lower.starts_with("while ") {
        return Some(build_step(
            algorithm,
            "loop",
            &format!("队列非空，继续广度优先搜索 [front={}, rear={}]", front, rear),
        ));
    }

    if line_lower.contains("queue[front++]") || (line_lower.contains("front") && line_lower.contains("++")) {
        return Some(build_step(algorithm, "dequeue", &format!("出队节点 u={}", u)));
    }

    if line_lower.contains("queue[rear++]") || (line_lower.contains("rear") && line_lower.contains("++")) {
        return Some(build_step(algorithm, "enqueue", "邻居节点入队"));
    }

    if line_lower.contains("visited") && line_lower.contains('=') {
        return Some(build_step(algorithm, "visit", &format!("标记节点 {} 为已访问", u)));
    }

    if line_lower.starts_with("return") {
        return Some(build_step(algorithm, "finish", "BFS 遍历完成"));
    }

    None
}
// ============================================================================
// DFS
// ============================================================================

pub(crate) fn infer_dfs(
    source_line: &str,
    vars: &VarMap,
    algorithm: &AlgorithmMatch,
    func_name: &str,
) -> Option<AlgorithmStepSnapshot> {
    let line_lower = source_line.to_lowercase();
    let u = vars.get_int_any(&["u", "v", "start"]).unwrap_or(-1);

    if source_line.contains(&format!("{}(", func_name)) {
        return Some(build_step(
            algorithm,
            "recursive",
            &format!("递归深入：从节点 {} 继续深度优先搜索", u),
        ));
    }

    if line_lower.contains("visited") && line_lower.contains('=') {
        return Some(build_step(algorithm, "visit", &format!("标记节点 {} 为已访问", u)));
    }

    if line_lower.starts_with("for ") || line_lower.starts_with("while ") {
        return Some(build_step(algorithm, "scan", &format!("扫描节点 {} 的邻居", u)));
    }

    if line_lower.starts_with("return") {
        return Some(build_step(algorithm, "finish", "DFS 遍历完成"));
    }

    None
}
// ============================================================================
// Prim 最小生成树
// ============================================================================

pub(crate) fn infer_prim_mst(
    source_line: &str,
    vars: &VarMap,
    algorithm: &AlgorithmMatch,
) -> Option<AlgorithmStepSnapshot> {
    let line_lower = source_line.to_lowercase();
    let k = vars.get_int_any(&["k"]).unwrap_or(-1);

    if line_lower.contains("lowcost") && line_lower.contains("min") && line_lower.contains("inf") {
        return Some(build_step(algorithm, "select_min", &format!("选择最小边，顶点 k={}", k)));
    }

    if line_lower.contains("lowcost")
        && line_lower.contains('0')
        && line_lower.contains('=')
        && !line_lower.contains("!=")
    {
        return Some(build_step(algorithm, "add_vertex", &format!("顶点 {} 加入生成树", k)));
    }

    if line_lower.contains("g[k]") && line_lower.contains("lowcost") && line_lower.contains('<') {
        return Some(build_step(algorithm, "update", "更新邻接顶点的最小边权"));
    }

    None
}
// ============================================================================
// Kruskal 最小生成树
// ============================================================================

pub(crate) fn infer_kruskal_mst(
    source_line: &str,
    _vars: &VarMap,
    algorithm: &AlgorithmMatch,
) -> Option<AlgorithmStepSnapshot> {
    let line_lower = source_line.to_lowercase();

    if line_lower.contains("edges") && line_lower.contains("w") && line_lower.contains('<') {
        return Some(build_step(algorithm, "sort", "按边权排序"));
    }

    if line_lower.contains("find") && line_lower.contains("parent") && line_lower.contains("!=") {
        return Some(build_step(algorithm, "check_cycle", "并查集判环"));
    }

    if line_lower.contains("union") && line_lower.contains("parent") {
        return Some(build_step(algorithm, "add_edge", "加入生成树并合并集合"));
    }

    None
}
// ============================================================================
// Dijkstra 最短路径
// ============================================================================

pub(crate) fn infer_dijkstra(
    source_line: &str,
    vars: &VarMap,
    algorithm: &AlgorithmMatch,
) -> Option<AlgorithmStepSnapshot> {
    let line_lower = source_line.to_lowercase();
    let u = vars.get_int_any(&["u"]).unwrap_or(-1);

    if line_lower.contains("dist") && line_lower.contains("min") && line_lower.contains("inf") {
        return Some(build_step(algorithm, "select", &format!("选择距离最小的未确定顶点 u={}", u)));
    }

    if line_lower.contains("visited") && line_lower.contains('=') && line_lower.contains('1') {
        return Some(build_step(algorithm, "confirm", &format!("顶点 {} 的最短距离已确定", u)));
    }

    if line_lower.contains("dist[u]") && line_lower.contains("g[u]") && line_lower.contains('+') {
        return Some(build_step(algorithm, "relax", "松弛操作，更新邻接顶点距离"));
    }

    None
}
// ============================================================================
// Floyd 最短路径
// ============================================================================

pub(crate) fn infer_floyd(
    source_line: &str,
    vars: &VarMap,
    algorithm: &AlgorithmMatch,
) -> Option<AlgorithmStepSnapshot> {
    let line_lower = source_line.to_lowercase();
    let k = vars.get_int_any(&["k"]).unwrap_or(-1);

    if line_lower.starts_with("for ") && line_lower.contains('k') {
        return Some(build_step(algorithm, "outer_loop", &format!("枚举中间顶点 k={}", k)));
    }

    if line_lower.contains("g[i][k]") && line_lower.contains("g[k][j]") && line_lower.contains('<') {
        return Some(build_step(algorithm, "relax", "检查经 k 中转是否更短"));
    }

    None
}
// ============================================================================
// 拓扑排序
// ============================================================================

pub(crate) fn infer_topological_sort(
    source_line: &str,
    vars: &VarMap,
    algorithm: &AlgorithmMatch,
) -> Option<AlgorithmStepSnapshot> {
    let line_lower = source_line.to_lowercase();
    let u = vars.get_int_any(&["u"]).unwrap_or(-1);

    if line_lower.contains("indegree") && line_lower.contains("0") && line_lower.contains("queue") {
        return Some(build_step(algorithm, "enqueue", "入度为 0 的顶点入队"));
    }

    if line_lower.contains("queue[front++]") || (line_lower.contains("front") && line_lower.contains("u")) {
        return Some(build_step(algorithm, "output", &format!("输出顶点 {}", u)));
    }

    if line_lower.contains("indegree") && line_lower.contains("--") {
        return Some(build_step(algorithm, "decrease", "删边，邻接点入度减 1"));
    }

    None
}
