using Cide.Client.Shared.Core;

namespace Cide.Client.Tests;

public class NativeMethodsTests
{
    [Fact]
    public void cide_session_create_returns_valid_handle()
    {
        try
        {
            var session = NativeMethods.cide_session_create();
            Assert.NotEqual(IntPtr.Zero, session);
            NativeMethods.cide_session_destroy(session);
        }
        catch (DllNotFoundException ex)
        {
            Assert.Fail($"Native library not available: {ex.Message}");
        }
    }

    [Fact]
    public void cide_compile_returns_error_for_empty_source()
    {
        try
        {
            var session = NativeMethods.cide_session_create();
            Assert.NotEqual(IntPtr.Zero, session);
            int rc = NativeMethods.cide_compile(session, "");
            // Empty source should fail (no main function).
            Assert.NotEqual(0, rc);
            NativeMethods.cide_session_destroy(session);
        }
        catch (DllNotFoundException ex)
        {
            Assert.Fail($"Native library not available: {ex.Message}");
        }
    }

    [Fact]
    public void cide_compile_hello_world_succeeds()
    {
        try
        {
            var session = NativeMethods.cide_session_create();
            Assert.NotEqual(IntPtr.Zero, session);
            const string src = """
                int main() {
                    printf("Hello");
                    return 0;
                }
                """;
            int rc = NativeMethods.cide_compile(session, src);
            Assert.Equal(0, rc);
            NativeMethods.cide_session_destroy(session);
        }
        catch (DllNotFoundException ex)
        {
            Assert.Fail($"Native library not available: {ex.Message}");
        }
    }
}
