#include <stdio.h>

int maxSubArray(int* nums, int numsSize) {
    int maxSum = nums[0];
    int curSum = nums[0];
    for (int i = 1; i < numsSize; i++) {
        if (curSum < 0) curSum = nums[i];
        else curSum += nums[i];
        if (curSum > maxSum) maxSum = curSum;
    }
    return maxSum;
}

int main() {
    int nums1[] = {-2, 1, -3, 4, -1, 2, 1, -5, 4};
    printf("%d\n", maxSubArray(nums1, 9));

    int nums2[] = {1};
    printf("%d\n", maxSubArray(nums2, 1));

    int nums3[] = {5, 4, -1, 7, 8};
    printf("%d\n", maxSubArray(nums3, 5));

    return 0;
}
