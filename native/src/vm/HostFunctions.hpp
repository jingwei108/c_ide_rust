#pragma once

#include "capi/CideSession.hpp"

namespace cide {

class CideVM;

// ============================================================================
// HostFunctions: Register built-in host functions (printf/scanf/malloc/free)
// into the CideVM. This layer encapsulates all execution semantics for
// host-side operations, keeping the capi layer thin.
// ============================================================================

class HostFunctions {
public:
    static void RegisterAll(CideSession* s, CideVM* vm, HostCtx* ctx);
};

} // namespace cide
