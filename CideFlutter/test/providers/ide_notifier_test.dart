import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:cide/models/code_template.dart';
import 'package:cide/models/learning_progress.dart';
import 'package:cide/providers/ide_provider.dart';

void main() {
  setUpAll(() {
    SharedPreferences.setMockInitialValues({});
  });

  group('IdeNotifier build', () {
    test('default state has main.c and TextEditingController', () async {
      final container = ProviderContainer();
      addTearDown(container.dispose);

      final notifier = container.read(ideProvider.notifier);
      expect(notifier.outputController, isNotNull);
      expect(container.read(ideProvider).currentFile, 'main.c');
      expect(container.read(ideProvider).files.length, 1);

      // Wait for async _loadProgress to complete.
      await Future.delayed(Duration.zero);
      expect(container.read(ideProvider).learningProgress, const LearningProgress());
    });
  });

  group('IdeNotifier file management', () {
    test('updateSource updates current file source', () {
      final container = ProviderContainer();
      addTearDown(container.dispose);

      final notifier = container.read(ideProvider.notifier);
      notifier.updateSource('int x;');

      final state = container.read(ideProvider);
      expect(state.source, 'int x;');
      expect(state.files.first.source, 'int x;');
    });

    test('addFile creates new file and switches to it', () {
      final container = ProviderContainer();
      addTearDown(container.dispose);

      final notifier = container.read(ideProvider.notifier);
      notifier.addFile('helper.c');

      final state = container.read(ideProvider);
      expect(state.files.length, 2);
      expect(state.currentFile, 'helper.c');
      expect(state.source, '');
    });

    test('addFile ignores duplicate filename', () {
      final container = ProviderContainer();
      addTearDown(container.dispose);

      final notifier = container.read(ideProvider.notifier);
      notifier.addFile('main.c');

      final state = container.read(ideProvider);
      expect(state.files.length, 1);
    });

    test('switchFile changes current file', () {
      final container = ProviderContainer();
      addTearDown(container.dispose);

      final notifier = container.read(ideProvider.notifier);
      notifier.addFile('helper.c');
      notifier.updateSource('int y;');
      notifier.switchFile('main.c');

      final state = container.read(ideProvider);
      expect(state.currentFile, 'main.c');
      expect(state.source.contains('Hello, Cide!'), isTrue);
    });

    test('removeFile removes file and switches when needed', () {
      final container = ProviderContainer();
      addTearDown(container.dispose);

      final notifier = container.read(ideProvider.notifier);
      notifier.addFile('helper.c');
      notifier.removeFile('main.c');

      final state = container.read(ideProvider);
      expect(state.files.length, 1);
      expect(state.currentFile, 'helper.c');
    });

    test('removeFile does not remove last file', () {
      final container = ProviderContainer();
      addTearDown(container.dispose);

      final notifier = container.read(ideProvider.notifier);
      notifier.removeFile('main.c');

      final state = container.read(ideProvider);
      expect(state.files.length, 1);
    });
  });

  group('IdeNotifier panel management', () {
    test('selectBottomTab updates index', () {
      final container = ProviderContainer();
      addTearDown(container.dispose);

      final notifier = container.read(ideProvider.notifier);
      notifier.selectBottomTab(2);

      expect(container.read(ideProvider).bottomActiveIndex, 2);
    });

    test('selectFloatingTab updates index', () {
      final container = ProviderContainer();
      addTearDown(container.dispose);

      final notifier = container.read(ideProvider.notifier);
      notifier.selectFloatingTab(3);

      expect(container.read(ideProvider).floatingActiveIndex, 3);
    });

    test('setBottomHeight clamps value', () {
      final container = ProviderContainer();
      addTearDown(container.dispose);

      final notifier = container.read(ideProvider.notifier);
      notifier.setBottomHeight(50);
      expect(container.read(ideProvider).bottomHeight, 120);

      notifier.setBottomHeight(600);
      expect(container.read(ideProvider).bottomHeight, 500);

      notifier.setBottomHeight(300);
      expect(container.read(ideProvider).bottomHeight, 300);
    });

    test('toggleFloating toggles open state', () {
      final container = ProviderContainer();
      addTearDown(container.dispose);

      final notifier = container.read(ideProvider.notifier);
      expect(container.read(ideProvider).isFloatingOpen, isFalse);

      notifier.toggleFloating();
      expect(container.read(ideProvider).isFloatingOpen, isTrue);

      notifier.closeFloating();
      expect(container.read(ideProvider).isFloatingOpen, isFalse);
    });

    test('openFloatingPanel sets active panel and closes menu', () {
      final container = ProviderContainer();
      addTearDown(container.dispose);

      final notifier = container.read(ideProvider.notifier);
      notifier.openFloatingPanel('memory');

      final state = container.read(ideProvider);
      expect(state.activeFloatingPanel, 'memory');
      expect(state.isFloatingOpen, isFalse);
    });

    test('closeFloatingPanel clears active panel', () {
      final container = ProviderContainer();
      addTearDown(container.dispose);

      final notifier = container.read(ideProvider.notifier);
      notifier.openFloatingPanel('memory');
      notifier.closeFloatingPanel();

      expect(container.read(ideProvider).activeFloatingPanel, isNull);
    });

    test('swapWithBottom moves panel to bottom, overflowing last bottom panel', () {
      final container = ProviderContainer();
      addTearDown(container.dispose);

      final notifier = container.read(ideProvider.notifier);
      final originalBottom = List<String>.from(container.read(ideProvider).bottomSlots);
      notifier.swapWithBottom('memory');

      final state = container.read(ideProvider);
      expect(state.bottomSlots.contains('memory'), isTrue);
      expect(state.floatingSlots.contains('memory'), isFalse);
      // Default bottom already has 4 slots; adding 'memory' overflows the last
      // bottom panel back into floating, so bottom length stays at 4.
      expect(state.bottomSlots.length, 4);
      // The overflow panel is pushed back to floating, keeping floating count stable.
      expect(state.floatingSlots.contains(originalBottom.last), isTrue);
    });

    test('swapWithFloating moves panel to floating', () {
      final container = ProviderContainer();
      addTearDown(container.dispose);

      final notifier = container.read(ideProvider.notifier);
      notifier.swapWithBottom('memory');
      notifier.swapWithFloating('memory');

      final state = container.read(ideProvider);
      expect(state.bottomSlots.contains('memory'), isFalse);
      expect(state.floatingSlots.contains('memory'), isTrue);
    });

    test('swapBottomPanels swaps positions', () {
      final container = ProviderContainer();
      addTearDown(container.dispose);

      final notifier = container.read(ideProvider.notifier);
      final before = container.read(ideProvider).bottomSlots;
      notifier.swapBottomPanels(0, 1);

      final after = container.read(ideProvider).bottomSlots;
      expect(after[0], before[1]);
      expect(after[1], before[0]);
    });

    test('swapBottomPanels ignores out of bounds', () {
      final container = ProviderContainer();
      addTearDown(container.dispose);

      final notifier = container.read(ideProvider.notifier);
      final before = List<String>.from(container.read(ideProvider).bottomSlots);
      notifier.swapBottomPanels(0, 100);

      expect(container.read(ideProvider).bottomSlots, before);
    });

    test('removeBottomPanel reports error when floating is full', () {
      final container = ProviderContainer();
      addTearDown(container.dispose);

      final notifier = container.read(ideProvider.notifier);
      notifier.removeBottomPanel(0);

      final state = container.read(ideProvider);
      expect(state.error, contains('悬浮球'));
    });
  });

  group('IdeNotifier output / error / highlight', () {
    test('clearOutput resets output', () {
      final container = ProviderContainer();
      addTearDown(container.dispose);

      final notifier = container.read(ideProvider.notifier);
      notifier.updateSource('int main() {}');
      // Directly mutate via copyWith is not possible from notifier, test clearOutput only.
      notifier.clearOutput();
      expect(container.read(ideProvider).output, '');
    });

    test('highlightLine and clearHighlight', () {
      final container = ProviderContainer();
      addTearDown(container.dispose);

      final notifier = container.read(ideProvider.notifier);
      notifier.highlightLine(10);
      expect(container.read(ideProvider).highlightedLine, 10);

      notifier.clearHighlight();
      expect(container.read(ideProvider).highlightedLine, 0);
    });
  });

  group('IdeNotifier watch expressions', () {
    test('addWatchExpression appends unique expressions', () {
      final container = ProviderContainer();
      addTearDown(container.dispose);

      final notifier = container.read(ideProvider.notifier);
      notifier.addWatchExpression('x');
      notifier.addWatchExpression('y');
      notifier.addWatchExpression('x'); // duplicate

      final state = container.read(ideProvider);
      expect(state.watchExpressions, ['x', 'y']);
    });

    test('removeWatchExpression removes expression', () {
      final container = ProviderContainer();
      addTearDown(container.dispose);

      final notifier = container.read(ideProvider.notifier);
      notifier.addWatchExpression('x');
      notifier.addWatchExpression('y');
      notifier.removeWatchExpression('x');

      expect(container.read(ideProvider).watchExpressions, ['y']);
    });

    test('clearWatchExpressions empties list', () {
      final container = ProviderContainer();
      addTearDown(container.dispose);

      final notifier = container.read(ideProvider.notifier);
      notifier.addWatchExpression('x');
      notifier.clearWatchExpressions();

      expect(container.read(ideProvider).watchExpressions, isEmpty);
    });
  });

  group('IdeNotifier execution speed / intro', () {
    test('setExecutionSpeed clamps value', () {
      final container = ProviderContainer();
      addTearDown(container.dispose);

      final notifier = container.read(ideProvider.notifier);
      notifier.setExecutionSpeed(-10);
      expect(container.read(ideProvider).executionSpeed, 0);

      notifier.setExecutionSpeed(600);
      expect(container.read(ideProvider).executionSpeed, 500);

      notifier.setExecutionSpeed(250);
      expect(container.read(ideProvider).executionSpeed, 250);
    });

    test('showIntro and hideIntro', () {
      final container = ProviderContainer();
      addTearDown(container.dispose);

      final notifier = container.read(ideProvider.notifier);
      notifier.showIntro();
      expect(container.read(ideProvider).showIntro, isTrue);

      notifier.hideIntro();
      expect(container.read(ideProvider).showIntro, isFalse);
    });
  });

  group('IdeNotifier tutorial', () {
    test('startTutorial sets active tutorial', () {
      final container = ProviderContainer();
      addTearDown(container.dispose);

      final notifier = container.read(ideProvider.notifier);
      final template = CodeTemplate(
        'bubble_sort',
        'Bubble Sort',
        'sort',
        '',
        tutorialSteps: [
          TutorialStep(
            title: 'Step 1',
            description: 'desc',
            focusLines: [1, 2],
          ),
        ],
      );
      notifier.startTutorial(template, 'generated code');

      final state = container.read(ideProvider);
      expect(state.activeTutorial, isNotNull);
      expect(state.activeTutorial!.templateKey, 'bubble_sort');
      expect(state.activeTutorial!.stepIndex, 0);
    });

    test('nextTutorialStep advances index', () {
      final container = ProviderContainer();
      addTearDown(container.dispose);

      final notifier = container.read(ideProvider.notifier);
      final template = CodeTemplate(
        'bubble_sort',
        'Bubble Sort',
        'sort',
        '',
        tutorialSteps: [
          TutorialStep(title: 'Step 1', description: 'd', focusLines: [1]),
          TutorialStep(title: 'Step 2', description: 'd', focusLines: [2]),
        ],
      );
      notifier.startTutorial(template, 'generated code');
      notifier.nextTutorialStep();

      expect(container.read(ideProvider).activeTutorial!.stepIndex, 1);
    });

    test('prevTutorialStep does not go below 0', () {
      final container = ProviderContainer();
      addTearDown(container.dispose);

      final notifier = container.read(ideProvider.notifier);
      final template = CodeTemplate(
        'bubble_sort',
        'Bubble Sort',
        'sort',
        '',
        tutorialSteps: [
          TutorialStep(title: 'Step 1', description: 'd', focusLines: [1]),
        ],
      );
      notifier.startTutorial(template, 'generated code');
      notifier.prevTutorialStep();

      expect(container.read(ideProvider).activeTutorial!.stepIndex, 0);
    });
  });

  group('IdeNotifier progress', () {
    test('recordKnowledgeCardView adds card id once', () async {
      final container = ProviderContainer();
      addTearDown(container.dispose);

      final notifier = container.read(ideProvider.notifier);
      await notifier.recordKnowledgeCardView('E1001');
      await notifier.recordKnowledgeCardView('E1001');

      expect(container.read(ideProvider).learningProgress.viewedKnowledgeCards, {'E1001'});
    });

    test('recordUnifiedRun updates totals', () async {
      final container = ProviderContainer();
      addTearDown(container.dispose);

      final notifier = container.read(ideProvider.notifier);
      await notifier.recordUnifiedRun(steps: 10, trapped: false);
      await notifier.recordUnifiedRun(steps: 20, trapped: true, trapMessage: 'boom');

      final progress = container.read(ideProvider).learningProgress;
      expect(progress.totalUnifiedRuns, 2);
      expect(progress.totalStepsExecuted, 30);
      expect(progress.maxStepsInSingleRun, 20);
      expect(progress.totalTraps, 1);
    });

    test('recordSeek increments totalSeeks', () async {
      final container = ProviderContainer();
      addTearDown(container.dispose);

      final notifier = container.read(ideProvider.notifier);
      await notifier.recordSeek();
      await notifier.recordSeek();

      expect(container.read(ideProvider).learningProgress.totalSeeks, 2);
    });

    test('resetProgress clears progress', () async {
      final container = ProviderContainer();
      addTearDown(container.dispose);

      final notifier = container.read(ideProvider.notifier);
      await notifier.recordSeek();
      await notifier.resetProgress();

      expect(container.read(ideProvider).learningProgress, const LearningProgress());
    });
  });
}
