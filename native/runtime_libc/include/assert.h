/* Cide assert.h stub */
void __cide_assert_fail(void);
#define assert(expr) if (!(expr)) __cide_assert_fail()
