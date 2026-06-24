use super::ErrorInfo;

pub(crate) fn entries() -> [(i32, ErrorInfo); 5] {
    [
        (4100, ErrorInfo {
            code: 4100,
            emoji: "🚰",
            title: "C++ 内存泄漏 (Memory Leak)",
            explanation: "使用 new 分配的内存没有被对应的 delete 释放。长期运行的程序会因此不断占用内存，最终导致资源耗尽。",
            common_causes: &["new 后忘记 delete", "异常路径导致 delete 未执行", "构造函数 new 但析构函数未 delete"],
        }),
        (4101, ErrorInfo {
            code: 4101,
            emoji: "🎣",
            title: "C++ 悬垂引用 (Dangling Reference)",
            explanation: "引用绑定到了一个生命周期即将结束的对象上。当被引用对象销毁后，引用指向的内存已经无效，继续使用会导致未定义行为。",
            common_causes: &["返回局部变量的引用", "引用绑定到临时对象", "引用的对象在引用使用前已被释放"],
        }),
        (4102, ErrorInfo {
            code: 4102,
            emoji: "🔪",
            title: "C++ 对象切片 (Object Slicing)",
            explanation: "把派生类对象赋值给基类值对象时，派生类特有的成员和数据会被'切掉'，只保留基类部分。这通常不是期望的行为。",
            common_causes: &["函数参数按值传递派生类", "用基类值对象接收派生类对象", "容器存储基类值对象"],
        }),
        (4103, ErrorInfo {
            code: 4103,
            emoji: "🪞",
            title: "C++ unique_ptr 所有权混乱",
            explanation: "unique_ptr 独占它所指向的对象所有权。通过拷贝、重复 delete 或在 move 后继续使用原指针，都会破坏所有权语义。",
            common_causes: &["对 unique_ptr 使用 std::move 后又通过原指针访问", "手动 delete unique_ptr 管理的对象", "两个 unique_ptr 指向同一地址"],
        }),
        (4104, ErrorInfo {
            code: 4104,
            emoji: "📤",
            title: "C++ move 后继续使用源对象",
            explanation: "对象被 std::move 后，其资源通常已被转移，源对象处于有效但未指定状态。把它当作仍有原值使用会导致错误结果。",
            common_causes: &["std::move 后仍读取原对象值", "move 后再次 move 同一对象", "不理解 move 语义把 move 当 copy 用"],
        })
    ]
}
