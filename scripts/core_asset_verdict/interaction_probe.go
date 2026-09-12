//go:build windows

// interaction_probe —— 交互切面探针的 Go 迁移（D5 探针集：防线级判定型）。
//
// 影子防线只做「compile → run → 比 stdout」批处理；本探针走 cide_cli serve 的
// JSON-lines 会话，做两件影子防线做不到的事：
//
//	A. 随机交互序列（固定种子加权选择 step/seek/payload.get/breakpoints/
//	   memory.regions/output.delta/config.set/input.feed），检查进程存活、
//	   每请求一行合法 JSON、id 关联、帧形状（result/error 恰一）、
//	   步级 payload 内部不变量（step_index 单调整数、code_line 在源码行域、
//	   local_vars 名字合法、pointer_snapshots[].status 四态、semantic_label 字符串）。
//	B. 恶意输入 fuzz（畸形 JSON/缺字段/类型错/负参/巨数/深嵌套），
//	   断言"一行畸形输入不得到达 panic"（路线图 U0 验收标准）。
//
// 与 Python 版（scripts/core_asset_verdict/interaction_probe.py）的对账口径：
//   - **RNG 逐比特复刻**（choices 加权 bisect_right / choice / sample 池洗牌与
//     set 拒绝两法），同 seed 下 op 序列与请求序列完全一致 → 与同一 cide_cli.exe
//     交互，per_program（含 panic 死亡点 died_at）、错误统计、不变量违反应逐字段一致；
//   - stderr 字段含进程 pid，跨轨对账时需洗去；rss 字段仅在 --rss 下有效。
//
// 与 Python 版的有意差异（记录在案）：**门禁加牙**——Python 版 main 恒 return 0；
// 本版在 会话死亡 / 非法响应 / 不变量违反 / fuzz 杀死进程 任一发生时 exit 1
// （U0 验收标准"一行畸形输入不得到达 panic"被打破就该红）。
//
// 用法：go run scripts/core_asset_verdict/interaction_probe.go [--seed 20260912] [--ops 200] [--rss]
package main

import (
	"bufio"
	"crypto/sha512"
	"encoding/json"
	"fmt"
	"math/bits"
	"os"
	"os/exec"
	"path/filepath"
	"regexp"
	"strings"
	"sync/atomic"
	"syscall"
	"time"
	"unsafe"
)

var (
	here = mustFindHere()
	cli  = mustFindCLI()
)

func mustFindHere() string {
	wd, err := os.Getwd()
	if err != nil {
		fmt.Fprintln(os.Stderr, "无法确定工作目录:", err)
		os.Exit(2)
	}
	dir := wd
	for i := 0; i < 5; i++ {
		ok := true
		for _, marker := range []string{"native", "scripts"} {
			if fi, err := os.Stat(filepath.Join(dir, marker)); err != nil || !fi.IsDir() {
				ok = false
			}
		}
		if ok {
			return filepath.Join(dir, "scripts", "core_asset_verdict")
		}
		parent := filepath.Dir(dir)
		if parent == dir {
			break
		}
		dir = parent
	}
	fmt.Fprintln(os.Stderr, "请在仓库内运行：go run scripts/core_asset_verdict/interaction_probe.go")
	os.Exit(2)
	return ""
}

func mustFindCLI() string {
	native := filepath.Join(filepath.Dir(filepath.Dir(here)), "native")
	for _, rel := range []string{
		filepath.Join("target", "release", "cide_cli.exe"),
		filepath.Join("target", "debug", "cide_cli.exe"),
	} {
		p := filepath.Join(native, rel)
		if _, err := os.Stat(p); err == nil {
			return p
		}
	}
	fatal("找不到 cide_cli.exe（请先 cd native && cargo build --release）")
	return ""
}

func fatal(format string, args ...any) {
	fmt.Fprintf(os.Stderr, "FATAL: "+format+"\n", args...)
	os.Exit(2)
}

func truncateStr(s string, n int) string {
	if len(s) > n {
		return s[:n]
	}
	return s
}

func mnum(v any) float64 {
	f, _ := v.(float64)
	return f
}

// ───────────────────────── CPython random 复刻（与 random_diff.go 同源，待包化收拢） ─────────────────────────

type pyRandom struct {
	mt  [624]uint32
	idx int
}

// newPyRandomInt：CPython 对 **int seed** 不走 sha512 路径——直接把整数的
// 32bit little-endian words 交给 init_by_array（random_seed 的 int 分支）。
// interaction_probe 的 --seed 是纯整数，必须用此构造器（字符串 seed 才用 newPyRandom）。
func newPyRandomInt(n uint64) *pyRandom {
	key := []uint32{0}
	if n > 0 {
		key = nil
		for n > 0 {
			key = append(key, uint32(n))
			n >>= 32
		}
	}
	r := &pyRandom{}
	r.initByArray(key)
	return r
}

func newPyRandom(seed string) *pyRandom {
	sb := []byte(seed)
	h := sha512.Sum512(sb)
	buf := append(sb, h[:]...)
	nbits := 0
	for i, b := range buf {
		if b != 0 {
			nbits = (len(buf)-1-i)*8 + bits.Len8(b)
			break
		}
	}
	keyUsed := 1
	if nbits > 0 {
		keyUsed = (nbits-1)/32 + 1
	}
	key := make([]uint32, keyUsed)
	L := len(buf)
	for i := 0; i < keyUsed; i++ {
		start := L - 4*(i+1)
		var w uint32
		for j := 0; j < 4; j++ {
			var b byte
			if start+j >= 0 {
				b = buf[start+j]
			}
			w = w<<8 | uint32(b)
		}
		key[i] = w
	}
	r := &pyRandom{}
	r.initByArray(key)
	return r
}

func (r *pyRandom) initGenrand(s uint32) {
	r.mt[0] = s
	for i := 1; i < 624; i++ {
		r.mt[i] = 1812433253*(r.mt[i-1]^(r.mt[i-1]>>30)) + uint32(i)
	}
	r.idx = 624
}

func (r *pyRandom) initByArray(key []uint32) {
	r.initGenrand(19650218)
	i, j := 1, 0
	k := 624
	if len(key) > k {
		k = len(key)
	}
	for ; k > 0; k-- {
		r.mt[i] = (r.mt[i] ^ (r.mt[i-1]^(r.mt[i-1]>>30))*1664525) + key[j] + uint32(j)
		i++
		j++
		if i >= 624 {
			r.mt[0] = r.mt[623]
			i = 1
		}
		if j >= len(key) {
			j = 0
		}
	}
	for k = 623; k > 0; k-- {
		r.mt[i] = (r.mt[i] ^ (r.mt[i-1]^(r.mt[i-1]>>30))*1566083941) - uint32(i)
		i++
		if i >= 624 {
			r.mt[0] = r.mt[623]
			i = 1
		}
	}
	r.mt[0] = 0x80000000
}

func (r *pyRandom) genrandUint32() uint32 {
	if r.idx >= 624 {
		for i := 0; i < 624; i++ {
			y := r.mt[i]&0x80000000 | r.mt[(i+1)%624]&0x7fffffff
			r.mt[i] = r.mt[(i+397)%624] ^ (y >> 1)
			if y&1 != 0 {
				r.mt[i] ^= 0x9908b0df
			}
		}
		r.idx = 0
	}
	y := r.mt[r.idx]
	r.idx++
	y ^= y >> 11
	y ^= (y << 7) & 0x9d2c5680
	y ^= (y << 15) & 0xefc60000
	y ^= y >> 18
	return y
}

func (r *pyRandom) getrandbits(k int) uint32 { return r.genrandUint32() >> (32 - k) }

func (r *pyRandom) randbelow(n int) int {
	k := bits.Len(uint(n))
	for {
		v := int(r.getrandbits(k))
		if v < n {
			return v
		}
	}
}

func (r *pyRandom) randint(a, b int) int { return a + r.randbelow(b-a+1) }

func (r *pyRandom) choice(n int) int { return r.randbelow(n) }

func (r *pyRandom) random() float64 {
	a := uint64(r.genrandUint32() >> 5)
	b := uint64(r.genrandUint32() >> 6)
	return float64(a*67108864+b) * (1.0 / 9007199254740992.0)
}

// choices 加权选择：对齐 CPython random.choices（accumulate + bisect_right）
func (r *pyRandom) choices(weights []int) int {
	total := 0.0
	cum := make([]float64, len(weights))
	for i, w := range weights {
		total += float64(w)
		cum[i] = total
	}
	x := r.random() * total
	// bisect_right(cum, x)：第一个 cum[i] > x 的下标
	lo, hi := 0, len(cum)
	for lo < hi {
		mid := (lo + hi) / 2
		if x < cum[mid] {
			hi = mid
		} else {
			lo = mid + 1
		}
	}
	if lo >= len(cum) {
		lo = len(cum) - 1
	}
	return lo
}

// sample 对齐 CPython random.sample（population 视为 [1..n]）：
// n ≤ setsize(21) 用池洗牌，否则 set 拒绝采样。
func (r *pyRandom) sample(n, k int) []int {
	result := make([]int, k)
	setsize := 21
	if k > 5 {
		lg := 0
		for p := 1; p < k*3; p *= 4 {
			lg++
		}
		setsize += 1 << (2 * lg)
	}
	if n <= setsize {
		pool := make([]int, n)
		for i := 0; i < n; i++ {
			pool[i] = i + 1
		}
		for i := 0; i < k; i++ {
			j := r.randbelow(n - i)
			result[i] = pool[j]
			pool[j] = pool[n-i-1]
		}
	} else {
		selected := map[int]bool{}
		for i := 0; i < k; i++ {
			j := r.randbelow(n)
			for selected[j] {
				j = r.randbelow(n)
			}
			result[i] = j + 1
			selected[j] = true
		}
	}
	return result
}

// ───────────────────────── winmem：psapi 驱动侧采样（口径同 winmem.py） ─────────────────────────

type processMemoryCounters struct {
	CB                         uint32
	PageFaultCount             uint32
	PeakWorkingSetSize         uintptr
	WorkingSetSize             uintptr
	QuotaPeakPagedPoolUsage    uintptr
	QuotaPagedPoolUsage        uintptr
	QuotaPeakNonPagedPoolUsage uintptr
	QuotaNonPagedPoolUsage     uintptr
	PagefileUsage              uintptr
	PeakPagefileUsage          uintptr
}

var (
	kernel32              = syscall.NewLazyDLL("kernel32.dll")
	procOpenProcess       = kernel32.NewProc("OpenProcess")
	procCloseHandle       = kernel32.NewProc("CloseHandle")
	psapiDLL              = syscall.NewLazyDLL("psapi.dll")
	procGetProcessMemInfo = psapiDLL.NewProc("GetProcessMemoryInfo")
)

func memCounters(pid int) *processMemoryCounters {
	const processQueryLimitedInformation = 0x1000
	h, _, _ := procOpenProcess.Call(processQueryLimitedInformation, 0, uintptr(pid))
	if h == 0 {
		return nil
	}
	defer procCloseHandle.Call(h)
	var c processMemoryCounters
	c.CB = uint32(unsafe.Sizeof(c))
	ok, _, _ := procGetProcessMemInfo.Call(h, uintptr(unsafe.Pointer(&c)), uintptr(unsafe.Sizeof(c)))
	if ok == 0 {
		return nil
	}
	return &c
}

func round1(f float64) float64 { return float64(int(f*10+0.5)) / 10 }

func commitMB(pid int) float64 {
	c := memCounters(pid)
	if c == nil {
		return -1
	}
	return round1(float64(c.PagefileUsage) / 1048576.0)
}

func peakCommitMB(pid int) float64 {
	c := memCounters(pid)
	if c == nil {
		return -1
	}
	return round1(float64(c.PeakPagefileUsage) / 1048576.0)
}

// ───────────────────────── Serve：serve 会话 + watchdog ─────────────────────────

const hardDeadline = 90 * time.Second

type Serve struct {
	errPath string
	errFile *os.File
	cmd     *exec.Cmd
	stdin   ioWriteCloser
	stdout  *bufio.Reader
	rid     int
	start   time.Time
	exited  atomic.Bool // cmd.Wait 的 goroutine 在进程退出后置位（poll() 语义）
	pid     int
}

type ioWriteCloser interface {
	Write(p []byte) (int, error)
	Close() error
}

func newServe(tag string) *Serve {
	s := &Serve{
		errPath: filepath.Join(here, fmt.Sprintf(".serve_%s.err.log", tag)),
		start:   time.Now(),
	}
	f, err := os.Create(s.errPath)
	if err != nil {
		fatal("无法创建 stderr 日志 %s: %v", s.errPath, err)
	}
	s.errFile = f
	cmd := exec.Command(cli, "serve")
	stdin, _ := cmd.StdinPipe()
	stdout, _ := cmd.StdoutPipe()
	cmd.Stderr = f
	if err := cmd.Start(); err != nil {
		fatal("无法启动 serve: %v", err)
	}
	s.cmd = cmd
	s.pid = cmd.Process.Pid
	s.stdin = stdin.(ioWriteCloser)
	s.stdout = bufio.NewReader(stdout)
	// Wait goroutine：exited 置位即 poll() != None；同时充当进程收割者
	go func() {
		cmd.Wait()
		s.exited.Store(true)
	}()
	// watchdog：超 HARD_DEADLINE 直接 kill（对齐 Python 守护线程）
	go func() {
		for !s.exited.Load() {
			if time.Since(s.start) > hardDeadline {
				s.cmd.Process.Kill()
				return
			}
			time.Sleep(200 * time.Millisecond)
		}
	}()
	return s
}

// alive 等价 Python proc.poll() is None
func (s *Serve) alive() bool { return !s.exited.Load() }

// req 发一帧请求并读一行响应；进程已死/写读失败 → nil。
// 响应行非合法 JSON → __badjson__ 标记帧（对齐 Python）。
func (s *Serve) req(method string, params map[string]any) map[string]any {
	if !s.alive() {
		return nil
	}
	s.rid++
	req := map[string]any{"id": s.rid, "method": method}
	if params != nil {
		req["params"] = params
	}
	line, _ := json.Marshal(req)
	if _, err := s.stdin.Write(append(line, '\n')); err != nil {
		s.exited.Store(true)
		return nil
	}
	respLine, err := s.stdout.ReadBytes('\n')
	if err != nil && len(respLine) == 0 {
		s.exited.Store(true)
		return nil
	}
	var resp map[string]any
	if json.Unmarshal(respLine, &resp) != nil {
		return map[string]any{
			"__badjson__": truncateStr(string(respLine), 200),
			"__id__":      float64(s.rid),
		}
	}
	resp["__id__"] = float64(s.rid)
	return resp
}

// writeRaw 直接写一行原始文本（Part B fuzz 用）
func (s *Serve) writeRaw(line string) error {
	return s.writeLine([]byte(line))
}

func (s *Serve) writeLine(b []byte) error {
	_, err := s.stdin.Write(append(b, '\n'))
	return err
}

// readLine 读一行原始响应
func (s *Serve) readLine() ([]byte, bool) {
	line, err := s.stdout.ReadBytes('\n')
	if err != nil && len(line) == 0 {
		return nil, false
	}
	return line, true
}

func (s *Serve) close() {
	s.stdin.Close()
	done := make(chan struct{})
	go func() { s.cmd.Wait(); close(done) }()
	select {
	case <-done:
	case <-time.After(5 * time.Second):
		s.cmd.Process.Kill()
		<-done
	}
	s.errFile.Close()
}

func (s *Serve) stderrTail(n int) string {
	data, err := os.ReadFile(s.errPath)
	if err != nil {
		return ""
	}
	sv := string(data)
	if len(sv) > n {
		sv = sv[len(sv)-n:]
	}
	return sv
}

// ───────────────────────── payload 不变量（内部一致性，非独立 oracle） ─────────────────────────

var (
	validStatus  = map[string]bool{"Valid": true, "Freed": true, "Null": true, "Dangling": true}
	identRe      = regexp.MustCompile(`^[A-Za-z_][A-Za-z0-9_]*$`)
	opsWeights   = []int{55, 8, 12, 5, 10, 4, 3, 3}
	opsNames     = []string{"step.next", "seek", "payload.get", "breakpoints.set", "memory.regions", "output.delta", "config.set", "input.feed"}
	seekChoices  = []float64{0, 1, 5, 50, 200, 1000, 4999, 50000}
	pgStarts     = []float64{0, 1, 10}
	pgSpans      = []float64{1, 10, 500, 5000}
	deltaCursors = []float64{0, 1, 100}
	maxStepsOpts = []float64{1000, 100000, 10000000}
)

func checkPayloadInvariants(resp map[string]any, srcLines int, viol *[]invariant, program string) {
	res, _ := resp["result"].(map[string]any)
	if res == nil {
		return
	}
	arr, ok := res["payloads"].([]any)
	if !ok {
		return
	}
	add := func(kind, detail string) {
		*viol = append(*viol, invariant{Program: program, Kind: kind, Detail: detail})
	}
	var prev float64
	hasPrev := false
	for _, item := range arr {
		p, ok := item.(map[string]any)
		if !ok {
			add("payload_not_object", truncateStr(fmt.Sprint(item), 80))
			continue
		}
		si, siIsNum := p["step_index"].(float64)
		if !siIsNum {
			add("step_index_not_int", fmt.Sprint(p["step_index"]))
		} else if hasPrev && si < prev {
			add("step_index_not_monotonic", fmt.Sprintf("%v->%v", prev, si))
		}
		if siIsNum {
			prev = si
			hasPrev = true
		}
		if cl, ok := p["code_line"].(float64); ok && srcLines > 0 && (cl < 0 || cl > float64(srcLines)+2) {
			add("code_line_out_of_range", fmt.Sprintf("line=%v src_lines=%d", cl, srcLines))
		}
		for _, v := range asArr(p["local_vars"]) {
			vm, ok := v.(map[string]any)
			if !ok {
				continue
			}
			if nm, ok := vm["name"].(string); ok && nm != "" && !identRe.MatchString(nm) {
				add("local_var_bad_name", nm)
			}
		}
		for _, ps := range asArr(p["pointer_snapshots"]) {
			pm, ok := ps.(map[string]any)
			if !ok {
				continue
			}
			if st, ok := pm["status"].(string); ok && !validStatus[st] {
				add("pointer_status_unknown", st)
			}
		}
		if lbl, has := p["semantic_label"]; has && lbl != nil {
			if _, isStr := lbl.(string); !isStr {
				add("semantic_label_not_str", fmt.Sprint(lbl))
			}
		}
	}
}

func asArr(v any) []any {
	a, _ := v.([]any)
	return a
}

// ───────────────────────── Part A：随机交互序列 ─────────────────────────

var curated = []string{
	"doubly_linked_list", "bst_insert_search", "tree_level_order",
	"linked_queue", "linked_stack", "switch_case", "float_basic", "for_empty_cond",
}

const longProg = "#include <stdio.h>\n" +
	"int main() {\n" +
	"    int s = 0;\n" +
	"    for (int i = 0; i < 5000; i++) { s += i % 7; }\n" +
	"    printf(\"%d\\n\", s);\n" +
	"    return 0;\n" +
	"}\n"

type invariant struct {
	Program string `json:"program"`
	Kind    string `json:"kind"`
	Detail  string `json:"detail"`
}

type rssSample struct {
	Program  string  `json:"program"`
	Op       int     `json:"op"`
	CommitMB float64 `json:"commit_mb"`
}

// per_program 两变体显式构造（对齐 Python dict 形态：died 变体与正常变体键集不同）
func deadRecord(name string, diedAfter int, diedAt, stderr string) map[string]any {
	return map[string]any{
		"program": name, "died_after_ops": diedAfter, "died_at": diedAt, "stderr": stderr,
	}
}

func normalRecord(name string, ops int, alive bool, maxStepSeen, maxCollected, mbStart, mbPeak, mbPeakAPI float64) map[string]any {
	return map[string]any{
		"program": name, "ops": ops, "alive": alive,
		"max_step_seen": maxStepSeen, "max_collected_step": maxCollected,
		"commit_mb_start": mbStart, "commit_mb_peak": mbPeak, "commit_mb_peak_psapi": mbPeakAPI,
	}
}

type partAResult struct {
	OpsTotal    int              `json:"ops_total"`
	Reqs        int              `json:"reqs"`
	DeadSession int              `json:"dead_sessions"`
	BadJSON     int              `json:"badjson"`
	ProtocolErr int              `json:"protocol_errors"`
	StateErr    int              `json:"state_errors"`
	Violations  []invariant      `json:"invariant_violations"`
	PerProgram  []map[string]any `json:"per_program"`
	RSSSamples  []rssSample      `json:"rss_samples"`
	StatusHist  struct{}         `json:"status_hist"`
}

func partA(seed, ops int, doRSS bool) partAResult {
	rng := newPyRandomInt(uint64(seed))
	var programs []struct {
		name string
		src  string
	}
	casesDir := filepath.Join(filepath.Dir(filepath.Dir(here)), "native", "tests", "cases", "baseline")
	for _, name := range curated {
		p := filepath.Join(casesDir, name+".c")
		if data, err := os.ReadFile(p); err == nil {
			programs = append(programs, struct{ name, src string }{name, string(data)})
		}
	}
	programs = append(programs, struct{ name, src string }{"long_loop_5000", longProg})

	out := partAResult{Violations: []invariant{}, PerProgram: []map[string]any{}, RSSSamples: []rssSample{}}
	for _, prog := range programs {
		s := newServe("a_" + prog.name)
		var viol []invariant
		nOps := 0
		maxStepSeen := -1.0
		maxCollected := -1.0
		rss0, rssPeak := -1.0, -1.0
		if doRSS {
			rss0 = commitMB(s.pid)
			rssPeak = rss0
		}
		srcLines := strings.Count(prog.src, "\n")
		if !strings.HasSuffix(prog.src, "\n") {
			srcLines++
		}
		appendDead := func(diedAfter int, diedAt string) {
			out.PerProgram = append(out.PerProgram,
				deadRecord(prog.name, diedAfter, diedAt, s.stderrTail(1500)))
		}
		got := s.req("compile", map[string]any{"source": prog.src})
		if got == nil {
			out.DeadSession++
			appendDead(0, "compile")
			s.close()
			continue
		}
		got = s.req("step.begin", nil)
		if got == nil {
			out.DeadSession++
			appendDead(0, "step.begin")
			s.close()
			continue
		}
	died:
		for i := 0; i < ops; i++ {
			nOps++
			op := opsNames[rng.choices(opsWeights)]
			var params map[string]any
			switch op {
			case "seek":
				params = map[string]any{"step": seekChoices[rng.choice(len(seekChoices))]}
			case "payload.get":
				choiceIdx := rng.choice(len(pgStarts) + 1)
				start := pgStarts[0]
				if choiceIdx < len(pgStarts) {
					start = pgStarts[choiceIdx]
				} else if maxStepSeen > 0 {
					start = maxStepSeen
				}
				span := pgSpans[rng.choice(len(pgSpans))]
				params = map[string]any{"start": start, "end": start + span}
			case "breakpoints.set":
				n := maxInt(1, srcLines)
				k := minInt(3, n)
				lines := rng.sample(n, k)
				params = map[string]any{"lines": lines}
			case "output.delta":
				params = map[string]any{"cursor": deltaCursors[rng.choice(len(deltaCursors))]}
			case "config.set":
				params = map[string]any{"max_steps": maxStepsOpts[rng.choice(len(maxStepsOpts))]}
			case "input.feed":
				params = map[string]any{"text": "1 2 3\n"}
			}
			resp := s.req(op, params)
			if resp == nil {
				out.DeadSession++
				appendDead(nOps, op)
				break died
			}
			out.Reqs++
			if _, bad := resp["__badjson__"]; bad {
				out.BadJSON++
				continue
			}
			if mnum(resp["id"]) != mnum(resp["__id__"]) {
				viol = append(viol, invariant{prog.name, "id_mismatch",
					fmt.Sprintf("%v != %v", resp["id"], resp["__id__"])})
			}
			_, hasResult := resp["result"]
			_, hasError := resp["error"]
			if hasResult == hasError {
				raw, _ := json.Marshal(resp)
				viol = append(viol, invariant{prog.name, "frame_shape", truncateStr(string(raw), 120)})
			}
			if errObj, has := resp["error"]; has {
				kind := ""
				if m := errObj.(map[string]any); m != nil {
					kind, _ = m["kind"].(string)
				}
				if kind == "protocol" {
					out.ProtocolErr++
				} else {
					out.StateErr++
				}
			}
			switch op {
			case "step.next":
				checkPayloadInvariants(resp, srcLines, &viol, prog.name)
				if res, ok := resp["result"].(map[string]any); ok {
					for _, p := range asArr(res["payloads"]) {
						if pm, ok := p.(map[string]any); ok {
							if si, ok := pm["step_index"].(float64); ok && si > maxStepSeen {
								maxStepSeen = si
							}
						}
					}
					if mcs, ok := res["max_collected_step"].(float64); ok && mcs > maxCollected {
						maxCollected = mcs
					}
				}
			case "payload.get":
				if res, ok := resp["result"].(map[string]any); ok {
					if mcs, ok := res["max_collected_step"].(float64); ok && mcs > maxCollected {
						maxCollected = mcs
					}
				}
			}
			if doRSS && nOps%25 == 0 {
				c := commitMB(s.pid)
				if c > 0 {
					if c > rssPeak {
						rssPeak = c
					}
					out.RSSSamples = append(out.RSSSamples, rssSample{prog.name, nOps, c})
				}
			}
		}
		out.OpsTotal += nOps
		peakAPI := -1.0
		if doRSS {
			peakAPI = peakCommitMB(s.pid)
		}
		out.PerProgram = append(out.PerProgram, normalRecord(
			prog.name, nOps, s.alive(), maxStepSeen, maxCollected, rss0, rssPeak, peakAPI))
		out.Violations = append(out.Violations, viol...)
		s.close()
	}
	return out
}

func maxInt(a, b int) int {
	if a > b {
		return a
	}
	return b
}

func minInt(a, b int) int {
	if a < b {
		return a
	}
	return b
}

// ───────────────────────── Part B：恶意输入 fuzz ─────────────────────────

var malformed = buildMalformed()

func buildMalformed() []string {
	return []string{
		`{"id":1,"method":"ping"}`,
		`not json at all`,
		`{"id":2,"method":`,
		`{"id":3}`,
		`{"id":4,"method":123}`,
		`{"id":5,"method":"ping","params":[]}`,
		`{"id":6,"method":"no.such.method"}`,
		`{"id":7,"method":"compile"}`,
		`{"id":8,"method":"compile","params":{"source":123}}`,
		`{"id":9,"method":"payload.get","params":{"start":0,"end":-1}}`,
		`{"id":10,"method":"payload.get","params":{"start":-2147483648,"end":2147483647}}`,
		`{"id":11,"method":"payload.get","params":{"start":0,"end":-9223372036854775808}}`,
		`{"id":12,"method":"seek","params":{"step":-2147483648}}`,
		`{"id":13,"method":"seek","params":{"step":999999999999}}`,
		`{"id":14,"method":"seek","params":{}}`,
		`{"id":15,"method":"breakpoints.set","params":{"lines":[-1,0,2147483647]}}`,
		`{"id":16,"method":"config.set","params":{"max_steps":-1}}`,
		`{"id":17,"method":"config.set","params":{"quarantine_budget":-99999999}}`,
		`{"id":18,"method":"config.set","params":{"call_depth_limit":0}}`,
		`{"id":19,"method":"session.reset"}`,
		`{"id":20,"method":"step.begin"}`,
		`{"id":21,"method":"step.next"}`,
		`{"id":22,"method":"memory.regions"}`,
		`{"id":23,"method":"run","params":{"argv":["a"]*10000}}`,
		`{"id":24,"method":"compile","params":{"source":"` + strings.Repeat("int x=", 2000) + `1;"}}`,
		`{"id":25,"method":"compile","params":{"source":"#define A(x) x x\nA(A(A(A(1))))"}}`,
		strings.Repeat("[", 200) + strings.Repeat("]", 200),
		`{"id":26,"method":"ping","params":{"nested":` + strings.Repeat("[", 500) + strings.Repeat("]", 500) + `}}`,
		`{"id":27,"method":"semantic_labels"}`,
		`{"id":28,"method":"shutdown"}`,
	}
}

type fuzzDetail struct {
	I    int     `json:"i"`
	Ok   *bool   `json:"ok,omitempty"`
	Kind *string `json:"kind,omitempty"`
	Raw  string  `json:"raw,omitempty"`
}

type partBResult struct {
	Inputs     int          `json:"inputs"`
	Responses  int          `json:"responses"`
	BadJSONOut int          `json:"badjson_out"`
	Died       bool         `json:"died"`
	DiedAt     string       `json:"died_at"`
	Stderr     string       `json:"stderr"`
	Detail     []fuzzDetail `json:"responses_detail"`
	AliveAtEnd bool         `json:"alive_at_end"`
}

func partB() partBResult {
	s := newServe("b_fuzz")
	out := partBResult{Inputs: len(malformed), Detail: []fuzzDetail{}}
	s.req("compile", map[string]any{"source": longProg})
	s.req("step.begin", nil)
	for i, line := range malformed {
		if !s.alive() {
			out.Died = true
			out.DiedAt = fmt.Sprintf("before input #%d: %s", i+1, truncateStr(line, 60))
			break
		}
		if err := s.writeRaw(line); err != nil {
			out.Died = true
			out.DiedAt = fmt.Sprintf("input #%d write: %v", i+1, err)
			break
		}
		respLine, ok := s.readLine()
		if !ok {
			out.Died = true
			out.DiedAt = fmt.Sprintf("input #%d (no response): %s", i+1, truncateStr(line, 80))
			break
		}
		out.Responses++
		var r map[string]any
		if json.Unmarshal(respLine, &r) == nil {
			d := fuzzDetail{I: i + 1}
			if okv, has := r["ok"]; has {
				b := okv == true
				d.Ok = &b
			}
			if e, has := r["error"]; has {
				if em, ok := e.(map[string]any); ok {
					kind := ""
					if k, has := em["kind"]; has {
						kind, _ = k.(string)
					}
					d.Kind = &kind
				}
			}
			out.Detail = append(out.Detail, d)
		} else {
			out.BadJSONOut++
			out.Detail = append(out.Detail, fuzzDetail{I: i + 1, Raw: truncateStr(string(respLine), 120)})
		}
	}
	out.AliveAtEnd = s.alive()
	out.Stderr = s.stderrTail(2000)
	s.close()
	return out
}

// ───────────────────────── main ─────────────────────────

func main() {
	seed, ops, doRSS := 20260912, 200, false
	args := os.Args[1:]
	for i := 0; i < len(args); i++ {
		switch args[i] {
		case "--seed":
			fmt.Sscan(args[i+1], &seed)
		case "--ops":
			fmt.Sscan(args[i+1], &ops)
		case "--rss":
			doRSS = true
		}
	}
	start := time.Now()
	fmt.Printf("cide_cli: %s\nseed=%d ops/程序=%d rss采样=%v\n\n", cli, seed, ops, doRSS)

	a := partA(seed, ops, doRSS)
	fmt.Println("== A 随机交互序列 ==")
	fmt.Printf("  请求总数 %d  会话死亡 %d  非法响应 %d\n", a.Reqs, a.DeadSession, a.BadJSON)
	fmt.Printf("  protocol 错误 %d  state 错误 %d\n", a.ProtocolErr, a.StateErr)
	fmt.Printf("  不变量违反 %d 条\n", len(a.Violations))
	for i, v := range a.Violations {
		if i >= 20 {
			break
		}
		fmt.Printf("    - [%s] %s: %s\n", v.Program, v.Kind, v.Detail)
	}
	for _, pp := range a.PerProgram {
		name, _ := pp["program"].(string)
		view := make(map[string]any, len(pp)-1)
		for k, v := range pp {
			if k != "program" {
				view[k] = v
			}
		}
		sub, _ := json.Marshal(view)
		fmt.Printf("  %-22s %s\n", name, string(sub))
	}
	b := partB()
	fmt.Println("\n== B 恶意输入 fuzz ==")
	fmt.Printf("  输入 %d  得到响应 %d  输出非法 JSON %d\n", b.Inputs, b.Responses, b.BadJSONOut)
	fmt.Printf("  serve 进程在 fuzz 中死亡: %v  位置: %s\n", b.Died, b.DiedAt)
	fmt.Printf("  fuzz 结束后仍存活: %v\n", b.AliveAtEnd)
	if strings.TrimSpace(b.Stderr) != "" {
		fmt.Println("  stderr 尾部:")
		tail := b.Stderr
		if len(tail) > 1200 {
			tail = tail[len(tail)-1200:]
		}
		fmt.Println("   " + strings.ReplaceAll(tail, "\n", "\n   "))
	}
	out := map[string]any{"seed": seed, "ops": ops, "part_a": a, "part_b": b}
	data, _ := json.MarshalIndent(out, "", " ")
	p := filepath.Join(here, "interaction_probe.json")
	if err := os.WriteFile(p, data, 0o644); err != nil {
		fatal("JSON 写出失败: %v", err)
	}
	fmt.Printf("\nJSON 已写出: %s\n耗时: %.1fs\n", p, time.Since(start).Seconds())

	// 门禁加牙（有意差异，Python 版恒 return 0）：
	// 会话死亡 / 非法响应 / 不变量违反 / fuzz 杀死进程 任一 → exit 1（U0 验收标准）
	if a.DeadSession > 0 || a.BadJSON > 0 || len(a.Violations) > 0 || b.Died || b.BadJSONOut > 0 {
		os.Exit(1)
	}
}
