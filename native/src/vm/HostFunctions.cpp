#include "vm/HostFunctions.hpp"
#include "capi/CideSession.hpp"
#include "vm/CideVM.hpp"

#include <algorithm>
#include <cstdint>
#include <cstdlib>
#include <string>

namespace cide {

void HostFunctions::RegisterAll(CideSession* s, CideVM* vm, HostCtx* ctx) {
    (void)s;
    vm->SetUserData(ctx);

    // __cide_output (host id 0)
    vm->RegisterHostFunction(0, [](std::vector<int32_t>& stack, CideVM* /*vm*/, void* ud) {
        auto* c = static_cast<HostCtx*>(ud);
        int32_t val = stack.back(); stack.pop_back();
        c->session->runtime.outputLines.push_back(std::to_string(val) + "\n");
    });

    // __cide_step (host id 1)
    vm->RegisterHostFunction(1, [](std::vector<int32_t>& stack, CideVM* /*vm*/, void* ud) {
        auto* c = static_cast<HostCtx*>(ud);
        int32_t line = stack.back(); stack.pop_back();
        c->session->runtime.currentLine = line;
        c->session->runtime.trace.push_back({line, "step"});
    });

    // malloc (host id 2)
    vm->RegisterHostFunction(2, [](std::vector<int32_t>& stack, CideVM* vm, void* ud) {
        auto* c = static_cast<HostCtx*>(ud);
        int32_t size = stack.back(); stack.pop_back();
        if (size <= 0) {
            stack.push_back(0); // NULL for non-positive size
            return;
        }
        uint32_t alignedSize = (size + 3) & ~3;
        uint32_t addr = 0;
        // Try to reuse from free list (first-fit)
        for (auto it = c->session->memory.freeList.begin(); it != c->session->memory.freeList.end(); ++it) {
            if (static_cast<uint32_t>(it->size) >= alignedSize) {
                addr = it->addr;
                if (static_cast<uint32_t>(it->size) > alignedSize) {
                    // Split the block
                    it->addr += alignedSize;
                    it->size -= static_cast<int>(alignedSize);
                } else {
                    // Exact fit: remove from free list
                    c->session->memory.freeList.erase(it);
                }
                break;
            }
        }
        if (addr == 0) {
            // No suitable free block, allocate from heap offset
            addr = c->session->memory.heapOffset;
            uint32_t newOffset = addr + alignedSize;
            if (newOffset > vm->GetMemorySize()) {
                stack.push_back(0); // NULL
                return;
            }
            c->session->memory.heapOffset = newOffset;
        }
        // Reuse existing freed region record or add new one
        bool found = false;
        for (auto& r : c->session->memory.regions) {
            if (r.addr == addr && r.isFreed) {
                r.isFreed = false;
                r.size = size;
                found = true;
                break;
            }
        }
        if (!found) {
            c->session->memory.regions.push_back({addr, size, "heap_" + std::to_string(++c->session->memory.allocCounter), "int", true, false});
        }
        stack.push_back(static_cast<int32_t>(addr));
    });

    // free (host id 3)
    vm->RegisterHostFunction(3, [](std::vector<int32_t>& stack, CideVM* /*vm*/, void* ud) {
        auto* c = static_cast<HostCtx*>(ud);
        int32_t addr = stack.back(); stack.pop_back();
        for (auto& r : c->session->memory.regions) {
            if (r.addr == static_cast<uint32_t>(addr) && !r.isFreed) {
                r.isFreed = true;
                // Add to free list and merge adjacent blocks
                auto& fl = c->session->memory.freeList;
                fl.push_back({r.addr, static_cast<int>((r.size + 3) & ~3)});
                std::sort(fl.begin(), fl.end(), [](const FreeBlock& a, const FreeBlock& b) {
                    return a.addr < b.addr;
                });
                std::vector<FreeBlock> merged;
                for (auto& block : fl) {
                    if (!merged.empty() && merged.back().addr + static_cast<uint32_t>(merged.back().size) == block.addr) {
                        merged.back().size += block.size;
                    } else {
                        merged.push_back(block);
                    }
                }
                fl = std::move(merged);
                break;
            }
        }
    });

    // __cide_printf_0 (host id 10)
    vm->RegisterHostFunction(10, [](std::vector<int32_t>& stack, CideVM* vm, void* ud) {
        auto* c = static_cast<HostCtx*>(ud);
        int32_t fmtAddr = stack.back(); stack.pop_back();
        uint8_t* mem = vm->GetMemory();
        uint32_t memSize = vm->GetMemorySize();
        std::string out;
        if (mem && static_cast<uint32_t>(fmtAddr) < memSize) {
            for (uint32_t i = fmtAddr; i < memSize && mem[i] != '\0'; i++) {
                out += static_cast<char>(mem[i]);
            }
        }
        c->session->runtime.outputLines.push_back(out);
    });

    // __cide_printf_1 (host id 11) — supports %d, %s, %c, %%
    vm->RegisterHostFunction(11, [](std::vector<int32_t>& stack, CideVM* vm, void* ud) {
        auto* c = static_cast<HostCtx*>(ud);
        int32_t fmtAddr = stack.back(); stack.pop_back();
        int32_t arg = stack.back(); stack.pop_back();
        uint8_t* mem = vm->GetMemory();
        uint32_t memSize = vm->GetMemorySize();
        std::string fmt;
        if (mem && static_cast<uint32_t>(fmtAddr) < memSize) {
            for (uint32_t i = fmtAddr; i < memSize && mem[i] != '\0'; i++) {
                fmt += static_cast<char>(mem[i]);
            }
        }
        std::string out;
        bool used = false;
        for (size_t i = 0; i < fmt.size(); i++) {
            if (!used && fmt[i] == '%' && i + 1 < fmt.size()) {
                char spec = fmt[i+1];
                if (spec == 'd') {
                    out += std::to_string(arg);
                    i++; used = true;
                } else if (spec == 's') {
                    if (mem && static_cast<uint32_t>(arg) < memSize) {
                        for (uint32_t j = static_cast<uint32_t>(arg); j < memSize && mem[j] != '\0'; j++) {
                            out += static_cast<char>(mem[j]);
                        }
                    }
                    i++; used = true;
                } else if (spec == 'c') {
                    out += static_cast<char>(arg);
                    i++; used = true;
                } else if (spec == '%') {
                    out += '%';
                    i++;
                } else {
                    out += fmt[i];
                }
            } else {
                out += fmt[i];
            }
        }
        c->session->runtime.outputLines.push_back(out);
    });

    // __cide_printf_2 (host id 12) — supports %d, %s, %c, %% for 2 args
    vm->RegisterHostFunction(12, [](std::vector<int32_t>& stack, CideVM* vm, void* ud) {
        auto* c = static_cast<HostCtx*>(ud);
        int32_t fmtAddr = stack.back(); stack.pop_back();
        int32_t arg1 = stack.back(); stack.pop_back();
        int32_t arg2 = stack.back(); stack.pop_back();
        uint8_t* mem = vm->GetMemory();
        uint32_t memSize = vm->GetMemorySize();
        std::string fmt;
        if (mem && static_cast<uint32_t>(fmtAddr) < memSize) {
            for (uint32_t i = fmtAddr; i < memSize && mem[i] != '\0'; i++) {
                fmt += static_cast<char>(mem[i]);
            }
        }
        std::string out;
        int used = 0;
        for (size_t i = 0; i < fmt.size(); i++) {
            if (used < 2 && fmt[i] == '%' && i + 1 < fmt.size()) {
                char spec = fmt[i+1];
                int32_t arg = (used == 0) ? arg1 : arg2;
                if (spec == 'd') {
                    out += std::to_string(arg);
                    i++; used++;
                } else if (spec == 's') {
                    if (mem && static_cast<uint32_t>(arg) < memSize) {
                        for (uint32_t j = static_cast<uint32_t>(arg); j < memSize && mem[j] != '\0'; j++) {
                            out += static_cast<char>(mem[j]);
                        }
                    }
                    i++; used++;
                } else if (spec == 'c') {
                    out += static_cast<char>(arg);
                    i++; used++;
                } else if (spec == '%') {
                    out += '%';
                    i++;
                } else {
                    out += fmt[i];
                }
            } else {
                out += fmt[i];
            }
        }
        c->session->runtime.outputLines.push_back(out);
    });

    // __cide_printf_n (host id 15) — generic printf with any number of args
    vm->RegisterHostFunction(15, [](std::vector<int32_t>& stack, CideVM* vm, void* ud) {
        auto* c = static_cast<HostCtx*>(ud);
        int32_t fmtAddr = stack.back(); stack.pop_back();
        uint8_t* mem = vm->GetMemory();
        uint32_t memSize = vm->GetMemorySize();
        std::string fmt;
        if (mem && static_cast<uint32_t>(fmtAddr) < memSize) {
            for (uint32_t i = fmtAddr; i < memSize && mem[i] != '\0'; i++) {
                fmt += static_cast<char>(mem[i]);
            }
        }
        // Count format specifiers (excluding %%)
        int specCount = 0;
        for (size_t i = 0; i < fmt.size(); i++) {
            if (fmt[i] == '%' && i + 1 < fmt.size()) {
                if (fmt[i+1] == '%') {
                    i++;
                } else {
                    specCount++;
                    i++;
                }
            }
        }
        // Pop arguments from stack (first % corresponds to first arg pushed after fmt)
        std::vector<int32_t> args;
        args.reserve(specCount);
        for (int i = 0; i < specCount; i++) {
            args.push_back(stack.back()); stack.pop_back();
        }
        std::string out;
        int used = 0;
        for (size_t i = 0; i < fmt.size(); i++) {
            if (used < specCount && fmt[i] == '%' && i + 1 < fmt.size()) {
                char spec = fmt[i+1];
                if (spec == '%') {
                    out += '%';
                    i++;
                } else {
                    int32_t arg = args[used];
                    if (spec == 'd') {
                        out += std::to_string(arg);
                    } else if (spec == 's') {
                        if (mem && static_cast<uint32_t>(arg) < memSize) {
                            for (uint32_t j = static_cast<uint32_t>(arg); j < memSize && mem[j] != '\0'; j++) {
                                out += static_cast<char>(mem[j]);
                            }
                        }
                    } else if (spec == 'c') {
                        out += static_cast<char>(arg);
                    } else {
                        out += fmt[i];
                        out += spec;
                    }
                    i++; used++;
                }
            } else {
                out += fmt[i];
            }
        }
        c->session->runtime.outputLines.push_back(out);
    });

    // __cide_scanf_1 (host id 20)
    vm->RegisterHostFunction(20, [](std::vector<int32_t>& stack, CideVM* vm, void* ud) {
        auto* c = static_cast<HostCtx*>(ud);
        int32_t fmtAddr = stack.back(); stack.pop_back();
        int32_t p1 = stack.back(); stack.pop_back();
        uint32_t memSize = vm->GetMemorySize();

        // Parse format string safely
        char fmtSpec = 'd';
        uint64_t fmtAddr64 = static_cast<uint64_t>(static_cast<int64_t>(fmtAddr));
        if (fmtAddr64 < memSize) {
            for (uint64_t i = fmtAddr64; i < memSize; i++) {
                int8_t ch = vm->LoadI8(static_cast<uint32_t>(i));
                if (ch == '\0') break;
                if (ch == '%' && i + 1 < memSize) {
                    int8_t next = vm->LoadI8(static_cast<uint32_t>(i + 1));
                    if (next != '\0') {
                        fmtSpec = static_cast<char>(next);
                        break;
                    }
                }
            }
        }

        if (c->session->runtime.inputIndex >= c->session->runtime.inputLines.size()) {
            return;
        }
        const std::string& inputLine = c->session->runtime.inputLines[c->session->runtime.inputIndex++];
        if (fmtSpec == 'c') {
            vm->StoreI8(static_cast<uint32_t>(p1), inputLine.empty() ? 0 : static_cast<int32_t>(inputLine[0]));
        } else {
            // Default: %d
            int value = std::atoi(inputLine.c_str());
            vm->StoreI32(static_cast<uint32_t>(p1), value);
        }
    });
}

} // namespace cide
