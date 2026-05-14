class AlgorithmTestCase {
  final String description;
  final List<int> inputArray;
  final int? searchTarget;
  AlgorithmTestCase(this.description, this.inputArray, [this.searchTarget]);
}

class AlgorithmValidationResult {
  final bool passed;
  final String message;
  AlgorithmValidationResult(this.passed, this.message);
}
