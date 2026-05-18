import 'package:flutter/material.dart';
import '../models/code_template.dart';

/// 模板参数输入对话框
///
/// 根据模板的 params 动态生成表单，收集学生输入后返回参数映射。
class TemplateParamDialog extends StatefulWidget {
  final CodeTemplate template;
  final void Function(Map<String, String> params) onConfirm;

  const TemplateParamDialog({
    super.key,
    required this.template,
    required this.onConfirm,
  });

  @override
  State<TemplateParamDialog> createState() => _TemplateParamDialogState();
}

class _TemplateParamDialogState extends State<TemplateParamDialog> {
  final _formKey = GlobalKey<FormState>();
  late final Map<String, TextEditingController> _controllers;

  @override
  void initState() {
    super.initState();
    _controllers = {
      for (final p in widget.template.params)
        p.key: TextEditingController(text: p.defaultValue),
    };
  }

  @override
  void dispose() {
    for (final c in _controllers.values) {
      c.dispose();
    }
    super.dispose();
  }

  void _submit() {
    if (!_formKey.currentState!.validate()) return;
    final result = <String, String>{};
    for (final entry in _controllers.entries) {
      result[entry.key] = entry.value.text.trim();
    }
    Navigator.of(context).pop();
    widget.onConfirm(result);
  }

  String? _validator(String? value, ParamType type) {
    if (value == null || value.trim().isEmpty) {
      return '不能为空';
    }
    if (type == ParamType.int) {
      final n = int.tryParse(value.trim());
      if (n == null) {
        return '请输入整数';
      }
      if (n < 0) {
        return '不能为负数';
      }
      if (n > 1000) {
        return '数值过大（最大 1000）';
      }
    }
    return null;
  }

  @override
  Widget build(BuildContext context) {
    final isDark = Theme.of(context).brightness == Brightness.dark;
    final bgColor = isDark ? const Color(0xFF1E1E1E) : Colors.white;
    final textColor = isDark ? const Color(0xFFD4D4D4) : const Color(0xFF333333);

    return Container(
      decoration: BoxDecoration(
        color: bgColor,
        borderRadius: const BorderRadius.vertical(top: Radius.circular(16)),
      ),
      padding: EdgeInsets.only(
        bottom: MediaQuery.of(context).viewInsets.bottom + 16,
        left: 20,
        right: 20,
        top: 16,
      ),
      child: Form(
        key: _formKey,
        child: Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            // 拖动条
            Center(
              child: Container(
                width: 36,
                height: 4,
                decoration: BoxDecoration(
                  color: Colors.grey.withValues(alpha: 0.4),
                  borderRadius: BorderRadius.circular(2),
                ),
              ),
            ),
            const SizedBox(height: 16),
            // 标题
            Text(
              '模板参数：${widget.template.displayName}',
              style: TextStyle(
                fontSize: 18,
                fontWeight: FontWeight.bold,
                color: textColor,
              ),
            ),
            const SizedBox(height: 4),
            Text(
              '填写以下参数，生成你的专属代码',
              style: TextStyle(
                fontSize: 13,
                color: textColor.withValues(alpha: 0.6),
              ),
            ),
            const SizedBox(height: 16),
            // 表单字段
            ...widget.template.params.map((param) {
              return Padding(
                padding: const EdgeInsets.only(bottom: 12),
                child: TextFormField(
                  controller: _controllers[param.key],
                  keyboardType: param.type == ParamType.int
                      ? TextInputType.number
                      : TextInputType.text,
                  validator: (v) => _validator(v, param.type),
                  style: TextStyle(color: textColor),
                  decoration: InputDecoration(
                    labelText: param.label,
                    labelStyle: TextStyle(color: textColor.withValues(alpha: 0.7)),
                    hintText: '默认值: ${param.defaultValue}',
                    hintStyle: TextStyle(color: textColor.withValues(alpha: 0.3)),
                    border: OutlineInputBorder(
                      borderRadius: BorderRadius.circular(8),
                    ),
                    enabledBorder: OutlineInputBorder(
                      borderRadius: BorderRadius.circular(8),
                      borderSide: BorderSide(
                        color: isDark ? const Color(0xFF3E4451) : const Color(0xFFE5E5E5),
                      ),
                    ),
                    focusedBorder: OutlineInputBorder(
                      borderRadius: BorderRadius.circular(8),
                      borderSide: const BorderSide(color: Colors.blueAccent),
                    ),
                    contentPadding: const EdgeInsets.symmetric(
                      horizontal: 12,
                      vertical: 12,
                    ),
                  ),
                ),
              );
            }),
            const SizedBox(height: 8),
            // 按钮
            Row(
              children: [
                Expanded(
                  child: OutlinedButton(
                    onPressed: () => Navigator.of(context).pop(),
                    style: OutlinedButton.styleFrom(
                      padding: const EdgeInsets.symmetric(vertical: 12),
                      shape: RoundedRectangleBorder(
                        borderRadius: BorderRadius.circular(8),
                      ),
                    ),
                    child: const Text('取消'),
                  ),
                ),
                const SizedBox(width: 12),
                Expanded(
                  child: ElevatedButton(
                    onPressed: _submit,
                    style: ElevatedButton.styleFrom(
                      padding: const EdgeInsets.symmetric(vertical: 12),
                      backgroundColor: Colors.blueAccent,
                      foregroundColor: Colors.white,
                      shape: RoundedRectangleBorder(
                        borderRadius: BorderRadius.circular(8),
                      ),
                    ),
                    child: const Text('确定'),
                  ),
                ),
              ],
            ),
            const SizedBox(height: 8),
          ],
        ),
      ),
    );
  }
}

/// 显示模板参数对话框的便捷函数
Future<void> showTemplateParamDialog({
  required BuildContext context,
  required CodeTemplate template,
  required void Function(Map<String, String> params) onConfirm,
}) async {
  await showModalBottomSheet(
    context: context,
    isScrollControlled: true,
    backgroundColor: Colors.transparent,
    builder: (context) => TemplateParamDialog(
      template: template,
      onConfirm: onConfirm,
    ),
  );
}
