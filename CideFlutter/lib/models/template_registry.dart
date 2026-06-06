export 'code_template.dart';
import 'code_template.dart';
import 'templates/sort.dart';
import 'templates/search.dart';
import 'templates/graph.dart';
import 'templates/dp.dart';
import 'templates/string.dart';
import 'templates/basic.dart';
import 'templates/recursion.dart';
import 'templates/linked_list.dart';
import 'templates/tree.dart';
import 'templates/stack_queue.dart';
import 'templates/other_struct.dart';

/// 所有内置代码模板的聚合列表。
///
/// 按分类排序：排序 → 查找 → 图算法 → 动态规划 → 字符串 → 基础 → 递归 → 数据结构
const List<CodeTemplate> allTemplates = [
  ...sortTemplates,
  ...searchTemplates,
  ...graphTemplates,
  ...dpTemplates,
  ...stringTemplates,
  ...basicTemplates,
  ...recursionTemplates,
  ...linkedListTemplates,
  ...treeTemplates,
  ...stackQueueTemplates,
  ...otherStructTemplates,
];
