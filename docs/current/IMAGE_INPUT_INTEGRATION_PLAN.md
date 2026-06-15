# 拍照与本地图片输入集成方案

> 目标：为 c_ide_rust IDE 添加拍照和相册选图能力，支持 OCR 识别代码、用户头像设置、教材拍图等场景。

---

## 1. 技术选型

| 需求 | 插件 | 版本 | 理由 |
|------|------|------|------|
| 拍照 + 相册选图 | `image_picker` | ^1.1.2 | Google 官方维护，跨平台最稳定 |
| 图片裁剪（可选） | `image_cropper` | ^8.0.2 | 拍代码时裁掉多余边缘 |
| 文件路径转字节 | Flutter 内置 | - | `File(path).readAsBytes()` |
| 缓存目录获取 | `path_provider` | ^2.1.5 | 保存临时文件 |

**不选 `camera` 插件的理由**：`camera` 需要在 Flutter 层自建相机 UI，对于 IDE 场景过重；`image_picker` 直接调系统相机 App，足够轻量。

---

## 2. Flutter 端配置

### 2.1 pubspec.yaml 追加依赖

```yaml
# CideFlutter/pubspec.yaml

dependencies:
  flutter:
    sdk: flutter
  flutter_riverpod: ^2.6.1
  flutter_rust_bridge: ^2.12.0
  shared_preferences: ^2.5.0

  # ========== 图片输入新增 ==========
  image_picker: ^1.1.2
  image_cropper: ^8.0.2
  path_provider: ^2.1.5
  # ==================================

  cide_native:
    path: rust_builder
```

然后运行：
```bash
flutter pub get
```

### 2.2 Android 权限配置

```xml
<!-- CideFlutter/android/app/src/main/AndroidManifest.xml -->
<manifest xmlns:android="http://schemas.android.com/apk/res/android">

    <!-- Android 13+ 用 READ_MEDIA_IMAGES，低版本用 READ_EXTERNAL_STORAGE -->
    <uses-permission android:name="android.permission.READ_MEDIA_IMAGES" />
    <uses-permission android:name="android.permission.READ_EXTERNAL_STORAGE"
        android:maxSdkVersion="32" />
    
    <!-- 拍照权限 -->
    <uses-permission android:name="android.permission.CAMERA" />

    <application ...>
        <!-- 原有内容不变 -->
    </application>
</manifest>
```

**注意**：Android 6.0+ 需要运行时权限申请（`image_picker` 内部已处理大部分相册逻辑，但拍照需要 `CAMERA` 权限）。

### 2.3 iOS 权限配置

```xml
<!-- CideFlutter/ios/Runner/Info.plist -->
<!-- 在 <dict> 内追加以下键值 -->

<!-- 相册读取 -->
<key>NSPhotoLibraryUsageDescription</key>
<string>需要访问相册以选择图片中的代码</string>

<!-- 相册写入（拍照后保存） -->
<key>NSPhotoLibraryAddUsageDescription</key>
<string>需要保存拍摄的照片到相册</string>

<!-- 相机 -->
<key>NSCameraUsageDescription</key>
<string>需要使用相机拍摄代码或教材页面</string>

<!-- 麦克风（拍照不需要，但如果未来录视频则需要） -->
<key>NSMicrophoneUsageDescription</key>
<string>录制视频时需要使用麦克风</string>
```

---

## 3. Flutter 端封装层

创建统一的图片输入服务，隐藏平台细节：

```dart
// CideFlutter/lib/services/image_input_service.dart

import 'dart:io';
import 'dart:typed_data';
import 'package:flutter/material.dart';
import 'package:image_picker/image_picker.dart';
import 'package:image_cropper/image_cropper.dart';

/// 图片输入来源
enum ImageSourceType { camera, gallery }

/// 图片输入结果
class ImageInputResult {
  final String? filePath;
  final Uint8List? bytes;
  final String? mimeType;

  ImageInputResult({this.filePath, this.bytes, this.mimeType});

  bool get hasData => bytes != null && bytes!.isNotEmpty;
}

class ImageInputService {
  final ImagePicker _picker = ImagePicker();

  /// 拍照或选图，返回图片字节
  Future<ImageInputResult?> pickImage({
    required ImageSourceType source,
    bool crop = false,
    int? maxWidth = 1920,
    int? maxHeight = 1920,
    int imageQuality = 85,
  }) async {
    final XFile? pickedFile = await _picker.pickImage(
      source: source == ImageSourceType.camera
          ? ImageSource.camera
          : ImageSource.gallery,
      maxWidth: maxWidth?.toDouble(),
      maxHeight: maxHeight?.toDouble(),
      imageQuality: imageQuality,
    );

    if (pickedFile == null) return null;

    String? finalPath = pickedFile.path;

    // 可选：裁剪
    if (crop) {
      final CroppedFile? cropped = await ImageCropper().cropImage(
        sourcePath: pickedFile.path,
        aspectRatioPresets: [
          CropAspectRatioPreset.square,
          CropAspectRatioPreset.ratio3x2,
          CropAspectRatioPreset.original,
        ],
        uiSettings: [
          AndroidUiSettings(
            toolbarTitle: '裁剪代码区域',
            toolbarColor: Colors.deepOrange,
            initAspectRatio: CropAspectRatioPreset.original,
            lockAspectRatio: false,
          ),
          IOSUiSettings(
            title: '裁剪代码区域',
          ),
        ],
      );
      if (cropped != null) {
        finalPath = cropped.path;
      }
    }

    final bytes = await File(finalPath!).readAsBytes();
    final mimeType = _inferMimeType(finalPath);

    return ImageInputResult(
      filePath: finalPath,
      bytes: bytes,
      mimeType: mimeType,
    );
  }

  /// 批量选图（相册多选）
  Future<List<ImageInputResult>> pickMultipleImages({
    int imageQuality = 85,
  }) async {
    final List<XFile> files = await _picker.pickMultiImage(
      imageQuality: imageQuality,
    );

    final results = <ImageInputResult>[];
    for (final f in files) {
      final bytes = await File(f.path).readAsBytes();
      results.add(ImageInputResult(
        filePath: f.path,
        bytes: bytes,
        mimeType: _inferMimeType(f.path),
      ));
    }
    return results;
  }

  String? _inferMimeType(String path) {
    final lower = path.toLowerCase();
    if (lower.endsWith('.png')) return 'image/png';
    if (lower.endsWith('.jpg') || lower.endsWith('.jpeg')) return 'image/jpeg';
    if (lower.endsWith('.gif')) return 'image/gif';
    if (lower.endsWith('.webp')) return 'image/webp';
    return 'application/octet-stream';
  }
}
```

---

## 4. Rust 端接收图片（通过 FRB）

如果图片需要在 Rust 端处理（如 OCR 识别代码），需要新增 API：

### 4.1 Rust API 定义

```rust
// native/src/api/cide.rs（追加）

#[frb]
#[derive(Debug, Clone)]
pub struct ImageData {
    pub bytes: Vec<u8>,
    pub mime_type: String,
}

#[frb]
pub fn process_image(image: ImageData) -> String {
    // 这里可以接入 OCR 库（如 leptess/tesseract-rs）
    // 或进行图像预处理
    format!("收到图片: {} bytes, MIME: {}", image.bytes.len(), image.mime_type)
}
```

### 4.2 Dart 调用示例

```dart
import 'package:cide/src/rust/api/cide.dart' as rust;

Future<void> onImagePicked(ImageInputResult image) async {
  if (!image.hasData) return;

  final result = await rust.processImage(
    image: rust.ImageData(
      bytes: image.bytes!,
      mimeType: image.mimeType ?? 'image/jpeg',
    ),
  );

  // result 可能是 OCR 识别出的代码文本
  print(result);
}
```

---

## 5. 与 IDE 的集成场景

### 场景 A：拍教材上的代码 → OCR → 填入编辑器

```dart
Future<void> _onCameraOcrPressed() async {
  final service = ImageInputService();
  final image = await service.pickImage(
    source: ImageSourceType.camera,
    crop: true,
    maxWidth: 2048,
    imageQuality: 90,
  );
  if (image == null) return;

  final codeText = await rust.processImage(
    image: rust.ImageData(
      bytes: image.bytes!,
      mimeType: image.mimeType!,
    ),
  );

  // 填入编辑器
  _editorController.text = codeText;
}
```

### 场景 B：从相册选代码截图

```dart
Future<void> _onGalleryPickPressed() async {
  final service = ImageInputService();
  final images = await service.pickMultipleImages(imageQuality: 85);

  for (final img in images) {
    final result = await rust.processImage(...);
    // 批量处理...
  }
}
```

### 场景 C：用户头像设置（不需要传给 Rust）

```dart
Future<void> _onAvatarPressed() async {
  final service = ImageInputService();
  final image = await service.pickImage(
    source: ImageSourceType.gallery,
    maxWidth: 256,
    maxHeight: 256,
    imageQuality: 80,
  );
  if (image == null) return;

  setState(() {
    _avatarBytes = image.bytes;
  });
}
```

---

## 6. Rust 端图像处理（可选）

当前 `native/Cargo.toml`：

```toml
[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
flutter_rust_bridge = "=2.12.0"
```

**追加图像处理依赖**（如果 Rust 端需要处理图片）：

```toml
[dependencies]
# ... 原有依赖 ...
image = "0.25"
base64 = "0.22"
# leptess = "0.14"  # Tesseract OCR（可选，体积大）
```

**Rust 端图像预处理示例**：

```rust
use image::imageops::FilterType;

pub fn preprocess_for_ocr(bytes: &[u8]) -> Vec<u8> {
    let img = image::load_from_memory(bytes).expect("解码失败");
    let gray = img.to_luma8();
    let resized = image::imageops::resize(&gray, 1024, 768, FilterType::Lanczos3);

    let mut output = Vec::new();
    let mut encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut output, 90);
    encoder.encode_image(&resized).unwrap();
    output
}
```

---

## 7. 桌面端（Windows/macOS/Linux）的特殊处理

`image_picker` 从 v1.0 起已支持 **Windows/macOS/Linux**：

| 平台 | 配置 |
|------|------|
| **Windows** | 无需额外配置，使用系统文件选择器 |
| **macOS** | `macos/Runner/Info.plist` 追加 `NSPhotoLibraryUsageDescription` |
| **Linux** | 依赖 `xdg-desktop-portal`，现代发行版已预装 |

---

## 8. 最小可行集成（MVP）

如果只需要**最简单的"拍照/选图 → 显示在 Flutter"**，最短路径是：

```dart
final picker = ImagePicker();
final file = await picker.pickImage(source: ImageSource.camera);
if (file != null) {
  final bytes = await File(file.path).readAsBytes();
  // 直接显示
  Image.memory(bytes);
  // 或传给 Rust
  rust.processImage(image: rust.ImageData(bytes: bytes, mimeType: 'image/jpeg'));
}
```

---

## 9. 实施清单

| 步骤 | 文件 | 动作 |
|------|------|------|
| 1 | `pubspec.yaml` | 加 `image_picker`, `image_cropper`, `path_provider` |
| 2 | `AndroidManifest.xml` | 加 `CAMERA` + `READ_MEDIA_IMAGES` |
| 3 | `ios/Runner/Info.plist` | 加相机/相册权限描述 |
| 4 | 新建 | `lib/services/image_input_service.dart` |
| 5 | 可选 | `native/src/api/cide.rs` 加 `ImageData` + `process_image` |
| 6 | IDE 工具栏 | 新增相机图标按钮，绑定 `ImageInputService` |
