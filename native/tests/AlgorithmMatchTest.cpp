#include <iostream>
#include <cstring>
#include "../include/cide_capi.h"

bool testBinarySearchDetection() {
    const char* source = R"(
int binarySearch(int arr[], int n, int target) {
    int left = 0;
    int right = n - 1;
    while (left <= right) {
        int mid = (left + right) / 2;
        if (arr[mid] == target) {
            return mid;
        } else if (arr[mid] < target) {
            left = mid + 1;
        } else {
            right = mid - 1;
        }
    }
    return -1;
}

int main() {
    int arr[5];
    arr[0] = 1; arr[1] = 3; arr[2] = 5; arr[3] = 7; arr[4] = 9;
    return binarySearch(arr, 5, 5);
}
)";

    CideSession* s = cide_session_create();
    int result = cide_compile(s, source);
    if (result != 0) {
        const char* err = cide_get_compile_errors(s);
        std::cout << "[FAIL] Compile error: " << (err ? err : "unknown") << std::endl;
        cide_session_destroy(s);
        return false;
    }

    int count = cide_algorithm_match_count(s);
    if (count == 0) {
        std::cout << "[FAIL] No algorithm detected" << std::endl;
        cide_session_destroy(s);
        return false;
    }

    bool foundBinarySearch = false;
    for (int i = 0; i < count; i++) {
        char name[64] = {0};
        char displayName[64] = {0};
        int confidence = 0;
        char suggestion[512] = {0};
        int line = 0;
        char funcName[64] = {0};
        cide_algorithm_match_get(s, i, name, sizeof(name), displayName, sizeof(displayName), funcName, sizeof(funcName),
                                 &confidence, suggestion, sizeof(suggestion), &line);
        std::cout << "  Match[" << i << "]: " << name << " (" << displayName << ") confidence=" << confidence << " line=" << line << std::endl;
        if (std::strcmp(name, "binary_search") == 0) {
            foundBinarySearch = true;
        }
    }

    cide_session_destroy(s);
    if (foundBinarySearch) {
        std::cout << "[OK]   Binary search detected" << std::endl;
        return true;
    }
    std::cout << "[FAIL] Binary search not detected" << std::endl;
    return false;
}

bool testBubbleSortDetection() {
    const char* source = R"(
void bubbleSort(int arr[], int n) {
    int i;
    for (i = 0; i < n - 1; i = i + 1) {
        int j;
        for (j = 0; j < n - i - 1; j = j + 1) {
            if (arr[j] > arr[j + 1]) {
                int temp = arr[j];
                arr[j] = arr[j + 1];
                arr[j + 1] = temp;
            }
        }
    }
}

int main() {
    int arr[3];
    arr[0] = 3; arr[1] = 1; arr[2] = 2;
    bubbleSort(arr, 3);
    return arr[0];
}
)";

    CideSession* s = cide_session_create();
    int result = cide_compile(s, source);
    if (result != 0) {
        const char* err = cide_get_compile_errors(s);
        std::cout << "[FAIL] Compile error: " << (err ? err : "unknown") << std::endl;
        cide_session_destroy(s);
        return false;
    }

    int count = cide_algorithm_match_count(s);
    bool foundBubble = false;
    for (int i = 0; i < count; i++) {
        char name[64] = {0};
        char displayName[64] = {0};
        int confidence = 0;
        char suggestion[512] = {0};
        int line = 0;
        char funcName[64] = {0};
        cide_algorithm_match_get(s, i, name, sizeof(name), displayName, sizeof(displayName), funcName, sizeof(funcName),
                                 &confidence, suggestion, sizeof(suggestion), &line);
        if (std::strcmp(name, "bubble_sort") == 0) {
            foundBubble = true;
        }
    }

    cide_session_destroy(s);
    if (foundBubble) {
        std::cout << "[OK]   Bubble sort detected" << std::endl;
        return true;
    }
    std::cout << "[FAIL] Bubble sort not detected" << std::endl;
    return false;
}

bool testQuickSortDetection() {
    const char* source = R"(
void quickSort(int arr[], int left, int right) {
    if (left >= right) return;
    int i = left;
    int j = right;
    int pivot = arr[left];
    while (i < j) {
        while (i < j && arr[j] >= pivot) {
            j = j - 1;
        }
        while (i < j && arr[i] <= pivot) {
            i = i + 1;
        }
        if (i < j) {
            int temp = arr[i];
            arr[i] = arr[j];
            arr[j] = temp;
        }
    }
    arr[left] = arr[i];
    arr[i] = pivot;
    quickSort(arr, left, i - 1);
    quickSort(arr, i + 1, right);
}

int main() {
    int arr[5];
    arr[0] = 5; arr[1] = 3; arr[2] = 8; arr[3] = 1; arr[4] = 2;
    quickSort(arr, 0, 4);
    return arr[0];
}
)";

    CideSession* s = cide_session_create();
    int result = cide_compile(s, source);
    if (result != 0) {
        const char* err = cide_get_compile_errors(s);
        std::cout << "[FAIL] Compile error: " << (err ? err : "unknown") << std::endl;
        cide_session_destroy(s);
        return false;
    }

    int count = cide_algorithm_match_count(s);
    bool foundQuickSort = false;
    for (int i = 0; i < count; i++) {
        char name[64] = {0};
        char displayName[64] = {0};
        int confidence = 0;
        char suggestion[512] = {0};
        int line = 0;
        char funcName[64] = {0};
        cide_algorithm_match_get(s, i, name, sizeof(name), displayName, sizeof(displayName), funcName, sizeof(funcName),
                                 &confidence, suggestion, sizeof(suggestion), &line);
        std::cout << "  Match[" << i << "]: " << name << " (" << displayName << ") confidence=" << confidence << " line=" << line << std::endl;
        if (std::strcmp(name, "quick_sort") == 0) {
            foundQuickSort = true;
        }
    }

    cide_session_destroy(s);
    if (foundQuickSort) {
        std::cout << "[OK]   Quick sort detected" << std::endl;
        return true;
    }
    std::cout << "[FAIL] Quick sort not detected" << std::endl;
    return false;
}

bool testMergeSortDetection() {
    const char* source = R"(
void mergeSort(int arr[], int left, int right) {
    if (left >= right) return;
    int mid = (left + right) / 2;
    mergeSort(arr, left, mid);
    mergeSort(arr, mid + 1, right);
    int i = left;
    int j = mid + 1;
    int k = 0;
    int temp[100];
    while (i <= mid && j <= right) {
        if (arr[i] <= arr[j]) {
            temp[k] = arr[i];
            i = i + 1;
        } else {
            temp[k] = arr[j];
            j = j + 1;
        }
        k = k + 1;
    }
    while (i <= mid) {
        temp[k] = arr[i];
        i = i + 1;
        k = k + 1;
    }
    while (j <= right) {
        temp[k] = arr[j];
        j = j + 1;
        k = k + 1;
    }
    k = 0;
    for (i = left; i <= right; i = i + 1) {
        arr[i] = temp[k];
        k = k + 1;
    }
}

int main() {
    int arr[5];
    arr[0] = 5; arr[1] = 3; arr[2] = 8; arr[3] = 1; arr[4] = 2;
    mergeSort(arr, 0, 4);
    return arr[0];
}
)";

    CideSession* s = cide_session_create();
    int result = cide_compile(s, source);
    if (result != 0) {
        const char* err = cide_get_compile_errors(s);
        std::cout << "[FAIL] Compile error: " << (err ? err : "unknown") << std::endl;
        cide_session_destroy(s);
        return false;
    }

    int count = cide_algorithm_match_count(s);
    bool foundMergeSort = false;
    for (int i = 0; i < count; i++) {
        char name[64] = {0};
        char displayName[64] = {0};
        int confidence = 0;
        char suggestion[512] = {0};
        int line = 0;
        char funcName[64] = {0};
        cide_algorithm_match_get(s, i, name, sizeof(name), displayName, sizeof(displayName), funcName, sizeof(funcName),
                                 &confidence, suggestion, sizeof(suggestion), &line);
        std::cout << "  Match[" << i << "]: " << name << " (" << displayName << ") confidence=" << confidence << " line=" << line << std::endl;
        if (std::strcmp(name, "merge_sort") == 0) {
            foundMergeSort = true;
        }
    }

    cide_session_destroy(s);
    if (foundMergeSort) {
        std::cout << "[OK]   Merge sort detected" << std::endl;
        return true;
    }
    std::cout << "[FAIL] Merge sort not detected" << std::endl;
    return false;
}

bool testLinkedListInsertDetection() {
    const char* source = R"(
struct Node { int data; struct Node* next; };

struct Node* insertHead(struct Node* head, int val) {
    struct Node* newNode = 0;
    newNode->data = val;
    newNode->next = head;
    head = newNode;
    return head;
}

int main() {
    struct Node* head = 0;
    head = insertHead(head, 1);
    head = insertHead(head, 2);
    return 0;
}
)";

    CideSession* s = cide_session_create();
    int result = cide_compile(s, source);
    if (result != 0) {
        const char* err = cide_get_compile_errors(s);
        std::cout << "[FAIL] Compile error: " << (err ? err : "unknown") << std::endl;
        cide_session_destroy(s);
        return false;
    }

    int count = cide_algorithm_match_count(s);
    bool foundInsert = false;
    for (int i = 0; i < count; i++) {
        char name[64] = {0};
        char displayName[64] = {0};
        int confidence = 0;
        char suggestion[512] = {0};
        int line = 0;
        char funcName[64] = {0};
        cide_algorithm_match_get(s, i, name, sizeof(name), displayName, sizeof(displayName), funcName, sizeof(funcName),
                                 &confidence, suggestion, sizeof(suggestion), &line);
        std::cout << "  Match[" << i << "]: " << name << " (" << displayName << ") confidence=" << confidence << " line=" << line << std::endl;
        if (std::strcmp(name, "linked_list_insert") == 0) {
            foundInsert = true;
        }
    }

    cide_session_destroy(s);
    if (foundInsert) {
        std::cout << "[OK]   Linked list insert detected" << std::endl;
        return true;
    }
    std::cout << "[FAIL] Linked list insert not detected" << std::endl;
    return false;
}

bool testLinkedListDeleteDetection() {
    const char* source = R"(
struct Node { int data; struct Node* next; };

struct Node* deleteNode(struct Node* head, int val) {
    struct Node* p = head;
    while (p != 0 && p->next != 0) {
        if (p->next->data == val) {
            struct Node* temp = p->next;
            p->next = p->next->next;
        }
        p = p->next;
    }
    return head;
}

int main() {
    return 0;
}
)";

    CideSession* s = cide_session_create();
    int result = cide_compile(s, source);
    if (result != 0) {
        const char* err = cide_get_compile_errors(s);
        std::cout << "[FAIL] Compile error: " << (err ? err : "unknown") << std::endl;
        cide_session_destroy(s);
        return false;
    }

    int count = cide_algorithm_match_count(s);
    bool foundDelete = false;
    for (int i = 0; i < count; i++) {
        char name[64] = {0};
        char displayName[64] = {0};
        int confidence = 0;
        char suggestion[512] = {0};
        int line = 0;
        char funcName[64] = {0};
        cide_algorithm_match_get(s, i, name, sizeof(name), displayName, sizeof(displayName), funcName, sizeof(funcName),
                                 &confidence, suggestion, sizeof(suggestion), &line);
        std::cout << "  Match[" << i << "]: " << name << " (" << displayName << ") confidence=" << confidence << " line=" << line << std::endl;
        if (std::strcmp(name, "linked_list_delete") == 0) {
            foundDelete = true;
        }
    }

    cide_session_destroy(s);
    if (foundDelete) {
        std::cout << "[OK]   Linked list delete detected" << std::endl;
        return true;
    }
    std::cout << "[FAIL] Linked list delete not detected" << std::endl;
    return false;
}

int main() {
    std::cout << "=== Algorithm Match Test ===" << std::endl;
    int passed = 0;
    if (testBubbleSortDetection()) passed++;
    if (testBinarySearchDetection()) passed++;
    if (testQuickSortDetection()) passed++;
    if (testMergeSortDetection()) passed++;
    if (testLinkedListInsertDetection()) passed++;
    if (testLinkedListDeleteDetection()) passed++;
    std::cout << "=== Passed " << passed << "/6 ===" << std::endl;
    return (passed == 6) ? 0 : 1;
}
