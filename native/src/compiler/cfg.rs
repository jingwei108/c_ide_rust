//! Control-flow graph (CFG) construction and analysis (P3).
//!
//! Builds a CFG from a function AST, then provides loop detection and
//! dominance-tree computation to enhance algorithm detection accuracy.

use crate::compiler::ast::{Expr, FuncDecl, SourceLoc, Stmt};
use std::collections::{HashMap, HashSet};

/// A unique identifier for a basic block.
pub type BlockId = usize;

/// A single basic block: maximal sequence of statements with a single entry
/// and a single exit.
#[derive(Debug, Clone)]
pub struct BasicBlock {
    pub id: BlockId,
    pub stmts: Vec<Stmt>,
    pub terminator: Terminator,
}

/// The exit condition of a basic block.
#[derive(Debug, Clone)]
pub enum Terminator {
    Return,
    /// Unconditional jump to another block.
    Goto(BlockId),
    /// Conditional branch.
    Branch {
        cond: Expr,
        then_block: BlockId,
        else_block: BlockId,
    },
    /// Fall-through to the next block in sequential order.
    FallThrough(BlockId),
}

/// A control-flow graph for a single function.
#[derive(Debug, Clone)]
pub struct ControlFlowGraph {
    pub entry: BlockId,
    pub blocks: Vec<BasicBlock>,
    /// Directed edges (from, to).
    pub edges: Vec<(BlockId, BlockId)>,
}

impl ControlFlowGraph {
    /// Build a CFG from a function declaration.
    pub fn from_func(func: &FuncDecl) -> Option<Self> {
        let body = func.body.as_ref()?;
        let mut builder = CfgBuilder::new();
        let entry = builder.build_block(body);
        Some(builder.finish(entry))
    }

    /// Find all blocks that are unreachable from the entry.
    pub fn find_unreachable_blocks(&self) -> Vec<BlockId> {
        let mut reachable = HashSet::new();
        let mut stack = vec![self.entry];
        while let Some(id) = stack.pop() {
            if reachable.insert(id) {
                for &(from, to) in &self.edges {
                    if from == id {
                        stack.push(to);
                    }
                }
            }
        }
        self.blocks.iter().map(|b| b.id).filter(|id| !reachable.contains(id)).collect()
    }

    /// Identify natural loops in the CFG.
    /// Returns a list of (header_block, body_blocks).
    pub fn find_loops(&self) -> Vec<LoopInfo> {
        let mut loops = Vec::new();
        let mut backedges = Vec::new();

        // Find backedges using a simple DFS.
        let mut visited = HashSet::new();
        let mut path = HashSet::new();
        self.dfs_backedges(self.entry, &mut visited, &mut path, &mut backedges);

        for &(tail, header) in &backedges {
            let body = self.natural_loop_body(header, tail);
            loops.push(LoopInfo { header, body });
        }
        loops
    }

    fn dfs_backedges(
        &self,
        id: BlockId,
        visited: &mut HashSet<BlockId>,
        path: &mut HashSet<BlockId>,
        backedges: &mut Vec<(BlockId, BlockId)>,
    ) {
        if !visited.insert(id) {
            return;
        }
        path.insert(id);
        for &(from, to) in &self.edges {
            if from == id {
                if path.contains(&to) {
                    backedges.push((from, to));
                } else {
                    self.dfs_backedges(to, visited, path, backedges);
                }
            }
        }
        path.remove(&id);
    }

    fn natural_loop_body(&self, header: BlockId, tail: BlockId) -> HashSet<BlockId> {
        let mut body = HashSet::new();
        let mut stack = vec![tail];
        body.insert(header);
        body.insert(tail);
        while let Some(id) = stack.pop() {
            for &(from, to) in &self.edges {
                if to == id && !body.contains(&from) {
                    body.insert(from);
                    stack.push(from);
                }
            }
        }
        body
    }

    /// Compute the immediate dominator of each block.
    /// Returns map: block_id → idom_block_id (entry maps to itself).
    pub fn compute_dominators(&self) -> HashMap<BlockId, BlockId> {
        let mut dom = HashMap::new();
        let all_blocks: HashSet<BlockId> = self.blocks.iter().map(|b| b.id).collect();

        // Initialize: dom(entry) = {entry}, dom(others) = all blocks.
        for block in &self.blocks {
            if block.id == self.entry {
                dom.insert(block.id, HashSet::from([block.id]));
            } else {
                dom.insert(block.id, all_blocks.clone());
            }
        }

        // Iterative fixed-point.
        let mut changed = true;
        while changed {
            changed = false;
            for block in &self.blocks {
                if block.id == self.entry {
                    continue;
                }
                let preds: Vec<BlockId> = self
                    .edges
                    .iter()
                    .filter(|&&(_, to)| to == block.id)
                    .map(|&(from, _)| from)
                    .collect();
                if preds.is_empty() {
                    continue;
                }
                let mut new_dom = all_blocks.clone();
                for &p in &preds {
                    if let Some(p_dom) = dom.get(&p) {
                        new_dom = new_dom.intersection(p_dom).copied().collect();
                    }
                }
                new_dom.insert(block.id);
                if let Some(old) = dom.get(&block.id) {
                    if *old != new_dom {
                        changed = true;
                        dom.insert(block.id, new_dom);
                    }
                }
            }
        }

        // Convert to immediate dominators.
        let mut idom = HashMap::new();
        idom.insert(self.entry, self.entry);
        for block in &self.blocks {
            if block.id == self.entry {
                continue;
            }
            if let Some(doms) = dom.get(&block.id) {
                // idom is the unique element in dom(n) \ {n} that is dominated by all others.
                let mut candidates: Vec<BlockId> = doms.iter().copied().filter(|&d| d != block.id).collect();
                candidates.sort_by_key(|&d| dom.get(&d).map(|s| s.len()).unwrap_or(0));
                if let Some(&candidate) = candidates.last() {
                    idom.insert(block.id, candidate);
                }
            }
        }
        idom
    }
}

/// Information about a natural loop.
#[derive(Debug, Clone)]
pub struct LoopInfo {
    pub header: BlockId,
    pub body: HashSet<BlockId>,
}

// ===================================================================
// CFG Builder
// ===================================================================

struct CfgBuilder {
    blocks: Vec<BasicBlock>,
    edges: Vec<(BlockId, BlockId)>,
    next_id: BlockId,
}

impl CfgBuilder {
    fn new() -> Self {
        Self {
            blocks: Vec::new(),
            edges: Vec::new(),
            next_id: 0,
        }
    }

    fn alloc_id(&mut self) -> BlockId {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    fn add_block(&mut self, stmts: Vec<Stmt>, terminator: Terminator) -> BlockId {
        let id = self.alloc_id();
        self.blocks.push(BasicBlock { id, stmts, terminator });
        id
    }

    fn add_edge(&mut self, from: BlockId, to: BlockId) {
        self.edges.push((from, to));
    }

    fn finish(self, entry: BlockId) -> ControlFlowGraph {
        ControlFlowGraph {
            entry,
            blocks: self.blocks,
            edges: self.edges,
        }
    }

    /// Build a CFG fragment for a statement and return the entry block id.
    fn build_block(&mut self, stmt: &Stmt) -> BlockId {
        match stmt {
            Stmt::Block { stmts, .. } => self.build_seq(stmts),
            Stmt::If { cond, then_stmt, else_stmt, loc, .. } => {
                let then_entry = self.build_block(then_stmt);
                let else_entry = else_stmt.as_ref().map(|s| self.build_block(s)).unwrap_or_else(|| {
                    let id = self.alloc_id();
                    self.blocks.push(BasicBlock {
                        id,
                        stmts: vec![],
                        terminator: Terminator::FallThrough(id),
                    });
                    id
                });
                let merge = self.alloc_id();
                self.blocks.push(BasicBlock {
                    id: merge,
                    stmts: vec![],
                    terminator: Terminator::FallThrough(merge),
                });

                self.add_edge(then_entry, merge);
                self.add_edge(else_entry, merge);

                // B35: 条件块只需保留条件表达式与源位置，避免克隆整个 If AST 子树。
                let cond_stmt = Stmt::Expr {
                    expr: cond.clone(),
                    loc: *loc,
                };
                let cond_block = self.add_block(
                    vec![cond_stmt],
                    Terminator::Branch {
                        cond: cond.clone(),
                        then_block: then_entry,
                        else_block: else_entry,
                    },
                );
                self.add_edge(cond_block, then_entry);
                self.add_edge(cond_block, else_entry);
                cond_block
            }
            Stmt::While { cond, body, .. } => {
                let header = self.alloc_id();
                let body_entry = self.build_block(body);
                let exit = self.alloc_id();
                self.blocks.push(BasicBlock {
                    id: exit,
                    stmts: vec![],
                    terminator: Terminator::FallThrough(exit),
                });

                // Loop header contains the condition check.
                self.blocks.push(BasicBlock {
                    id: header,
                    stmts: vec![stmt.clone()],
                    terminator: Terminator::Branch {
                        cond: cond.clone(),
                        then_block: body_entry,
                        else_block: exit,
                    },
                });
                self.add_edge(header, body_entry);
                self.add_edge(header, exit);
                self.add_edge(body_entry, header);
                header
            }
            Stmt::DoWhile { body, cond, .. } => {
                let header = self.alloc_id();
                let body_entry = self.build_block(body);
                let exit = self.alloc_id();
                self.blocks.push(BasicBlock {
                    id: exit,
                    stmts: vec![],
                    terminator: Terminator::FallThrough(exit),
                });
                self.blocks.push(BasicBlock {
                    id: header,
                    stmts: vec![stmt.clone()],
                    terminator: Terminator::Branch {
                        cond: cond.clone(),
                        then_block: body_entry,
                        else_block: exit,
                    },
                });
                self.add_edge(body_entry, header);
                self.add_edge(header, body_entry);
                self.add_edge(header, exit);
                body_entry
            }
            Stmt::For { init, cond, body, .. } => {
                let init_block = init.as_ref().map(|s| self.build_block(s)).unwrap_or_else(|| {
                    let id = self.alloc_id();
                    self.blocks.push(BasicBlock {
                        id,
                        stmts: vec![],
                        terminator: Terminator::FallThrough(id),
                    });
                    id
                });
                let header = self.alloc_id();
                let body_entry = self.build_block(body);
                let exit = self.alloc_id();
                self.blocks.push(BasicBlock {
                    id: exit,
                    stmts: vec![],
                    terminator: Terminator::FallThrough(exit),
                });

                let cond_expr = cond.clone().unwrap_or(Expr::Literal {
                    value: 1,
                    loc: SourceLoc::default(),
                    ty: crate::compiler::ast::Type::int(),
                });
                self.blocks.push(BasicBlock {
                    id: header,
                    stmts: vec![stmt.clone()],
                    terminator: Terminator::Branch {
                        cond: cond_expr,
                        then_block: body_entry,
                        else_block: exit,
                    },
                });
                self.add_edge(init_block, header);
                self.add_edge(header, body_entry);
                self.add_edge(header, exit);
                self.add_edge(body_entry, header);
                init_block
            }
            Stmt::Switch { body, .. } => {
                let body_entry = self.build_block(body);
                let exit = self.alloc_id();
                self.blocks.push(BasicBlock {
                    id: exit,
                    stmts: vec![],
                    terminator: Terminator::FallThrough(exit),
                });
                let switch_block = self.add_block(vec![stmt.clone()], Terminator::Goto(body_entry));
                self.add_edge(switch_block, body_entry);
                self.add_edge(body_entry, exit);
                switch_block
            }
            Stmt::Return { .. } => self.add_block(vec![stmt.clone()], Terminator::Return),
            _ => {
                let next = self.alloc_id();
                self.add_block(vec![stmt.clone()], Terminator::FallThrough(next))
            }
        }
    }

    fn build_seq(&mut self, stmts: &[Stmt]) -> BlockId {
        if stmts.is_empty() {
            let id = self.alloc_id();
            self.blocks.push(BasicBlock {
                id,
                stmts: vec![],
                terminator: Terminator::FallThrough(id),
            });
            return id;
        }

        let mut entries = Vec::new();
        for stmt in stmts {
            entries.push(self.build_block(stmt));
        }

        // Sequential fall-through: connect each block to the next.
        for i in 0..entries.len().saturating_sub(1) {
            let from = entries[i];
            let to = entries[i + 1];
            // Only add fall-through edge if the block doesn't already have a terminator
            // that resolves to a different target.
            let needs_edge = if let Some(block) = self.blocks.iter().find(|b| b.id == from) {
                // B36: Return 块不应再向后 fall-through；仅 FallThrough 需要连接下一块。
                matches!(block.terminator, Terminator::FallThrough(_))
            } else {
                false
            };
            if needs_edge {
                self.add_edge(from, to);
            }
        }

        entries[0]
    }
}

// ===================================================================
// Unit tests
// ===================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_func(source: &str) -> FuncDecl {
        let (tokens, _) = crate::compiler::lexer::Lexer::new(source).tokenize();
        let (program, _) = crate::compiler::parser::Parser::new(tokens).parse();
        program.unwrap().funcs.into_iter().next().unwrap()
    }

    #[test]
    fn test_cfg_simple_function() {
        let func = parse_func("int main() { return 0; }");
        let cfg = ControlFlowGraph::from_func(&func).unwrap();
        assert_eq!(cfg.entry, 0);
        assert!(!cfg.blocks.is_empty());
    }

    #[test]
    fn test_cfg_if_statement() {
        let func = parse_func("int main() { if (1) { return 1; } else { return 0; } }");
        let cfg = ControlFlowGraph::from_func(&func).unwrap();
        let branches: Vec<_> = cfg
            .blocks
            .iter()
            .filter(|b| matches!(b.terminator, Terminator::Branch { .. }))
            .collect();
        assert!(!branches.is_empty());
    }

    #[test]
    fn test_cfg_while_loop() {
        let func = parse_func("int main() { int i = 0; while (i < 3) { i = i + 1; } return i; }");
        let cfg = ControlFlowGraph::from_func(&func).unwrap();
        let loops = cfg.find_loops();
        assert!(!loops.is_empty(), "Should detect at least one loop");
    }

    #[test]
    fn test_cfg_for_loop() {
        let func = parse_func("int main() { int s = 0; for (int i = 0; i < 5; i = i + 1) { s = s + i; } return s; }");
        let cfg = ControlFlowGraph::from_func(&func).unwrap();
        let loops = cfg.find_loops();
        assert!(!loops.is_empty(), "Should detect for-loop");
    }

    #[test]
    fn test_dominators() {
        let func = parse_func("int main() { int x = 1; if (x) { return 1; } return 0; }");
        let cfg = ControlFlowGraph::from_func(&func).unwrap();
        let dom = cfg.compute_dominators();
        assert!(dom.contains_key(&cfg.entry));
    }

    #[test]
    fn test_cfg_if_cond_block_does_not_clone_whole_if() {
        // B35: If 条件块的 stmts 应只保留条件表达式，而不是克隆整个 If AST 子树。
        let func = parse_func("int main() { int x = 1; if (x) { return 1; } else { return 0; } }");
        let cfg = ControlFlowGraph::from_func(&func).unwrap();
        let cond_blocks: Vec<_> = cfg
            .blocks
            .iter()
            .filter(|b| matches!(b.terminator, Terminator::Branch { .. }))
            .collect();
        assert_eq!(cond_blocks.len(), 1);
        let cond_block = cond_blocks[0];
        assert_eq!(cond_block.stmts.len(), 1, "条件块应只包含一个占位语句");
        assert!(
            !matches!(cond_block.stmts[0], Stmt::If { .. }),
            "条件块不应包含完整的 If 语句"
        );
        assert!(
            matches!(cond_block.stmts[0], Stmt::Expr { .. }),
            "条件块应只包含 Expr 占位语句"
        );
    }

    #[test]
    fn test_cfg_return_no_fall_through_edge() {
        // B36: return 终结的块不应再向后 fall-through。
        let func = parse_func("int main() { return 1; int x = 2; return x; }");
        let cfg = ControlFlowGraph::from_func(&func).unwrap();
        let return_blocks: Vec<_> = cfg
            .blocks
            .iter()
            .filter(|b| matches!(b.terminator, Terminator::Return))
            .collect();
        assert!(!return_blocks.is_empty(), "应存在 Return 终结的块");
        for rb in &return_blocks {
            let has_outgoing = cfg.edges.iter().any(|(from, _)| *from == rb.id);
            assert!(
                !has_outgoing,
                "Return 块 {} 不应有任何出边",
                rb.id
            );
        }
    }
}
