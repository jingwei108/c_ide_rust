import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:cide/models/code_template.dart';
import 'package:cide/widgets/template_tutorial_panel.dart';
import '../helpers/pump_app.dart';

void main() {
  group('TemplateTutorialPanel', () {
    final step = TutorialStep(
      title: '初始化',
      description: '设置循环变量',
      focusLines: const [1, 2],
      explanations: [
        LineExplanation(
          line: 1,
          short: '声明变量',
          detail: '这里声明了排序所需的临时变量。',
        ),
      ],
    );

    testWidgets('renders title, step counter and description', (tester) async {
      await pumpWidget(
        tester,
        child: TemplateTutorialPanel(
          templateName: '冒泡排序',
          currentStep: 0,
          totalSteps: 3,
          step: step,
          isDark: false,
          onNext: () {},
          onPrev: () {},
          onSkip: () {},
          onRun: () {},
        ),
      );

      expect(find.text('教程模式'), findsOneWidget);
      expect(find.text('1 / 3  初始化'), findsOneWidget);
      expect(find.text('设置循环变量'), findsOneWidget);
    });

    testWidgets('prev button is disabled on first step', (tester) async {
      await pumpWidget(
        tester,
        child: TemplateTutorialPanel(
          templateName: '冒泡排序',
          currentStep: 0,
          totalSteps: 3,
          step: step,
          isDark: false,
          onNext: () {},
          onPrev: () {},
          onSkip: () {},
          onRun: () {},
        ),
      );

      final prevButton = find.widgetWithText(OutlinedButton, '上一步');
      expect(prevButton, findsOneWidget);
      expect(tester.widget<OutlinedButton>(prevButton).onPressed, isNull);
    });

    testWidgets('last step shows run button', (tester) async {
      await pumpWidget(
        tester,
        child: TemplateTutorialPanel(
          templateName: '冒泡排序',
          currentStep: 2,
          totalSteps: 3,
          step: step,
          isDark: false,
          onNext: () {},
          onPrev: () {},
          onSkip: () {},
          onRun: () {},
        ),
      );

      expect(find.text('运行代码'), findsOneWidget);
      expect(find.byIcon(Icons.play_arrow), findsOneWidget);
    });

    testWidgets('calls callbacks on button taps', (tester) async {
      String? action;
      await pumpWidget(
        tester,
        child: TemplateTutorialPanel(
          templateName: '冒泡排序',
          currentStep: 1,
          totalSteps: 3,
          step: step,
          isDark: false,
          onNext: () => action = 'next',
          onPrev: () => action = 'prev',
          onSkip: () => action = 'skip',
          onRun: () => action = 'run',
        ),
      );

      await tester.tap(find.text('跳过'));
      await tester.pump();
      expect(action, 'skip');

      await tester.tap(find.text('上一步'));
      await tester.pump();
      expect(action, 'prev');

      await tester.tap(find.text('下一步'));
      await tester.pump();
      expect(action, 'next');
    });

    testWidgets('expands line explanation on tap', (tester) async {
      await pumpWidget(
        tester,
        child: TemplateTutorialPanel(
          templateName: '冒泡排序',
          currentStep: 0,
          totalSteps: 1,
          step: step,
          isDark: false,
          onNext: () {},
          onPrev: () {},
          onSkip: () {},
          onRun: () {},
        ),
      );

      expect(find.text('这里声明了排序所需的临时变量。'), findsNothing);

      await tester.tap(find.text('声明变量'));
      await tester.pump();

      expect(find.text('这里声明了排序所需的临时变量。'), findsOneWidget);
    });
  });
}
