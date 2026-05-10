import json
import os

cards = [
    {"file": "E2005_expected_semicolon.json", "code": 2005, "emoji": "⏹️", "title": "预期分号", "explanation": "C 语言的每条语句末尾都需要分号 ; 作为结束标志。就像中文一句话末尾需要句号。忘记写分号是初学者最常见的错误之一。", "wrongCode": "int a = 10", "correctCode": "int a = 10;", "exercise": "找出下面代码中缺少的分号：\nint x = 5\nprintf(\"%d\", x)\nreturn 0", "difficulty": 1},
    {"file": "E2006_expected_closing_brace.json", "code": 2006, "emoji": "🗂️", "title": "预期右花括号", "explanation": "代码块由一对花括号 { } 包裹。编译器读到了块的末尾，但没有找到配对的 }。请检查是否遗漏了右花括号，或前面的 { 太多了。", "wrongCode": "int main() {\n    int x = 10;\n    return 0;", "correctCode": "int main() {\n    int x = 10;\n    return 0;\n}", "exercise": "给下面的代码补上缺失的 }：\nint main() {\n    if (x > 0) {\n        printf(\"yes\");\n    return 0;", "difficulty": 1},
    {"file": "E2007_expected_closing_paren.json", "code": 2007, "emoji": "🗂️", "title": "预期右圆括号", "explanation": "圆括号 ( ) 必须成对出现。常见于 if/while/for 的条件表达式或函数调用参数列表。请检查是否遗漏了右圆括号。", "wrongCode": "if (x == 5 {", "correctCode": "if (x == 5) {", "exercise": "修正下面代码的括号：\nwhile (i < 10 {", "difficulty": 1},
    {"file": "E2008_expected_closing_bracket.json", "code": 2008, "emoji": "🗂️", "title": "预期右方括号", "explanation": "方括号 [ ] 必须成对出现。常见于数组声明和数组索引访问。请检查是否遗漏了右方括号。", "wrongCode": "int arr[5 = {1,2,3};", "correctCode": "int arr[5] = {1,2,3};", "exercise": "修正下面代码：\narr[0 = 10;", "difficulty": 1},
    {"file": "E1002_unterminated_string.json", "code": 1002, "emoji": "💬", "title": "字符串未闭合", "explanation": "字符串的双引号只写了一个开头，没有找到配对的结尾。字符串必须在一对双引号之间。", "wrongCode": "char s[10] = \"hello;", "correctCode": "char s[10] = \"hello\";", "exercise": "修正下面代码：\nprintf(\"Hello World);", "difficulty": 1},
    {"file": "E1004_unsupported_op.json", "code": 1004, "emoji": "🔧", "title": "不支持的操作符", "explanation": "检测到可能是逻辑运算符的误写。在 C 语言中，逻辑或写为 ||，逻辑与写为 &&。单个 | 和 & 是位运算符，很少在条件里使用。", "wrongCode": "if (a | b) { ... }", "correctCode": "if (a || b) { ... }", "exercise": "把下面的位运算改成逻辑运算：\nif (x > 0 & y < 10)", "difficulty": 2},
    {"file": "E3023_undeclared_var.json", "code": 3023, "emoji": "❓", "title": "变量未声明", "explanation": "在使用变量之前，必须先声明它的类型（如 int x;）。C 语言不会自动创建变量。请检查变量名拼写，以及声明位置是否在使用之前。", "wrongCode": "x = 5;\nprintf(\"%d\", x);", "correctCode": "int x = 5;\nprintf(\"%d\", x);", "exercise": "下面代码缺少什么？\na = 10;\nprintf(\"%d\", a);", "difficulty": 1},
    {"file": "E3035_scanf_arg_type.json", "code": 3035, "emoji": "⌨️", "title": "scanf 参数类型错误", "explanation": "scanf 的参数必须是变量的地址（加 & 符号）。例如 scanf(\"%d\", &a);。如果参数已经是指针，则不需要 &。", "wrongCode": "scanf(\"%d\", a);", "correctCode": "scanf(\"%d\", &a);", "exercise": "下面代码错在哪里？\nint x;\nscanf(\"%d\", x);", "difficulty": 1},
    {"file": "E3041_member_non_struct.json", "code": 3041, "emoji": "🏗️", "title": "对非结构体使用成员访问", "explanation": ". 和 -> 只能用于结构体类型。如果变量是结构体变量，用 .（点）；如果是指向结构体的指针，用 ->（箭头）。", "wrongCode": "struct Point p;\np->x = 10;", "correctCode": "struct Point p;\np.x = 10;", "exercise": "下面该用 . 还是 -> ？\nstruct Point *p = malloc(sizeof(struct Point));\np___x = 10;", "difficulty": 2},
    {"file": "E3043_assign_to_rvalue.json", "code": 3043, "emoji": "🔒", "title": "向右值赋值", "explanation": "赋值号 = 左边必须是可修改的变量（左值）。不能给常量、表达式结果或数组名赋值。", "wrongCode": "a + b = 10;", "correctCode": "int c = a + b;", "exercise": "为什么下面代码是错的？\n5 = x;", "difficulty": 1},
    {"file": "E3013_missing_return.json", "code": 3013, "emoji": "↩️", "title": "非 void 函数缺少返回值", "explanation": "声明了返回类型（如 int）的函数，必须在所有执行路径上都返回一个对应类型的值。如果函数最后没有 return，程序行为将不确定。", "wrongCode": "int add(int a, int b) {\n    int c = a + b;\n}", "correctCode": "int add(int a, int b) {\n    int c = a + b;\n    return c;\n}", "exercise": "给下面函数补上 return：\nint max(int a, int b) {\n    if (a > b)\n        return a;\n}", "difficulty": 2},
]

desktop_dir = "Cide.Client/Assets/KnowledgeCards"
maui_dir = "Cide.Client.Maui/Resources/Raw/KnowledgeCards"

for c in cards:
    data = {
        "errorCode": c["code"],
        "messageContains": "",
        "emoji": c["emoji"],
        "title": c["title"],
        "plainExplanation": c["explanation"],
        "wrongCode": c["wrongCode"],
        "correctCode": c["correctCode"],
        "memoryAnimationDescription": "",
        "exercise": c["exercise"],
        "difficulty": c["difficulty"]
    }
    text = json.dumps(data, ensure_ascii=False, indent=2)
    with open(os.path.join(desktop_dir, c["file"]), "w", encoding="utf-8") as f:
        f.write(text)
    with open(os.path.join(maui_dir, c["file"]), "w", encoding="utf-8") as f:
        f.write(text)

print("Done")
