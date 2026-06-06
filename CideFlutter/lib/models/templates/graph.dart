import '../code_template.dart';

const List<CodeTemplate> graphTemplates = [
    CodeTemplate(
      'bfs',
      'BFS 广度优先',
      '图算法',
      '#include <stdio.h>\n'
      '\n'
      'int graph[5][5] = {\n'
      '    {0, 1, 1, 0, 0},\n'
      '    {1, 0, 0, 1, 1},\n'
      '    {1, 0, 0, 0, 0},\n'
      '    {0, 1, 0, 0, 0},\n'
      '    {0, 1, 0, 0, 0}\n'
      '};\n'
      'int visited[5] = {0, 0, 0, 0, 0};\n'
      'int queue[5];\n'
      'int front = 0, rear = 0;\n'
      '\n'
      'void bfs(int start, int n) {\n'
      '    visited[start] = 1;\n'
      '    queue[rear++] = start;\n'
      '    while (front < rear) {\n'
      '        int u = queue[front++];\n'
      '        printf("%d ", u);\n'
      '        for (int v = 0; v < n; v++) {\n'
      '            if (graph[u][v] == 1 && visited[v] == 0) {\n'
      '                visited[v] = 1;\n'
      '                queue[rear++] = v;\n'
      '            }\n'
      '        }\n'
      '    }\n'
      '}\n'
      '\n'
      'int main() {\n'
      '    int n = {{n:5}};\n'
      '    bfs(0, n);\n'
      '    printf("\\n");\n'
      '    return 0;\n'
      '}',
      params: [
        TemplateParam(key: 'n', label: '节点数', defaultValue: '5', type: ParamType.int),
      ],
      tutorialSteps: [
        TutorialStep(
          title: '图与队列',
          description: 'BFS 使用队列实现。graph 是邻接矩阵，visited 标记已访问节点，queue 存储待访问节点。',
          focusLines: [3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13],
          explanations: [
            LineExplanation(line: 3, short: '邻接矩阵', detail: 'graph[i][j] = 1 表示节点 i 和 j 之间有边。'),
            LineExplanation(line: 10, short: '访问标记', detail: 'visited 数组防止节点被重复访问。'),
            LineExplanation(line: 11, short: '队列', detail: 'queue 存放按层序待访问的节点。'),
          ],
        ),
        TutorialStep(
          title: 'BFS 过程',
          description: '从起点出发，标记为已访问并入队。每次出队一个节点，访问其所有未访问的邻居并入队。',
          focusLines: [15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25],
          explanations: [
            LineExplanation(line: 15, short: '标记起点', detail: 'visited[start] = 1，标记起点已访问。'),
            LineExplanation(line: 16, short: '起点入队', detail: 'queue[rear++] = start，起点入队。'),
            LineExplanation(line: 17, short: '循环条件', detail: 'front < rear 表示队列非空。'),
            LineExplanation(line: 18, short: '出队', detail: 'u = queue[front++]，取出队头节点。'),
            LineExplanation(line: 20, short: '扫描邻居', detail: 'v 从 0 到 n-1 扫描所有可能邻居。'),
            LineExplanation(line: 21, short: '未访问邻居', detail: 'graph[u][v] == 1 且 visited[v] == 0 说明是未访问的邻居。'),
            LineExplanation(line: 22, short: '标记并入队', detail: '先标记 visited[v]，再入队，保证同一节点不会重复入队。'),
          ],
        ),
      ],
    ),

    CodeTemplate(
      'dfs',
      'DFS 深度优先',
      '图算法',
      '#include <stdio.h>\n'
      '\n'
      'int graph[5][5] = {\n'
      '    {0, 1, 1, 0, 0},\n'
      '    {1, 0, 0, 1, 1},\n'
      '    {1, 0, 0, 0, 0},\n'
      '    {0, 1, 0, 0, 0},\n'
      '    {0, 1, 0, 0, 0}\n'
      '};\n'
      'int visited[5] = {0, 0, 0, 0, 0};\n'
      '\n'
      'void dfs(int u, int n) {\n'
      '    visited[u] = 1;\n'
      '    printf("%d ", u);\n'
      '    for (int v = 0; v < n; v++) {\n'
      '        if (graph[u][v] == 1 && visited[v] == 0) {\n'
      '            dfs(v, n);\n'
      '        }\n'
      '    }\n'
      '}\n'
      '\n'
      'int main() {\n'
      '    int n = {{n:5}};\n'
      '    dfs(0, n);\n'
      '    printf("\\n");\n'
      '    return 0;\n'
      '}',
      params: [
        TemplateParam(key: 'n', label: '节点数', defaultValue: '5', type: ParamType.int),
      ],
      tutorialSteps: [
        TutorialStep(
          title: '图与访问标记',
          description: 'DFS 使用递归实现。graph 是邻接矩阵，visited 标记已访问节点，防止循环访问。',
          focusLines: [3, 4, 5, 6, 7, 8, 9, 10],
          explanations: [
            LineExplanation(line: 3, short: '邻接矩阵', detail: 'graph[i][j] = 1 表示节点 i 和 j 之间有边。'),
            LineExplanation(line: 10, short: '访问标记', detail: 'visited 数组防止节点被重复访问和无限递归。'),
          ],
        ),
        TutorialStep(
          title: 'DFS 过程',
          description: '从当前节点出发，标记为已访问并输出。然后对每个未访问的邻居递归调用 dfs，一条路走到黑再回溯。',
          focusLines: [12, 13, 14, 15, 16, 17, 18, 19, 20],
          explanations: [
            LineExplanation(line: 12, short: '标记访问', detail: 'visited[u] = 1，标记当前节点已访问。'),
            LineExplanation(line: 13, short: '输出节点', detail: '打印当前节点编号。'),
            LineExplanation(line: 14, short: '扫描邻居', detail: 'v 从 0 到 n-1 扫描所有可能邻居。'),
            LineExplanation(line: 15, short: '未访问邻居', detail: 'graph[u][v] == 1 且 visited[v] == 0 说明是未访问的邻居。'),
            LineExplanation(line: 16, short: '递归深入', detail: 'dfs(v, n) 递归访问邻居，直到没有未访问邻居才返回。'),
          ],
        ),
      ],
    ),

    CodeTemplate(
      'primMST',
      'Prim 最小生成树',
      '图算法',
      '#include <stdio.h>\n'
      '#define MAXV 5\n'
      '#define INF 65535\n'
      '\n'
      'void Prim(int G[][MAXV], int n) {\n'
      '    int lowcost[MAXV];\n'
      '    int adjvex[MAXV];\n'
      '    lowcost[0] = 0;\n'
      '    adjvex[0] = 0;\n'
      '    for (int i = 1; i < n; i++) {\n'
      '        lowcost[i] = G[0][i];\n'
      '        adjvex[i] = 0;\n'
      '    }\n'
      '    for (int i = 1; i < n; i++) {\n'
      '        int min = INF;\n'
      '        int k = 0;\n'
      '        for (int j = 1; j < n; j++) {\n'
      '            if (lowcost[j] != 0 && lowcost[j] < min) {\n'
      '                min = lowcost[j];\n'
      '                k = j;\n'
      '            }\n'
      '        }\n'
      '        printf("(%d,%d)=%d\\n", adjvex[k], k, min);\n'
      '        lowcost[k] = 0;\n'
      '        for (int j = 1; j < n; j++) {\n'
      '            if (lowcost[j] != 0 && G[k][j] < lowcost[j]) {\n'
      '                lowcost[j] = G[k][j];\n'
      '                adjvex[j] = k;\n'
      '            }\n'
      '        }\n'
      '    }\n'
      '}\n'
      '\n'
      'int main() {\n'
      '    int G[MAXV][MAXV] = {\n'
      '        {0, 2, INF, 6, INF},\n'
      '        {2, 0, 3, 8, 5},\n'
      '        {INF, 3, 0, INF, 7},\n'
      '        {6, 8, INF, 0, 9},\n'
      '        {INF, 5, 7, 9, 0}\n'
      '    };\n'
      '    Prim(G, MAXV);\n'
      '    return 0;\n'
      '}',
      params: [],
      tutorialSteps: [
        TutorialStep(
          title: 'Prim 算法思想',
          description: 'Prim 算法从某一顶点出发，每次选择连接"已选顶点集"和"未选顶点集"的最小权值边，逐步扩展生成树。',
          focusLines: [6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37],
          explanations: [
            LineExplanation(line: 7, short: 'lowcost', detail: 'lowcost[j] 记录顶点 j 到已选集合的最小边权。'),
            LineExplanation(line: 8, short: 'adjvex', detail: 'adjvex[j] 记录这条最小边来自哪个已选顶点。'),
            LineExplanation(line: 10, short: '初始化', detail: '从顶点 0 出发，lowcost[0]=0 表示已选。'),
            LineExplanation(line: 18, short: '选最小边', detail: '在 lowcost 非零的顶点中选最小值，即连接两集合的最小边。'),
            LineExplanation(line: 25, short: '输出边', detail: 'printf 输出选中的边 (adjvex[k], k)。'),
            LineExplanation(line: 26, short: '加入集合', detail: 'lowcost[k] = 0，顶点 k 加入已选集合。'),
            LineExplanation(line: 29, short: '更新 lowcost', detail: '检查 k 的邻接点，如果通过 k 到未选顶点的边更短，就更新。'),
          ],
        ),
        TutorialStep(
          title: '贪心正确性',
          description: 'Prim 算法每次选择连接已选集和未选集的最小权边，这条边一定在某棵最小生成树中。',
          focusLines: [6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37],
          explanations: [
            LineExplanation(line: 7, short: '割性质', detail: '已选集和未选集形成图的割，最小横跨边必在 MST 中。'),
            LineExplanation(line: 18, short: '选最小', detail: 'lowcost 数组高效维护了割上的最小横跨边。'),
          ],
        ),
      ],
    ),

    CodeTemplate(
      'kruskalMST',
      'Kruskal 最小生成树',
      '图算法',
      '#include <stdio.h>\n'
      '#define MAXE 10\n'
      '#define MAXV 5\n'
      '\n'
      'typedef struct {\n'
      '    int u;\n'
      '    int v;\n'
      '    int w;\n'
      '} Edge;\n'
      '\n'
      'int Find(int parent[], int x) {\n'
      '    if (parent[x] != x) parent[x] = Find(parent, parent[x]);\n'
      '    return parent[x];\n'
      '}\n'
      '\n'
      'void Union(int parent[], int x, int y) {\n'
      '    parent[Find(parent, x)] = Find(parent, y);\n'
      '}\n'
      '\n'
      'void Kruskal(Edge edges[], int n, int e) {\n'
      '    int parent[MAXV];\n'
      '    for (int i = 0; i < n; i++) parent[i] = i;\n'
      '    for (int i = 0; i < e - 1; i++) {\n'
      '        int min = i;\n'
      '        for (int j = i + 1; j < e; j++) {\n'
      '            if (edges[j].w < edges[min].w) min = j;\n'
      '        }\n'
      '        if (min != i) {\n'
      '            Edge tmp = edges[i];\n'
      '            edges[i] = edges[min];\n'
      '            edges[min] = tmp;\n'
      '        }\n'
      '    }\n'
      '    for (int i = 0; i < e; i++) {\n'
      '        int u = edges[i].u;\n'
      '        int v = edges[i].v;\n'
      '        if (Find(parent, u) != Find(parent, v)) {\n'
      '            printf("(%d,%d)=%d\\n", u, v, edges[i].w);\n'
      '            Union(parent, u, v);\n'
      '        }\n'
      '    }\n'
      '}\n'
      '\n'
      'int main() {\n'
      '    Edge edges[MAXE] = {\n'
      '        {0, 1, 2}, {0, 3, 6}, {1, 2, 3},\n'
      '        {1, 3, 8}, {1, 4, 5}, {2, 4, 7},\n'
      '        {3, 4, 9}\n'
      '    };\n'
      '    Kruskal(edges, MAXV, 7);\n'
      '    return 0;\n'
      '}',
      params: [],
      tutorialSteps: [
        TutorialStep(
          title: '边排序',
          description: 'Kruskal 算法先把所有边按权值从小到大排序，然后用贪心策略依次选择不形成环的边。',
          focusLines: [21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34],
          explanations: [
            LineExplanation(line: 23, short: '初始化', detail: 'parent[i] = i，每个顶点自成一集合。'),
            LineExplanation(line: 25, short: '选择排序', detail: '对边数组进行简单选择排序，按权值升序排列。'),
            LineExplanation(line: 30, short: '交换', detail: '把最小边交换到第 i 个位置。'),
          ],
        ),
        TutorialStep(
          title: '并查集判环',
          description: '依次检查每条边，如果两个端点不在同一集合，就加入生成树并合并集合；如果在同一集合，加入会形成环，就跳过。',
          focusLines: [35, 36, 37, 38, 39, 40, 41, 42, 43, 44],
          explanations: [
            LineExplanation(line: 39, short: '判环', detail: 'Find(parent, u) != Find(parent, v) 说明 u 和 v 不在同一集合，加入不会成环。'),
            LineExplanation(line: 40, short: '输出边', detail: '将这条边加入最小生成树。'),
            LineExplanation(line: 41, short: '合并集合', detail: 'Union 把 u 和 v 所在集合合并，表示这两个连通分量已连接。'),
          ],
        ),
      ],
    ),

    CodeTemplate(
      'dijkstra',
      'Dijkstra 最短路径',
      '图算法',
      '#include <stdio.h>\n'
      '#define MAXV 5\n'
      '#define INF 65535\n'
      '\n'
      'void Dijkstra(int G[][MAXV], int n, int v0) {\n'
      '    int dist[MAXV];\n'
      '    int visited[MAXV];\n'
      '    for (int i = 0; i < n; i++) {\n'
      '        dist[i] = G[v0][i];\n'
      '        visited[i] = 0;\n'
      '    }\n'
      '    visited[v0] = 1;\n'
      '    for (int i = 1; i < n; i++) {\n'
      '        int min = INF;\n'
      '        int u = -1;\n'
      '        for (int j = 0; j < n; j++) {\n'
      '            if (!visited[j] && dist[j] < min) {\n'
      '                min = dist[j];\n'
      '                u = j;\n'
      '            }\n'
      '        }\n'
      '        if (u == -1) break;\n'
      '        visited[u] = 1;\n'
      '        for (int j = 0; j < n; j++) {\n'
      '            if (!visited[j] && G[u][j] != INF && dist[u] + G[u][j] < dist[j]) {\n'
      '                dist[j] = dist[u] + G[u][j];\n'
      '            }\n'
      '        }\n'
      '    }\n'
      '    for (int i = 0; i < n; i++) {\n'
      '        printf("%d ", dist[i]);\n'
      '    }\n'
      '    printf("\\n");\n'
      '}\n'
      '\n'
      'int main() {\n'
      '    int G[MAXV][MAXV] = {\n'
      '        {0, 2, INF, 6, INF},\n'
      '        {2, 0, 3, 8, 5},\n'
      '        {INF, 3, 0, INF, 7},\n'
      '        {6, 8, INF, 0, 9},\n'
      '        {INF, 5, 7, 9, 0}\n'
      '    };\n'
      '    Dijkstra(G, MAXV, 0);\n'
      '    return 0;\n'
      '}',
      params: [],
      tutorialSteps: [
        TutorialStep(
          title: '初始化',
          description: 'Dijkstra 算法用 dist[] 记录源点到各顶点的当前最短距离，visited[] 标记已确定最短路径的顶点。',
          focusLines: [6, 7, 8, 9, 10, 11, 12, 13, 14],
          explanations: [
            LineExplanation(line: 7, short: '距离数组', detail: 'dist[i] 初始为源点 v0 到 i 的直接边权。'),
            LineExplanation(line: 8, short: '访问标记', detail: 'visited[i] = 0 表示尚未确定最短路径。'),
            LineExplanation(line: 10, short: '源点已访问', detail: 'visited[v0] = 1，源点到自身的距离已确定为 0。'),
          ],
        ),
        TutorialStep(
          title: '选最近顶点与松弛',
          description: '每次从未访问顶点中选 dist 最小者 u，标记为已访问，然后用 u 的所有邻接边进行松弛：如果经过 u 到 j 更短，就更新 dist[j]。',
          focusLines: [15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34],
          explanations: [
            LineExplanation(line: 16, short: '选最小', detail: '在未访问顶点中选 dist 最小的 u。'),
            LineExplanation(line: 23, short: '标记确定', detail: 'visited[u] = 1，u 的最短距离已确定。'),
            LineExplanation(line: 27, short: '松弛条件', detail: '!visited[j] && G[u][j] != INF 表示 j 未访问且 u 到 j 有边。'),
            LineExplanation(line: 28, short: '更新距离', detail: 'dist[u] + G[u][j] < dist[j] 时更新 dist[j]。'),
          ],
        ),
      ],
    ),

    CodeTemplate(
      'floyd',
      'Floyd 最短路径',
      '图算法',
      '#include <stdio.h>\n'
      '#define MAXV 4\n'
      '#define INF 65535\n'
      '\n'
      'void Floyd(int G[][MAXV], int n) {\n'
      '    for (int k = 0; k < n; k++) {\n'
      '        for (int i = 0; i < n; i++) {\n'
      '            for (int j = 0; j < n; j++) {\n'
      '                if (G[i][k] + G[k][j] < G[i][j]) {\n'
      '                    G[i][j] = G[i][k] + G[k][j];\n'
      '                }\n'
      '            }\n'
      '        }\n'
      '    }\n'
      '    for (int i = 0; i < n; i++) {\n'
      '        for (int j = 0; j < n; j++) {\n'
      '            if (G[i][j] == INF) printf("INF ");\n'
      '            else printf("%d ", G[i][j]);\n'
      '        }\n'
      '        printf("\\n");\n'
      '    }\n'
      '}\n'
      '\n'
      'int main() {\n'
      '    int G[MAXV][MAXV] = {\n'
      '        {0, 2, 6, INF},\n'
      '        {INF, 0, 3, INF},\n'
      '        {INF, INF, 0, 1},\n'
      '        {INF, INF, INF, 0}\n'
      '    };\n'
      '    Floyd(G, MAXV);\n'
      '    return 0;\n'
      '}',
      params: [],
      tutorialSteps: [
        TutorialStep(
          title: '动态规划思想',
          description: 'Floyd 算法用三重循环，枚举中间顶点 k，检查从 i 经 k 到 j 是否比直接从 i 到 j 更短。',
          focusLines: [6, 7, 8, 9, 10, 11, 12, 13, 14],
          explanations: [
            LineExplanation(line: 7, short: '中间点', detail: 'k 作为中间顶点，依次允许经过顶点 0、1、...、n-1。'),
            LineExplanation(line: 8, short: '起点', detail: 'i 遍历所有起点。'),
            LineExplanation(line: 9, short: '终点', detail: 'j 遍历所有终点。'),
            LineExplanation(line: 10, short: '松弛', detail: 'G[i][k] + G[k][j] < G[i][j] 时更新为更短路径。'),
          ],
        ),
        TutorialStep(
          title: '输出结果',
          description: '三重循环结束后，G[i][j] 就是顶点 i 到 j 的最短路径长度。',
          focusLines: [15, 16, 17, 18, 19, 20, 21, 22],
          explanations: [
            LineExplanation(line: 18, short: '无穷大', detail: 'G[i][j] == INF 表示 i 无法到达 j。'),
            LineExplanation(line: 19, short: '最短距离', detail: '输出 i 到 j 的最短路径长度。'),
          ],
        ),
      ],
    ),

    CodeTemplate(
      'topologicalSort',
      '拓扑排序',
      '图算法',
      '#include <stdio.h>\n'
      '#define MAXV 6\n'
      '\n'
      'void TopologicalSort(int G[][MAXV], int n) {\n'
      '    int indegree[MAXV] = {0};\n'
      '    int queue[MAXV];\n'
      '    int front = 0, rear = 0;\n'
      '    for (int i = 0; i < n; i++) {\n'
      '        for (int j = 0; j < n; j++) {\n'
      '            if (G[i][j] != 0) indegree[j]++;\n'
      '        }\n'
      '    }\n'
      '    for (int i = 0; i < n; i++) {\n'
      '        if (indegree[i] == 0) queue[rear++] = i;\n'
      '    }\n'
      '    while (front < rear) {\n'
      '        int u = queue[front++];\n'
      '        printf("%d ", u);\n'
      '        for (int v = 0; v < n; v++) {\n'
      '            if (G[u][v] != 0) {\n'
      '                indegree[v]--;\n'
      '                if (indegree[v] == 0) queue[rear++] = v;\n'
      '            }\n'
      '        }\n'
      '    }\n'
      '    printf("\\n");\n'
      '}\n'
      '\n'
      'int main() {\n'
      '    int G[MAXV][MAXV] = {\n'
      '        {0, 1, 1, 0, 0, 0},\n'
      '        {0, 0, 0, 1, 1, 0},\n'
      '        {0, 0, 0, 1, 0, 0},\n'
      '        {0, 0, 0, 0, 0, 1},\n'
      '        {0, 0, 0, 0, 0, 1},\n'
      '        {0, 0, 0, 0, 0, 0}\n'
      '    };\n'
      '    TopologicalSort(G, MAXV);\n'
      '    return 0;\n'
      '}',
      params: [],
      tutorialSteps: [
        TutorialStep(
          title: '计算入度',
          description: '拓扑排序先计算每个顶点的入度（指向该顶点的边数），然后把所有入度为 0 的顶点入队。',
          focusLines: [6, 7, 8, 9, 10, 11, 12, 13, 14, 15],
          explanations: [
            LineExplanation(line: 7, short: '入度数组', detail: 'indegree[j] 记录顶点 j 的入度。'),
            LineExplanation(line: 10, short: '统计入度', detail: 'G[i][j] != 0 表示存在从 i 到 j 的边，j 的入度加 1。'),
            LineExplanation(line: 14, short: '入队', detail: '入度为 0 的顶点没有前驱，可以作为拓扑序列的起点。'),
          ],
        ),
        TutorialStep(
          title: '输出与更新',
          description: '依次出队并输出，然后删除该顶点的所有出边（对应邻接点的入度减 1）。如果某个邻接点入度变为 0，就入队。',
          focusLines: [16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27],
          explanations: [
            LineExplanation(line: 17, short: '循环条件', detail: 'front < rear 表示队列非空，还有入度为 0 的顶点未处理。'),
            LineExplanation(line: 18, short: '出队输出', detail: '取出队头顶点 u 并输出。'),
            LineExplanation(line: 22, short: '删边', detail: 'indegree[v]--，删除 u 到 v 的边，v 的入度减 1。'),
            LineExplanation(line: 23, short: '新入度为 0', detail: '如果 v 的入度减为 0，说明所有前驱都已输出，v 可以入队。'),
          ],
        ),
      ],
    ),

    CodeTemplate(
      'criticalPath',
      '关键路径',
      '图算法',
      '#include <stdio.h>\n'
      '#define MAXV 6\n'
      '#define INF 65535\n'
      '\n'
      'void CriticalPath(int G[][MAXV], int n) {\n'
      '    int indegree[MAXV] = {0};\n'
      '    int queue[MAXV];\n'
      '    int front = 0, rear = 0;\n'
      '    int ve[MAXV] = {0};\n'
      '    int vl[MAXV];\n'
      '    int topo[MAXV];\n'
      '    int topoCount = 0;\n'
      '    for (int i = 0; i < n; i++)\n'
      '        for (int j = 0; j < n; j++)\n'
      '            if (G[i][j] != INF && G[i][j] != 0) indegree[j]++;\n'
      '    for (int i = 0; i < n; i++)\n'
      '        if (indegree[i] == 0) queue[rear++] = i;\n'
      '    while (front < rear) {\n'
      '        int u = queue[front++];\n'
      '        topo[topoCount++] = u;\n'
      '        for (int v = 0; v < n; v++) {\n'
      '            if (G[u][v] != INF && G[u][v] != 0) {\n'
      '                if (ve[u] + G[u][v] > ve[v]) ve[v] = ve[u] + G[u][v];\n'
      '                indegree[v]--;\n'
      '                if (indegree[v] == 0) queue[rear++] = v;\n'
      '            }\n'
      '        }\n'
      '    }\n'
      '    for (int i = 0; i < n; i++) vl[i] = ve[topo[n - 1]];\n'
      '    for (int i = n - 1; i >= 0; i--) {\n'
      '        int u = topo[i];\n'
      '        for (int v = 0; v < n; v++) {\n'
      '            if (G[u][v] != INF && G[u][v] != 0) {\n'
      '                if (vl[v] - G[u][v] < vl[u]) vl[u] = vl[v] - G[u][v];\n'
      '            }\n'
      '        }\n'
      '    }\n'
      '    for (int u = 0; u < n; u++) {\n'
      '        for (int v = 0; v < n; v++) {\n'
      '            if (G[u][v] != INF && G[u][v] != 0) {\n'
      '                int e = ve[u];\n'
      '                int l = vl[v] - G[u][v];\n'
      '                if (e == l) printf("(%d,%d) ", u, v);\n'
      '            }\n'
      '        }\n'
      '    }\n'
      '    printf("\\n");\n'
      '}\n'
      '\n'
      'int main() {\n'
      '    int G[MAXV][MAXV] = {\n'
      '        {INF, 3, 2, INF, INF, INF},\n'
      '        {INF, INF, INF, 2, 3, INF},\n'
      '        {INF, INF, INF, 4, INF, 3},\n'
      '        {INF, INF, INF, INF, INF, 2},\n'
      '        {INF, INF, INF, INF, INF, 1},\n'
      '        {INF, INF, INF, INF, INF, INF}\n'
      '    };\n'
      '    CriticalPath(G, MAXV);\n'
      '    return 0;\n'
      '}',
      params: [],
      tutorialSteps: [
        TutorialStep(
          title: '拓扑排序与最早时间',
          description: '先进行拓扑排序，同时计算每个事件的最早发生时间 ve。ve[j] = max(ve[i] + weight(i,j))。',
          focusLines: [6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29],
          explanations: [
            LineExplanation(line: 10, short: '统计入度', detail: '计算每个顶点的入度，用于拓扑排序。'),
            LineExplanation(line: 21, short: '更新 ve', detail: 've[u] + G[u][v] > ve[v] 时更新，取所有前驱中最大的完成时间。'),
          ],
        ),
        TutorialStep(
          title: '最迟时间与关键活动',
          description: '逆拓扑序计算最迟发生时间 vl。然后对每条边计算 e 和 l，e == l 的边就是关键活动。',
          focusLines: [30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49],
          explanations: [
            LineExplanation(line: 31, short: '初始化 vl', detail: 'vl 全部初始化为总工期（最后一个事件的 ve）。'),
            LineExplanation(line: 36, short: '逆推 vl', detail: 'vl[u] = min(vl[v] - G[u][v])，逆拓扑序更新。'),
            LineExplanation(line: 43, short: '判断关键活动', detail: 'e == l 说明该活动没有机动时间，是关键活动。'),
          ],
        ),
      ],
    ),

    CodeTemplate(
      'bellmanFord',
      'Bellman-Ford 最短路径',
      '图算法',
      '#include <stdio.h>\n'
      '#define MAXV 5\n'
      '#define INF 65535\n'
      '\n'
      'struct Edge {\n'
      '    int u, v, w;\n'
      '};\n'
      '\n'
      'void BellmanFord(struct Edge edges[], int e, int n, int v0) {\n'
      '    int dist[MAXV];\n'
      '    for (int i = 0; i < n; i++) dist[i] = INF;\n'
      '    dist[v0] = 0;\n'
      '    for (int i = 1; i < n; i++) {\n'
      '        for (int j = 0; j < e; j++) {\n'
      '            int u = edges[j].u, v = edges[j].v, w = edges[j].w;\n'
      '            if (dist[u] != INF && dist[u] + w < dist[v])\n'
      '                dist[v] = dist[u] + w;\n'
      '        }\n'
      '    }\n'
      '    for (int j = 0; j < e; j++) {\n'
      '        int u = edges[j].u, v = edges[j].v, w = edges[j].w;\n'
      '        if (dist[u] != INF && dist[u] + w < dist[v]) {\n'
      '            printf("negative cycle\\n");\n'
      '            return;\n'
      '        }\n'
      '    }\n'
      '    for (int i = 0; i < n; i++) printf("%d ", dist[i]);\n'
      '    printf("\\n");\n'
      '}\n'
      '\n'
      'int main() {\n'
      '    struct Edge edges[] = {\n'
      '        {0, 1, -1}, {0, 2, 4}, {1, 2, 3}, {1, 3, 2},\n'
      '        {1, 4, 2}, {3, 2, 5}, {3, 1, 1}, {4, 3, -3}\n'
      '    };\n'
      '    BellmanFord(edges, 8, 5, 0);\n'
      '    return 0;\n'
      '}',
      params: [],
      tutorialSteps: [
        TutorialStep(
          title: '松弛操作',
          description: 'Bellman-Ford 对每条边进行 n-1 轮松弛。如果 dist[u] + w < dist[v]，就更新 dist[v]。',
          focusLines: [10, 11, 12, 13, 14, 15, 16, 17],
          explanations: [
            LineExplanation(line: 11, short: '初始化', detail: 'dist 全部设为 INF，只有源点 dist[v0] = 0。'),
            LineExplanation(line: 13, short: 'n-1 轮', detail: '最多进行 n-1 轮松弛，每轮遍历所有边。'),
            LineExplanation(line: 16, short: '松弛条件', detail: 'dist[u] != INF 保证 u 可达，dist[u]+w < dist[v] 说明找到更短路径。'),
          ],
        ),
        TutorialStep(
          title: '负权环检测',
          description: 'n-1 轮松弛后，如果还能松弛，说明图中存在负权环，最短路径不存在。',
          focusLines: [18, 19, 20, 21, 22, 23, 24, 25],
          explanations: [
            LineExplanation(line: 20, short: '再扫描一轮', detail: '对所有边再执行一次松弛检查。'),
            LineExplanation(line: 22, short: '存在负环', detail: '如果还能更新 dist，说明存在负权回路。'),
          ],
        ),
      ],
    ),

    CodeTemplate(
      'spfa',
      'SPFA 最短路径',
      '图算法',
      '#include <stdio.h>\n'
      '#define MAXV 5\n'
      '#define INF 65535\n'
      '\n'
      'struct Edge {\n'
      '    int u, v, w;\n'
      '};\n'
      '\n'
      'void SPFA(struct Edge edges[], int e, int n, int v0) {\n'
      '    int dist[MAXV];\n'
      '    int inqueue[MAXV] = {0};\n'
      '    int queue[MAXV];\n'
      '    int front = 0, rear = 0;\n'
      '    int count[MAXV] = {0};\n'
      '    for (int i = 0; i < n; i++) dist[i] = INF;\n'
      '    dist[v0] = 0;\n'
      '    queue[rear++] = v0;\n'
      '    inqueue[v0] = 1;\n'
      '    count[v0]++;\n'
      '    while (front < rear) {\n'
      '        int u = queue[front++];\n'
      '        inqueue[u] = 0;\n'
      '        for (int i = 0; i < e; i++) {\n'
      '            if (edges[i].u == u) {\n'
      '                int v = edges[i].v, w = edges[i].w;\n'
      '                if (dist[u] + w < dist[v]) {\n'
      '                    dist[v] = dist[u] + w;\n'
      '                    if (!inqueue[v]) {\n'
      '                        queue[rear++] = v;\n'
      '                        inqueue[v] = 1;\n'
      '                        count[v]++;\n'
      '                        if (count[v] > n) {\n'
      '                            printf("negative cycle\\n");\n'
      '                            return;\n'
      '                        }\n'
      '                    }\n'
      '                }\n'
      '            }\n'
      '        }\n'
      '    }\n'
      '    for (int i = 0; i < n; i++) printf("%d ", dist[i]);\n'
      '    printf("\\n");\n'
      '}\n'
      '\n'
      'int main() {\n'
      '    struct Edge edges[] = {\n'
      '        {0, 1, -1}, {0, 2, 4}, {1, 2, 3}, {1, 3, 2},\n'
      '        {1, 4, 2}, {3, 2, 5}, {3, 1, 1}, {4, 3, -3}\n'
      '    };\n'
      '    SPFA(edges, 8, 5, 0);\n'
      '    return 0;\n'
      '}',
      params: [],
      tutorialSteps: [
        TutorialStep(
          title: '队列优化',
          description: 'SPFA 是 Bellman-Ford 的队列优化。只有某个节点的最短距离被更新时，它的出边才可能引起其他节点的更新，因此只把这些节点入队。',
          focusLines: [10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21],
          explanations: [
            LineExplanation(line: 11, short: '入队标记', detail: 'inqueue[v] 表示 v 是否已在队列中，避免重复入队。'),
            LineExplanation(line: 14, short: '计数器', detail: 'count[v] 记录 v 入队次数，超过 n 次说明存在负权环。'),
          ],
        ),
        TutorialStep(
          title: '松弛与入队',
          description: '取出队头节点 u，遍历 u 的所有出边。如果能松弛邻接点 v 且 v 不在队列中，就把 v 入队。',
          focusLines: [22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41],
          explanations: [
            LineExplanation(line: 23, short: '出队', detail: 'u = queue[front++]，取出队头并标记为不在队列中。'),
            LineExplanation(line: 29, short: '松弛', detail: 'dist[u] + w < dist[v] 时更新 v 的最短距离。'),
            LineExplanation(line: 31, short: '入队', detail: 'v 被更新且不在队列中时，入队等待后续处理。'),
          ],
        ),
      ],
    ),
];
